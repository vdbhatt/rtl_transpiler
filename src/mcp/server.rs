use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::{
        CallToolRequestParam, ListToolsRequestParam, Tool as RmcpTool, ToolInputSchema,
        Content, TextContent, RawContent,
    },
    transport::StdioTransport,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use crate::tools::{Tool, TranspileTool, TextEditorTool, VHDLAnalyzeTool};

/// MCP Server that exposes VHDL transpiler tools
pub struct MCPServer {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl MCPServer {
    pub fn new() -> Self {
        let mut tools: HashMap<String, Arc<dyn Tool>> = HashMap::new();
        
        // Add transpiler tool
        let transpile_tool = Arc::new(TranspileTool::new(vec![])); // Allow all folders for MCP
        tools.insert("transpile_vhdl_to_verilog".to_string(), transpile_tool);
        
        // Add file editor tool
        let edit_tool = Arc::new(TextEditorTool::new("mcp".to_string(), vec![])); // Allow all folders for MCP
        tools.insert("edit_file".to_string(), edit_tool);
        
        // Add VHDL analysis tool
        let analyze_tool = Arc::new(VHDLAnalyzeTool::new(vec![])); // Allow all folders for MCP
        tools.insert("analyze_vhdl".to_string(), analyze_tool);
        
        Self { tools }
    }

    pub async fn run(&self) -> Result<()> {
        let server = self.clone();
        
        // Create the MCP service
        let service = server.serve(StdioTransport::new()).await?;
        
        // Keep the server running
        service.await?;
        
        Ok(())
    }

    fn convert_tool_to_rmcp(&self, tool: &Arc<dyn Tool>) -> RmcpTool {
        let schema = tool.schema();
        
        // Convert parameters to input schema
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        
        for param in &schema.parameters {
            let mut prop_schema = serde_json::Map::new();
            prop_schema.insert("type".to_string(), json!(param.param_type));
            prop_schema.insert("description".to_string(), json!(param.description));
            
            if let Some(default) = &param.default {
                prop_schema.insert("default".to_string(), default.clone());
            }
            
            properties.insert(param.name.clone(), Value::Object(prop_schema));
            
            if param.required {
                required.push(param.name.clone());
            }
        }
        
        let input_schema = ToolInputSchema {
            type_: "object".to_string(),
            properties: Some(properties),
            required: Some(required),
            additional_properties: None,
        };
        
        RmcpTool {
            name: schema.name.into(),
            description: Some(schema.description),
            input_schema,
        }
    }
}

impl Clone for MCPServer {
    fn clone(&self) -> Self {
        Self {
            tools: self.tools.clone(),
        }
    }
}

#[rmcp::async_trait]
impl rmcp::Service for MCPServer {
    async fn list_tools(&self, _params: ListToolsRequestParam) -> rmcp::Result<Vec<RmcpTool>> {
        let tools: Vec<RmcpTool> = self.tools
            .values()
            .map(|tool| self.convert_tool_to_rmcp(tool))
            .collect();
        
        Ok(tools)
    }

    async fn call_tool(&self, params: CallToolRequestParam) -> rmcp::Result<Vec<Content>> {
        let tool_name = params.name.to_string();
        
        if let Some(tool) = self.tools.get(&tool_name) {
            // Convert arguments from rmcp format to serde_json::Value
            let arguments = if let Some(args) = params.arguments {
                // Convert rmcp Object to serde_json::Value
                serde_json::to_value(args).map_err(|e| {
                    rmcp::Error::InvalidRequest(format!("Failed to convert arguments: {}", e))
                })?
            } else {
                json!({})
            };
            
            // Execute the tool
            match tool.execute(&arguments) {
                Ok(result) => {
                    let content = Content {
                        type_: "text".to_string(),
                        raw: RawContent::Text(TextContent {
                            text: result,
                        }),
                    };
                    Ok(vec![content])
                }
                Err(e) => Err(rmcp::Error::InternalError(format!("Tool execution failed: {}", e))),
            }
        } else {
            Err(rmcp::Error::InvalidRequest(format!("Unknown tool: {}", tool_name)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = MCPServer::new();
        assert!(server.tools.contains_key("transpile_vhdl_to_verilog"));
        assert!(server.tools.contains_key("edit_file"));
    }

    #[tokio::test]
    async fn test_list_tools() {
        let server = MCPServer::new();
        let tools = server.list_tools(ListToolsRequestParam {}).await.unwrap();
        
        assert!(tools.len() >= 2);
        
        let tool_names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
        assert!(tool_names.contains(&"transpile_vhdl_to_verilog".to_string()));
        assert!(tool_names.contains(&"edit_file".to_string()));
    }
}
