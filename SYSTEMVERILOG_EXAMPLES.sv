// Expected SystemVerilog 2012 Output Examples
// These show what the transpiler will generate after rebuilding

// ============================================================================
// Example 1: Simple 2-to-1 Multiplexer (Combinational Logic)
// ============================================================================

// VHDL Input:
// entity mux2to1 is
//     Port (
//         a : in STD_LOGIC;
//         b : in STD_LOGIC;
//         sel : in STD_LOGIC;
//         y : out STD_LOGIC
//     );
// end mux2to1;
//
// architecture Behavioral of mux2to1 is
// begin
//     process(a, b, sel)
//     begin
//         if sel = '0' then
//             y <= a;
//         else
//             y <= b;
//         end if;
//     end process;
// end Behavioral;

// SystemVerilog Output:
module mux2to1 (
    input logic a,
    input logic b,
    input logic sel,
    output logic y
);

    always_comb begin
        if (sel == 1'b0) begin
            y <= a;
        end else begin
            y <= b;
        end
    end
endmodule

// Key Features:
// - 'logic' type instead of wire/reg (no more confusion!)
// - 'always_comb' clearly indicates combinational logic
// - Better synthesis optimization opportunities

// ============================================================================
// Example 2: 8-bit Counter (Sequential Logic)
// ============================================================================

// SystemVerilog Output:
module counter (
    input logic clk,
    input logic reset,
    input logic enable,
    output logic [7:0] count
);

    logic [7:0] cnt;

    always_ff @(posedge clk) begin
        if (reset == 1'b1) begin
            cnt <= '0;  // SystemVerilog '0 = all zeros
        end else if (enable == 1'b1) begin
            cnt <= cnt + 1;
        end
    end
    
    assign count = cnt;
endmodule

// Key Features:
// - 'always_ff' explicitly marks flip-flop logic
// - '0 notation for all-zeros (cleaner than 8'b0)
// - Better timing analysis by synthesis tools
// - Clear distinction from combinational logic

// ============================================================================
// Example 3: 2-to-4 Decoder (Case Statement)
// ============================================================================

// SystemVerilog Output:
module decoder_2to4 (
    input logic [1:0] sel,
    output logic [3:0] y
);

    always_comb begin
        unique case (sel)
            2'b00: begin
                y <= 4'b0001;
            end
            2'b01: begin
                y <= 4'b0010;
            end
            2'b10: begin
                y <= 4'b0100;
            end
            2'b11: begin
                y <= 4'b1000;
            end
            default: begin
                y <= 4'b0000;
            end
        endcase
    end
endmodule

// Key Features:
// - 'unique case' tells synthesizer cases are mutually exclusive
// - Better optimization and resource usage
// - Synthesis warnings if cases overlap

// ============================================================================
// Example 4: FSM with Async Reset (Sequential + Combinational)
// ============================================================================

// SystemVerilog Output:
module simple_fsm (
    input logic clk,
    input logic arst_n,  // Active-low async reset
    input logic start,
    output logic [1:0] state,
    output logic done
);

    typedef enum logic [1:0] {
        IDLE   = 2'b00,
        ACTIVE = 2'b01,
        FINISH = 2'b10
    } state_t;

    state_t current_state, next_state;

    // Sequential logic with async reset
    always_ff @(posedge clk or negedge arst_n) begin
        if (!arst_n) begin
            current_state <= IDLE;
        end else begin
            current_state <= next_state;
        end
    end

    // Combinational next-state logic
    always_comb begin
        next_state = current_state;
        done = 1'b0;
        
        unique case (current_state)
            IDLE: begin
                if (start) begin
                    next_state = ACTIVE;
                end
            end
            ACTIVE: begin
                next_state = FINISH;
            end
            FINISH: begin
                done = 1'b1;
                next_state = IDLE;
            end
            default: begin
                next_state = IDLE;
            end
        endcase
    end

    assign state = current_state;
endmodule

// Key Features:
// - Separate always_ff and always_comb blocks (2-process FSM)
// - Enumerated types for state encoding (optional)
// - Clear separation of sequential and combinational logic
// - 'unique case' for better synthesis

// ============================================================================
// Comparison: Verilog vs SystemVerilog
// ============================================================================

// VERILOG-2001 (Old Way):
// -----------------------
// module mux2to1 (
//     input wire a,        // Must specify wire
//     input wire b,
//     input wire sel,
//     output reg y         // Must use reg for procedural assignment!
// );
//
//     always @(*) begin    // Generic sensitivity
//         if (sel == 1'b0) begin
//             y <= a;
//         end else begin
//             y <= b;
//         end
//     end
// endmodule

// SYSTEMVERILOG-2012 (New Way):
// ------------------------------
// module mux2to1 (
//     input logic a,       // 'logic' works everywhere
//     input logic b,
//     input logic sel,
//     output logic y       // No wire/reg distinction needed!
// );
//
//     always_comb begin    // Clear intent: combinational
//         if (sel == 1'b0) begin
//             y <= a;
//         end else begin
//             y <= b;
//         end
//     end
// endmodule

// ============================================================================
// Migration Benefits Summary
// ============================================================================

// 1. NO MORE WIRE/REG ERRORS
//    ❌ Verilog: "cannot assign to wire y in always block"
//    ✅ SystemVerilog: 'logic' works in all contexts

// 2. CLEARER INTENT
//    ❌ Verilog: always @(*) - is this comb or just poor coding?
//    ✅ SystemVerilog: always_comb - explicitly combinational

// 3. BETTER SYNTHESIS
//    ❌ Verilog: case (sel) - synthesizer guesses optimization
//    ✅ SystemVerilog: unique case (sel) - explicit hint

// 4. MODERN TOOLS
//    ❌ Verilog: Supported but legacy
//    ✅ SystemVerilog: Recommended by all modern tools

// 5. CLEANER SYNTAX
//    ❌ Verilog: count = 8'b00000000;
//    ✅ SystemVerilog: count = '0;

// ============================================================================
// Tool Support Matrix
// ============================================================================

// Synthesis Tools:
// ✅ Xilinx Vivado (2015+)      - Full SV 2012 support, recommended
// ✅ Intel Quartus Prime (15+)  - Full SV support
// ✅ Synopsys DC                - Industry standard SV synthesis
// ✅ Cadence Genus              - Complete SV-2012 compliance
// ✅ Yosys (0.9+)              - Open source, growing SV support

// Simulation Tools:
// ✅ ModelSim/QuestaSim         - Full SV support
// ✅ VCS (Synopsys)            - Complete SV-2012
// ✅ Xcelium (Cadence)         - Full SV support
// ✅ Verilator (4.0+)          - Fast SV simulation & linting
// ✅ Icarus Verilog            - Basic SV support

// ============================================================================
// Usage Instructions
// ============================================================================

// 1. Build the transpiler:
//    cd /Users/vijaybhatt/repos/rtl_transpiler
//    cargo build --release

// 2. Transpile to SystemVerilog (DEFAULT):
//    ./target/release/rtl-transpiler-mcp transpile design.vhd -o design.sv

// 3. Transpile to SystemVerilog (EXPLICIT):
//    ./target/release/rtl-transpiler-mcp transpile design.vhd -o design.sv --format systemverilog

// 4. Transpile to Verilog (LEGACY):
//    ./target/release/rtl-transpiler-mcp transpile design.vhd -o design.v --format verilog

// 5. Using MCP Server:
//    {
//      "vhdl_file": "design.vhd",
//      "output_file": "design.sv",
//      "format": "systemverilog"  // or "verilog"
//    }

// ============================================================================
// Best Practices
// ============================================================================

// ✅ DO: Use always_comb for combinational logic
always_comb begin
    y = a & b;
end

// ✅ DO: Use always_ff for sequential logic
always_ff @(posedge clk) begin
    q <= d;
end

// ✅ DO: Use 'logic' for all signal types
logic [7:0] data;
logic valid;

// ✅ DO: Use unique case for mutually exclusive cases
unique case (state)
    IDLE: ...
    ACTIVE: ...
endcase

// ❌ DON'T: Mix wire and reg anymore
// wire [7:0] data;  // Old way
// reg [7:0] result; // Old way

// ❌ DON'T: Use generic always @(*) 
// always @(*) begin  // Use always_comb instead
//     ...
// end

// ❌ DON'T: Forget to specify unique/priority for case
// case (state)  // Synthesizer can't optimize well
//     ...
// endcase
