use anyhow::{Context, Result};
use crossterm::style::{Color, Stylize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

use crate::config::ModelConfig;
use crate::llm::basics::{LLMMessage, LLMResponse, LLMUsage};
use crate::llm::client::LLMClient;
use crate::tools::{Tool, ToolCall};

#[derive(Debug, Clone, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
    temperature: f32,
    top_p: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    #[serde(default)]
    usage: Option<OpenAIUsage>,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    #[serde(default)]
    prompt_tokens: i32,
    #[serde(default)]
    completion_tokens: i32,
    #[serde(default)]
    total_tokens: i32,
}

pub struct OpenAIClient {
    config: ModelConfig,
    client: reqwest::blocking::Client,
    base_url: String,
}

impl OpenAIClient {
    pub fn new(config: ModelConfig) -> Result<Self> {
        let provider = config
            .model_provider
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Model provider not configured"))?;

        let base_url = provider.base_url.clone().unwrap_or_else(|| {
            match provider.provider.to_lowercase().as_str() {
                "openai" => "https://api.openai.com/v1".to_string(),
                "anthropic" => "https://api.anthropic.com/v1".to_string(),
                "openrouter" => "https://openrouter.ai/api/v1".to_string(),
                _ => "https://api.openai.com/v1".to_string(),
            }
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let auth_header = if provider.provider.to_lowercase() == "anthropic" {
            format!("x-api-key {}", provider.api_key)
        } else {
            format!("Bearer {}", provider.api_key)
        };
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_header)?);

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(300))
            .build()?;

        Ok(Self {
            config,
            client,
            base_url,
        })
    }

    fn convert_messages(&self, messages: &[LLMMessage]) -> Vec<OpenAIMessage> {
        messages
            .iter()
            .map(|msg| match msg {
                LLMMessage::System { content } => OpenAIMessage {
                    role: "system".to_string(),
                    content: Some(content.clone()),
                    tool_calls: None,
                    tool_call_id: None,
                },
                LLMMessage::User { content } => OpenAIMessage {
                    role: "user".to_string(),
                    content: Some(content.clone()),
                    tool_calls: None,
                    tool_call_id: None,
                },
                LLMMessage::Assistant { content, tool_calls } => OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(content.clone()),
                    tool_calls: tool_calls.as_ref().map(|calls| {
                        calls
                            .iter()
                            .map(|call| OpenAIToolCall {
                                id: call.id.clone(),
                                call_type: "function".to_string(),
                                function: OpenAIFunction {
                                    name: call.name.clone(),
                                    arguments: call.arguments.to_string(),
                                },
                            })
                            .collect()
                    }),
                    tool_call_id: None,
                },
                LLMMessage::Tool {
                    tool_call_id,
                    content,
                } => OpenAIMessage {
                    role: "tool".to_string(),
                    content: Some(content.clone()),
                    tool_calls: None,
                    tool_call_id: Some(tool_call_id.clone()),
                },
            })
            .collect()
    }

    fn make_request(&self, request: OpenAIRequest) -> Result<OpenAIResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        // Log outgoing request
        tracing::info!("Sending request to OpenAI API: {}", url);
        tracing::debug!("Request payload: {}", serde_json::to_string(&request).unwrap_or_default());
        tracing::trace!("Full request: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .context("Failed to send request to OpenAI API")?;

        let status = response.status();

        // Get raw response text for debugging
        let response_text = response.text().context("Failed to read response body")?;

        // Parse the response text back to JSON
        let parsed_response: serde_json::Value = serde_json::from_str(&response_text)
            .context("Failed to parse response as JSON")?;

        // Log incoming response
        tracing::info!("Received response from OpenAI API: status={}", status);
        tracing::debug!("Response: {}", &response_text);
        tracing::trace!("Full response: {}", serde_json::to_string_pretty(&parsed_response).unwrap_or_default());

        // Check if it's an error response
        if let Some(error) = parsed_response.get("error") {
            let error_text = serde_json::to_string(error).unwrap_or_default();
            tracing::error!("API error: {}", error_text);
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        // Parse the JSON directly into OpenAIResponse
        let openai_response = serde_json::from_value::<OpenAIResponse>(parsed_response)
            .context("Failed to parse OpenAI response")?;

        Ok(openai_response)
    }

    /// Log formatted JSON (removed visual formatting as it's now logged to file)
    fn print_json_box(&self, json_str: &str, tag: &str) {
        // This function is kept for compatibility but doesn't print anything
        // Logging is handled in make_request
        tracing::trace!("{} JSON: {}", tag, json_str);
    }

    /// Flatten JSON into hierarchical table rows
    fn flatten_json(&self, value: &serde_json::Value, path: &mut Vec<String>, rows: &mut Vec<Vec<String>>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    path.push(key.clone());
                    self.flatten_json(val, path, rows);
                    path.pop();
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, item) in arr.iter().enumerate() {
                    path.push(format!("[{}]", i));
                    self.flatten_json(item, path, rows);
                    path.pop();
                }
            }
            _ => {
                // This is a leaf value, create a row
                let mut row = vec!["".to_string(); 4]; // 4 columns max
                for (i, path_part) in path.iter().enumerate() {
                    if i < 4 {
                        row[i] = path_part.clone();
                    }
                }
                
                // Put the value in the last column (no truncation since we wrap)
                let value_str = match value {
                    serde_json::Value::String(s) => s.replace('\n', " ").replace('\r', " "),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => "null".to_string(),
                    _ => value.to_string().replace('\n', " ").replace('\r', " "),
                };
                
                if row.len() > 3 {
                    row[3] = value_str;
                } else {
                    row.push(value_str);
                }
                
                rows.push(row);
            }
        }
    }

    /// Wrap text to fit within specified width, respecting word boundaries
    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        // Convert to character vector for safe indexing
        let chars: Vec<char> = text.chars().collect();
        
        if chars.len() <= width {
            return vec![text.to_string()];
        }
        
        let mut lines = Vec::new();
        let mut current_pos = 0;
        
        while current_pos < chars.len() {
            let end_pos = (current_pos + width).min(chars.len());
            
            // If we're not at the end of the text, try to break at a word boundary
            if end_pos < chars.len() {
                // Look backwards from end_pos to find a space or newline
                let mut break_pos = end_pos;
                for i in (current_pos..end_pos).rev() {
                    if chars[i] == ' ' || chars[i] == '\n' {
                        break_pos = i;
                        break;
                    }
                }
                
                // If we found a word boundary, use it; otherwise use the original end_pos
                if break_pos > current_pos {
                    let line: String = chars[current_pos..break_pos].iter().collect();
                    lines.push(line.trim_end().to_string());
                    current_pos = break_pos + 1; // Skip the space/newline
                } else {
                    // No word boundary found, force break at width
                    let line: String = chars[current_pos..end_pos].iter().collect();
                    lines.push(line);
                    current_pos = end_pos;
                }
            } else {
                // Last chunk, just take what's left
                let line: String = chars[current_pos..end_pos].iter().collect();
                lines.push(line);
                current_pos = end_pos;
            }
        }
        
        // Ensure all lines are within the width limit
        lines.into_iter().map(|line| {
            if line.len() > width {
                line.chars().take(width).collect()
            } else {
                line
            }
        }).collect()
    }
}

impl LLMClient for OpenAIClient {
    fn complete(
        &self,
        messages: &[LLMMessage],
        tools: Option<Vec<Arc<dyn Tool>>>,
    ) -> Result<LLMResponse> {
        let openai_messages = self.convert_messages(messages);

        let tools_json: Option<Vec<serde_json::Value>> = tools.as_ref().map(|tool_list| {
            tool_list
                .iter()
                .map(|tool| {
                    serde_json::json!({
                        "type": "function",
                        "function": tool.to_openai_function()
                    })
                })
                .collect()
        });

        // Pretty print tools info if available
        if let Some(ref tool_list) = tools {
            if !tool_list.is_empty() {
                tracing::info!("Available tools: {} total", tool_list.len());
                for (i, tool_json) in tools_json.as_ref().unwrap().iter().enumerate() {
                    if let Some(name) = tool_json.get("function").and_then(|f| f.get("name")).and_then(|n| n.as_str()) {
                        tracing::debug!("  {}. {}", i + 1, name);
                    }
                }
            }
        }

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: openai_messages,
            tools: tools_json,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            max_tokens: if self.config.should_use_max_completion_tokens() {
                None
            } else {
                Some(self.config.get_max_tokens_param())
            },
            max_completion_tokens: if self.config.should_use_max_completion_tokens() {
                Some(self.config.get_max_tokens_param())
            } else {
                None
            },
            stop: self.config.stop_sequences.clone(),
        };

        let mut last_error = None;
        for attempt in 0..self.config.max_retries {
            if attempt > 0 {
                std::thread::sleep(Duration::from_secs(2u64.pow(attempt as u32)));
            }

            match self.make_request(request.clone()) {
                Ok(response) => {
                    let choice = response
                        .choices
                        .first()
                        .ok_or_else(|| anyhow::anyhow!("No choices in response"))?;

                    let tool_calls = choice.message.tool_calls.as_ref().map(|calls| {
                        calls
                            .iter()
                            .map(|call| {
                                ToolCall::with_id(
                                    call.id.clone(),
                                    call.function.name.clone(),
                                    serde_json::from_str(&call.function.arguments)
                                        .unwrap_or(json!({})),
                                )
                            })
                            .collect()
                    });

                    let usage = response.usage.map(|u| LLMUsage {
                        input_tokens: u.prompt_tokens,
                        output_tokens: u.completion_tokens,
                        ..Default::default()
                    });

                    return Ok(LLMResponse {
                        content: choice.message.content.clone(),
                        usage,
                        model: response.model,
                        finish_reason: choice.finish_reason.clone(),
                        tool_calls,
                    });
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Max retries exceeded")))
    }

    fn get_model_name(&self) -> &str {
        &self.config.model
    }
}