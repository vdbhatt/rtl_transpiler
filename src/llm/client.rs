use anyhow::Result;
use std::sync::Arc;

use crate::config::ModelConfig;
use crate::llm::basics::{LLMMessage, LLMResponse};
use crate::tools::Tool;

pub trait LLMClient: Send + Sync {
    fn complete(
        &self,
        messages: &[LLMMessage],
        tools: Option<Vec<Arc<dyn Tool>>>,
    ) -> Result<LLMResponse>;

    fn get_model_name(&self) -> &str;
}

pub fn create_llm_client(config: &ModelConfig) -> Result<Arc<dyn LLMClient>> {
    let provider = config
        .model_provider
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Model provider not configured"))?;

    match provider.provider.to_lowercase().as_str() {
        "mock" => {
            // Use mock client for testing
            let client = crate::llm::mock::MockLLMClient::new(config.clone())?;
            Ok(Arc::new(client))
        }
        // "openai" => {
        //     let client = crate::llm::openai::OpenAIClient::new(config.clone())?;
        //     Ok(Arc::new(client))
        // }
        // "anthropic" => {
        //     let client = crate::llm::openai::OpenAIClient::new(config.clone())?; // Using OpenAI-compatible for now
        //     Ok(Arc::new(client))
        // }
        // "openrouter" => {
        //     let client = crate::llm::openai::OpenAIClient::new(config.clone())?;
        //     Ok(Arc::new(client))
        // }
        // "infineon" => {
        //     let client = crate::llm::infineon::InfineonClient::new(config.clone())?;
        //     Ok(Arc::new(client))
        // }
        _ => {
            // Default to mock for unsupported providers
            let client = crate::llm::mock::MockLLMClient::new(config.clone())?;
            Ok(Arc::new(client))
        }
    }
}