use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::agent::basics::{AgentError, AgentExecution, AgentState, AgentStep};
use crate::config::{AgentConfig, ModelConfig};
use crate::llm::{LLMClient, LLMMessage, LLMResponse};
use crate::tools::{Tool, ToolCall, ToolExecutor, ToolResult};
use crate::utils::{CLIConsole, TrajectoryRecorder};

pub trait BaseAgent: Send + Sync {
    fn get_name(&self) -> &str;
    fn get_max_steps(&self) -> u32;
    fn get_tools(&self) -> Vec<Arc<dyn Tool>>;
    fn get_tool_executor(&self) -> Arc<ToolExecutor>;
    fn get_llm_client(&self) -> Arc<dyn LLMClient>;
    fn get_trajectory_recorder(&self) -> Option<Arc<Mutex<TrajectoryRecorder>>>;
    fn get_cli_console(&self) -> Option<Arc<dyn CLIConsole>>;

    fn initialize(&mut self) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;

    fn prepare_system_message(&self, task: &str, task_args: &serde_json::Value) -> String;

    fn process_response(
        &self,
        response: &LLMResponse,
        execution: &mut AgentExecution,
    ) -> Result<Vec<ToolResult>>;

    fn run_step(
        &self,
        messages: &mut Vec<LLMMessage>,
        execution: &mut AgentExecution,
        cancel_flag: Arc<AtomicBool>,
        step_num: u32,
    ) -> Result<bool>;

    fn run(
        &self,
        task: String,
        task_args: serde_json::Value,
        cancel_flag: Arc<AtomicBool>,
    ) -> Result<String> {
        let mut execution = AgentExecution::new(task.clone());
        execution.start();

        // Record task start
        if let Some(recorder) = self.get_trajectory_recorder() {
            let mut recorder = recorder.lock().unwrap();
            recorder.record_task(&task)?;
        }

        // Prepare initial message
        let system_message = self.prepare_system_message(&task, &task_args);
        let mut messages = vec![LLMMessage::system(system_message)];
        messages.push(LLMMessage::user(task.clone()));

        // Main execution loop
        let max_steps = self.get_max_steps();
        for step_num in 0..max_steps {
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                execution.stop();
                return Err(AgentError::Cancelled.into());
            }

            let done = self.run_step(&mut messages, &mut execution, cancel_flag.clone(), step_num + 1)?;

            if done {
                break;
            }
        }

        // Check if we exceeded max steps
        if execution.step_count() >= max_steps as usize && execution.state != AgentState::Finished {
            execution.finish_with_error(format!("Maximum steps ({}) exceeded", max_steps));
            return Err(AgentError::MaxStepsExceeded(max_steps).into());
        }

        // Return result
        match execution.state {
            AgentState::Finished => Ok(execution.result.unwrap_or_default()),
            AgentState::Error => Err(AgentError::Other(execution.error.unwrap_or_default()).into()),
            AgentState::Stopped => Err(AgentError::Cancelled.into()),
            _ => Err(AgentError::Other("Unexpected agent state".to_string()).into()),
        }
    }
}

pub struct BaseAgentImpl {
    pub name: String,
    pub config: AgentConfig,
    pub llm_client: Arc<dyn LLMClient>,
    pub tools: Vec<Arc<dyn Tool>>,
    pub tool_executor: Arc<ToolExecutor>,
    pub trajectory_recorder: Option<Arc<Mutex<TrajectoryRecorder>>>,
    pub cli_console: Option<Arc<dyn CLIConsole>>,
}

impl BaseAgentImpl {
    pub fn new(
        name: String,
        config: AgentConfig,
        llm_client: Arc<dyn LLMClient>,
        trajectory_recorder: Option<Arc<Mutex<TrajectoryRecorder>>>,
        cli_console: Option<Arc<dyn CLIConsole>>,
    ) -> Result<Self> {
        let mut tools: Vec<Arc<dyn Tool>> = Vec::new();

        // Initialize tools based on config
        for tool_name in &config.tools {
            let tool = crate::tools::create_tool(
                tool_name,
                config.allowed_folders.clone(),
                config.model_config.as_ref().and_then(|m| m.model_provider.as_ref()),
            )?;
            tools.push(tool);
        }

        let tool_executor = Arc::new(ToolExecutor::new(tools.clone()));

        Ok(Self {
            name,
            config,
            llm_client,
            tools,
            tool_executor,
            trajectory_recorder,
            cli_console,
        })
    }

    pub fn close_tools(&mut self) -> Result<()> {
        for tool in &self.tools {
            tool.cleanup()?;
        }
        Ok(())
    }
}

impl BaseAgent for BaseAgentImpl {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_max_steps(&self) -> u32 {
        self.config.max_steps
    }

    fn get_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    fn get_tool_executor(&self) -> Arc<ToolExecutor> {
        self.tool_executor.clone()
    }

    fn get_llm_client(&self) -> Arc<dyn LLMClient> {
        self.llm_client.clone()
    }

    fn get_trajectory_recorder(&self) -> Option<Arc<Mutex<TrajectoryRecorder>>> {
        self.trajectory_recorder.clone()
    }

    fn get_cli_console(&self) -> Option<Arc<dyn CLIConsole>> {
        self.cli_console.clone()
    }

    fn initialize(&mut self) -> Result<()> {
        // Initialize tools
        for tool in &self.tools {
            tool.initialize()?;
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.close_tools()?;
        Ok(())
    }

    fn prepare_system_message(&self, task: &str, task_args: &serde_json::Value) -> String {
        // This should be overridden by specific agent implementations
        format!(
            "You are an AI agent tasked with: {}\n\nTask arguments: {}",
            task,
            serde_json::to_string_pretty(task_args).unwrap_or_default()
        )
    }

    fn process_response(
        &self,
        response: &LLMResponse,
        execution: &mut AgentExecution,
    ) -> Result<Vec<ToolResult>> {
        let mut results = Vec::new();

        if let Some(tool_calls) = &response.tool_calls {
            for tool_call in tool_calls {
                let result = self.tool_executor.execute(tool_call)?;
                results.push(result);
            }
        }

        Ok(results)
    }

    fn run_step(
        &self,
        messages: &mut Vec<LLMMessage>,
        execution: &mut AgentExecution,
        cancel_flag: Arc<AtomicBool>,
        step_num: u32,
    ) -> Result<bool> {
        // Print step header
        if let Some(console) = &self.cli_console {
            console.print_step(step_num, self.get_max_steps());
            console.print_thinking(step_num);
        }

        // Debug: Print the complete prompt being sent to LLM
        self.print_prompt_box(messages);

        let response = self.llm_client.complete(messages, Some(self.tools.clone()))?;

        // Record LLM response to trajectory
        if let Some(recorder) = &self.trajectory_recorder {
            let mut rec = recorder.lock().unwrap();
            if let Some(content) = &response.content {
                rec.record_thought(content).ok();
            }
        }

        // Print LLM response
        if let Some(console) = &self.cli_console {
            if let Some(content) = &response.content {
                if !content.is_empty() {
                    console.print_agent_message(content);
                }
            }
        }

        // Check if task is done via tool call
        if let Some(tool_calls) = &response.tool_calls {
            for tool_call in tool_calls {
                if tool_call.name == "task_done" {
                    if let Some(console) = &self.cli_console {
                        console.print_success("Task completed!");
                    }

                    // Record completion
                    if let Some(recorder) = &self.trajectory_recorder {
                        let mut rec = recorder.lock().unwrap();
                        rec.record_result(
                            &response.content.clone().unwrap_or("Task completed".to_string())
                        ).ok();
                    }

                    execution.finish_with_result(
                        response.content.clone().unwrap_or("Task completed".to_string())
                    );
                    return Ok(true);
                }
            }
        }

        // Process tool calls
        let tool_results = self.process_response(&response, execution)?;

        // Record and print tool usage
        if let Some(tool_calls) = &response.tool_calls {
            for tool_call in tool_calls {
                // Record to trajectory
                if let Some(recorder) = &self.trajectory_recorder {
                    let mut rec = recorder.lock().unwrap();
                    rec.record_action(&tool_call.name, &tool_call.arguments).ok();
                }

                // Print to console
                if let Some(console) = &self.cli_console {
                    let args_str = serde_json::to_string(&tool_call.arguments).unwrap_or_default();
                    console.print_tool_use(&tool_call.name, &args_str);
                }
            }
        }

        // Add assistant message
        messages.push(LLMMessage::assistant(
            response.content.clone().unwrap_or_default(),
            response.tool_calls.clone(),
        ));

        // Add tool results as messages
        for result in &tool_results {
            // Record to trajectory
            if let Some(recorder) = &self.trajectory_recorder {
                let mut rec = recorder.lock().unwrap();
                rec.record_observation(&result.content).ok();
            }

            // Print to console
            if let Some(console) = &self.cli_console {
                console.print_tool_result(&result.content);
            }

            messages.push(LLMMessage::tool_result(
                result.tool_call_id.clone(),
                result.content.clone(),
            ));
        }

        // Check for cancellation
        if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            execution.stop();
            return Ok(true);
        }

        Ok(false)
    }
}

impl BaseAgentImpl {
    /// Log prompt messages in a structured format
    fn print_prompt_box(&self, messages: &[crate::llm::LLMMessage]) {
        tracing::debug!("=== CONVERSATION PROMPT ({} messages) ===", messages.len());
        
        for (i, message) in messages.iter().enumerate() {
            let role = message.role();

            tracing::debug!("Message {}: role={}", i + 1, role);

            // Log content if available
            if let Some(content) = message.content() {
                if !content.is_empty() {
                    let preview = if content.len() > 200 {
                        format!("{}...", &content[..200])
                    } else {
                        content.to_string()
                    };
                    tracing::debug!("  Content: {}", preview);
                    tracing::trace!("  Full content: {}", content);
                }
            }

            // Handle different message types
            match message {
                crate::llm::LLMMessage::Assistant { tool_calls, .. } => {
                    if let Some(calls) = tool_calls {
                        tracing::debug!("  Tool calls: {} call(s)", calls.len());
                        for call in calls {
                            tracing::debug!("    - {}: {:?}", call.name, call.arguments);
                        }
                    }
                }
                crate::llm::LLMMessage::Tool { tool_call_id, .. } => {
                    tracing::debug!("  Tool call ID: {}", tool_call_id);
                }
                _ => {}
            }
        }
    }

    /// Wrap text to fit within specified width, respecting word boundaries
    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
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
                    let line_chars = &chars[current_pos..break_pos];
                    let line = line_chars.iter().collect::<String>().trim_end().to_string();
                    lines.push(line);
                    current_pos = break_pos + 1; // Skip the space/newline
                } else {
                    // No word boundary found, force break at width
                    let line_chars = &chars[current_pos..end_pos];
                    let line = line_chars.iter().collect::<String>();
                    lines.push(line);
                    current_pos = end_pos;
                }
            } else {
                // Last chunk, just take what's left
                let line_chars = &chars[current_pos..end_pos];
                let line = line_chars.iter().collect::<String>();
                lines.push(line);
                current_pos = end_pos;
            }
        }
        
        // Ensure all lines are within the width limit
        lines.into_iter().map(|line| {
            if line.chars().count() > width {
                line.chars().take(width).collect()
            } else {
                line
            }
        }).collect()
    }
}