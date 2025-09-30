use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, Tool as RmcpTool},
    transport::{ConfigureCommandExt, TokioChildProcess},
    object,
};
use tokio::process::Command;

use crate::config::MCPServerConfig;

#[derive(Debug, Clone)]
pub enum MCPServerStatus {
    Disconnected,
    Connecting,
    Connected,
}

// Store the actual client connection data
#[derive(Clone)]
pub struct MCPConnection {
    name: String,
    config: MCPServerConfig,
    status: MCPServerStatus,
}

impl MCPConnection {
    pub fn new(name: String, config: MCPServerConfig) -> Self {
        Self {
            name,
            config,
            status: MCPServerStatus::Disconnected,
        }
    }

    pub async fn list_tools(&self) -> Result<Vec<RmcpTool>> {
        if !matches!(self.status, MCPServerStatus::Connected) {
            return Err(anyhow::anyhow!("Client not connected"));
        }

        // Create a fresh connection to list tools
        tracing::debug!("Creating client connection to list tools for server: {}", self.name);

        if let Some(command) = &self.config.command {
            let mut cmd = Command::new(command);

            if let Some(args) = &self.config.args {
                cmd.args(args);
            }

            let client = ()
                .serve(TokioChildProcess::new(cmd.configure(|process_cmd| {
                    if let Some(env_vars) = &self.config.env {
                        for (key, value) in env_vars {
                            process_cmd.env(key, value);
                        }
                    }

                    if let Some(cwd) = &self.config.cwd {
                        process_cmd.current_dir(cwd);
                    }
                }))?)
                .await?;

            Ok(client.list_all_tools().await?)
        } else {
            Err(anyhow::anyhow!("No command specified for MCP server"))
        }
    }

    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<String> {
        if !matches!(self.status, MCPServerStatus::Connected) {
            return Err(anyhow::anyhow!("Client not connected"));
        }

        let arguments_obj = match arguments {
            serde_json::Value::Object(map) => {
                let mut obj = object!({});
                for (key, value) in map {
                    obj.insert(key, value);
                }
                Some(obj)
            }
            serde_json::Value::Null => None,
            _ => Some(object!({})),
        };

        if let Some(command) = &self.config.command {
            let mut cmd = Command::new(command);

            if let Some(args) = &self.config.args {
                cmd.args(args);
            }

            let client = ()
                .serve(TokioChildProcess::new(cmd.configure(|process_cmd| {
                    if let Some(env_vars) = &self.config.env {
                        for (key, value) in env_vars {
                            process_cmd.env(key, value);
                        }
                    }

                    if let Some(cwd) = &self.config.cwd {
                        process_cmd.current_dir(cwd);
                    }
                }))?)
                .await?;

            let tool_result = client.call_tool(CallToolRequestParam {
                name: name.to_string().into(),
                arguments: arguments_obj,
            }).await?;

            // Extract text content
            let mut result_text = String::new();
            for content in &tool_result.content {
                if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                    result_text.push_str(&text_content.text);
                    result_text.push('\n');
                }
            }

            Ok(result_text.trim().to_string())
        } else {
            Err(anyhow::anyhow!("No command specified for MCP server"))
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.status = MCPServerStatus::Connecting;
        tracing::info!("Connecting to MCP server: {}", self.name);

        // Test the connection by creating a client
        if let Some(command) = &self.config.command {
            let mut cmd = Command::new(command);

            if let Some(args) = &self.config.args {
                cmd.args(args);
            }

            let client = ()
                .serve(TokioChildProcess::new(cmd.configure(|process_cmd| {
                    if let Some(env_vars) = &self.config.env {
                        for (key, value) in env_vars {
                            process_cmd.env(key, value);
                        }
                    }

                    if let Some(cwd) = &self.config.cwd {
                        process_cmd.current_dir(cwd);
                    }
                }))?)
                .await?;

            let server_info = client.peer_info();
            tracing::info!("Connected to MCP server '{}': {:?}", self.name, server_info);

            self.status = MCPServerStatus::Connected;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No command specified for MCP server"))
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.status, MCPServerStatus::Connected)
    }
}