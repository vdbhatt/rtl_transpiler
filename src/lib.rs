pub mod agent;
pub mod llm;
pub mod tools;
// pub mod mcp;  // Commented out - MCP not fully implemented
pub mod mcp_stub;  // MCP stub
pub use mcp_stub as mcp;  // Re-export as mcp
pub mod ir;
pub mod parser;
pub mod config;
pub mod constants;
pub mod utils;

// Re-export commonly used types
pub use agent::{Agent, AgentType, BaseAgent};
pub use ir::{Entity, Port, PortDirection, VHDLType};
pub use parser::VHDLParser;