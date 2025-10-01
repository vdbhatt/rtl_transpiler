use crate::ir::{Entity, Architecture, Port, PortDirection, VHDLType};
use anyhow::Result;

/// Generate SystemVerilog 2012 module from Entity IR
/// This generator produces synthesizable SystemVerilog code following IEEE 1800-2012
pub struct SystemVerilogGenerator {
    indent: String,
}

impl SystemVerilogGenerator {
    pub fn new() -> Self {
        Self {
            indent: "    ".to_string(),
        }
    }

    pub fn with_indent(indent: String) -> Self {
        Self { indent }
    }

    /// Generate complete SystemVerilog module from entity
    pub fn generate(&self, entity: &Entity) -> Result<String> {
        let mut output = String::new();

        // Module header with ports in SystemVerilog ANSI-style
        output.push_str(&self.generate_module_header(entity)?);

        // Module body
        output.push_str(&self.generate_module_body(entity)?);

        // Module footer
        output.push_str("endmodule\n");

        Ok(output)
    }

    fn generate_module_header(&self, entity: &Entity) -> Result<String> {
        let mut output = String::new();

        // Start module declaration
        output.push_str(&format!("module {} (\n", entity.name));

        // Generate port list in ANSI style (SystemVerilog)
        if !entity.ports.is_empty() {
            for (i, port) in entity.ports.iter().enumerate() {
                output.push_str(&self.indent);
                
                let direction = port.direction.to_systemverilog();
                let sv_type = port.port_type.to_systemverilog();
                
                // SystemVerilog ANSI-style: direction type name
                output.push_str(&format!("{} {} {}", direction, sv_type, port.name));

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

        // Generate signal declarations using 'logic' type
        if !arch.signals.is_empty() {
            output.push('\n');
            for signal in &arch.signals {
                output.push_str(&self.indent);
                let sv_type = signal.signal_type.to_systemverilog();
                output.push_str(&format!("{} {};\n", sv_type, signal.name));
            }
        }

        // Generate processes as always_comb or always_ff blocks
        for process in &arch.processes {
            output.push('\n');
            output.push_str(&self.generate_process(process)?);
        }

        // Generate concurrent statements as continuous assignments
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
            // Sequential logic - always_ff @(posedge clk)
            let mut edge_signals = Vec::new();
            let mut has_async_reset = false;
            let mut async_reset_edge = String::new();
            
            for sig in &process.sensitivity_list {
                if sig.contains("clk") || sig.contains("clock") {
                    edge_signals.push(format!("posedge {}", sig));
                } else if sig.contains("reset") || sig.contains("rst") {
                    // Check if active high or low reset
                    if process.body.contains(&format!("{} = '1'", sig)) || process.body.contains(&format!("{} = \"1\"", sig)) {
                        async_reset_edge = format!("posedge {}", sig);
                    } else {
                        async_reset_edge = format!("negedge {}", sig);
                    }
                    has_async_reset = true;
                }
            }

            if edge_signals.is_empty() {
                edge_signals.push("posedge clk".to_string());
            }

            if has_async_reset {
                edge_signals.push(async_reset_edge);
                output.push_str(&format!("always_ff @({}) begin\n", edge_signals.join(" or ")));
            } else {
                output.push_str(&format!("always_ff @({}) begin\n", edge_signals.join(" or ")));
            }
        } else {
            // Combinational logic - always_comb
            output.push_str("always_comb begin\n");
        }

        // Convert VHDL process body to SystemVerilog
        let sv_body = self.convert_process_body(&process.body)?;
        output.push_str(&sv_body);

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
        let mut indent_level = 0;

        for line in vhdl_body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }

            let mut sv_line = trimmed.to_string();

            // Skip lines with rising_edge/falling_edge as they're handled in sensitivity list
            if sv_line.starts_with("if") && (sv_line.contains("rising_edge") || sv_line.contains("falling_edge")) {
                continue;
            }

            // Convert hex literals: x"0" -> 4'h0, x"FF" -> 8'hFF
            let hex_re = regex::Regex::new(r#"x"([0-9A-Fa-f]+)""#).unwrap();
            sv_line = hex_re.replace_all(&sv_line, |caps: &regex::Captures| {
                let hex_value = &caps[1];
                let bit_width = hex_value.len() * 4;
                format!("{}'h{}", bit_width, hex_value)
            }).to_string();

            // Convert bit literals and comparison operators
            sv_line = sv_line.replace("='1'", " == 1'b1");
            sv_line = sv_line.replace("='0'", " == 1'b0");
            sv_line = sv_line.replace(" = '1'", " == 1'b1");
            sv_line = sv_line.replace(" = '0'", " == 1'b0");
            sv_line = sv_line.replace("'1'", "1'b1");
            sv_line = sv_line.replace("'0'", "1'b0");

            // Convert others => value to '0 (SystemVerilog replication)
            if sv_line.contains("(others =>") {
                // Extract the replicated value
                if sv_line.contains("1'b0") {
                    sv_line = sv_line.replace("(others => 1'b0)", "'0");
                } else if sv_line.contains("1'b1") {
                    sv_line = sv_line.replace("(others => 1'b1)", "'1");
                }
            }

            // Convert case statements to unique case (for synthesis)
            if sv_line.starts_with("case ") && sv_line.contains(" is") {
                sv_line = sv_line.replace(" is", "");
                sv_line = sv_line.replacen("case ", "unique case (", 1);
                if !sv_line.ends_with(")") {
                    sv_line.push(')');
                }
                in_case = true;
                case_branch_has_stmt = false;
            } else if sv_line.starts_with("when ") {
                // Close previous case branch if it had statements
                if in_case && case_branch_has_stmt {
                    output.push_str(&format!("{}end\n", &double_indent));
                    case_branch_has_stmt = false;
                }

                if let Some(value_end) = sv_line.find(" =>") {
                    let value_part = &sv_line[5..value_end];
                    let value = value_part.trim();
                    if value == "others" {
                        sv_line = "default: begin".to_string();
                    } else if value.starts_with('"') && value.ends_with('"') {
                        let binary = value.trim_matches('"');
                        let width = binary.len();
                        sv_line = format!("{}'b{}: begin", width, binary);
                    } else {
                        sv_line = format!("{}: begin", value);
                    }
                }
            } else if sv_line == "end case" || sv_line == "end case;" {
                if in_case && case_branch_has_stmt {
                    output.push_str(&format!("{}end\n", &double_indent));
                }
                sv_line = "endcase".to_string();
                in_case = false;
                case_branch_has_stmt = false;
            }

            // Handle if/elsif/else/then keywords
            let is_if = sv_line.starts_with("if ") || sv_line.starts_with("if(");
            let is_elsif = sv_line.starts_with("elsif ") || sv_line.starts_with("elsif(");
            let is_else = sv_line.trim() == "else";
            let is_endif = sv_line == "end if" || sv_line == "end if;";

            if is_if {
                if sv_line.starts_with("if(") {
                    sv_line = sv_line.replacen("if(", "if (", 1);
                }
                if sv_line.contains(" then") {
                    sv_line = sv_line.replace(" then", " begin");
                    if !sv_line.contains(")") && sv_line.matches('(').count() > 0 {
                        let begin_pos = sv_line.find(" begin").unwrap();
                        sv_line.insert(begin_pos, ')');
                    }
                } else if sv_line.contains(" begin") {
                    if !sv_line.contains(')') && sv_line.contains('(') {
                        sv_line = sv_line.replace(" begin", ") begin");
                    }
                } else {
                    if sv_line.contains('(') && !sv_line.contains(')') {
                        sv_line.push_str(") begin");
                    } else if !sv_line.ends_with("begin") {
                        sv_line.push_str(" begin");
                    }
                }
            } else if is_elsif {
                if sv_line.contains("rising_edge") || sv_line.contains("falling_edge") {
                    sv_line = "end else begin".to_string();
                } else {
                    sv_line = sv_line.replacen("elsif ", "end else if (", 1);
                    sv_line = sv_line.replace(" then", ") begin");
                    if !sv_line.contains(") begin") {
                        sv_line.push_str(" begin");
                    }
                }
            } else if is_else {
                sv_line = "end else begin".to_string();
            } else if is_endif {
                sv_line = "end".to_string();
            }

            // Convert logical operators
            sv_line = sv_line.replace(" and ", " & ");
            sv_line = sv_line.replace(" or ", " | ");
            sv_line = sv_line.replace(" xor ", " ^ ");
            sv_line = sv_line.replace(" not ", " ~");

            // Convert type conversions - SystemVerilog doesn't need most of these
            sv_line = sv_line.replace("std_logic_vector(unsigned(", "");
            sv_line = sv_line.replace("std_logic_vector(signed(", "");
            sv_line = sv_line.replace("std_logic_vector(", "");
            sv_line = sv_line.replace("unsigned(", "");
            sv_line = sv_line.replace("signed(", "");
            sv_line = sv_line.replace("to_unsigned(", "");
            sv_line = sv_line.replace("to_signed(", "");
            sv_line = sv_line.replace("to_integer(", "int'(");

            // Remove extra closing parens
            let mut paren_diff = sv_line.matches(')').count() as i32 - sv_line.matches('(').count() as i32;
            while paren_diff > 0 {
                if let Some(pos) = sv_line.rfind(')') {
                    sv_line.remove(pos);
                    paren_diff -= 1;
                } else {
                    break;
                }
            }

            // Don't add semicolons to control flow keywords
            let is_control_flow = sv_line.contains("begin") ||
                                   (sv_line.starts_with("end") && !sv_line.starts_with("endcase")) ||
                                   sv_line == "else" ||
                                   sv_line.ends_with(":") ||
                                   sv_line.starts_with("unique case") ||
                                   sv_line.starts_with("case") ||
                                   sv_line == "endcase";

            // Adjust indent level
            if sv_line.starts_with("end") {
                if indent_level > 0 {
                    indent_level -= 1;
                }
            }

            let current_indent = match indent_level {
                0 => double_indent.clone(),
                1 => triple_indent.clone(),
                _ => format!("{}{}", triple_indent, self.indent.repeat(indent_level - 1)),
            };

            output.push_str(&current_indent);
            output.push_str(&sv_line);

            if !is_control_flow && !sv_line.ends_with(';') {
                output.push(';');
            }

            output.push('\n');

            if sv_line.contains("begin") {
                indent_level += 1;
            }

            if in_case && !is_control_flow && sv_line.contains(" <= ") {
                case_branch_has_stmt = true;
            }
        }

        Ok(output)
    }

    fn convert_concurrent_statement(&self, stmt: &str) -> Result<String> {
        let mut sv = stmt.to_string();

        // Remove type conversions
        sv = sv.replace("std_logic_vector(", "");
        sv = sv.replace("unsigned(", "");
        sv = sv.replace("signed(", "");

        let mut paren_diff = sv.matches(')').count() as i32 - sv.matches('(').count() as i32;
        while paren_diff > 0 {
            if let Some(pos) = sv.rfind(')') {
                sv.remove(pos);
                paren_diff -= 1;
            } else {
                break;
            }
        }

        // Handle with...select statements
        if sv.contains("with ") && sv.contains(" select") {
            return Ok(format!("// TODO: Convert VHDL 'with...select' to SystemVerilog case:\n    // {}",
                sv.replace("\n", "\n    // ")));
        }

        // Handle conditional assignments
        if sv.contains(" when ") && sv.contains(" else ") {
            let parts: Vec<&str> = sv.split(" <= ").collect();
            if parts.len() == 2 {
                let target = parts[0].trim();
                let rest = parts[1];

                if let Some(when_pos) = rest.find(" when ") {
                    if let Some(else_pos) = rest.find(" else ") {
                        let value1 = rest[..when_pos].trim();
                        let condition = rest[when_pos+6..else_pos].trim();
                        let value2 = rest[else_pos+6..].trim();

                        let mut cond_conv = condition.to_string();
                        cond_conv = cond_conv.replace(" = ", " == ");
                        
                        // Convert binary literals in conditions
                        if cond_conv.contains('"') {
                            let mut result = String::new();
                            let mut chars = cond_conv.chars().peekable();
                            
                            while let Some(ch) = chars.next() {
                                if ch == '"' {
                                    let mut binary = String::new();
                                    while let Some(&next_ch) = chars.peek() {
                                        if next_ch == '"' {
                                            chars.next();
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

                        sv = format!("assign {} = {} ? {} : {};", target, cond_conv, val1_conv, val2_conv);
                        return Ok(sv);
                    }
                }
            }
        }

        sv = sv.replace(" <= ", " = ");
        sv = sv.replace("'1'", "1'b1");
        sv = sv.replace("'0'", "1'b0");

        if sv.contains(" = ") && !sv.starts_with("assign ") {
            sv = format!("assign {};", sv.trim_end_matches(';'));
        }

        Ok(sv)
    }
}

impl Default for SystemVerilogGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Add SystemVerilog conversion methods to existing types
impl PortDirection {
    pub fn to_systemverilog(&self) -> &str {
        match self {
            PortDirection::In => "input",
            PortDirection::Out => "output",
            PortDirection::InOut => "inout",
            PortDirection::Buffer => "output",
        }
    }
}

impl VHDLType {
    pub fn to_systemverilog(&self) -> String {
        match self {
            VHDLType::StdLogic => "logic".to_string(),
            VHDLType::StdLogicVector(range) => format!("logic {}", range.to_systemverilog()),
            VHDLType::Integer => "logic signed [31:0]".to_string(),
            VHDLType::Natural => "logic [31:0]".to_string(),
            VHDLType::Positive => "logic [31:0]".to_string(),
            VHDLType::Boolean => "logic".to_string(),
            VHDLType::Bit => "logic".to_string(),
            VHDLType::BitVector(range) => format!("logic {}", range.to_systemverilog()),
            VHDLType::Signed(range) => format!("logic signed {}", range.to_systemverilog()),
            VHDLType::Unsigned(range) => format!("logic {}", range.to_systemverilog()),
            VHDLType::Custom(name) => format!("logic /* {} */", name),
        }
    }
}

use crate::ir::VectorRange;

impl VectorRange {
    pub fn to_systemverilog(&self) -> String {
        // SystemVerilog uses [msb:lsb] format like Verilog
        if self.downto {
            format!("[{}:{}]", self.left, self.right)
        } else {
            format!("[{}:{}]", self.right, self.left)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{PortDirection, VHDLType, VectorRange};

    #[test]
    fn test_generate_simple_sv_module() {
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

        let generator = SystemVerilogGenerator::new();
        let sv = generator.generate(&entity).unwrap();

        println!("Generated SystemVerilog:\n{}", sv);

        assert!(sv.contains("module counter"));
        assert!(sv.contains("input logic clk"));
        assert!(sv.contains("input logic reset"));
        assert!(sv.contains("output logic [7:0] count"));
        assert!(sv.contains("endmodule"));
    }

    #[test]
    fn test_always_comb_generation() {
        let mut entity = Entity::new("mux".to_string());
        let mut arch = Architecture {
            name: "rtl".to_string(),
            signals: vec![],
            processes: vec![crate::ir::Process {
                label: None,
                sensitivity_list: vec!["a".to_string(), "b".to_string(), "sel".to_string()],
                body: "if sel = '0' then\n    y <= a;\nelse\n    y <= b;\nend if;".to_string(),
            }],
            concurrent_statements: vec![],
        };
        entity.architecture = Some(arch);

        let generator = SystemVerilogGenerator::new();
        let sv = generator.generate(&entity).unwrap();

        assert!(sv.contains("always_comb"));
    }
}
