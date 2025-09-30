// MCP stub module - not fully implemented yet
use anyhow::Result;
use crate::config::MCPServerConfig;

pub struct MCPClient {
    name: String,
}

impl MCPClient {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn connect(&mut self, _config: &MCPServerConfig) -> Result<()> {
        Ok(())
    }

    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        Ok(vec![])
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct ToolDefinition {
    pub name: String,
    pub description: String,
}
