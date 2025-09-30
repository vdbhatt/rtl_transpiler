use anyhow::Result;
use crate::tools::{BaseToolImpl, Tool, ToolParameter, ToolSchema};

pub struct TaskDoneTool {
    base: BaseToolImpl,
}

impl TaskDoneTool {
    pub fn new() -> Self {
        let parameters = vec![
            ToolParameter {
                name: "result".to_string(),
                param_type: "string".to_string(),
                description: "The final result or summary of the task".to_string(),
                required: false,
                default: None,
            },
        ];

        let base = BaseToolImpl::new(
            "task_done".to_string(),
            "Mark the task as completed".to_string(),
            parameters,
        );

        Self { base }
    }
}

impl Default for TaskDoneTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for TaskDoneTool {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn schema(&self) -> ToolSchema {
        self.base.schema.clone()
    }

    fn execute(&self, arguments: &serde_json::Value) -> Result<String> {
        let result = arguments
            .get("result")
            .and_then(|v| v.as_str())
            .unwrap_or("Task completed");

        Ok(result.to_string())
    }
}