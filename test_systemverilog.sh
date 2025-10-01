#!/bin/bash

# SystemVerilog Transpiler Test Script
# Demonstrates the new SystemVerilog 2012 output

echo "=== SystemVerilog 2012 Transpiler Test ==="
echo ""

# Create test VHDL files
echo "Creating test VHDL files..."

# Test 1: Simple Combinational Logic (Mux)
cat > test_comb_mux.vhd << 'EOF'
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity mux2to1 is
    Port (
        a : in STD_LOGIC;
        b : in STD_LOGIC;
        sel : in STD_LOGIC;
        y : out STD_LOGIC
    );
end mux2to1;

architecture Behavioral of mux2to1 is
begin
    process(a, b, sel)
    begin
        if sel = '0' then
            y <= a;
        else
            y <= b;
        end if;
    end process;
end Behavioral;
EOF

# Test 2: Sequential Logic (Counter)
cat > test_seq_counter.vhd << 'EOF'
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;
use IEEE.NUMERIC_STD.ALL;

entity counter is
    Port (
        clk : in STD_LOGIC;
        reset : in STD_LOGIC;
        enable : in STD_LOGIC;
        count : out STD_LOGIC_VECTOR(7 downto 0)
    );
end counter;

architecture Behavioral of counter is
    signal cnt : unsigned(7 downto 0) := (others => '0');
begin
    process(clk)
    begin
        if rising_edge(clk) then
            if reset = '1' then
                cnt <= (others => '0');
            elsif enable = '1' then
                cnt <= cnt + 1;
            end if;
        end if;
    end process;
    
    count <= std_logic_vector(cnt);
end Behavioral;
EOF

# Test 3: Case Statement (Decoder)
cat > test_case_decoder.vhd << 'EOF'
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity decoder_2to4 is
    Port (
        sel : in STD_LOGIC_VECTOR(1 downto 0);
        y : out STD_LOGIC_VECTOR(3 downto 0)
    );
end decoder_2to4;

architecture Behavioral of decoder_2to4 is
begin
    process(sel)
    begin
        case sel is
            when "00" =>
                y <= "0001";
            when "01" =>
                y <= "0010";
            when "10" =>
                y <= "0100";
            when "11" =>
                y <= "1000";
            when others =>
                y <= "0000";
        end case;
    end process;
end Behavioral;
EOF

echo "✓ Test VHDL files created"
echo ""

# Rebuild project
echo "Building project..."
cd /Users/vijaybhatt/repos/rtl_transpiler
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true
echo ""

# Test transpilation
echo "=== Test 1: Combinational Logic (Mux) ==="
echo "Transpiling to SystemVerilog (default)..."
./target/release/rtl-transpiler-mcp transpile test_comb_mux.vhd -o test_comb_mux.sv 2>&1 || true
echo ""
echo "Generated SystemVerilog:"
cat test_comb_mux.sv 2>/dev/null || echo "File not generated yet (needs rebuild)"
echo ""

echo "Transpiling to Verilog (legacy)..."
./target/release/rtl-transpiler-mcp transpile test_comb_mux.vhd -o test_comb_mux.v --format verilog 2>&1 || true
echo ""
echo "Generated Verilog:"
cat test_comb_mux.v 2>/dev/null || echo "File not generated yet (needs rebuild)"
echo ""

echo "=== Test 2: Sequential Logic (Counter) ==="
echo "Transpiling to SystemVerilog..."
./target/release/rtl-transpiler-mcp transpile test_seq_counter.vhd -o test_seq_counter.sv 2>&1 || true
echo ""
echo "Generated SystemVerilog:"
cat test_seq_counter.sv 2>/dev/null || echo "File not generated yet (needs rebuild)"
echo ""

echo "=== Test 3: Case Statement (Decoder) ==="
echo "Transpiling to SystemVerilog..."
./target/release/rtl-transpiler-mcp transpile test_case_decoder.vhd -o test_case_decoder.sv 2>&1 || true
echo ""
echo "Generated SystemVerilog:"
cat test_case_decoder.sv 2>/dev/null || echo "File not generated yet (needs rebuild)"
echo ""

echo "=== Comparison Summary ==="
echo ""
echo "Verilog vs SystemVerilog Key Differences:"
echo "1. wire/reg → logic (unified type)"
echo "2. always @(*) → always_comb (clear intent)"
echo "3. always @(posedge clk) → always_ff (sequential logic)"
echo "4. case → unique case (optimization hint)"
echo "5. (others => '0') → '0 (cleaner syntax)"
echo ""

echo "=== Test Complete ==="
echo ""
echo "To rebuild and test:"
echo "  cd /Users/vijaybhatt/repos/rtl_transpiler"
echo "  cargo build --release"
echo "  ./test_systemverilog.sh"
