pub mod model;
pub mod verilog_gen;  // Keep for backward compatibility
pub mod systemverilog_gen;

pub use model::{Entity, Port, PortDirection, VHDLType, VectorRange, Generic, Architecture, Signal, Process};
pub use systemverilog_gen::SystemVerilogGenerator;
// VerilogGenerator still available if needed for legacy code
pub use verilog_gen::VerilogGenerator;