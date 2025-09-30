pub mod model;
pub mod verilog_gen;

pub use model::{Entity, Port, PortDirection, VHDLType, VectorRange, Generic, Architecture, Signal, Process};
pub use verilog_gen::VerilogGenerator;