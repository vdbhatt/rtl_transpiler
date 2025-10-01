# RTL Transpiler - VHDL to Verilog

A Rust-based AI-powered transpiler that converts VHDL entity declarations to Verilog module declarations.

## Features

✅ **Entity to Module Conversion**: Converts VHDL entities to Verilog modules with matching ports
✅ **Type Mapping**: Automatically maps VHDL types to Verilog equivalents
✅ **Port Direction Mapping**: Correctly translates port directions (in/out/inout/buffer)
✅ **Vector Support**: Handles std_logic_vector, signed, unsigned with proper bit ordering
✅ **AI Agent Architecture**: Extensible agent-based framework for complex translations

## Supported Type Mappings

| VHDL Type | Verilog Type |
|-----------|--------------|
| `std_logic` | `wire` |
| `std_logic_vector(N downto 0)` | `wire [N:0]` |
| `bit` | `wire` |
| `integer` | `wire signed [31:0]` |
| `natural` | `wire [31:0]` |
| `signed(N downto 0)` | `wire signed [N:0]` |
| `unsigned(N downto 0)` | `wire [N:0]` |

## Example

**Input VHDL:**
```vhdl
entity counter is
    port(
        clk    : in  std_logic;
        reset  : in  std_logic;
        enable : in  std_logic;
        count  : out std_logic_vector(7 downto 0)
    );
end entity counter;
```

**Output Verilog:**
```verilog
module counter (
    input  wire       clk,
    input  wire       reset,
    input  wire       enable,
    output wire [7:0] count
);
endmodule
```

## Architecture

The transpiler uses a multi-stage pipeline:

1. **Parser** (`src/parser/vhdl.rs`): Regex-based VHDL parser that extracts entities and ports
2. **IR** (`src/ir/`): Intermediate representation for language-agnostic design storage
3. **Generator** (`src/ir/verilog_gen.rs`): Verilog code generator from IR
4. **Tools** (`src/tools/transpile.rs`): Tool interface for transpilation operations
5. **Agent** (`src/agent/transpiler_agent.rs`): AI agent for orchestrating complex conversions

## Project Structure

```
rtl_transpiler/
├── src/
│   ├── agent/           # Agent framework (AlanAgent, TranspilerAgent)
│   ├── ir/              # Intermediate representation
│   │   ├── model.rs     # Entity, Port, Type definitions
│   │   └── verilog_gen.rs  # Verilog code generation
│   ├── parser/          # VHDL parsing
│   │   └── vhdl.rs      # Regex-based VHDL parser
│   ├── tools/           # Tool implementations
│   │   └── transpile.rs # Transpilation tool
│   ├── llm/             # LLM client interfaces
│   ├── config.rs        # Configuration structures
│   └── lib.rs           # Library root
├── tests/
│   ├── fixtures/        # Test VHDL files
│   └── integration_test.rs  # Integration tests
└── Cargo.toml
```

## Testing

Run the test suite:

```bash
cargo test
```

All tests pass successfully:
- ✅ `test_counter_transpilation` - Basic counter entity
- ✅ `test_alu_transpilation` - Multi-port ALU entity
- ✅ `test_type_conversions` - Various VHDL type mappings

## Usage (Library)

```rust
use rtl_transpiler::{VHDLParser, VerilogGenerator};

// Parse VHDL
let parser = VHDLParser::from_file("counter.vhd")?;
let entities = parser.parse_entities()?;

// Generate Verilog
let generator = VerilogGenerator::new();
let verilog = generator.generate(&entities[0])?;

println!("{}", verilog);
```

## Current Limitations

- Only entity-to-module conversion (no architecture/implementation conversion yet)
- AST-based parsing using tree-sitter for robust VHDL analysis
- No support for generics/parameters yet
- No support for VHDL packages/libraries

## Future Enhancements

- [x] Full tree-sitter integration for robust parsing
- [ ] Architecture body conversion (processes, signals, logic)
- [ ] Generic/parameter support
- [ ] Package and library handling
- [ ] Testbench generation
- [ ] Full LLM integration for semantic understanding
- [ ] CLI tool for standalone usage

## Dependencies

- `regex` - VHDL parsing
- `serde` / `serde_json` - Serialization
- `anyhow` - Error handling
- `tokio` - Async runtime (for agent framework)
- Agent framework infrastructure (LLM client, tools, MCP)

## License

MIT

---

**Status**: ✅ Core functionality implemented and tested
**Test Results**: 3/3 passing
**Generated**: 2025-09-30
