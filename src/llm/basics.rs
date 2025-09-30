use serde::{Deserialize, Serialize};
use std::ops::Add;

use crate::tools::ToolCall;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum LLMMessage {
    #[serde(rename = "system")]
    System { content: String },

    #[serde(rename = "user")]
    User { content: String },

    #[serde(rename = "assistant")]
    Assistant {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
    },

    #[serde(rename = "tool")]
    Tool {
        tool_call_id: String,
        content: String,
    },
}

impl LLMMessage {
    pub fn system(content: String) -> Self {
        LLMMessage::System { content }
    }

    pub fn user(content: String) -> Self {
        LLMMessage::User { content }
    }

    pub fn assistant(content: String, tool_calls: Option<Vec<ToolCall>>) -> Self {
        LLMMessage::Assistant { content, tool_calls }
    }

    pub fn tool_result(tool_call_id: String, content: String) -> Self {
        LLMMessage::Tool {
            tool_call_id,
            content,
        }
    }

    pub fn role(&self) -> &str {
        match self {
            LLMMessage::System { .. } => "system",
            LLMMessage::User { .. } => "user",
            LLMMessage::Assistant { .. } => "assistant",
            LLMMessage::Tool { .. } => "tool",
        }
    }

    pub fn content(&self) -> Option<&str> {
        match self {
            LLMMessage::System { content } => Some(content),
            LLMMessage::User { content } => Some(content),
            LLMMessage::Assistant { content, .. } => Some(content),
            LLMMessage::Tool { content, .. } => Some(content),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LLMUsage {
    pub input_tokens: i32,
    pub output_tokens: i32,
    #[serde(default)]
    pub cache_creation_input_tokens: i32,
    #[serde(default)]
    pub cache_read_input_tokens: i32,
    #[serde(default)]
    pub reasoning_tokens: i32,
}

impl Add for LLMUsage {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        LLMUsage {
            input_tokens: self.input_tokens + other.input_tokens,
            output_tokens: self.output_tokens + other.output_tokens,
            cache_creation_input_tokens: self.cache_creation_input_tokens
                + other.cache_creation_input_tokens,
            cache_read_input_tokens: self.cache_read_input_tokens + other.cache_read_input_tokens,
            reasoning_tokens: self.reasoning_tokens + other.reasoning_tokens,
        }
    }
}

impl std::fmt::Display for LLMUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LLMUsage(input_tokens={}, output_tokens={}, cache_creation_input_tokens={}, cache_read_input_tokens={}, reasoning_tokens={})",
            self.input_tokens,
            self.output_tokens,
            self.cache_creation_input_tokens,
            self.cache_read_input_tokens,
            self.reasoning_tokens
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: Option<String>,
    #[serde(default)]
    pub usage: Option<LLMUsage>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl LLMResponse {
    pub fn new(content: String) -> Self {
        Self {
            content: Some(content),
            usage: None,
            model: None,
            finish_reason: None,
            tool_calls: None,
        }
    }

    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }

    pub fn with_usage(mut self, usage: LLMUsage) -> Self {
        self.usage = Some(usage);
        self
    }
}