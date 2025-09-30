use anyhow::Result;
use std::path::PathBuf;

/// CLI console trait for output
pub trait CLIConsole: Send + Sync {
    fn print_step(&self, step: u32, max_steps: u32);
    fn print_thinking(&self, step: u32);
    fn print_agent_message(&self, message: &str);
    fn print_tool_use(&self, tool_name: &str, args: &str);
    fn print_tool_result(&self, result: &str);
    fn print_success(&self, message: &str);
    fn print_error(&self, message: &str);
    fn print_info(&self, message: &str);
}

/// Simple console implementation
pub struct SimpleConsole;

impl CLIConsole for SimpleConsole {
    fn print_step(&self, step: u32, max_steps: u32) {
        println!("\n=== Step {}/{} ===", step, max_steps);
    }

    fn print_thinking(&self, _step: u32) {
        println!("Thinking...");
    }

    fn print_agent_message(&self, message: &str) {
        println!("Agent: {}", message);
    }

    fn print_tool_use(&self, tool_name: &str, args: &str) {
        println!("Tool: {} ({})", tool_name, args);
    }

    fn print_tool_result(&self, result: &str) {
        println!("Result: {}", result);
    }

    fn print_success(&self, message: &str) {
        println!("✓ {}", message);
    }

    fn print_error(&self, message: &str) {
        eprintln!("✗ {}", message);
    }

    fn print_info(&self, message: &str) {
        println!("ℹ {}", message);
    }
}

/// Trajectory recorder for agent actions
pub struct TrajectoryRecorder {
    _output_path: Option<PathBuf>,
}

impl TrajectoryRecorder {
    pub fn new(output_path: Option<PathBuf>) -> Result<Self> {
        Ok(Self {
            _output_path: output_path,
        })
    }

    pub fn record_task(&mut self, _task: &str) -> Result<()> {
        Ok(())
    }

    pub fn record_thought(&mut self, _thought: &str) -> Result<()> {
        Ok(())
    }

    pub fn record_action(&mut self, _action: &str, _args: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    pub fn record_observation(&mut self, _observation: &str) -> Result<()> {
        Ok(())
    }

    pub fn record_result(&mut self, _result: &str) -> Result<()> {
        Ok(())
    }
}