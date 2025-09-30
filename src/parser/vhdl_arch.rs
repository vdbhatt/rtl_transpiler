// Architecture parsing extension for VHDL parser
use anyhow::{Context, Result};
use regex::Regex;
use crate::ir::{Architecture, Signal, Process, VHDLType};
use crate::parser::VHDLParser;

impl VHDLParser {
    pub fn parse_architecture(&self, entity_name: &str) -> Result<Architecture> {
        // Find architecture for this entity
        let arch_re = Regex::new(
            &format!(r"(?is)architecture\s+(\w+)\s+of\s+{}\s+is(.*?)begin(.*?)end\s+(?:architecture\s+)?(?:\w+\s*)?;", entity_name)
        ).context("Failed to compile architecture regex")?;

        if let Some(cap) = arch_re.captures(&self.content) {
            let arch_name = cap.get(1).unwrap().as_str().to_string();
            let declarations = cap.get(2).unwrap().as_str();
            let body = cap.get(3).unwrap().as_str();

            let signals = self.parse_signals(declarations)?;
            let processes = self.parse_processes(body)?;
            let concurrent_statements = self.parse_concurrent_statements(body)?;

            Ok(Architecture {
                name: arch_name,
                signals,
                processes,
                concurrent_statements,
            })
        } else {
            Err(anyhow::anyhow!("No architecture found for entity: {}", entity_name))
        }
    }

    fn parse_signals(&self, declarations: &str) -> Result<Vec<Signal>> {
        let mut signals = Vec::new();

        // Match signal declarations: signal name : type;
        let signal_re = Regex::new(
            r"(?i)signal\s+(\w+)\s*:\s*([^;]+);"
        ).context("Failed to compile signal regex")?;

        for cap in signal_re.captures_iter(declarations) {
            let name = cap.get(1).unwrap().as_str().to_string();
            let type_str = cap.get(2).unwrap().as_str().trim();

            // Reuse type parsing from main parser
            let signal_type = self.parse_type(type_str)?;

            signals.push(Signal {
                name,
                signal_type,
            });
        }

        Ok(signals)
    }

    fn parse_processes(&self, body: &str) -> Result<Vec<Process>> {
        let mut processes = Vec::new();

        // Match process blocks
        let process_re = Regex::new(
            r"(?is)(?:(\w+)\s*:\s*)?process\s*\(([^)]*)\)(.*?)end\s+process"
        ).context("Failed to compile process regex")?;

        for cap in process_re.captures_iter(body) {
            let label = cap.get(1).map(|m| m.as_str().to_string());
            let sensitivity = cap.get(2).unwrap().as_str();
            let process_body = cap.get(3).unwrap().as_str();

            let sensitivity_list: Vec<String> = sensitivity
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            processes.push(Process {
                label,
                sensitivity_list,
                body: process_body.to_string(),
            });
        }

        Ok(processes)
    }

    fn parse_concurrent_statements(&self, body: &str) -> Result<Vec<String>> {
        let mut statements = Vec::new();

        // Remove process blocks from body to get only concurrent statements
        let process_re = Regex::new(
            r"(?is)(?:\w+\s*:\s*)?process\s*\([^)]*\).*?end\s+process\s*;"
        ).context("Failed to compile process regex for removal")?;

        let body_without_processes = process_re.replace_all(body, "");

        // Split by semicolon and filter out empty lines
        for line in body_without_processes.split(';') {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with("--") {
                statements.push(line.to_string());
            }
        }

        Ok(statements)
    }
}