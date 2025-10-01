# RTL Transpiler MCP Server Usage

This document describes how to use the RTL Transpiler MCP (Model Context Protocol) server for VHDL to Verilog transpilation.

## Overview

The RTL Transpiler MCP Server exposes VHDL transpilation and analysis tools via the Model Context Protocol. It provides tools for:

1. **Single File Transpilation**: Convert a single VHDL file to Verilog
2. **Batch Folder Transpilation**: Convert all VHDL files in a folder to Verilog
3. **VHDL Analysis**: Extract structural information from VHDL files
4. **Text Editing**: View and edit files

## Building the Server

```bash
cargo build --release --bin rtl-transpiler-mcp
```

The binary will be located at `target/release/rtl-transpiler-mcp`.

## Available Tools

### 1. `transpile_vhdl_to_verilog`

Transpile a single VHDL entity to a Verilog module. Extracts both entity declaration and architecture implementation, converting ports, signals, processes, and concurrent statements.

**Parameters:**
- `vhdl_file` (string, required): Path to the VHDL file to transpile
- `output_file` (string, optional): Path to the output Verilog file. If not provided, returns the generated Verilog as text.

**Example:**
```json
{
  "vhdl_file": "/path/to/counter.vhd",
  "output_file": "/path/to/counter.v"
}
```

### 2. `transpile_vhdl_folder`

Batch transpile all VHDL files in a folder to Verilog modules. Processes all `.vhd` and `.vhdl` files in the specified directory.

**Parameters:**
- `vhdl_folder` (string, required): Path to the folder containing VHDL files
- `output_folder` (string, optional): Path to the output folder. Defaults to the same folder as input.
- `recursive` (boolean, optional): Whether to recursively process subdirectories. Default: false.

**Example:**
```json
{
  "vhdl_folder": "/path/to/vhdl_files",
  "output_folder": "/path/to/verilog_output",
  "recursive": false
}
```

**Output:**
Returns a detailed report showing:
- Number of files found and processed
- List of successful transpilations with input/output paths
- Any errors encountered
- Summary statistics

### 3. `analyze_vhdl`

Analyze VHDL files to extract entities, ports, signals, processes, and other structural information.

**Parameters:**
- `vhdl_file` (string, required): Path to the VHDL file to analyze
- `analysis_type` (string, optional): Type of analysis to perform. Default: "all"

**Example:**
```json
{
  "vhdl_file": "/path/to/design.vhd",
  "analysis_type": "all"
}
```

### 4. `str_replace_based_edit_tool`

Custom editing tool for viewing, creating, and editing files.

**Parameters:**
- `command` (string, required): Command to execute: "view", "create", "str_replace", or "insert"
- `path` (string, required): Path to the file to operate on
- `old_str` (string, optional): Old string for replacement operations
- `new_str` (string, optional): New string for replacement operations
- `file_text` (string, optional): File content for create operations
- `insert_line` (integer, optional): Line number for insert operations
- `view_range` (array, optional): Range for view operations [start_line, end_line]

## Features

### Architecture Parsing

The transpiler now fully supports VHDL architecture parsing and conversion, including:

- **Process statements** with sensitivity lists
- **Sequential logic** (if/elsif/else statements)
- **Combinational logic** (case statements)
- **Concurrent signal assignments**
- **Internal signals and registers**
- **Type conversions** (std_logic_vector, unsigned, signed)
- **Hex and bit literals** (x"FF" → 8'hFF, '1' → 1'b1)

### Example Conversion

**Input VHDL (counter.vhd):**
```vhdl
entity UP_COUNTER is
    port(
        clk    : in  std_logic;
        reset  : in  std_logic;
        counter: out std_logic_vector(3 downto 0)
    );
end UP_COUNTER;

architecture Behavioral of UP_COUNTER is
    signal counter_up: std_logic_vector(3 downto 0);
begin
    process(clk)
    begin
        if(rising_edge(clk)) then
            if(reset='1') then
                counter_up <= x"0";
            else
                counter_up <= counter_up + x"1";
            end if;
        end if;
    end process;
    counter <= counter_up;
end Behavioral;
```

**Output Verilog (counter.v):**
```verilog
module UP_COUNTER (
    input wire clk,
    input wire reset,
    output wire [3:0] counter
);

    reg [3:0] counter_up;

    always @(posedge clk) begin
        if (reset == 1'b1) begin
        counter_up <= 4'h0;
        end else begin
        counter_up <= counter_up + 4'h1;
        end
        end
    end

    assign counter = counter_up;
endmodule
```

## Running the Server

The MCP server communicates over stdio (standard input/output). To run it:

```bash
./target/release/rtl-transpiler-mcp
```

The server will start and wait for MCP protocol messages on stdin. It will respond with MCP protocol messages on stdout.

## Connecting with MCP Clients

### Claude Desktop

Add the following to your Claude Desktop configuration file:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "rtl-transpiler": {
      "command": "/path/to/rtl_transpiler/target/release/rtl-transpiler-mcp",
      "args": []
    }
  }
}
```

After restarting Claude Desktop, you can use the transpilation tools in your conversations.

### Other MCP Clients

Any MCP-compatible client can connect to the server by launching it as a subprocess and communicating over stdio using the Model Context Protocol.

## Testing

Run the folder transpilation test:

```bash
cargo run --example test_folder_transpile
```

This will transpile all VHDL files in `tests/fixtures/` to Verilog in `tests/output/`.

Run unit tests:

```bash
cargo test --lib transpile_folder
```

## Security

The server includes path validation to restrict file access to allowed folders. By default, all paths are allowed. To restrict access, modify the `allowed_folders` parameter when creating tool instances in `src/mcp/rmcp_server.rs`.

## Architecture Details

The transpiler uses tree-sitter for parsing VHDL files. The parsing flow is:

1. **Parse VHDL** → Tree-sitter AST
2. **Extract IR** → Internal representation (Entity, Architecture, Process, etc.)
3. **Generate Verilog** → Convert IR to Verilog syntax

Key components:
- `src/parser/ast_parser.rs`: VHDL parsing using tree-sitter
- `src/ir/model.rs`: Internal representation data structures
- `src/ir/verilog_gen.rs`: Verilog code generation
- `src/tools/transpile.rs`: Single file transpilation tool
- `src/tools/transpile_folder.rs`: Batch folder transpilation tool
- `src/mcp/rmcp_server.rs`: MCP server implementation

## Recent Improvements

- ✅ Fixed architecture parsing (now extracts process bodies correctly)
- ✅ Fixed sensitivity list extraction
- ✅ Fixed concurrent statement parsing
- ✅ Improved VHDL to Verilog conversion:
  - Hex literals (x"0" → 4'h0)
  - Bit literals ('1' → 1'b1)
  - Comparison operators (= → ==)
  - If/elsif/else statements
  - Case statements
  - Type conversions removal
  - Rising_edge/falling_edge handling
- ✅ Added batch folder transpilation tool
- ✅ Integrated folder transpilation into MCP server

## Troubleshooting

**Problem:** Server not responding
- Check that the binary is built (`cargo build --release`)
- Verify the path in your MCP client configuration
- Check server logs for errors

**Problem:** Files not being transpiled
- Ensure VHDL files have `.vhd` or `.vhdl` extension
- Check file permissions
- Verify paths are correct and accessible

**Problem:** Syntax errors in generated Verilog
- The transpiler handles most common VHDL constructs
- Some complex VHDL features may not be fully supported yet
- Check the input VHDL for unsupported constructs
- File an issue at the project repository with the problematic VHDL code

## License

See the main project LICENSE file.
