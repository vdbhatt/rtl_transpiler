// MCP Tool stub - not fully implemented yet
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::tools::base::{Tool, ToolSchema, ToolParameter};
use crate::mcp::MCPClient;

pub struct MCPTool {
    _client: Arc<Mutex<MCPClient>>,
    schema: ToolSchema,
}

impl MCPTool {
    pub fn new(_client: Arc<Mutex<MCPClient>>, tool_def: crate::mcp::ToolDefinition) -> Self {
        let schema = ToolSchema {
            name: tool_def.name,
            description: tool_def.description,
            parameters: vec![],
        };

        Self {
            _client,
            schema,
        }
    }
}

impl Tool for MCPTool {
    fn name(&self) -> &str {
        &self.schema.name
    }

    fn description(&self) -> &str {
        &self.schema.description
    }

    fn schema(&self) -> ToolSchema {
        self.schema.clone()
    }

    fn execute(&self, _arguments: &serde_json::Value) -> Result<String> {
        Ok("MCP tool not implemented yet".to_string())
    }
}