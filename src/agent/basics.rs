use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    Init,
    Running,
    Finished,
    Error,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStepState {
    Thinking,
    Acting,
    Observing,
    Finished,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub id: String,
    pub state: AgentStepState,
    pub thoughts: Option<String>,
    pub action: Option<String>,
    pub observation: Option<String>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AgentStep {
    pub fn new(id: String) -> Self {
        Self {
            id,
            state: AgentStepState::Thinking,
            thoughts: None,
            action: None,
            observation: None,
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_thought(mut self, thought: String) -> Self {
        self.thoughts = Some(thought);
        self.state = AgentStepState::Acting;
        self
    }

    pub fn with_action(mut self, action: String) -> Self {
        self.action = Some(action);
        self.state = AgentStepState::Observing;
        self
    }

    pub fn with_observation(mut self, observation: String) -> Self {
        self.observation = Some(observation);
        self.state = AgentStepState::Finished;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self.state = AgentStepState::Error;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecution {
    pub id: String,
    pub state: AgentState,
    pub task: String,
    pub steps: Vec<AgentStep>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl AgentExecution {
    pub fn new(task: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state: AgentState::Init,
            task,
            steps: Vec::new(),
            result: None,
            error: None,
            started_at: chrono::Utc::now(),
            finished_at: None,
        }
    }

    pub fn start(&mut self) {
        self.state = AgentState::Running;
    }

    pub fn add_step(&mut self, step: AgentStep) {
        self.steps.push(step);
    }

    pub fn finish_with_result(&mut self, result: String) {
        self.state = AgentState::Finished;
        self.result = Some(result);
        self.finished_at = Some(chrono::Utc::now());
    }

    pub fn finish_with_error(&mut self, error: String) {
        self.state = AgentState::Error;
        self.error = Some(error);
        self.finished_at = Some(chrono::Utc::now());
    }

    pub fn stop(&mut self) {
        self.state = AgentState::Stopped;
        self.finished_at = Some(chrono::Utc::now());
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("LLM error: {0}")]
    LLM(String),

    #[error("Maximum steps ({0}) exceeded")]
    MaxStepsExceeded(u32),

    #[error("Task cancelled by user")]
    Cancelled,

    #[error("Agent error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for AgentError {
    fn from(err: anyhow::Error) -> Self {
        AgentError::Other(err.to_string())
    }
}