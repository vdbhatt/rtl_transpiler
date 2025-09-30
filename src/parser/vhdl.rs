use anyhow::{Context, Result};
use regex::Regex;
use crate::ir::{Entity, Port, PortDirection, VHDLType, VectorRange, Architecture, Signal, Process};

/// Simple VHDL parser that extracts entity declarations
/// For now, using regex-based parsing. Tree-sitter integration can be added later.
pub struct VHDLParser {
    pub content: String,
}

impl VHDLParser {
    pub fn new(content: String) -> Self {
        Self { content }
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read VHDL file: {:?}", path))?;
        Ok(Self::new(content))
    }

    /// Parse and extract all entities from the VHDL content
    pub fn parse_entities(&self) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();

        // Find all entity declarations
        let entity_re = Regex::new(
            r"(?is)entity\s+(\w+)\s+is.*?end\s+(?:entity\s+)?(?:\w+\s*)?;"
        ).context("Failed to compile entity regex")?;

        for cap in entity_re.captures_iter(&self.content) {
            let entity_text = cap.get(0).unwrap().as_str();
            let entity_name = cap.get(1).unwrap().as_str().to_string();

            let mut entity = self.parse_entity(entity_name.clone(), entity_text)?;

            // Try to find and parse architecture for this entity
            if let Ok(arch) = self.parse_architecture(&entity_name) {
                entity.architecture = Some(arch);
            }

            entities.push(entity);
        }

        Ok(entities)
    }

    fn parse_entity(&self, name: String, entity_text: &str) -> Result<Entity> {
        let mut entity = Entity::new(name);

        // Extract port clause - match from 'port(' to ');' allowing nested parentheses
        // Use a greedy match .* to get everything between port( and the last );
        let port_re = Regex::new(
            r"(?is)port\s*\((.*)\)\s*;"
        ).context("Failed to compile port regex")?;

        if let Some(port_cap) = port_re.captures(entity_text) {
            let ports_text = port_cap.get(1).unwrap().as_str();
            let ports = self.parse_ports(ports_text)?;
            for port in ports {
                entity.add_port(port);
            }
        }

        Ok(entity)
    }

    fn parse_ports(&self, ports_text: &str) -> Result<Vec<Port>> {
        let mut ports = Vec::new();

        // Split by semicolon to get individual port declarations
        for port_decl in ports_text.split(';') {
            let port_decl = port_decl.trim();
            if port_decl.is_empty() {
                continue;
            }

            eprintln!("DEBUG: Parsing port declaration: '{}'", port_decl);

            // Parse: "name1, name2 : direction type"
            let port_re = Regex::new(
                r"(?i)^\s*([\w,\s]+)\s*:\s*(\w+)\s+(.+)$"
            ).context("Failed to compile port declaration regex")?;

            if let Some(cap) = port_re.captures(port_decl) {
                let names_str = cap.get(1).unwrap().as_str();
                let direction_str = cap.get(2).unwrap().as_str();
                let type_str = cap.get(3).unwrap().as_str().trim();

                let direction = PortDirection::from_vhdl(direction_str)
                    .context(format!("Invalid port direction: {}", direction_str))?;

                let port_type = self.parse_type(type_str)?;

                // Handle multiple port names: "a, b, c : in std_logic"
                for name in names_str.split(',') {
                    let name = name.trim().to_string();
                    if !name.is_empty() {
                        ports.push(Port::new(name, direction.clone(), port_type.clone()));
                    }
                }
            }
        }

        Ok(ports)
    }

    pub fn parse_type(&self, type_str: &str) -> Result<VHDLType> {
        let type_str = type_str.trim().to_lowercase();

        if type_str == "std_logic" || type_str == "std_ulogic" {
            return Ok(VHDLType::StdLogic);
        }

        if type_str == "bit" {
            return Ok(VHDLType::Bit);
        }

        if type_str == "integer" {
            return Ok(VHDLType::Integer);
        }

        if type_str == "natural" {
            return Ok(VHDLType::Natural);
        }

        if type_str == "positive" {
            return Ok(VHDLType::Positive);
        }

        if type_str == "boolean" {
            return Ok(VHDLType::Boolean);
        }

        // Parse vector types: std_logic_vector(7 downto 0)
        let vector_re = Regex::new(
            r"(?i)^(std_logic_vector|bit_vector|signed|unsigned)\s*\(\s*(\d+)\s+(downto|to)\s+(\d+)\s*\)$"
        ).context("Failed to compile vector regex")?;

        if let Some(cap) = vector_re.captures(&type_str) {
            let base_type = cap.get(1).unwrap().as_str().to_lowercase();
            let left: i32 = cap.get(2).unwrap().as_str().parse()?;
            let direction = cap.get(3).unwrap().as_str().to_lowercase();
            let right: i32 = cap.get(4).unwrap().as_str().parse()?;

            let range = VectorRange {
                left,
                right,
                downto: direction == "downto",
            };

            return Ok(match base_type.as_str() {
                "std_logic_vector" => VHDLType::StdLogicVector(range),
                "bit_vector" => VHDLType::BitVector(range),
                "signed" => VHDLType::Signed(range),
                "unsigned" => VHDLType::Unsigned(range),
                _ => VHDLType::Custom(type_str.clone()),
            });
        }

        // If we can't parse it, treat as custom type
        Ok(VHDLType::Custom(type_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let parser = VHDLParser::new(vhdl.to_string());
        let entities = parser.parse_entities().unwrap();

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "counter");
        assert_eq!(entities[0].ports.len(), 3);

        assert_eq!(entities[0].ports[0].name, "clk");
        assert_eq!(entities[0].ports[0].direction, PortDirection::In);

        assert_eq!(entities[0].ports[2].name, "count");
        assert_eq!(entities[0].ports[2].direction, PortDirection::Out);
    }

    #[test]
    fn test_parse_multiple_names() {
        let vhdl = r#"
        entity test is
            port(
                a, b, c : in std_logic;
                d : out std_logic
            );
        end entity;
        "#;

        let parser = VHDLParser::new(vhdl.to_string());
        let entities = parser.parse_entities().unwrap();

        assert_eq!(entities[0].ports.len(), 4);
        assert_eq!(entities[0].ports[0].name, "a");
        assert_eq!(entities[0].ports[1].name, "b");
        assert_eq!(entities[0].ports[2].name, "c");
    }
}