pub mod base;
pub mod bash;
pub mod edit;
pub mod sequential_thinking;
pub mod task_done;
pub mod transpile;
pub mod transpile_folder;
pub mod vhdl_analyze;

use std::sync::Arc;
use anyhow::Result;

use crate::config::ModelProvider;
use crate::constants;

pub use base::{Tool, ToolCall, ToolExecutor, ToolResult, ToolParameter, ToolSchema, BaseToolImpl};
pub use bash::BashTool;
pub use edit::TextEditorTool;
pub use sequential_thinking::SequentialThinkingTool;
pub use task_done::TaskDoneTool;
pub use transpile::TranspileTool;
pub use transpile_folder::TranspileFolderTool;
pub use vhdl_analyze::VHDLAnalyzeTool;

pub fn create_tool(
    tool_name: &str,
    allowed_folders: Vec<String>,
    model_provider: Option<&ModelProvider>,
) -> Result<Arc<dyn Tool>> {
    let provider_name = model_provider
        .map(|p| p.provider.as_str())
        .unwrap_or("unknown");

    match tool_name {
        constants::TOOL_BASH => {
            Ok(Arc::new(BashTool::new(provider_name.to_string(), allowed_folders)))
        }
        constants::TOOL_STR_REPLACE_EDIT => {
            Ok(Arc::new(TextEditorTool::new(provider_name.to_string(), allowed_folders)))
        }
        constants::TOOL_SEQUENTIAL_THINKING => {
            Ok(Arc::new(SequentialThinkingTool::new(provider_name.to_string())))
        }
        constants::TOOL_TASK_DONE => {
            Ok(Arc::new(TaskDoneTool::new()))
        }
        "transpile_vhdl_to_verilog" => {
            Ok(Arc::new(TranspileTool::new(allowed_folders)))
        }
        "transpile_vhdl_folder" => {
            Ok(Arc::new(TranspileFolderTool::new(allowed_folders)))
        }
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}