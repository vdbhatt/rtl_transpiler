use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortDirection {
    In,
    Out,
    InOut,
    Buffer,
}

impl PortDirection {
    pub fn from_vhdl(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "in" => Some(PortDirection::In),
            "out" => Some(PortDirection::Out),
            "inout" => Some(PortDirection::InOut),
            "buffer" => Some(PortDirection::Buffer),
            _ => None,
        }
    }

    pub fn to_verilog(&self) -> &str {
        match self {
            PortDirection::In => "input",
            PortDirection::Out => "output",
            PortDirection::InOut => "inout",
            PortDirection::Buffer => "output", // Buffer maps to output in Verilog
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorRange {
    pub left: i32,
    pub right: i32,
    pub downto: bool, // true for "downto", false for "to"
}

impl VectorRange {
    pub fn to_verilog(&self) -> String {
        // Verilog uses [msb:lsb] format
        if self.downto {
            format!("[{}:{}]", self.left, self.right)
        } else {
            format!("[{}:{}]", self.right, self.left)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VHDLType {
    StdLogic,
    StdLogicVector(VectorRange),
    Integer,
    Natural,
    Positive,
    Boolean,
    Bit,
    BitVector(VectorRange),
    Signed(VectorRange),
    Unsigned(VectorRange),
    Custom(String), // For user-defined types
}

impl VHDLType {
    pub fn to_verilog(&self) -> String {
        match self {
            VHDLType::StdLogic => "wire".to_string(),
            VHDLType::StdLogicVector(range) => format!("wire {}", range.to_verilog()),
            VHDLType::Integer => "wire signed [31:0]".to_string(),
            VHDLType::Natural => "wire [31:0]".to_string(),
            VHDLType::Positive => "wire [31:0]".to_string(),
            VHDLType::Boolean => "wire".to_string(),
            VHDLType::Bit => "wire".to_string(),
            VHDLType::BitVector(range) => format!("wire {}", range.to_verilog()),
            VHDLType::Signed(range) => format!("wire signed {}", range.to_verilog()),
            VHDLType::Unsigned(range) => format!("wire {}", range.to_verilog()),
            VHDLType::Custom(name) => format!("wire /* {} */", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub name: String,
    pub direction: PortDirection,
    pub port_type: VHDLType,
}

impl Port {
    pub fn new(name: String, direction: PortDirection, port_type: VHDLType) -> Self {
        Self {
            name,
            direction,
            port_type,
        }
    }

    pub fn to_verilog(&self) -> String {
        let direction = self.direction.to_verilog();
        let verilog_type = self.port_type.to_verilog();

        // Format: "input wire [7:0] data" or "output wire clk"
        format!("{} {} {}", direction, verilog_type, self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub ports: Vec<Port>,
    pub generics: Vec<Generic>,
    pub architecture: Option<Architecture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Architecture {
    pub name: String,
    pub signals: Vec<Signal>,
    pub processes: Vec<Process>,
    pub concurrent_statements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub name: String,
    pub signal_type: VHDLType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub label: Option<String>,
    pub sensitivity_list: Vec<String>,
    pub body: String, // Store as raw text for now
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generic {
    pub name: String,
    pub generic_type: String,
    pub default_value: Option<String>,
}

impl Entity {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ports: Vec::new(),
            generics: Vec::new(),
            architecture: None,
        }
    }

    pub fn add_port(&mut self, port: Port) {
        self.ports.push(port);
    }

    pub fn add_generic(&mut self, generic: Generic) {
        self.generics.push(generic);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_direction_conversion() {
        assert_eq!(PortDirection::from_vhdl("in"), Some(PortDirection::In));
        assert_eq!(PortDirection::In.to_verilog(), "input");
    }

    #[test]
    fn test_vector_range_conversion() {
        let range = VectorRange {
            left: 7,
            right: 0,
            downto: true,
        };
        assert_eq!(range.to_verilog(), "[7:0]");
    }

    #[test]
    fn test_vhdl_type_conversion() {
        let std_logic = VHDLType::StdLogic;
        assert_eq!(std_logic.to_verilog(), "wire");

        let vector = VHDLType::StdLogicVector(VectorRange {
            left: 7,
            right: 0,
            downto: true,
        });
        assert_eq!(vector.to_verilog(), "wire [7:0]");
    }

    #[test]
    fn test_port_to_verilog() {
        let port = Port::new(
            "clk".to_string(),
            PortDirection::In,
            VHDLType::StdLogic,
        );
        assert_eq!(port.to_verilog(), "input wire clk");

        let port_vector = Port::new(
            "data".to_string(),
            PortDirection::Out,
            VHDLType::StdLogicVector(VectorRange {
                left: 7,
                right: 0,
                downto: true,
            }),
        );
        assert_eq!(port_vector.to_verilog(), "output wire [7:0] data");
    }
}