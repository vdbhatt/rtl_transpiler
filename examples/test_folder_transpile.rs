use rtl_transpiler::tools::{Tool, TranspileFolderTool};

fn main() -> anyhow::Result<()> {
    // Create the tool (no folder restrictions for this example)
    let tool = TranspileFolderTool::new(vec![]);

    // Transpile all VHDL files in the fixtures directory
    let fixtures_path = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string()) + "/tests/fixtures";

    let output_path = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string()) + "/tests/output";

    println!("Transpiling VHDL files from: {}", fixtures_path);
    println!("Output will be written to: {}\n", output_path);

    let args = serde_json::json!({
        "vhdl_folder": fixtures_path,
        "output_folder": output_path,
        "recursive": false
    });

    let result = tool.execute(&args)?;

    println!("{}", result);

    Ok(())
}
