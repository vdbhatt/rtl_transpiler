pub mod basics;
pub mod client;
// pub mod openai;  // Commented out - has compilation errors
pub mod mock;
// pub mod infineon;  // Commented out for now

pub use basics::{LLMMessage, LLMResponse, LLMUsage};
pub use client::{LLMClient, create_llm_client};