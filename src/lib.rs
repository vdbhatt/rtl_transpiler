pub mod agent;
pub mod llm;
pub mod tools;
pub mod mcp;  // MCP implementation
pub mod ir;
pub mod parser;
pub mod config;
pub mod constants;
pub mod utils;

// Re-export commonly used types
pub use agent::{Agent, AgentType, BaseAgent};
pub use ir::{Entity, Port, PortDirection, VHDLType};
pub use parser::ASTVHDLParser;