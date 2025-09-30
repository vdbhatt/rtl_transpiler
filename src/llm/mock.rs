// Mock LLM client for testing
use anyhow::Result;
use std::sync::Arc;

use crate::config::ModelConfig;
use crate::llm::{LLMClient, LLMMessage, LLMResponse};
use crate::tools::Tool;

pub struct MockLLMClient {
    model_name: String,
}

impl MockLLMClient {
    pub fn new(config: ModelConfig) -> Result<Self> {
        Ok(Self {
            model_name: config.model_name,
        })
    }
}

impl LLMClient for MockLLMClient {
    fn complete(
        &self,
        _messages: &[LLMMessage],
        _tools: Option<Vec<Arc<dyn Tool>>>,
    ) -> Result<LLMResponse> {
        // Return a simple mock response
        Ok(LLMResponse {
            content: Some("Mock LLM response".to_string()),
            tool_calls: None,
            usage: None,
            model: Some("mock".to_string()),
            finish_reason: Some("stop".to_string()),
        })
    }

    fn get_model_name(&self) -> &str {
        &self.model_name
    }
}