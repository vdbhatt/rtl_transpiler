use rtl_transpiler::{ASTVHDLParser, ir::VerilogGenerator};
use std::path::PathBuf;
use std::fs;

fn main() -> anyhow::Result<()> {
    // Setup paths
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/output");

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;

    // VHDL files with architecture
    let vhdl_files = vec![
        ("counter_with_arch.vhd", "counter_with_arch.v"),
        ("simple_alu_with_arch.vhd", "simple_alu_with_arch.v"),
        ("fsm_with_arch.vhd", "fsm_with_arch.v"),
    ];

    for (vhdl_file, verilog_file) in vhdl_files {
        let vhdl_path = fixtures_dir.join(vhdl_file);
        let output_path = output_dir.join(verilog_file);

        println!("\n========================================");
        println!("Transpiling {} -> {}", vhdl_file, verilog_file);
        println!("========================================");

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
            println!("\n📦 Converting entity: {}", entity.name);

            if let Some(arch) = &entity.architecture {
                println!("   ✓ Architecture: {}", arch.name);
                println!("   ✓ Signals: {}", arch.signals.len());
                println!("   ✓ Processes: {}", arch.processes.len());
                println!("   ✓ Concurrent statements: {}", arch.concurrent_statements.len());
            } else {
                println!("   ℹ No architecture found");
            }

            let verilog = generator.generate(entity)?;
            verilog_output.push_str(&verilog);
            verilog_output.push('\n');
        }

        // Write to file
        fs::write(&output_path, &verilog_output)?;
        println!("\n✅ Written to: {}", output_path.display());

        // Show generated code
        println!("\n📄 Generated Verilog:\n");
        println!("{}", verilog_output);
    }

    println!("\n========================================");
    println!("✨ Transpilation complete!");
    println!("========================================");
    Ok(())
}
