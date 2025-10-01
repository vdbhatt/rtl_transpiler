use crate::ir::{Entity, Architecture, Port, PortDirection, VHDLType};
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

        // Collect all signals assigned in processes (need to be reg)
        let procedural_signals = self.collect_procedural_signals(entity);

        // Module header with ports
        output.push_str(&self.generate_module_header(entity, &procedural_signals)?);

        // Module body (empty for now, just entity conversion)
        output.push_str(&self.generate_module_body(entity)?);

        // Module footer
        output.push_str("endmodule\n");

        Ok(output)
    }

    fn collect_procedural_signals(&self, entity: &Entity) -> std::collections::HashSet<String> {
        let mut procedural_signals = std::collections::HashSet::new();
        
        if let Some(arch) = &entity.architecture {
            for process in &arch.processes {
                // Extract signal names assigned in process body
                for line in process.body.lines() {
                    let trimmed = line.trim();
                    if let Some(pos) = trimmed.find(" <=") {
                        let signal_name = trimmed[..pos].trim();
                        procedural_signals.insert(signal_name.to_string());
                    }
                }
            }
        }
        
        procedural_signals
    }

    fn generate_module_header(&self, entity: &Entity, procedural_signals: &std::collections::HashSet<String>) -> Result<String> {
        let mut output = String::new();

        // Start module declaration
        output.push_str(&format!("module {} (\n", entity.name));

        // Generate port list
        if !entity.ports.is_empty() {
            for (i, port) in entity.ports.iter().enumerate() {
                output.push_str(&self.indent);
                
                // Check if this port is assigned in a process and needs to be reg
                let is_procedural = procedural_signals.contains(&port.name);
                let direction = port.direction.to_verilog();
                let mut verilog_type = port.port_type.to_verilog();
                
                // If output port is assigned in process, change wire to reg
                if is_procedural && matches!(port.direction, PortDirection::Out | PortDirection::Buffer) {
                    verilog_type = verilog_type.replace("wire", "reg");
                }
                
                output.push_str(&format!("{} {} {}", direction, verilog_type, port.name));

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

        // Generate signal declarations (internal signals are always reg when assigned in processes)
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
        let triple_indent = format!("{}{}{}", self.indent, self.indent, self.indent);
        let mut in_case = false;
        let mut case_branch_has_stmt = false;
        let mut indent_level = 0; // Track nesting level for proper indentation

        for line in vhdl_body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }

            let mut verilog_line = trimmed.to_string();

            // Skip lines with rising_edge/falling_edge as they're handled in sensitivity list
            if verilog_line.starts_with("if") && (verilog_line.contains("rising_edge") || verilog_line.contains("falling_edge")) {
                continue;
            }

            // Convert hex literals first (before other conversions)
            // x"0" -> 4'h0, x"1" -> 4'h1, x"FF" -> 8'hFF
            let hex_re = regex::Regex::new(r#"x"([0-9A-Fa-f]+)""#).unwrap();
            verilog_line = hex_re.replace_all(&verilog_line, |caps: &regex::Captures| {
                let hex_value = &caps[1];
                let bit_width = hex_value.len() * 4; // Each hex digit is 4 bits
                format!("{}'h{}", bit_width, hex_value)
            }).to_string();

            // Convert bit literals and comparison operators
            // Handle '=' comparisons with bit literals (with or without spaces)
            verilog_line = verilog_line.replace("='1'", " == 1'b1");
            verilog_line = verilog_line.replace("='0'", " == 1'b0");
            verilog_line = verilog_line.replace(" = '1'", " == 1'b1");
            verilog_line = verilog_line.replace(" = '0'", " == 1'b0");

            // Convert remaining bit literals
            verilog_line = verilog_line.replace("'1'", "1'b1");
            verilog_line = verilog_line.replace("'0'", "1'b0");

            // Convert others => value
            verilog_line = verilog_line.replace("(others => 1'b0)", "8'b0");
            verilog_line = verilog_line.replace("(others => 1'b1)", "8'b1");

            // Convert case statements
            if verilog_line.starts_with("case ") && verilog_line.contains(" is") {
                verilog_line = verilog_line.replace(" is", "");
                verilog_line = verilog_line.replacen("case ", "case (", 1);
                if !verilog_line.ends_with(")") {
                    verilog_line.push(')');
                }
                in_case = true;
                case_branch_has_stmt = false;
            } else if verilog_line.starts_with("when ") {
                // Close previous case branch if it had statements
                if in_case && case_branch_has_stmt {
                    output.push_str(&format!("{}end\n", &double_indent));
                    case_branch_has_stmt = false;
                }

                // "when "00" =>" -> "2'b00: begin" or "when IDLE =>" -> "IDLE: begin"
                if let Some(value_end) = verilog_line.find(" =>") {
                    let value_part = &verilog_line[5..value_end]; // Skip "when "
                    let value = value_part.trim();
                    if value == "others" {
                        verilog_line = "default: begin".to_string();
                    } else if value.starts_with('"') && value.ends_with('"') {
                        // Binary literal: "00" -> 2'b00: begin
                        let binary = value.trim_matches('"');
                        let width = binary.len();
                        verilog_line = format!("{}'b{}: begin", width, binary);
                    } else {
                        // Enum or identifier: IDLE -> IDLE: begin
                        verilog_line = format!("{}: begin", value);
                    }
                }
            } else if verilog_line == "end case" || verilog_line == "end case;" {
                // Close last case branch
                if in_case && case_branch_has_stmt {
                    output.push_str(&format!("{}end\n", &double_indent));
                }
                verilog_line = "endcase".to_string();
                in_case = false;
                case_branch_has_stmt = false;
            }

            // Handle if/elsif/else/then keywords
            let is_if = verilog_line.starts_with("if ") || verilog_line.starts_with("if(");
            let is_elsif = verilog_line.starts_with("elsif ") || verilog_line.starts_with("elsif(");
            let is_else = verilog_line.trim() == "else";
            let is_endif = verilog_line == "end if" || verilog_line == "end if;";

            if is_if {
                // "if(reset == 1'b1) then" -> "if (reset == 1'b1) begin"
                // First, add space after 'if' if needed
                if verilog_line.starts_with("if(") {
                    verilog_line = verilog_line.replacen("if(", "if (", 1);
                }
                // Remove 'then' and add 'begin'
                if verilog_line.contains(" then") {
                    verilog_line = verilog_line.replace(" then", ") begin");
                } else if verilog_line.contains(" begin") {
                    // Already has begin, ensure proper parentheses
                    if !verilog_line.contains(')') && verilog_line.contains('(') {
                        verilog_line = verilog_line.replace(" begin", ") begin");
                    }
                } else {
                    // No 'then' or 'begin', add them
                    if verilog_line.contains('(') && !verilog_line.contains(')') {
                        verilog_line.push_str(") begin");
                    } else if !verilog_line.ends_with("begin") {
                        verilog_line.push_str(" begin");
                    }
                }
            } else if is_elsif {
                // "elsif rising_edge(clk) then" -> "end else begin"
                // (rising_edge is already handled in sensitivity list)
                if verilog_line.contains("rising_edge") || verilog_line.contains("falling_edge") {
                    verilog_line = "end else begin".to_string();
                } else {
                    verilog_line = verilog_line.replacen("elsif ", "end else if (", 1);
                    verilog_line = verilog_line.replace(" then", ") begin");
                    if !verilog_line.contains(") begin") {
                        verilog_line.push_str(" begin");
                    }
                }
            } else if is_else {
                verilog_line = "end else begin".to_string();
            } else if is_endif {
                verilog_line = "end".to_string();
            }

            // Convert logical operators
            verilog_line = verilog_line.replace(" and ", " & ");
            verilog_line = verilog_line.replace(" or ", " | ");
            verilog_line = verilog_line.replace(" xor ", " ^ ");
            verilog_line = verilog_line.replace(" not ", " ~");

            // Convert type conversions - remove VHDL type casts
            // Handle nested type conversions
            verilog_line = verilog_line.replace("std_logic_vector(unsigned(", "");
            verilog_line = verilog_line.replace("std_logic_vector(signed(", "");
            verilog_line = verilog_line.replace("std_logic_vector(", "");
            verilog_line = verilog_line.replace("unsigned(", "");
            verilog_line = verilog_line.replace("signed(", "");
            verilog_line = verilog_line.replace("to_unsigned(", "");
            verilog_line = verilog_line.replace("to_signed(", "");
            verilog_line = verilog_line.replace("to_integer(", "");

            // Remove extra closing parens from type conversions
            let mut paren_diff = verilog_line.matches(')').count() as i32 - verilog_line.matches('(').count() as i32;
            while paren_diff > 0 {
                if let Some(pos) = verilog_line.rfind(')') {
                    verilog_line.remove(pos);
                    paren_diff -= 1;
                } else {
                    break;
                }
            }

            // Don't add semicolons to control flow keywords
            let is_control_flow = verilog_line.contains("begin") ||
                                   (verilog_line.starts_with("end") && !verilog_line.starts_with("endcase")) ||
                                   verilog_line == "else" ||
                                   verilog_line.ends_with(":") || // case labels
                                   verilog_line.starts_with("case") ||
                                   verilog_line == "endcase";

            // Adjust indent level based on control flow
            if verilog_line.starts_with("end") {
                if indent_level > 0 {
                    indent_level -= 1;
                }
            }

            // Choose appropriate indentation
            let current_indent = match indent_level {
                0 => double_indent.clone(),
                1 => triple_indent.clone(),
                _ => format!("{}{}", triple_indent, self.indent.repeat(indent_level - 1)),
            };

            output.push_str(&current_indent);
            output.push_str(&verilog_line);

            // Add semicolon to assignments only
            if !is_control_flow && !verilog_line.ends_with(';') {
                output.push(';');
            }

            output.push('\n');

            // Increase indent level after begin
            if verilog_line.contains("begin") {
                indent_level += 1;
            }

            // Track if we've added a statement to a case branch
            if in_case && !is_control_flow && verilog_line.contains(" <= ") {
                case_branch_has_stmt = true;
            }
        }

        Ok(output)
    }

    fn convert_concurrent_statement(&self, stmt: &str) -> Result<String> {
        // Convert VHDL concurrent statements to Verilog assign statements
        let mut verilog = stmt.to_string();

        // Remove type conversions
        verilog = verilog.replace("std_logic_vector(", "");
        verilog = verilog.replace("unsigned(", "");
        verilog = verilog.replace("signed(", "");

        // Remove extra closing parens from type conversions
        let mut paren_diff = verilog.matches(')').count() as i32 - verilog.matches('(').count() as i32;
        while paren_diff > 0 {
            if let Some(pos) = verilog.rfind(')') {
                verilog.remove(pos);
                paren_diff -= 1;
            } else {
                break;
            }
        }

        // Handle with...select statements
        if verilog.contains("with ") && verilog.contains(" select") {
            return Ok(format!("// TODO: Convert VHDL 'with...select' statement:\n    // {}",
                verilog.replace("\n", "\n    // ")));
        }

        // Handle conditional assignments: "signal <= '1' when condition else '0'"
        if verilog.contains(" when ") && verilog.contains(" else ") {
            // Parse: "target <= value1 when condition else value2"
            let parts: Vec<&str> = verilog.split(" <= ").collect();
            if parts.len() == 2 {
                let target = parts[0].trim();
                let rest = parts[1];

                if let Some(when_pos) = rest.find(" when ") {
                    if let Some(else_pos) = rest.find(" else ") {
                        let value1 = rest[..when_pos].trim();
                        let condition = rest[when_pos+6..else_pos].trim();
                        let value2 = rest[else_pos+6..].trim();

                        // Convert to ternary: target = condition ? value1 : value2
                        let mut cond_conv = condition.to_string();
                        cond_conv = cond_conv.replace(" = ", " == ");
                        // Convert string literals in conditions like "00000000" to 8'b00000000
                        if cond_conv.contains('"') {
                            // Simple string replacement for binary literals
                            let mut result = String::new();
                            let mut chars = cond_conv.chars().peekable();
                            
                            while let Some(ch) = chars.next() {
                                if ch == '"' {
                                    // Found start of string literal
                                    let mut binary = String::new();
                                    while let Some(&next_ch) = chars.peek() {
                                        if next_ch == '"' {
                                            chars.next(); // consume the closing quote
                                            break;
                                        }
                                        if next_ch == '0' || next_ch == '1' {
                                            binary.push(chars.next().unwrap());
                                        } else {
                                            chars.next();
                                        }
                                    }
                                    
                                    if !binary.is_empty() && binary.chars().all(|c| c == '0' || c == '1') {
                                        result.push_str(&format!("{}'b{}", binary.len(), binary));
                                    } else {
                                        result.push('"');
                                        result.push_str(&binary);
                                        result.push('"');
                                    }
                                } else {
                                    result.push(ch);
                                }
                            }
                            cond_conv = result;
                        }

                        let val1_conv = value1.replace("'1'", "1'b1").replace("'0'", "1'b0");
                        let val2_conv = value2.replace("'1'", "1'b1").replace("'0'", "1'b0");

                        verilog = format!("assign {} = {} ? {} : {};", target, cond_conv, val1_conv, val2_conv);
                        return Ok(verilog);
                    }
                }
            }
        }

        verilog = verilog.replace(" <= ", " = ");  // Concurrent assignment
        verilog = verilog.replace("'1'", "1'b1");
        verilog = verilog.replace("'0'", "1'b0");

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