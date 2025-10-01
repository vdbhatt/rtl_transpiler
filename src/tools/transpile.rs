use anyhow::{Context, Result};
use std::path::Path;

use crate::ir::VerilogGenerator;
use crate::parser::ASTVHDLParser;
use crate::tools::{BaseToolImpl, Tool, ToolParameter, ToolSchema};

/// Tool for transpiling VHDL entities to Verilog modules
pub struct TranspileTool {
    base: BaseToolImpl,
    allowed_folders: Vec<String>,
}

impl TranspileTool {
    pub fn new(allowed_folders: Vec<String>) -> Self {
        let parameters = vec![
            ToolParameter {
                name: "vhdl_file".to_string(),
                param_type: "string".to_string(),
                description: "Path to the VHDL file to transpile".to_string(),
                required: true,
                default: None,
            },
            ToolParameter {
                name: "output_file".to_string(),
                param_type: "string".to_string(),
                description: "Path to the output Verilog file (optional)".to_string(),
                required: false,
                default: None,
            },
        ];

        let base = BaseToolImpl::new(
            "transpile_vhdl_to_verilog".to_string(),
            "Transpile VHDL entity to Verilog module. Extracts entity declaration and converts it to a Verilog module with matching ports.".to_string(),
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
}

impl Tool for TranspileTool {
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
        let vhdl_file = arguments
            .get("vhdl_file")
            .and_then(|v| v.as_str())
            .context("Missing 'vhdl_file' argument")?;

        let output_file = arguments
            .get("output_file")
            .and_then(|v| v.as_str());

        let vhdl_path = Path::new(vhdl_file);

        // Check if path is allowed
        if !self.is_path_allowed(vhdl_path) {
            return Err(anyhow::anyhow!(
                "Access denied: '{}' is not in allowed folders",
                vhdl_file
            ));
        }

        // Parse VHDL using AST parser
        tracing::info!("Parsing VHDL file: {}", vhdl_file);
        let mut parser = ASTVHDLParser::from_file(vhdl_path)
            .context(format!("Failed to parse VHDL file: {}", vhdl_file))?;

        let entities = parser.parse_entities()
            .context("Failed to extract entities from VHDL")?;

        if entities.is_empty() {
            return Err(anyhow::anyhow!("No entities found in VHDL file"));
        }

        // Generate Verilog for all entities
        let generator = VerilogGenerator::new();
        let mut verilog_output = String::new();

        for entity in &entities {
            tracing::info!("Generating Verilog for entity: {}", entity.name);
            let verilog = generator.generate(entity)
                .context(format!("Failed to generate Verilog for entity: {}", entity.name))?;

            verilog_output.push_str(&verilog);
            verilog_output.push('\n');
        }

        // Write to file if output path provided
        if let Some(output_path) = output_file {
            let out_path = Path::new(output_path);

            // Check output path is allowed
            if !self.is_path_allowed(out_path.parent().unwrap_or(Path::new("."))) {
                return Err(anyhow::anyhow!(
                    "Access denied: output path '{}' is not in allowed folders",
                    output_path
                ));
            }

            std::fs::write(out_path, &verilog_output)
                .context(format!("Failed to write Verilog to: {}", output_path))?;

            tracing::info!("Verilog written to: {}", output_path);

            Ok(format!(
                "Successfully transpiled {} entity(ies) from '{}' to '{}'\n\nGenerated Verilog:\n{}",
                entities.len(),
                vhdl_file,
                output_path,
                verilog_output
            ))
        } else {
            Ok(format!(
                "Successfully transpiled {} entity(ies) from '{}'\n\nGenerated Verilog:\n{}",
                entities.len(),
                vhdl_file,
                verilog_output
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_transpile_tool() {
        let vhdl_content = r#"
        entity counter is
            port(
                clk    : in  std_logic;
                reset  : in  std_logic;
                count  : out std_logic_vector(7 downto 0)
            );
        end entity counter;
        "#;

        // Create temp VHDL file
        let mut vhdl_file = NamedTempFile::new().unwrap();
        vhdl_file.write_all(vhdl_content.as_bytes()).unwrap();
        let vhdl_path = vhdl_file.path().to_str().unwrap();

        // Create tool with allowed folders (allow all)
        let tool = TranspileTool::new(vec![]);

        // Execute
        let args = serde_json::json!({
            "vhdl_file": vhdl_path
        });

        let result = tool.execute(&args).unwrap();

        println!("Result:\n{}", result);

        assert!(result.contains("Successfully transpiled"));
        assert!(result.contains("module counter"));
        assert!(result.contains("input wire clk"));
        assert!(result.contains("output wire [7:0] count"));
    }
}