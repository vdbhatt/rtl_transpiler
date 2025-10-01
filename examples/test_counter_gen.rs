use rtl_transpiler::{ASTVHDLParser, ir::VerilogGenerator};

fn main() -> anyhow::Result<()> {
    let vhdl_path = "tests/fixtures/counter.vhd";

    let mut parser = ASTVHDLParser::from_file(std::path::Path::new(vhdl_path))?;
    let entities = parser.parse_entities()?;

    if let Some(entity) = entities.first() {
        let generator = VerilogGenerator::new();
        let verilog = generator.generate(entity)?;

        println!("Generated Verilog:\n");
        println!("{}", verilog);
    }

    Ok(())
}
