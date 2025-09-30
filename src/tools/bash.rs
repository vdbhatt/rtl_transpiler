use anyhow::Result;
use crate::tools::{BaseToolImpl, Tool, ToolParameter, ToolSchema};

pub struct BashTool {
    base: BaseToolImpl,
    _provider: String,
    _allowed_folders: Vec<String>,
}

impl BashTool {
    pub fn new(provider: String, allowed_folders: Vec<String>) -> Self {
        let parameters = vec![
            ToolParameter {
                name: "command".to_string(),
                param_type: "string".to_string(),
                description: "The bash command to execute".to_string(),
                required: true,
                default: None,
            },
        ];

        let base = BaseToolImpl::new(
            "bash".to_string(),
            "Execute a bash command".to_string(),
            parameters,
        );

        Self {
            base,
            _provider: provider,
            _allowed_folders: allowed_folders,
        }
    }
}

impl Tool for BashTool {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn schema(&self) -> ToolSchema {
        self.base.schema.clone()
    }

    fn execute(&self, _arguments: &serde_json::Value) -> Result<String> {
        // Stub implementation
        Ok("Bash tool not implemented yet".to_string())
    }
}