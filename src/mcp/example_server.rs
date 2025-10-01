use rmcp::{
    model::{CallToolResult, Content, ErrorData as McpError, ServerCapabilities, ServerInfo, ToolsCapability},
    tool, tool_handler, tool_router, ServerHandler,
    handler::server::tool::Parameters,
    transport::io::stdio,
    ServiceExt,
};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct SumRequest {
    a: i32,
    b: i32,
}

#[derive(Clone)]
pub struct HelloWorldServer {
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

#[tool_router]
impl HelloWorldServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Returns a simple hello world message")]
    async fn hello_world(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            "Hello, World from RUST MCP Server!".to_string(),
        )]))
    }
    
    #[tool(description = "Returns sum of two numbers")]
    async fn sum(&self, params: Parameters<SumRequest>) -> Result<CallToolResult, McpError> {
        let SumRequest { a, b } = params.0;
        Ok(CallToolResult::success(vec![Content::text(
            (a + b).to_string(),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for HelloWorldServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple hello world MCP server".to_string()),
            capabilities: ServerCapabilities { tools: Some(ToolsCapability { list_changed: Some(false) }), ..Default::default() },
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = HelloWorldServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}