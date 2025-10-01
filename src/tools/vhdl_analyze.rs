use anyhow::{Context, Result};
use std::path::Path;

use crate::parser::ASTVHDLParser;
use crate::tools::{BaseToolImpl, Tool, ToolParameter, ToolSchema};

/// Tool for analyzing VHDL files and extracting information
pub struct VHDLAnalyzeTool {
    base: BaseToolImpl,
    allowed_folders: Vec<String>,
}

impl VHDLAnalyzeTool {
    pub fn new(allowed_folders: Vec<String>) -> Self {
        let parameters = vec![
            ToolParameter {
                name: "vhdl_file".to_string(),
                param_type: "string".to_string(),
                description: "Path to the VHDL file to analyze".to_string(),
                required: true,
                default: None,
            },
            ToolParameter {
                name: "analysis_type".to_string(),
                param_type: "string".to_string(),
                description: "Type of analysis: 'entities', 'ports', 'signals', 'processes', or 'all'".to_string(),
                required: false,
                default: Some(serde_json::json!("all")),
            },
        ];

        let base = BaseToolImpl::new(
            "analyze_vhdl".to_string(),
            "Analyze VHDL files to extract entities, ports, signals, processes, and other structural information.".to_string(),
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

impl Tool for VHDLAnalyzeTool {
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

        let analysis_type = arguments
            .get("analysis_type")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        let vhdl_path = Path::new(vhdl_file);

        // Check if path is allowed
        if !self.is_path_allowed(vhdl_path) {
            return Err(anyhow::anyhow!(
                "Access denied: '{}' is not in allowed folders",
                vhdl_file
            ));
        }

        // Parse VHDL using AST parser
        tracing::info!("Analyzing VHDL file: {}", vhdl_file);
        let mut parser = ASTVHDLParser::from_file(vhdl_path)
            .context(format!("Failed to parse VHDL file: {}", vhdl_file))?;

        let entities = parser.parse_entities()
            .context("Failed to extract entities from VHDL")?;

        if entities.is_empty() {
            return Ok("No entities found in VHDL file".to_string());
        }

        let mut result = String::new();

        match analysis_type {
            "entities" => {
                result.push_str(&format!("Found {} entities:\n\n", entities.len()));
                for entity in &entities {
                    result.push_str(&format!("Entity: {}\n", entity.name));
                    result.push_str(&format!("  Ports: {}\n", entity.ports.len()));
                    result.push_str(&format!("  Generics: {}\n", entity.generics.len()));
                    if let Some(arch) = &entity.architecture {
                        result.push_str(&format!("  Architecture: {}\n", arch.name));
                    }
                    result.push('\n');
                }
            }
            "ports" => {
                result.push_str("Port Analysis:\n\n");
                for entity in &entities {
                    result.push_str(&format!("Entity: {}\n", entity.name));
                    if entity.ports.is_empty() {
                        result.push_str("  No ports\n");
                    } else {
                        for port in &entity.ports {
                            result.push_str(&format!("  {} : {:?} {:?}\n", 
                                port.name, port.direction, port.port_type));
                        }
                    }
                    result.push('\n');
                }
            }
            "signals" => {
                result.push_str("Signal Analysis:\n\n");
                for entity in &entities {
                    result.push_str(&format!("Entity: {}\n", entity.name));
                    if let Some(arch) = &entity.architecture {
                        result.push_str(&format!("  Architecture: {}\n", arch.name));
                        if arch.signals.is_empty() {
                            result.push_str("    No signals\n");
                        } else {
                            for signal in &arch.signals {
                                result.push_str(&format!("    {} : {:?}\n", 
                                    signal.name, signal.signal_type));
                            }
                        }
                    } else {
                        result.push_str("  No architecture found\n");
                    }
                    result.push('\n');
                }
            }
            "processes" => {
                result.push_str("Process Analysis:\n\n");
                for entity in &entities {
                    result.push_str(&format!("Entity: {}\n", entity.name));
                    if let Some(arch) = &entity.architecture {
                        result.push_str(&format!("  Architecture: {}\n", arch.name));
                        if arch.processes.is_empty() {
                            result.push_str("    No processes\n");
                        } else {
                        for (i, process) in arch.processes.iter().enumerate() {
                            let default_label = format!("process_{}", i);
                            let label = process.label.as_ref()
                                .map(|l| l.as_str())
                                .unwrap_or(&default_label);
                                result.push_str(&format!("    Process: {}\n", label));
                                result.push_str(&format!("      Sensitivity: {:?}\n", process.sensitivity_list));
                                result.push_str(&format!("      Body length: {} chars\n", process.body.len()));
                            }
                        }
                    } else {
                        result.push_str("  No architecture found\n");
                    }
                    result.push('\n');
                }
            }
            "all" | _ => {
                result.push_str(&format!("Complete VHDL Analysis for: {}\n", vhdl_file));
                result.push_str(&format!("Found {} entities\n\n", entities.len()));

                for entity in &entities {
                    result.push_str(&format!("Entity: {}\n", entity.name));
                    result.push_str(&format!("  Generics: {}\n", entity.generics.len()));
                    for generic in &entity.generics {
                        result.push_str(&format!("    {} : {}", generic.name, generic.generic_type));
                        if let Some(default) = &generic.default_value {
                            result.push_str(&format!(" := {}", default));
                        }
                        result.push('\n');
                    }
                    
                    result.push_str(&format!("  Ports: {}\n", entity.ports.len()));
                    for port in &entity.ports {
                        result.push_str(&format!("    {} : {:?} {:?}\n", 
                            port.name, port.direction, port.port_type));
                    }

                    if let Some(arch) = &entity.architecture {
                        result.push_str(&format!("  Architecture: {}\n", arch.name));
                        result.push_str(&format!("    Signals: {}\n", arch.signals.len()));
                        for signal in &arch.signals {
                            result.push_str(&format!("      {} : {:?}\n", 
                                signal.name, signal.signal_type));
                        }
                        
                        result.push_str(&format!("    Processes: {}\n", arch.processes.len()));
                        for (i, process) in arch.processes.iter().enumerate() {
                            let default_label = format!("process_{}", i);
                            let label = process.label.as_ref()
                                .map(|l| l.as_str())
                                .unwrap_or(&default_label);
                            result.push_str(&format!("      {}: sensitivity {:?}\n", 
                                label, process.sensitivity_list));
                        }
                        
                        result.push_str(&format!("    Concurrent statements: {}\n", arch.concurrent_statements.len()));
                    } else {
                        result.push_str("  No architecture found\n");
                    }
                    result.push('\n');
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_vhdl_analyze_tool() {
        let vhdl_content = r#"
        entity counter is
            generic(
                WIDTH : integer := 8
            );
            port(
                clk    : in  std_logic;
                reset  : in  std_logic;
                count  : out std_logic_vector(WIDTH-1 downto 0)
            );
        end entity counter;
        "#;

        // Create temp VHDL file
        let mut vhdl_file = NamedTempFile::new().unwrap();
        vhdl_file.write_all(vhdl_content.as_bytes()).unwrap();
        let vhdl_path = vhdl_file.path().to_str().unwrap();

        // Create tool with allowed folders (allow all)
        let tool = VHDLAnalyzeTool::new(vec![]);

        // Test entities analysis
        let args = serde_json::json!({
            "vhdl_file": vhdl_path,
            "analysis_type": "entities"
        });

        let result = tool.execute(&args).unwrap();
        assert!(result.contains("Entity: counter"));
        assert!(result.contains("Ports: 3"));
    }
}
