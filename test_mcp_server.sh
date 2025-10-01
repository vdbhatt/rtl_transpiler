#!/bin/bash
# Simple test script to verify MCP server can start

echo "Building MCP server..."
cargo build --release --bin rtl-transpiler-mcp

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo ""
echo "MCP Server binary location:"
ls -lh target/release/rtl-transpiler-mcp
echo ""

echo "To run the MCP server:"
echo "  ./target/release/rtl-transpiler-mcp"
echo ""
echo "To test with verbose logging:"
echo "  ./target/release/rtl-transpiler-mcp --verbose"
echo ""
echo "To add to Claude Desktop config (~/.claude/claude_desktop_config.json):"
echo '{'
echo '  "mcpServers": {'
echo '    "rtl-transpiler": {'
echo "      \"command\": \"$(pwd)/target/release/rtl-transpiler-mcp\","
echo '      "args": []'
echo '    }'
echo '  }'
echo '}'
