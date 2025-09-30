use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

impl ToolCall {
    pub fn new(name: String, arguments: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            arguments,
        }
    }

    pub fn with_id(id: String, name: String, arguments: serde_json::Value) -> Self {
        Self { id, name, arguments }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub success: bool,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(tool_call_id: String, content: String) -> Self {
        Self {
            tool_call_id,
            content,
            success: true,
            error: None,
        }
    }

    pub fn error(tool_call_id: String, error: String) -> Self {
        Self {
            tool_call_id,
            content: format!("Error: {}", error),
            success: false,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> ToolSchema;

    fn initialize(&self) -> Result<()> {
        Ok(())
    }

    fn execute(&self, arguments: &serde_json::Value) -> Result<String>;

    fn cleanup(&self) -> Result<()> {
        Ok(())
    }

    fn to_openai_function(&self) -> serde_json::Value {
        let schema = self.schema();
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &schema.parameters {
            let mut prop = serde_json::Map::new();
            prop.insert("type".to_string(), serde_json::json!(param.param_type));
            prop.insert("description".to_string(), serde_json::json!(param.description));

            // Handle array types by adding items specification
            if param.param_type == "array" {
                // For view_range, it should be an array of integers
                if param.name == "view_range" {
                    prop.insert("items".to_string(), serde_json::json!({
                        "type": "integer"
                    }));
                } else {
                    // Default to array of strings for other array parameters
                    prop.insert("items".to_string(), serde_json::json!({
                        "type": "string"
                    }));
                }
            }

            if let Some(default) = &param.default {
                prop.insert("default".to_string(), default.clone());
            }

            properties.insert(param.name.clone(), serde_json::Value::Object(prop));

            if param.required {
                required.push(param.name.clone());
            }
        }

        serde_json::json!({
            "name": schema.name,
            "description": schema.description,
            "parameters": {
                "type": "object",
                "properties": properties,
                "required": required,
            }
        })
    }

    fn to_anthropic_tool(&self) -> serde_json::Value {
        let schema = self.schema();
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &schema.parameters {
            let mut prop = serde_json::Map::new();
            prop.insert("type".to_string(), serde_json::json!(param.param_type));
            prop.insert("description".to_string(), serde_json::json!(param.description));

            if let Some(default) = &param.default {
                prop.insert("default".to_string(), default.clone());
            }

            properties.insert(param.name.clone(), serde_json::Value::Object(prop));

            if param.required {
                required.push(param.name.clone());
            }
        }

        serde_json::json!({
            "name": schema.name,
            "description": schema.description,
            "input_schema": {
                "type": "object",
                "properties": properties,
                "required": required,
            }
        })
    }
}

pub struct ToolExecutor {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolExecutor {
    pub fn new(tools: Vec<Arc<dyn Tool>>) -> Self {
        let mut tool_map = HashMap::new();
        for tool in tools {
            tool_map.insert(tool.name().to_string(), tool);
        }
        Self { tools: tool_map }
    }

    pub fn execute(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        match self.tools.get(&tool_call.name) {
            Some(tool) => {
                match tool.execute(&tool_call.arguments) {
                    Ok(result) => Ok(ToolResult::success(tool_call.id.clone(), result)),
                    Err(e) => Ok(ToolResult::error(tool_call.id.clone(), e.to_string())),
                }
            }
            None => Ok(ToolResult::error(
                tool_call.id.clone(),
                format!("Tool '{}' not found", tool_call.name),
            )),
        }
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

// Base implementation helper for tools
pub struct BaseToolImpl {
    pub name: String,
    pub description: String,
    pub schema: ToolSchema,
}

impl BaseToolImpl {
    pub fn new(name: String, description: String, parameters: Vec<ToolParameter>) -> Self {
        Self {
            name: name.clone(),
            description: description.clone(),
            schema: ToolSchema {
                name,
                description,
                parameters,
            },
        }
    }
}