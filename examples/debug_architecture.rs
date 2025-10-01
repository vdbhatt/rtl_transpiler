use rtl_transpiler::parser::tree_sitter_vhdl::TreeSitterVHDLParser;

fn print_node(node: &tree_sitter::Node, source: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let node_text = &source[node.start_byte()..node.end_byte()];
    let display_text = if node_text.len() > 60 {
        format!("{}...", &node_text[..60])
    } else {
        node_text.to_string()
    };

    println!("{}[{}] {}", indent, node.kind(), display_text.replace("\n", "\\n"));

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if depth < 8 {  // Limit depth to avoid too much output
            print_node(&child, source, depth + 1);
        }
    }
}

fn main() -> anyhow::Result<()> {
    let vhdl_source = std::fs::read_to_string("tests/fixtures/counter.vhd")?;

    let mut parser = TreeSitterVHDLParser::new()?;
    let tree = parser.parse(&vhdl_source)?;

    println!("=== AST Structure ===\n");
    print_node(&tree.root_node(), &vhdl_source, 0);

    Ok(())
}
