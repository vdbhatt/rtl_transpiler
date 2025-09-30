use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::agent::base::{BaseAgent, BaseAgentImpl};
use crate::agent::basics::AgentExecution;
use crate::config::AgentConfig;
use crate::llm::{create_llm_client, LLMMessage};
use crate::tools::ToolResult;
use crate::utils::{CLIConsole, TrajectoryRecorder};
use obfstr::obfstr;
use lazy_static::lazy_static;

lazy_static! {
    static ref TRANSPILER_AGENT_SYSTEM_PROMPT: String = obfstr!(r#"You are an expert VHDL to Verilog transpiler agent.

Your task is to convert VHDL entity declarations to Verilog module declarations with matching input and output ports.

**Core Conversion Rules:**

1. **Entity to Module:**
   - VHDL `entity X is` → Verilog `module X (`
   - VHDL `end entity X;` → Verilog `endmodule`

2. **Port Directions:**
   - VHDL `in` → Verilog `input wire`
   - VHDL `out` → Verilog `output wire`
   - VHDL `inout` → Verilog `inout wire`
   - VHDL `buffer` → Verilog `output wire`

3. **Type Mappings:**
   - VHDL `std_logic` → Verilog `wire` (1-bit)
   - VHDL `std_logic_vector(N downto 0)` → Verilog `wire [N:0]`
   - VHDL `std_logic_vector(N-1 downto 0)` → Verilog `wire [N-1:0]`
   - VHDL `bit` → Verilog `wire`
   - VHDL `bit_vector` → Verilog `wire [...]`
   - VHDL `integer` → Verilog `wire signed [31:0]`
   - VHDL `natural` → Verilog `wire [31:0]`
   - VHDL `signed(N downto 0)` → Verilog `wire signed [N:0]`
   - VHDL `unsigned(N downto 0)` → Verilog `wire [N:0]`

4. **Bit Ordering:**
   - VHDL `(N downto 0)` → Verilog `[N:0]` (MSB:LSB)
   - VHDL `(0 to N)` → Verilog `[N:0]` (flip the order)

**Example Conversion:**

VHDL Input:
```vhdl
entity counter is
    port(
        clk    : in  std_logic;
        reset  : in  std_logic;
        enable : in  std_logic;
        count  : out std_logic_vector(7 downto 0)
    );
end entity counter;
```

Verilog Output:
```verilog
module counter (
    input  wire       clk,
    input  wire       reset,
    input  wire       enable,
    output wire [7:0] count
);
endmodule
```

**Available Tool:**
- `transpile_vhdl_to_verilog`: Use this tool to transpile VHDL files to Verilog

**Workflow:**
1. Read the VHDL file provided by the user
2. Use the `transpile_vhdl_to_verilog` tool to convert it
3. If the user specifies an output file, provide it to the tool
4. Review the generated Verilog and report success
5. Call `task_done` when complete

**Important:**
- Focus ONLY on entity-to-module conversion (ports and interfaces)
- Do NOT attempt to convert architecture bodies or behavioral code yet
- Ensure port names and types match exactly
- Preserve signal naming conventions

# Current task:
Project Path: {project_path}
Task: {task}
"#).to_string();
}

pub struct TranspilerAgent {
    base: BaseAgentImpl,
    project_path: String,
}

impl TranspilerAgent {
    pub fn new(
        config: AgentConfig,
        trajectory_recorder: Option<Arc<Mutex<TrajectoryRecorder>>>,
        cli_console: Option<Arc<dyn CLIConsole>>,
    ) -> Result<Self> {
        // Create LLM client
        let model_config = config.model_config.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Model configuration required for TranspilerAgent"))?;

        let llm_client = create_llm_client(model_config)?;

        // Create base agent
        let base = BaseAgentImpl::new(
            "TranspilerAgent".to_string(),
            config.clone(),
            llm_client,
            trajectory_recorder,
            cli_console,
        )?;

        Ok(Self {
            base,
            project_path: String::new(),
        })
    }
}

impl BaseAgent for TranspilerAgent {
    fn get_name(&self) -> &str {
        self.base.get_name()
    }

    fn get_max_steps(&self) -> u32 {
        self.base.get_max_steps()
    }

    fn get_tools(&self) -> Vec<Arc<dyn crate::tools::Tool>> {
        self.base.get_tools()
    }

    fn get_tool_executor(&self) -> Arc<crate::tools::ToolExecutor> {
        self.base.get_tool_executor()
    }

    fn get_llm_client(&self) -> Arc<dyn crate::llm::LLMClient> {
        self.base.get_llm_client()
    }

    fn get_trajectory_recorder(&self) -> Option<Arc<Mutex<TrajectoryRecorder>>> {
        self.base.get_trajectory_recorder()
    }

    fn get_cli_console(&self) -> Option<Arc<dyn CLIConsole>> {
        self.base.get_cli_console()
    }

    fn initialize(&mut self) -> Result<()> {
        self.base.initialize()
    }

    fn shutdown(&mut self) -> Result<()> {
        self.base.shutdown()
    }

    fn prepare_system_message(&self, task: &str, task_args: &serde_json::Value) -> String {
        let project_path = task_args.get("project_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let system_prompt = TRANSPILER_AGENT_SYSTEM_PROMPT
            .replace("{project_path}", project_path)
            .replace("{task}", task);

        tracing::debug!("TranspilerAgent::prepare_system_message called");
        tracing::debug!("Task: {}", task);
        tracing::debug!("Project path: {}", project_path);

        system_prompt
    }

    fn process_response(
        &self,
        response: &crate::llm::LLMResponse,
        execution: &mut AgentExecution,
    ) -> Result<Vec<ToolResult>> {
        self.base.process_response(response, execution)
    }

    fn run_step(
        &self,
        messages: &mut Vec<LLMMessage>,
        execution: &mut AgentExecution,
        cancel_flag: Arc<AtomicBool>,
        step_num: u32,
    ) -> Result<bool> {
        self.base.run_step(messages, execution, cancel_flag, step_num)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_generation() {
        let prompt = TRANSPILER_AGENT_SYSTEM_PROMPT.clone();
        assert!(prompt.contains("VHDL to Verilog"));
        assert!(prompt.contains("entity"));
        assert!(prompt.contains("module"));
        assert!(prompt.contains("transpile_vhdl_to_verilog"));
    }
}