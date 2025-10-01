use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

use crate::ir::SystemVerilogGenerator;
use crate::parser::ASTVHDLParser;
use crate::tools::{BaseToolImpl, Tool, ToolParameter, ToolSchema};

/// Tool for batch transpiling VHDL files in a folder to SystemVerilog 2012 modules
pub struct TranspileFolderTool {
    base: BaseToolImpl,
    allowed_folders: Vec<String>,
}

impl TranspileFolderTool {
    pub fn new(allowed_folders: Vec<String>) -> Self {
        let parameters = vec![
            ToolParameter {
                name: "vhdl_folder".to_string(),
                param_type: "string".to_string(),
                description: "Path to the folder containing VHDL files to transpile".to_string(),
                required: true,
                default: None,
            },
            ToolParameter {
                name: "output_folder".to_string(),
                param_type: "string".to_string(),
                description: "Path to the output folder for SystemVerilog files (optional, defaults to same folder)".to_string(),
                required: false,
                default: None,
            },
            ToolParameter {
                name: "recursive".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to recursively process subdirectories (default: false)".to_string(),
                required: false,
                default: Some(serde_json::Value::Bool(false)),
            },
        ];

        let base = BaseToolImpl::new(
            "transpile_vhdl_folder_to_systemverilog".to_string(),
            "Batch transpile all VHDL files in a folder to SystemVerilog 2012 modules. Processes all .vhd and .vhdl files in the specified directory.".to_string(),
            parameters,
        );

        Self {
            base,
            allowed_folders,
        }
    }

    fn is_path_allowed(&self, path: &Path) -> bool {
        if self.allowed_folders.is_empty() {
            return true;
        }

        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        for allowed in &self.allowed_folders {
            let allowed_path = match Path::new(allowed).canonicalize() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if canonical_path.starts_with(&allowed_path) {
                return true;
            }
        }

        false
    }

    fn find_vhdl_files(&self, folder: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
        let mut vhdl_files = Vec::new();

        if !folder.is_dir() {
            return Err(anyhow::anyhow!("'{}' is not a directory", folder.display()));
        }

        let entries = fs::read_dir(folder)
            .context(format!("Failed to read directory: {}", folder.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "vhd" || ext_str == "vhdl" {
                        vhdl_files.push(path);
                    }
                }
            } else if path.is_dir() && recursive {
                let sub_files = self.find_vhdl_files(&path, recursive)?;
                vhdl_files.extend(sub_files);
            }
        }

        Ok(vhdl_files)
    }

    fn transpile_file(&self, vhdl_path: &Path, output_folder: &Path) -> Result<(String, String)> {
        // Parse VHDL using AST parser
        let mut parser = ASTVHDLParser::from_file(vhdl_path)
            .context(format!("Failed to parse VHDL file: {}", vhdl_path.display()))?;

        let entities = parser.parse_entities()
            .context("Failed to extract entities from VHDL")?;

        if entities.is_empty() {
            return Err(anyhow::anyhow!("No entities found in VHDL file"));
        }

        // Generate SystemVerilog for all entities
        let generator = SystemVerilogGenerator::new();
        let mut systemverilog_output = String::new();

        for entity in &entities {
            let systemverilog = generator.generate(entity)
                .context(format!("Failed to generate SystemVerilog for entity: {}", entity.name))?;

            systemverilog_output.push_str(&systemverilog);
            systemverilog_output.push('\n');
        }

        // Determine output file path
        let vhdl_filename = vhdl_path.file_stem()
            .ok_or_else(|| anyhow::anyhow!("Invalid VHDL filename"))?;
        let output_path = output_folder.join(format!("{}.sv", vhdl_filename.to_string_lossy()));

        // Write to file
        std::fs::write(&output_path, &systemverilog_output)
            .context(format!("Failed to write SystemVerilog to: {}", output_path.display()))?;

        Ok((
            vhdl_path.display().to_string(),
            output_path.display().to_string(),
        ))
    }
}

impl Tool for TranspileFolderTool {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn schema(&self) -> ToolSchema {
        self.base.schema.clone()
    }

    fn execute(&self, arguments: &serde_json::Value) -> Result<String> {
        let vhdl_folder = arguments
            .get("vhdl_folder")
            .and_then(|v| v.as_str())
            .context("Missing 'vhdl_folder' argument")?;

        let output_folder = arguments
            .get("output_folder")
            .and_then(|v| v.as_str())
            .unwrap_or(vhdl_folder);

        let recursive = arguments
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let vhdl_path = Path::new(vhdl_folder);
        let output_path = Path::new(output_folder);

        // Check if paths are allowed
        if !self.is_path_allowed(vhdl_path) {
            return Err(anyhow::anyhow!(
                "Access denied: '{}' is not in allowed folders",
                vhdl_folder
            ));
        }

        if !self.is_path_allowed(output_path) {
            return Err(anyhow::anyhow!(
                "Access denied: output path '{}' is not in allowed folders",
                output_folder
            ));
        }

        // Create output folder if it doesn't exist
        if !output_path.exists() {
            fs::create_dir_all(output_path)
                .context(format!("Failed to create output directory: {}", output_folder))?;
        }

        // Find all VHDL files
        tracing::info!("Searching for VHDL files in: {}", vhdl_folder);
        let vhdl_files = self.find_vhdl_files(vhdl_path, recursive)?;

        if vhdl_files.is_empty() {
            return Ok(format!("No VHDL files found in '{}'", vhdl_folder));
        }

        tracing::info!("Found {} VHDL file(s)", vhdl_files.len());

        // Transpile each file
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut success_count = 0;

        for vhdl_file in &vhdl_files {
            tracing::info!("Transpiling: {}", vhdl_file.display());

            match self.transpile_file(vhdl_file, output_path) {
                Ok((input, output)) => {
                    results.push(format!("✓ {} -> {}", input, output));
                    success_count += 1;
                }
                Err(e) => {
                    let error_msg = format!("✗ {}: {}", vhdl_file.display(), e);
                    errors.push(error_msg.clone());
                    tracing::error!("{}", error_msg);
                }
            }
        }

        // Build summary report
        let mut report = String::new();
        report.push_str(&format!("\n=== Batch VHDL to SystemVerilog Transpilation ===\n\n"));
        report.push_str(&format!("Input folder:  {}\n", vhdl_folder));
        report.push_str(&format!("Output folder: {}\n", output_folder));
        report.push_str(&format!("Recursive:     {}\n\n", recursive));
        report.push_str(&format!("Total files found:      {}\n", vhdl_files.len()));
        report.push_str(&format!("Successfully transpiled: {}\n", success_count));
        report.push_str(&format!("Failed:                 {}\n\n", errors.len()));

        if !results.is_empty() {
            report.push_str("=== Successful Transpilations ===\n");
            for result in results {
                report.push_str(&format!("{}\n", result));
            }
            report.push('\n');
        }

        if !errors.is_empty() {
            report.push_str("=== Errors ===\n");
            for error in errors {
                report.push_str(&format!("{}\n", error));
            }
            report.push('\n');
        }

        report.push_str(&format!("=== Transpilation Complete ===\n"));

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;

    #[test]
    fn test_transpile_folder_tool() {
        // Create temp directory with VHDL files
        let temp_dir = TempDir::new().unwrap();
        let vhdl_folder = temp_dir.path();

        // Create a couple of VHDL files
        let vhdl1 = r#"
        entity counter is
            port(
                clk    : in  std_logic;
                reset  : in  std_logic;
                count  : out std_logic_vector(7 downto 0)
            );
        end entity counter;
        "#;

        let vhdl2 = r#"
        entity buffer_entity is
            port(
                data_in  : in  std_logic;
                data_out : out std_logic
            );
        end entity buffer_entity;
        "#;

        fs::write(vhdl_folder.join("counter.vhd"), vhdl1).unwrap();
        fs::write(vhdl_folder.join("buffer.vhd"), vhdl2).unwrap();

        // Create tool with allowed folders (allow all)
        let tool = TranspileFolderTool::new(vec![]);

        // Execute
        let args = serde_json::json!({
            "vhdl_folder": vhdl_folder.to_str().unwrap(),
            "recursive": false
        });

        let result = tool.execute(&args).unwrap();

        println!("Result:\n{}", result);

        assert!(result.contains("Successfully transpiled: 2"));
        assert!(result.contains("counter.vhd"));
        assert!(result.contains("buffer.vhd"));

        // Verify output files exist
        assert!(vhdl_folder.join("counter.sv").exists());
        assert!(vhdl_folder.join("buffer.sv").exists());
    }
}
