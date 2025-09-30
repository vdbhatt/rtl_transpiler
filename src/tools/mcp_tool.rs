use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use rmcp::model::Tool as RmcpTool;

use crate::tools::base::{Tool, ToolSchema, ToolParameter};
use crate::mcp::MCPClient;

pub struct MCPTool {
    client: Arc<Mutex<MCPClient>>,
    tool_def: RmcpTool,
    schema: ToolSchema,
}

impl MCPTool {
    pub fn new(client: Arc<Mutex<MCPClient>>, tool_def: RmcpTool) -> Self {
        // Convert rmcp Tool to our ToolSchema
        let parameters = Self::extract_parameters(&tool_def);

        let schema = ToolSchema {
            name: tool_def.name.to_string(),
            description: tool_def.description.clone().unwrap_or_default().to_string(),
            parameters,
        };

        Self {
            client,
            tool_def,
            schema,
        }
    }

    fn extract_parameters(tool: &RmcpTool) -> Vec<ToolParameter> {
        let mut parameters = Vec::new();

        // Parse the input_schema from rmcp Tool
        let input_schema = &tool.input_schema;
        // input_schema is an Arc<serde_json::Map>
        if let Some(properties) = input_schema.get("properties").and_then(|p| p.as_object()) {
            let required = input_schema.get("required")
                .and_then(|r| r.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            for (name, prop) in properties {
                if let Some(prop_obj) = prop.as_object() {
                    let param = ToolParameter {
                        name: name.clone(),
                        param_type: prop_obj.get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("string")
                            .to_string(),
                        description: prop_obj.get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string(),
                        required: required.contains(name),
                        default: prop_obj.get("default").cloned(),
                    };
                    parameters.push(param);
                }
            }
        }

        parameters
    }
}

impl Tool for MCPTool {
    fn name(&self) -> &str {
        &self.tool_def.name
    }

    fn description(&self) -> &str {
        self.tool_def.description.as_deref().unwrap_or("")
    }

    fn schema(&self) -> ToolSchema {
        self.schema.clone()
    }

    fn execute(&self, arguments: &serde_json::Value) -> Result<String> {
        // Bridge async to sync using tokio runtime
        // Try to get existing runtime or create a new one
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            // Use existing runtime
            handle.block_on(self.execute_async(arguments))
        } else {
            // Create new runtime for this execution
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(self.execute_async(arguments))
        };

        result
    }
}

impl MCPTool {
    async fn execute_async(&self, arguments: &serde_json::Value) -> Result<String> {
        let client = self.client.lock().await;
        let tool_name = self.tool_def.name.clone();

        // Set a timeout for the tool call (similar to Python's 20 second timeout)
        let timeout_duration = tokio::time::Duration::from_secs(20);

        match tokio::time::timeout(
            timeout_duration,
            client.call_tool(&tool_name, arguments.clone())
        ).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(anyhow::anyhow!("Error running MCP tool: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Timeout calling MCP tool '{}' after 20 seconds", tool_name)),
        }
    }
}