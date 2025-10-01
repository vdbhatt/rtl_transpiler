//! RTL Transpiler MCP Server
//! 
//! This module provides an MCP (Model Context Protocol) server implementation
//! using the rmcp crate. It exposes VHDL transpilation and analysis tools
//! to AI agents and other MCP clients.

use rmcp::{
    model::{CallToolResult, Content, ErrorData as McpError, ServerCapabilities, ServerInfo, ToolsCapability},
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use schemars::JsonSchema;
use std::sync::Arc;
use std::future::Future;
use crate::tools::{TranspileTool, TranspileFolderTool, TextEditorTool, VHDLAnalyzeTool};
use crate::tools::base::Tool;

/// Request parameters for VHDL to Verilog transpilation
#[derive(Deserialize, JsonSchema)]
struct TranspileRequest {
    /// Path to the VHDL file to transpile
    vhdl_file: String,
    /// Optional output file path (if not provided, outputs to stdout)
    output_file: Option<String>,
}

/// Request parameters for batch VHDL folder transpilation
#[derive(Deserialize, JsonSchema)]
struct TranspileFolderRequest {
    /// Path to the folder containing VHDL files
    vhdl_folder: String,
    /// Optional output folder path (if not provided, uses same folder)
    output_folder: Option<String>,
    /// Whether to recursively process subdirectories
    recursive: Option<bool>,
}

/// Request parameters for VHDL analysis
#[derive(Deserialize, JsonSchema)]
struct AnalyzeRequest {
    /// Path to the VHDL file to analyze
    vhdl_file: String,
    /// Type of analysis to perform (defaults to "all")
    analysis_type: Option<String>,
}

/// Request parameters for file editing operations
#[derive(Deserialize, JsonSchema)]
struct EditRequest {
    /// Command to execute: "view", "create", "str_replace", or "insert"
    command: String,
    /// Path to the file to operate on
    path: String,
    /// Old string for replacement operations
    old_str: Option<String>,
    /// New string for replacement operations
    new_str: Option<String>,
    /// File content for create operations
    file_text: Option<String>,
    /// Line number for insert operations
    insert_line: Option<i32>,
    /// Range for view operations [start_line, end_line]
    view_range: Option<Vec<i32>>,
}

/// RTL Transpiler MCP Server
///
/// This server exposes VHDL transpilation and analysis tools via the Model Context Protocol.
/// It provides four main tools:
/// - VHDL to Verilog transpilation (single file)
/// - VHDL to Verilog batch transpilation (folder)
/// - VHDL file analysis
/// - Text file editing operations
#[derive(Clone)]
pub struct RTLTranspilerMCPServer {
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
    transpile_tool: Arc<TranspileTool>,
    transpile_folder_tool: Arc<TranspileFolderTool>,
    text_editor_tool: Arc<TextEditorTool>,
    vhdl_analyze_tool: Arc<VHDLAnalyzeTool>,
}

#[tool_router]
impl RTLTranspilerMCPServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            transpile_tool: Arc::new(TranspileTool::new(vec![])),
            transpile_folder_tool: Arc::new(TranspileFolderTool::new(vec![])),
            text_editor_tool: Arc::new(TextEditorTool::new("mcp".to_string(), vec![])),
            vhdl_analyze_tool: Arc::new(VHDLAnalyzeTool::new(vec![])),
        }
    }

    /// Transpile VHDL entity to Verilog module
    ///
    /// Extracts entity declaration from VHDL file and converts it to a Verilog module
    /// with matching ports, types, and generics. Uses AST-based parsing for robust analysis.
    #[tool(description = "Transpile VHDL entity to Verilog module. Extracts entity declaration and converts it to a Verilog module with matching ports.")]
    async fn transpile_vhdl_to_verilog(&self, params: rmcp::handler::server::tool::Parameters<TranspileRequest>) -> Result<CallToolResult, McpError> {
        let TranspileRequest { vhdl_file, output_file } = params.0;

        match self.transpile_tool.execute(&serde_json::json!({
            "vhdl_file": vhdl_file,
            "output_file": output_file
        })) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!("Error: {}", e))])),
        }
    }

    /// Batch transpile all VHDL files in a folder to Verilog modules
    ///
    /// Processes all .vhd and .vhdl files in the specified directory. Optionally processes
    /// subdirectories recursively. Each VHDL file is parsed and converted to a Verilog module
    /// with matching ports, signals, processes, and architecture implementation.
    #[tool(description = "Batch transpile all VHDL files in a folder to Verilog modules. Processes all .vhd and .vhdl files, converting entities and architectures.")]
    async fn transpile_vhdl_folder(&self, params: rmcp::handler::server::tool::Parameters<TranspileFolderRequest>) -> Result<CallToolResult, McpError> {
        let TranspileFolderRequest { vhdl_folder, output_folder, recursive } = params.0;

        match self.transpile_folder_tool.execute(&serde_json::json!({
            "vhdl_folder": vhdl_folder,
            "output_folder": output_folder,
            "recursive": recursive.unwrap_or(false)
        })) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!("Error: {}", e))])),
        }
    }

    /// Analyze VHDL files for structural information
    /// 
    /// Extracts entities, ports, signals, processes, and other structural information
    /// from VHDL files. Provides detailed analysis of the design hierarchy.
    #[tool(description = "Analyze VHDL files to extract entities, ports, signals, processes, and other structural information.")]
    async fn analyze_vhdl(&self, params: rmcp::handler::server::tool::Parameters<AnalyzeRequest>) -> Result<CallToolResult, McpError> {
        let AnalyzeRequest { vhdl_file, analysis_type } = params.0;
        
        match self.vhdl_analyze_tool.execute(&serde_json::json!({
            "vhdl_file": vhdl_file,
            "analysis_type": analysis_type.unwrap_or("all".to_string())
        })) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!("Error: {}", e))])),
        }
    }

    /// Edit text files with various operations
    /// 
    /// Supports multiple file operations including view, create, search/replace,
    /// and insert operations. Provides a comprehensive file editing interface.
    #[tool(description = "Custom editing tool for viewing, creating and editing files\n* State is persistent across command calls\n* The create command cannot be used if the path already exists\n* For str_replace: old_str must match EXACTLY and be unique in the file")]
    async fn str_replace_based_edit_tool(&self, params: rmcp::handler::server::tool::Parameters<EditRequest>) -> Result<CallToolResult, McpError> {
        let EditRequest { command, path, old_str, new_str, file_text, insert_line, view_range } = params.0;
        
        let mut args = serde_json::json!({
            "command": command,
            "path": path
        });

        if let Some(old_str) = old_str {
            args["old_str"] = serde_json::Value::String(old_str);
        }
        if let Some(new_str) = new_str {
            args["new_str"] = serde_json::Value::String(new_str);
        }
        if let Some(file_text) = file_text {
            args["file_text"] = serde_json::Value::String(file_text);
        }
        if let Some(insert_line) = insert_line {
            args["insert_line"] = serde_json::Value::Number(serde_json::Number::from(insert_line));
        }
        if let Some(view_range) = view_range {
            args["view_range"] = serde_json::Value::Array(
                view_range.into_iter().map(|i| serde_json::Value::Number(serde_json::Number::from(i))).collect()
            );
        }

        match self.text_editor_tool.execute(&args) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!("Error: {}", e))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for RTLTranspilerMCPServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("RTL Transpiler MCP Server - Exposes VHDL transpilation and analysis tools".to_string()),
            capabilities: ServerCapabilities { 
                tools: Some(ToolsCapability { list_changed: Some(false) }), 
                ..Default::default() 
            },
            ..Default::default()
        }
    }
}
