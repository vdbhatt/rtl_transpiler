use rtl_transpiler::parser::ASTVHDLParser;
use rtl_transpiler::ir::VerilogGenerator;
use std::path::PathBuf;

#[test]
fn test_counter_transpilation() {
    let vhdl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/counter.vhd");

    let mut parser = ASTVHDLParser::from_file(&vhdl_path).unwrap();
    let entities = parser.parse_entities().unwrap();

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].name, "counter");
    assert_eq!(entities[0].ports.len(), 4);

    let generator = VerilogGenerator::new();
    let verilog = generator.generate(&entities[0]).unwrap();

    println!("Generated Verilog:\n{}", verilog);

    // Verify key parts of the generated Verilog
    assert!(verilog.contains("module counter"));
    assert!(verilog.contains("input wire clk"));
    assert!(verilog.contains("input wire reset"));
    assert!(verilog.contains("input wire enable"));
    assert!(verilog.contains("output wire [7:0] count"));
    assert!(verilog.contains("endmodule"));

    // Verify port ordering (should match VHDL)
    let clk_pos = verilog.find("input wire clk").unwrap();
    let reset_pos = verilog.find("input wire reset").unwrap();
    let enable_pos = verilog.find("input wire enable").unwrap();
    let count_pos = verilog.find("output wire [7:0] count").unwrap();

    assert!(clk_pos < reset_pos);
    assert!(reset_pos < enable_pos);
    assert!(enable_pos < count_pos);
}

#[test]
fn test_alu_transpilation() {
    let vhdl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/alu.vhd");

    let mut parser = ASTVHDLParser::from_file(&vhdl_path).unwrap();
    let entities = parser.parse_entities().unwrap();

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].name, "alu");

    eprintln!("Found {} ports", entities[0].ports.len());
    for port in &entities[0].ports {
        eprintln!("Port: {} ({:?})", port.name, port.direction);
    }

    assert_eq!(entities[0].ports.len(), 6);

    let generator = VerilogGenerator::new();
    let verilog = generator.generate(&entities[0]).unwrap();

    println!("Generated Verilog:\n{}", verilog);

    // Verify key parts
    assert!(verilog.contains("module alu"));
    assert!(verilog.contains("input wire [15:0] a"));
    assert!(verilog.contains("input wire [15:0] b"));
    assert!(verilog.contains("input wire [2:0] opcode"));
    assert!(verilog.contains("output wire [15:0] result"));
    assert!(verilog.contains("output wire zero"));
    assert!(verilog.contains("output wire carry"));
    assert!(verilog.contains("endmodule"));
}

#[test]
fn test_type_conversions() {
    let vhdl = r#"
    entity type_test is
        port(
            bit_sig       : in  bit;
            logic_sig     : in  std_logic;
            int_sig       : in  integer;
            natural_sig   : in  natural;
            vec_sig       : in  std_logic_vector(31 downto 0);
            signed_sig    : in  signed(15 downto 0);
            unsigned_sig  : in  unsigned(7 downto 0)
        );
    end entity type_test;
    "#;

    let mut parser = ASTVHDLParser::new(vhdl.to_string()).unwrap();
    let entities = parser.parse_entities().unwrap();

    assert_eq!(entities.len(), 1);

    let generator = VerilogGenerator::new();
    let verilog = generator.generate(&entities[0]).unwrap();

    println!("Generated Verilog:\n{}", verilog);

    // Verify type mappings
    assert!(verilog.contains("input wire bit_sig"));
    assert!(verilog.contains("input wire logic_sig"));
    assert!(verilog.contains("input wire signed [31:0] int_sig"));
    assert!(verilog.contains("input wire [31:0] natural_sig"));
    assert!(verilog.contains("input wire [31:0] vec_sig"));
    assert!(verilog.contains("input wire signed [15:0] signed_sig"));
    assert!(verilog.contains("input wire [7:0] unsigned_sig"));
}