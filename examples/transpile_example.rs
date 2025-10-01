use rtl_transpiler::{ASTVHDLParser, ir::VerilogGenerator};
use std::path::PathBuf;
use std::fs;

fn main() -> anyhow::Result<()> {
    // Setup paths
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/output");

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;

    // List of VHDL files to transpile
    let vhdl_files = vec![
        ("counter.vhd", "counter.v"),
        ("alu.vhd", "alu.v"),
        ("fifo.vhd", "fifo.v"),
        ("uart.vhd", "uart.v"),
        ("spi_master.vhd", "spi_master.v"),
        ("memory_controller.vhd", "memory_controller.v"),
        ("axi_crossbar.vhd", "axi_crossbar.v"),
        ("pcie_endpoint.vhd", "pcie_endpoint.v"),
    ];

    for (vhdl_file, verilog_file) in vhdl_files {
        let vhdl_path = fixtures_dir.join(vhdl_file);
        let output_path = output_dir.join(verilog_file);

        println!("Transpiling {} -> {}", vhdl_file, verilog_file);

        // Parse VHDL
        let mut parser = ASTVHDLParser::from_file(&vhdl_path)?;
        let entities = parser.parse_entities()?;

        if entities.is_empty() {
            println!("  Warning: No entities found in {}", vhdl_file);
            continue;
        }

        // Generate Verilog for all entities
        let generator = VerilogGenerator::new();
        let mut verilog_output = String::new();

        for entity in &entities {
            println!("  Converting entity: {}", entity.name);
            let verilog = generator.generate(entity)?;
            verilog_output.push_str(&verilog);
            verilog_output.push('\n');
        }

        // Write to file
        fs::write(&output_path, verilog_output)?;
        println!("  ✓ Written to: {}", output_path.display());
    }

    println!("\nTranspilation complete!");
    Ok(())
}
