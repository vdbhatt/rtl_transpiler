use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_steps: u32,
    pub tools: Vec<String>,
    pub allowed_folders: Vec<String>,
    pub model_config: Option<ModelConfig>,
    pub allow_mcp_servers: Vec<String>,
    pub mcp_servers_config: Option<HashMap<String, MCPServerConfig>>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 50,
            tools: vec![
                "transpile_vhdl_to_verilog".to_string(),
                "task_done".to_string(),
            ],
            allowed_folders: vec![],
            model_config: None,
            allow_mcp_servers: vec![],
            mcp_servers_config: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_provider: Option<ModelProvider>,
    pub model_name: String,
    pub model: String,  // Alias for model_name
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub max_retries: u32,
}

impl ModelConfig {
    pub fn should_use_max_completion_tokens(&self) -> bool {
        true  // Simplified
    }

    pub fn get_max_tokens_param(&self) -> u32 {
        self.max_tokens.unwrap_or(4096)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}