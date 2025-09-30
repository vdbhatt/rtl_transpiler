use anyhow::Result;
use rmcp::model::Tool as RmcpTool;

use crate::config::MCPServerConfig;

pub mod working_client;

pub use working_client::{MCPConnection, MCPServerStatus};

pub struct MCPClient {
    connection: MCPConnection,
}

impl MCPClient {
    pub fn new(name: String) -> Self {
        // Create a placeholder config - this will be set during connect
        let config = MCPServerConfig {
            command: None,
            args: None,
            env: None,
            cwd: None,
            url: None,
            http_url: None,
        };

        Self {
            connection: MCPConnection::new(name, config),
        }
    }

    pub async fn connect(&mut self, config: &MCPServerConfig) -> Result<()> {
        // Update the connection with the actual config
        self.connection = MCPConnection::new(self.connection.name().to_string(), config.clone());
        self.connection.connect().await
    }

    pub async fn list_tools(&self) -> Result<Vec<RmcpTool>> {
        self.connection.list_tools().await
    }

    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<String> {
        self.connection.call_tool(name, arguments).await
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down MCP client: {}", self.connection.name());
        // No specific shutdown needed for our implementation
        Ok(())
    }

    pub fn name(&self) -> &str {
        self.connection.name()
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_connected()
    }
}

// Implement Clone manually
impl Clone for MCPClient {
    fn clone(&self) -> Self {
        // Clone the connection
        Self {
            connection: self.connection.clone(),
        }
    }
}