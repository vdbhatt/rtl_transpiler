use rtl_transpiler::{ASTVHDLParser};

fn main() -> anyhow::Result<()> {
    let vhdl_path = "tests/fixtures/counter.vhd";

    let mut parser = ASTVHDLParser::from_file(std::path::Path::new(vhdl_path))?;
    let entities = parser.parse_entities()?;

    println!("=== Parsed Entities ===");
    for entity in &entities {
        println!("\nEntity: {}", entity.name);
        println!("Ports: {}", entity.ports.len());

        if let Some(arch) = &entity.architecture {
            println!("\nArchitecture: {}", arch.name);
            println!("Signals: {}", arch.signals.len());
            for signal in &arch.signals {
                println!("  - {} : {:?}", signal.name, signal.signal_type);
            }

            println!("\nProcesses: {}", arch.processes.len());
            for (i, process) in arch.processes.iter().enumerate() {
                println!("\nProcess {}:", i);
                println!("  Label: {:?}", process.label);
                println!("  Sensitivity list: {:?}", process.sensitivity_list);
                println!("  Body length: {} chars", process.body.len());
                println!("  Body:\n{}", process.body);
            }

            println!("\nConcurrent statements: {}", arch.concurrent_statements.len());
            for stmt in &arch.concurrent_statements {
                println!("  - {}", stmt);
            }
        } else {
            println!("No architecture found!");
        }
    }

    Ok(())
}
