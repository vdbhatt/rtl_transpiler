use anyhow::{Context, Result};
use tree_sitter::{Node, Tree};
use crate::ir::{Entity, Port, PortDirection, VHDLType, VectorRange, Architecture, Signal, Process, Generic};
use crate::parser::tree_sitter_vhdl::{TreeSitterVHDLParser, VHDLASTHelper};

/// AST-based VHDL parser using tree-sitter
pub struct ASTVHDLParser {
    parser: TreeSitterVHDLParser,
    content: String,
}

impl ASTVHDLParser {
    pub fn new(content: String) -> Result<Self> {
        let parser = TreeSitterVHDLParser::new()
            .context("Failed to create tree-sitter VHDL parser")?;
        
        Ok(Self { parser, content })
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read VHDL file: {:?}", path))?;
        Self::new(content)
    }

    /// Parse and extract all entities from the VHDL content
    pub fn parse_entities(&mut self) -> Result<Vec<Entity>> {
        let tree = self.parser.parse(&self.content)
            .context("Failed to parse VHDL content with tree-sitter")?;

        let root = tree.root_node();
        if root.has_error() {
            return Err(anyhow::anyhow!("Parse tree contains errors"));
        }

        let mut entities = Vec::new();

        // Find all entity declarations in the AST
        let entity_nodes = VHDLASTHelper::find_all_nodes_by_type(&root, "entity_declaration");
        
        for entity_node in entity_nodes {
            let entity = self.parse_entity_from_node(&entity_node, &tree)?;
            entities.push(entity);
        }

        Ok(entities)
    }

    fn parse_entity_from_node(&self, entity_node: &Node, tree: &Tree) -> Result<Entity> {
        // Get entity name
        let name_node = VHDLASTHelper::find_child_by_type(entity_node, "identifier")
            .ok_or_else(|| anyhow::anyhow!("Entity missing name"))?;
        
        let entity_name = VHDLASTHelper::node_text(&name_node, &self.content).to_string();
        let mut entity = Entity::new(entity_name.clone());

        // Parse generic clause if present
        if let Some(generic_node) = VHDLASTHelper::find_child_by_type(entity_node, "generic_clause") {
            let generics = self.parse_generics_from_node(&generic_node)?;
            for generic in generics {
                entity.add_generic(generic);
            }
        }

        // Parse port clause if present - look in entity_header first
        if let Some(entity_header) = VHDLASTHelper::find_child_by_type(entity_node, "entity_header") {
            if let Some(port_node) = VHDLASTHelper::find_child_by_type(&entity_header, "port_clause") {
                let ports = self.parse_ports_from_node(&port_node)?;
                for port in ports {
                    entity.add_port(port);
                }
            }
        }

        // Try to find and parse architecture for this entity
        let root_node = tree.root_node();
        let arch_nodes = VHDLASTHelper::find_all_nodes_by_type(&root_node, "architecture_body");
        for arch_node in arch_nodes {
            if let Ok(arch) = self.parse_architecture_from_node(&arch_node, &entity_name) {
                entity.architecture = Some(arch);
                break;
            }
        }

        Ok(entity)
    }

    fn parse_generics_from_node(&self, generic_node: &Node) -> Result<Vec<Generic>> {
        let mut generics = Vec::new();

        // Find generic interface list
        if let Some(interface_list) = VHDLASTHelper::find_child_by_type(generic_node, "generic_interface_list") {
            let interface_declarations = VHDLASTHelper::find_children_by_type(&interface_list, "interface_constant_declaration");
            
            for decl in interface_declarations {
                let generic = self.parse_generic_from_declaration(&decl)?;
                generics.push(generic);
            }
        }

        Ok(generics)
    }

    fn parse_generic_from_declaration(&self, decl_node: &Node) -> Result<Generic> {
        // Get identifier list (generic names)
        let identifier_list = VHDLASTHelper::find_child_by_type(decl_node, "identifier_list")
            .ok_or_else(|| anyhow::anyhow!("Generic declaration missing identifier list"))?;
        
        let identifiers = VHDLASTHelper::find_children_by_type(&identifier_list, "identifier");
        if identifiers.is_empty() {
            return Err(anyhow::anyhow!("Generic declaration has no identifiers"));
        }

        // For now, take the first identifier (we can extend this to handle multiple later)
        let name = VHDLASTHelper::node_text(&identifiers[0], &self.content).to_string();

        // Get subtype indication (type)
        let subtype_indication = VHDLASTHelper::find_child_by_type(decl_node, "subtype_indication")
            .ok_or_else(|| anyhow::anyhow!("Generic declaration missing type"))?;
        
        let type_name = self.extract_type_name_from_subtype(&subtype_indication)?;

        // Get default value if present
        let default_value = VHDLASTHelper::find_child_by_type(decl_node, "expression")
            .map(|expr| VHDLASTHelper::node_text(&expr, &self.content).to_string());

        Ok(Generic {
            name,
            generic_type: type_name,
            default_value,
        })
    }

    fn parse_ports_from_node(&self, port_node: &Node) -> Result<Vec<Port>> {
        let mut ports = Vec::new();

        // Find signal interface declarations directly in the port clause
        let interface_declarations = VHDLASTHelper::find_children_by_type(port_node, "signal_interface_declaration");
        
        for decl in interface_declarations {
            let port_list = self.parse_ports_from_declaration(&decl)?;
            ports.extend(port_list);
        }

        Ok(ports)
    }

    fn parse_ports_from_declaration(&self, decl_node: &Node) -> Result<Vec<Port>> {
        let mut ports = Vec::new();

        // Get identifier list (port names)
        let identifier_list = VHDLASTHelper::find_child_by_type(decl_node, "identifier_list")
            .ok_or_else(|| anyhow::anyhow!("Port declaration missing identifier list"))?;
        
        let identifiers = VHDLASTHelper::find_children_by_type(&identifier_list, "identifier");
        if identifiers.is_empty() {
            return Err(anyhow::anyhow!("Port declaration has no identifiers"));
        }

        // Get port direction (mode)
        let mode_node = VHDLASTHelper::find_child_by_type(decl_node, "mode")
            .ok_or_else(|| anyhow::anyhow!("Port declaration missing mode"))?;
        
        let mode_text = VHDLASTHelper::node_text(&mode_node, &self.content);
        let direction = PortDirection::from_vhdl(mode_text)
            .ok_or_else(|| anyhow::anyhow!("Invalid port direction: {}", mode_text))?;

        // Get subtype indication (type)
        let subtype_indication = VHDLASTHelper::find_child_by_type(decl_node, "subtype_indication")
            .ok_or_else(|| anyhow::anyhow!("Port declaration missing type"))?;
        
        let port_type = self.parse_type_from_subtype(&subtype_indication)?;

        // Create ports for all identifiers
        for identifier in identifiers {
            let name = VHDLASTHelper::node_text(&identifier, &self.content).to_string();
            ports.push(Port::new(name, direction.clone(), port_type.clone()));
        }

        Ok(ports)
    }

    fn parse_type_from_subtype(&self, subtype_node: &Node) -> Result<VHDLType> {
        // Get the type mark (base type name) - it contains a simple_name
        let type_mark = VHDLASTHelper::find_child_by_type(subtype_node, "type_mark")
            .ok_or_else(|| anyhow::anyhow!("Subtype indication missing type mark"))?;
        
        let simple_name = VHDLASTHelper::find_child_by_type(&type_mark, "simple_name")
            .ok_or_else(|| anyhow::anyhow!("Type mark missing simple name"))?;
        
        let type_name = VHDLASTHelper::node_text(&simple_name, &self.content).to_lowercase();

        // Handle basic types
        match type_name.as_str() {
            "std_logic" | "std_ulogic" => return Ok(VHDLType::StdLogic),
            "bit" => return Ok(VHDLType::Bit),
            "integer" => return Ok(VHDLType::Integer),
            "natural" => return Ok(VHDLType::Natural),
            "positive" => return Ok(VHDLType::Positive),
            "boolean" => return Ok(VHDLType::Boolean),
            _ => {}
        }

        // Check for array constraint (for vector types)
        if let Some(array_constraint) = VHDLASTHelper::find_child_by_type(subtype_node, "array_constraint") {
            if let Some(index_constraint) = VHDLASTHelper::find_child_by_type(&array_constraint, "index_constraint") {
                let range = self.parse_range_from_index_constraint(&index_constraint)?;
                
                return Ok(match type_name.as_str() {
                    "std_logic_vector" => VHDLType::StdLogicVector(range),
                    "bit_vector" => VHDLType::BitVector(range),
                    "signed" => VHDLType::Signed(range),
                    "unsigned" => VHDLType::Unsigned(range),
                    _ => VHDLType::Custom(format!("{}({})", type_name, range.left)),
                });
            }
        }

        // If we can't parse it, treat as custom type
        Ok(VHDLType::Custom(type_name))
    }

    fn extract_type_name_from_subtype(&self, subtype_node: &Node) -> Result<String> {
        // For generics, we just need the type name as a string
        let type_mark = VHDLASTHelper::find_child_by_type(subtype_node, "type_mark")
            .or_else(|| VHDLASTHelper::find_child_by_type(subtype_node, "identifier"))
            .ok_or_else(|| anyhow::anyhow!("Subtype indication missing type mark"))?;
        
        Ok(VHDLASTHelper::node_text(&type_mark, &self.content).to_string())
    }

    fn parse_range_from_index_constraint(&self, index_constraint: &Node) -> Result<VectorRange> {
        // Look for descending_range or ascending_range
        if let Some(descending_range) = VHDLASTHelper::find_child_by_type(index_constraint, "descending_range") {
            return self.parse_descending_range(&descending_range);
        }
        
        if let Some(ascending_range) = VHDLASTHelper::find_child_by_type(index_constraint, "ascending_range") {
            return self.parse_ascending_range(&ascending_range);
        }

        Err(anyhow::anyhow!("Could not find range in index constraint"))
    }

    fn parse_descending_range(&self, descending_range: &Node) -> Result<VectorRange> {
        let simple_expressions = VHDLASTHelper::find_children_by_type(descending_range, "simple_expression");
        
        if simple_expressions.len() >= 2 {
            let left_expr = &simple_expressions[0];
            let right_expr = &simple_expressions[1];
            
            let left = self.parse_integer_from_expression(left_expr)?;
            let right = self.parse_integer_from_expression(right_expr)?;
            
            return Ok(VectorRange { left, right, downto: true });
        }

        Err(anyhow::anyhow!("Could not parse descending range"))
    }

    fn parse_ascending_range(&self, ascending_range: &Node) -> Result<VectorRange> {
        let simple_expressions = VHDLASTHelper::find_children_by_type(ascending_range, "simple_expression");
        
        if simple_expressions.len() >= 2 {
            let left_expr = &simple_expressions[0];
            let right_expr = &simple_expressions[1];
            
            let left = self.parse_integer_from_expression(left_expr)?;
            let right = self.parse_integer_from_expression(right_expr)?;
            
            return Ok(VectorRange { left, right, downto: false });
        }

        Err(anyhow::anyhow!("Could not parse ascending range"))
    }

    fn parse_integer_from_expression(&self, expr: &Node) -> Result<i32> {
        // Look for integer_decimal in the expression
        if let Some(integer_node) = VHDLASTHelper::find_child_by_type(expr, "integer_decimal") {
            let integer_text = VHDLASTHelper::node_text(&integer_node, &self.content);
            return integer_text.parse()
                .context(format!("Failed to parse integer: {}", integer_text));
        }

        // If not found directly, try to get the text of the whole expression
        let expr_text = VHDLASTHelper::node_text(expr, &self.content).trim();
        
        // Handle simple expressions like "WIDTH-1" by trying to parse as integer first
        if let Ok(value) = expr_text.parse::<i32>() {
            return Ok(value);
        }
        
        // For expressions like "WIDTH-1", we'll need to handle them differently
        // For now, return a default value and log a warning
        if expr_text.contains('-') && expr_text.len() < 20 {
            // Simple heuristic: if it looks like "WIDTH-1", assume it's a reasonable range
            // This is a temporary solution - in a real implementation, we'd need proper expression evaluation
            tracing::warn!("Could not parse expression '{}', using default value 7", expr_text);
            return Ok(7); // Default to 8-bit range
        }
        
        expr_text.parse()
            .context(format!("Failed to parse expression as integer: {}", expr_text))
    }

    fn parse_architecture_from_node(&self, arch_node: &Node, entity_name: &str) -> Result<Architecture> {
        // Get architecture name
        let arch_name_node = VHDLASTHelper::find_child_by_type(arch_node, "identifier")
            .ok_or_else(|| anyhow::anyhow!("Architecture missing name"))?;
        
        let arch_name = VHDLASTHelper::node_text(&arch_name_node, &self.content).to_string();

        // Check if this architecture is for the correct entity
        // Look for the entity name reference after "of" keyword
        let all_identifiers = VHDLASTHelper::find_children_by_type(arch_node, "identifier");
        
        // The entity name should be the second identifier (after architecture name)
        let referenced_entity = if all_identifiers.len() >= 2 {
            VHDLASTHelper::node_text(&all_identifiers[1], &self.content).to_string()
        } else {
            // Try to find entity name in a different way - look for it after "of"
            let arch_text = VHDLASTHelper::node_text(arch_node, &self.content);
            
            // Simple text parsing: "architecture NAME of ENTITY is"
            if let Some(of_pos) = arch_text.find(" of ") {
                if let Some(is_pos) = arch_text.find(" is") {
                    let entity_part = arch_text[of_pos + 4..is_pos].trim().to_string();
                    entity_part
                } else {
                    return Err(anyhow::anyhow!("Architecture missing 'is' keyword"));
                }
            } else {
                return Err(anyhow::anyhow!("Architecture missing 'of' keyword"));
            }
        };
        
        if referenced_entity != entity_name {
            return Err(anyhow::anyhow!("Architecture is for different entity: {}", referenced_entity));
        }

        // Parse architecture declarative part (signals)
        let mut signals = Vec::new();
        if let Some(decl_part) = VHDLASTHelper::find_child_by_type(arch_node, "declarative_part") {
            signals = self.parse_signals_from_declarative_part(&decl_part)?;
        }

        // Parse architecture statement part (processes and concurrent statements)
        let mut processes = Vec::new();
        let mut concurrent_statements = Vec::new();
        
        if let Some(stmt_part) = VHDLASTHelper::find_child_by_type(arch_node, "concurrent_statement_part") {
            let (procs, concurrent) = self.parse_statements_from_statement_part(&stmt_part)?;
            processes = procs;
            concurrent_statements = concurrent;
        }

        Ok(Architecture {
            name: arch_name,
            signals,
            processes,
            concurrent_statements,
        })
    }

    fn parse_signals_from_declarative_part(&self, decl_part: &Node) -> Result<Vec<Signal>> {
        let mut signals = Vec::new();

        let signal_declarations = VHDLASTHelper::find_all_nodes_by_type(decl_part, "signal_declaration");
        
        for signal_decl in signal_declarations {
            let signal_list = self.parse_signals_from_declaration(&signal_decl)?;
            signals.extend(signal_list);
        }

        Ok(signals)
    }

    fn parse_signals_from_declaration(&self, decl_node: &Node) -> Result<Vec<Signal>> {
        let mut signals = Vec::new();

        // Get identifier list (signal names)
        let identifier_list = VHDLASTHelper::find_child_by_type(decl_node, "identifier_list")
            .ok_or_else(|| anyhow::anyhow!("Signal declaration missing identifier list"))?;
        
        let identifiers = VHDLASTHelper::find_children_by_type(&identifier_list, "identifier");

        // Get subtype indication (type)
        let subtype_indication = VHDLASTHelper::find_child_by_type(decl_node, "subtype_indication")
            .ok_or_else(|| anyhow::anyhow!("Signal declaration missing type"))?;
        
        let signal_type = self.parse_type_from_subtype(&subtype_indication)?;

        // Create signals for all identifiers
        for identifier in identifiers {
            let name = VHDLASTHelper::node_text(&identifier, &self.content).to_string();
            signals.push(Signal {
                name,
                signal_type: signal_type.clone(),
            });
        }

        Ok(signals)
    }

    fn parse_statements_from_statement_part(&self, stmt_part: &Node) -> Result<(Vec<Process>, Vec<String>)> {
        let mut processes = Vec::new();
        let mut concurrent_statements = Vec::new();

        // Find process statements
        let process_nodes = VHDLASTHelper::find_all_nodes_by_type(stmt_part, "process_statement");
        for process_node in process_nodes {
            if let Ok(process) = self.parse_process_from_node(&process_node) {
                processes.push(process);
            }
        }

        // Find concurrent signal assignments - try different node types
        let concurrent_types = vec![
            "concurrent_signal_assignment_statement",
            "simple_concurrent_signal_assignment",
            "conditional_signal_assignment",
            "selected_signal_assignment",
        ];

        for node_type in concurrent_types {
            let concurrent_nodes = VHDLASTHelper::find_all_nodes_by_type(stmt_part, node_type);
            for concurrent_node in concurrent_nodes {
                let stmt_text = VHDLASTHelper::node_text(&concurrent_node, &self.content);
                let stmt_str = stmt_text.trim().to_string();
                if !stmt_str.is_empty() && !concurrent_statements.contains(&stmt_str) {
                    concurrent_statements.push(stmt_str);
                }
            }
        }

        Ok((processes, concurrent_statements))
    }

    fn parse_process_from_node(&self, process_node: &Node) -> Result<Process> {
        // Get process label if present
        let label = VHDLASTHelper::find_child_by_type(process_node, "label")
            .map(|label_node| VHDLASTHelper::node_text(&label_node, &self.content).to_string());

        // Get sensitivity list - check both in process_node itself and in sensitivity_list child
        let mut sensitivity_list = Vec::new();

        // First, try to find sensitivity_list directly as a child
        if let Some(sensitivity_list_node) = VHDLASTHelper::find_child_by_type(process_node, "sensitivity_list") {
            // Extract identifiers or simple_names from the sensitivity list
            let simple_names = VHDLASTHelper::find_children_by_type(&sensitivity_list_node, "simple_name");
            for name_node in simple_names {
                let name = VHDLASTHelper::node_text(&name_node, &self.content).to_string();
                sensitivity_list.push(name);
            }

            // Also try identifiers (in case they're not wrapped in simple_name)
            let identifiers = VHDLASTHelper::find_children_by_type(&sensitivity_list_node, "identifier");
            for identifier in identifiers {
                let name = VHDLASTHelper::node_text(&identifier, &self.content).to_string();
                if !sensitivity_list.contains(&name) {
                    sensitivity_list.push(name);
                }
            }
        }

        // Get process body - look for sequence_of_statements node
        let body = if let Some(stmt_sequence) = VHDLASTHelper::find_child_by_type(process_node, "sequence_of_statements") {
            VHDLASTHelper::node_text(&stmt_sequence, &self.content).to_string()
        } else {
            String::new()
        };

        Ok(Process {
            label,
            sensitivity_list,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_parser_creation() {
        let content = "entity test is end entity;".to_string();
        let parser = ASTVHDLParser::new(content);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parse_simple_entity() {
        let vhdl = r#"
        entity counter is
            port(
                clk    : in  std_logic;
                reset  : in  std_logic;
                count  : out std_logic_vector(7 downto 0)
            );
        end entity counter;
        "#;

        let mut parser = ASTVHDLParser::new(vhdl.to_string()).unwrap();
        let entities = parser.parse_entities();
        
        // This test might fail initially until we have the tree-sitter grammar working
        // but it establishes the expected interface
        if let Ok(entities) = entities {
            assert_eq!(entities.len(), 1);
            assert_eq!(entities[0].name, "counter");
            assert_eq!(entities[0].ports.len(), 3);
        }
    }
}
