//! RTL Transpiler MCP Server Binary
//! 
//! This binary provides a standalone MCP server that exposes VHDL transpilation
//! and analysis tools via the Model Context Protocol using the rmcp crate.

use anyhow::Result;
use clap::Parser;
use rtl_transpiler::mcp::RTLTranspilerMCPServer;
use tracing_subscriber;
use rmcp::ServiceExt;

#[derive(Parser)]
#[command(name = "rtl-transpiler-mcp")]
#[command(about = "RTL Transpiler MCP Server - Exposes VHDL transpilation and analysis tools via Model Context Protocol")]
#[command(version)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Filter out empty arguments that some MCP clients may pass
    let args: Vec<String> = std::env::args()
        .filter(|arg| !arg.is_empty())
        .collect();
    
    let args = Args::parse_from(args);
    
    // Initialize logging
    let log_level = if args.debug {
        tracing::Level::DEBUG
    } else if args.verbose {
        tracing::Level::INFO
    } else {
        tracing::Level::WARN
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    tracing::info!("Starting RTL Transpiler MCP Server (rmcp)");
    
    // Create and run the MCP server - following the example_server.rs pattern
    let server = RTLTranspilerMCPServer::new();
    let service = server.serve(rmcp::transport::io::stdio()).await?;
    
    tracing::info!("MCP Server initialized with tools:");
    tracing::info!("  - transpile_vhdl_to_verilog: Convert VHDL entities to Verilog modules");
    tracing::info!("  - analyze_vhdl: Analyze VHDL files for entities, ports, signals, and processes");
    tracing::info!("  - edit_file: Edit text files with search/replace functionality");
    
    tracing::info!("Server ready, listening on stdio...");
    
    // Run the server (this will block until the server shuts down)
    service.waiting().await?;
    
    tracing::info!("MCP Server shutting down");
    Ok(())
}