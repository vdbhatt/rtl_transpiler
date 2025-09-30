pub mod alan_agent;
pub mod base;
pub mod basics;
pub mod transpiler_agent;

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::config::AgentConfig;
use crate::utils::{CLIConsole, TrajectoryRecorder};

pub use base::{BaseAgent, BaseAgentImpl};
pub use basics::{AgentError, AgentExecution, AgentState, AgentStep, AgentStepState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentType {
    AlanAgent,
    TranspilerAgent,
}

impl AgentType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "alan_agent" => Ok(AgentType::AlanAgent),
            "transpiler_agent" => Ok(AgentType::TranspilerAgent),
            _ => Err(anyhow::anyhow!("Unknown agent type: {}", s)),
        }
    }
}

pub struct Agent {
    agent_type: AgentType,
    inner: Box<dyn BaseAgent>,
    alan_agent: Option<alan_agent::AlanAgent>,  // Keep a separate reference for async MCP operations
    trajectory_recorder: Option<Arc<Mutex<TrajectoryRecorder>>>,
}

impl Agent {
    pub fn new(
        agent_type: AgentType,
        config: AgentConfig,
        trajectory_file: Option<PathBuf>,
        cli_console: Box<dyn CLIConsole>,
    ) -> Result<Self> {
        let trajectory_recorder = if let Some(path) = trajectory_file {
            Some(Arc::new(Mutex::new(TrajectoryRecorder::new(Some(path))?)))
        } else {
            Some(Arc::new(Mutex::new(TrajectoryRecorder::new(None)?)))
        };

        let cli_console: Arc<dyn CLIConsole> = Arc::from(cli_console);

        let (inner, alan_agent): (Box<dyn BaseAgent>, Option<alan_agent::AlanAgent>) = match agent_type {
            AgentType::AlanAgent => {
                let agent = alan_agent::AlanAgent::new(
                    config,
                    trajectory_recorder.clone(),
                    Some(cli_console),
                )?;
                let agent_clone = agent.clone();
                (Box::new(agent) as Box<dyn BaseAgent>, Some(agent_clone))
            }
            AgentType::TranspilerAgent => {
                let agent = transpiler_agent::TranspilerAgent::new(
                    config,
                    trajectory_recorder.clone(),
                    Some(cli_console),
                )?;
                (Box::new(agent) as Box<dyn BaseAgent>, None)
            }
        };

        Ok(Self {
            agent_type,
            inner,
            alan_agent,
            trajectory_recorder,
        })
    }

    pub fn initialize_mcp(&mut self) -> Result<()> {
        // First initialize the base agent
        self.inner.initialize()?;

        // Then initialize MCP if it's an AlanAgent
        if let Some(ref mut alan_agent) = self.alan_agent {
            // Check if MCP servers are configured
            let has_mcp = alan_agent.allow_mcp_servers.len() > 0;
            if has_mcp {
                // Create a runtime for async MCP initialization
                let runtime = tokio::runtime::Runtime::new()?;
                runtime.block_on(alan_agent.initialize_mcp())?;

                // CRITICAL FIX: Replace the inner instance with the MCP-initialized one
                // This ensures that the agent used for execution has all the MCP tools
                self.inner = Box::new(alan_agent.clone());
            }
        }

        Ok(())
    }

    pub fn close_tools(&mut self) -> Result<()> {
        self.inner.shutdown()
    }

    pub fn cleanup_mcp_clients(&mut self) -> Result<()> {
        // TODO: Implement MCP client cleanup
        Ok(())
    }

    pub fn get_tool_names(&self) -> Vec<String> {
        self.inner
            .get_tools()
            .iter()
            .map(|t| t.name().to_string())
            .collect()
    }

    pub fn get_trajectory_path(&self) -> Option<PathBuf> {
        // Note: This is a simplified version
        // In a real implementation, we'd get the path from the recorder
        None
    }

    pub fn run(
        &mut self,
        task: String,
        task_args: serde_json::Value,
        cancel_flag: Arc<AtomicBool>,
    ) -> Result<String> {
        self.inner.run(task, task_args, cancel_flag)
    }
}