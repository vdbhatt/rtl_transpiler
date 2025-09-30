use crate::ir::{Entity, Architecture};
use anyhow::Result;

/// Generate Verilog module from Entity IR
pub struct VerilogGenerator {
    indent: String,
}

impl VerilogGenerator {
    pub fn new() -> Self {
        Self {
            indent: "    ".to_string(),
        }
    }

    pub fn with_indent(indent: String) -> Self {
        Self { indent }
    }

    /// Generate complete Verilog module from entity
    pub fn generate(&self, entity: &Entity) -> Result<String> {
        let mut output = String::new();

        // Module header with ports
        output.push_str(&self.generate_module_header(entity)?);

        // Module body (empty for now, just entity conversion)
        output.push_str(&self.generate_module_body(entity)?);

        // Module footer
        output.push_str("endmodule\n");

        Ok(output)
    }

    fn generate_module_header(&self, entity: &Entity) -> Result<String> {
        let mut output = String::new();

        // Start module declaration
        output.push_str(&format!("module {} (\n", entity.name));

        // Generate port list
        if !entity.ports.is_empty() {
            for (i, port) in entity.ports.iter().enumerate() {
                output.push_str(&self.indent);
                output.push_str(&port.to_verilog());

                // Add comma if not last port
                if i < entity.ports.len() - 1 {
                    output.push(',');
                }
                output.push('\n');
            }
        }

        output.push_str(");\n");

        Ok(output)
    }

    fn generate_module_body(&self, entity: &Entity) -> Result<String> {
        let mut output = String::new();

        // If there's an architecture, generate the implementation
        if let Some(arch) = &entity.architecture {
            output.push_str(&self.generate_architecture_body(arch)?);
        }

        Ok(output)
    }

    fn generate_architecture_body(&self, arch: &Architecture) -> Result<String> {
        let mut output = String::new();

        // Generate signal declarations
        if !arch.signals.is_empty() {
            output.push('\n');
            for signal in &arch.signals {
                output.push_str(&self.indent);
                let verilog_type = signal.signal_type.to_verilog();
                output.push_str(&format!("{} {};\n", verilog_type.replace("wire ", "reg "), signal.name));
            }
        }

        // Generate processes as always blocks
        for process in &arch.processes {
            output.push('\n');
            output.push_str(&self.generate_process(process)?);
        }

        // Generate concurrent statements as assign statements
        for stmt in &arch.concurrent_statements {
            output.push('\n');
            output.push_str(&self.indent);
            output.push_str(&self.convert_concurrent_statement(stmt)?);
            output.push('\n');
        }

        Ok(output)
    }

    fn generate_process(&self, process: &crate::ir::Process) -> Result<String> {
        let mut output = String::new();

        // Determine if it's sequential or combinational based on sensitivity list
        let is_sequential = process.sensitivity_list.iter()
            .any(|s| s.contains("clk") || s.contains("clock") || s.contains("rising_edge") || s.contains("falling_edge"));

        output.push_str(&self.indent);

        if is_sequential {
            // Sequential logic - always @(posedge clk)
            let mut edge_signals = Vec::new();
            for sig in &process.sensitivity_list {
                if sig.contains("clk") || sig.contains("clock") {
                    edge_signals.push(format!("posedge {}", sig));
                } else if sig.contains("reset") || sig.contains("rst") {
                    // Check if active high or low reset
                    if process.body.contains(&format!("{} = '1'", sig)) || process.body.contains(&format!("{} = \"1\"", sig)) {
                        edge_signals.push(format!("posedge {}", sig));
                    } else {
                        edge_signals.push(format!("negedge {}", sig));
                    }
                }
            }

            if edge_signals.is_empty() {
                edge_signals.push("posedge clk".to_string());
            }

            output.push_str(&format!("always @({}) begin\n", edge_signals.join(" or ")));
        } else {
            // Combinational logic - always @(*)
            output.push_str("always @(*) begin\n");
        }

        // Convert VHDL process body to Verilog
        let verilog_body = self.convert_process_body(&process.body)?;
        output.push_str(&verilog_body);

        output.push_str(&self.indent);
        output.push_str("end\n");

        Ok(output)
    }

    fn convert_process_body(&self, vhdl_body: &str) -> Result<String> {
        let mut output = String::new();
        let double_indent = format!("{}{}", self.indent, self.indent);

        // Simple conversion - this is a starting point
        // In a full implementation, this would need proper VHDL parsing

        for line in vhdl_body.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("--") {
                continue;
            }

            let mut verilog_line = line.to_string();

            // Convert VHDL syntax to Verilog
            verilog_line = verilog_line.replace(" := ", " = ");          // Assignment
            verilog_line = verilog_line.replace(" <= ", " <= ");         // Non-blocking (already correct)
            verilog_line = verilog_line.replace("'1'", "1'b1");          // Bit literals
            verilog_line = verilog_line.replace("'0'", "1'b0");
            verilog_line = verilog_line.replace("\"1\"", "1'b1");
            verilog_line = verilog_line.replace("\"0\"", "1'b0");
            verilog_line = verilog_line.replace(" then", "");            // Remove 'then'
            verilog_line = verilog_line.replace("elsif", "else if");     // elsif -> else if
            verilog_line = verilog_line.replace("end if", "end");        // end if -> end
            verilog_line = verilog_line.replace("rising_edge(", "posedge "); // rising_edge
            verilog_line = verilog_line.replace(")", "");                // Remove closing paren from rising_edge

            // Convert if statements
            if verilog_line.starts_with("if ") {
                verilog_line = verilog_line.replace("if ", "if (") + ")";
            }

            // Handle others
            verilog_line = verilog_line.replace("(others => '0')", "0");
            verilog_line = verilog_line.replace("(others => '1')", "~0");

            output.push_str(&double_indent);
            output.push_str(&verilog_line);
            if !verilog_line.ends_with(';') && !verilog_line.ends_with("begin") && !verilog_line.trim().is_empty() {
                output.push(';');
            }
            output.push('\n');
        }

        Ok(output)
    }

    fn convert_concurrent_statement(&self, stmt: &str) -> Result<String> {
        // Convert VHDL concurrent statements to Verilog assign statements
        let mut verilog = stmt.to_string();

        verilog = verilog.replace(" <= ", " = ");  // Concurrent assignment

        // If it doesn't look like an assignment, wrap it in assign
        if verilog.contains(" = ") && !verilog.starts_with("assign ") {
            verilog = format!("assign {};", verilog.trim_end_matches(';'));
        }

        Ok(verilog)
    }

    /// Generate port declarations in Verilog-2001 style (separate from module header)
    pub fn generate_port_declarations(&self, entity: &Entity) -> Result<String> {
        let mut output = String::new();

        for port in &entity.ports {
            output.push_str(&self.indent);
            output.push_str(&port.to_verilog());
            output.push_str(";\n");
        }

        Ok(output)
    }
}

impl Default for VerilogGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{PortDirection, VHDLType, VectorRange};

    #[test]
    fn test_generate_simple_module() {
        let mut entity = Entity::new("counter".to_string());
        entity.add_port(Port::new(
            "clk".to_string(),
            PortDirection::In,
            VHDLType::StdLogic,
        ));
        entity.add_port(Port::new(
            "reset".to_string(),
            PortDirection::In,
            VHDLType::StdLogic,
        ));
        entity.add_port(Port::new(
            "count".to_string(),
            PortDirection::Out,
            VHDLType::StdLogicVector(VectorRange {
                left: 7,
                right: 0,
                downto: true,
            }),
        ));

        let generator = VerilogGenerator::new();
        let verilog = generator.generate(&entity).unwrap();

        println!("Generated Verilog:\n{}", verilog);

        assert!(verilog.contains("module counter"));
        assert!(verilog.contains("input wire clk"));
        assert!(verilog.contains("input wire reset"));
        assert!(verilog.contains("output wire [7:0] count"));
        assert!(verilog.contains("endmodule"));
    }

    #[test]
    fn test_generate_with_multiple_types() {
        let mut entity = Entity::new("test".to_string());
        entity.add_port(Port::new(
            "int_signal".to_string(),
            PortDirection::In,
            VHDLType::Integer,
        ));
        entity.add_port(Port::new(
            "bit_signal".to_string(),
            PortDirection::Out,
            VHDLType::Bit,
        ));

        let generator = VerilogGenerator::new();
        let verilog = generator.generate(&entity).unwrap();

        assert!(verilog.contains("input wire signed [31:0] int_signal"));
        assert!(verilog.contains("output wire bit_signal"));
    }
}