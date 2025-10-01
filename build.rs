use std::path::PathBuf;

fn main() {
    let dir: PathBuf = ["vendor", "tree-sitter-vhdl", "src"].iter().collect();

    let mut build = cc::Build::new();
    build.include(&dir)
         .file(dir.join("parser.c"));

    // Only include scanner.c if it exists
    let scanner_path = dir.join("scanner.c");
    if scanner_path.exists() {
        build.file(scanner_path);
    }

    build.compile("tree-sitter-vhdl");
}
