#!/bin/bash

# Test script for RTL Transpiler fixes
# This script creates a test VHDL file, transpiles it, and checks the output

echo "=== RTL Transpiler Fix Verification ==="
echo ""

# Create test VHDL file
echo "Creating test VHDL file..."
cat > test_mux_verify.vhd << 'EOF'
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

-- Simple 2-to-1 Multiplexer
entity mux2to1_test is
    Port (
        a : in STD_LOGIC;
        b : in STD_LOGIC;
        sel : in STD_LOGIC;
        y : out STD_LOGIC
    );
end mux2to1_test;

architecture Behavioral of mux2to1_test is
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

echo "✓ Test VHDL file created"
echo ""

# Rebuild the project
echo "Rebuilding project..."
cargo build --release 2>&1 | tail -n 3
echo ""

# Run transpiler
echo "Running transpiler..."
./target/release/rtl-transpiler-mcp transpile test_mux_verify.vhd -o test_mux_verify.v 2>&1 || true
echo ""

# Check if output file exists
if [ -f test_mux_verify.v ]; then
    echo "✓ Verilog file generated"
    echo ""
    
    echo "Generated Verilog content:"
    echo "=========================="
    cat test_mux_verify.v
    echo "=========================="
    echo ""
    
    # Check for correct output type
    if grep -q "output reg y" test_mux_verify.v; then
        echo "✅ PASS: Output port correctly declared as 'reg'"
    elif grep -q "output wire y" test_mux_verify.v; then
        echo "❌ FAIL: Output port incorrectly declared as 'wire'"
    else
        echo "⚠️  WARNING: Could not find output declaration"
    fi
    
    # Check for proper indentation (looking for properly indented if statement)
    if grep -q "        if " test_mux_verify.v; then
        echo "✅ PASS: Proper indentation detected"
    else
        echo "❌ FAIL: Indentation issues detected"
    fi
    
    # Check for proper parentheses in if statement
    if grep -q "if (sel ==" test_mux_verify.v || grep -q "if(sel ==" test_mux_verify.v; then
        echo "✅ PASS: Conditional statement properly formatted"
    else
        echo "⚠️  WARNING: Conditional statement format needs review"
    fi
    
else
    echo "❌ FAIL: Verilog file not generated"
fi

echo ""
echo "=== Test Complete ==="
