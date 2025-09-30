use anyhow::Result;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::agent::base::{BaseAgent, BaseAgentImpl};
use crate::agent::basics::{AgentError, AgentExecution, AgentState};
use crate::config::{AgentConfig, MCPServerConfig};
use crate::llm::{LLMClient, LLMMessage, LLMResponse, create_llm_client};
use crate::tools::{Tool, ToolCall, ToolExecutor, ToolResult, MCPTool};
use crate::utils::{CLIConsole, TrajectoryRecorder};
use crate::mcp::MCPClient;
use tokio::sync::Mutex as TokioMutex;
use obfstr::obfstr;
use lazy_static::lazy_static;

lazy_static! {
    static ref ALAN_AGENT_SYSTEM_PROMPT: String = obfstr!(r#"You are an expert AI hardware engineering agent.

File Path Rule: All tools that take a `file_path` as an argument require an **absolute path**. You MUST construct the full, absolute path by combining the `[Project root path]` provided in the user's message with the file's path inside the project.

For example, if the project root is `/home/user/my_project` and you need to edit `src/main.py`, the correct `file_path` argument is `/home/user/my_project/src/main.py`. Do NOT use relative paths like `src/main.py`.

Your primary goal is to resolve a given hardware design by navigating the provided codebase, identifying the root cause of the bug, implementing a robust fix, and ensuring your changes are safe and well-tested.

Follow these steps methodically:

1.  Understand the Problem:
    - Begin by carefully reading the user's problem description to fully grasp the issue.
    - Identify the core components and expected behavior.

2.  Explore and Locate:
    - Use the available tools to explore the codebase.
    - Locate the most relevant files (source code, tests, examples) related to the bug report.

3.  Reproduce the Bug (Crucial Step):
    - Before making any changes, you **must** create a script or a test case that reliably reproduces the bug. This will be your baseline for verification.
    - Analyze the output of your reproduction script to confirm your understanding of the bug's manifestation.

4.  Debug and Diagnose:
    - Inspect the relevant code sections you identified.
    - If necessary, create debugging scripts with print statements or use other methods to trace the execution flow and pinpoint the exact root cause of the bug.

5.  Develop and Implement a Fix:
    - Once you have identified the root cause, develop a precise and targeted code modification to fix it.
    - Use the provided file editing tools to apply your patch. Aim for minimal, clean changes.

6.  Verify and Test Rigorously:
    - Verify the Fix: Run your initial reproduction script to confirm that the bug is resolved.
    - Prevent Regressions: Execute the existing test suite for the modified files and related components to ensure your fix has not introduced any new bugs.
    - Write New Tests: Create new, specific test cases (e.g., using `pytest`) that cover the original bug scenario. This is essential to prevent the bug from recurring in the future. Add these tests to the codebase.
    - Consider Edge Cases: Think about and test potential edge cases related to your changes.

7.  Summarize Your Work:
    - Conclude your trajectory with a clear and concise summary. Explain the nature of the bug, the logic of your fix, and the steps you took to verify its correctness and safety.

**Guiding Principle:** Act like a senior hardware engineer. Prioritize correctness, safety, and high-quality, test-driven development.

# GUIDE FOR HOW TO USE "sequential_thinking" TOOL:
- Your thinking should be thorough and so it's fine if it's very long. Set total_thoughts to at least 5, but setting it up to 25 is fine as well. You'll need more total thoughts when you are considering multiple possible solutions or root causes for an issue.
- Use this tool as much as you find necessary to improve the quality of your answers.
- You can run bash commands (like tests, a reproduction script, or 'grep'/'find' to find relevant context) in between thoughts.
- The sequential_thinking tool can help you break down complex problems, analyze issues step-by-step, and ensure a thorough approach to problem-solving.
- Don't hesitate to use it multiple times throughout your thought process to enhance the depth and accuracy of your solutions.

If you are sure the issue has been solved, you should call the `task_done` to finish the task.

If there are questions or you need additional information, call the `task_done` to finish the task.

# if there is a custom_instructions.md file, follow the instructions in the file

# Add RAG search before conversion
Before starting the conversion:
1. Use the search_knowledge_chunk tool to search the knowledge base for VHDL/SystemVerilog syntax differences
2. Look for similar conversion examples in the knowledge base
3. Reference specific conversion patterns for the constructs you encounter
4. Use the sequential_thinking tool to plan the conversion strategy"

# ADDITIONAL INSTRUCTIONS FOR HDL CONVERSION:

When converting VHDL to SystemVerilog, follow these critical guidelines:

1. **Syntax Mapping**:
   - VHDL 'entity/architecture' → SystemVerilog 'module/endmodule'
   - VHDL 'signal' → SystemVerilog 'wire' or 'logic'
   - VHDL 'process' → SystemVerilog 'always_ff', 'always_comb', or 'always_latch'
   - VHDL '<=' (signal assignment) → SystemVerilog '<=' (non-blocking) or '=' (blocking)
   - VHDL port modes: 'in/out/inout/buffer' → SystemVerilog 'input/output/inout'

2. **Type System Differences**:
   - VHDL 'STD_LOGIC'/'STD_LOGIC_VECTOR' → SystemVerilog 'logic' type
   - VHDL 'integer' ranges → SystemVerilog sized integers
   - VHDL 'std_logic_1164' library → SystemVerilog built-in types
   - Handle VHDL enumerated types → SystemVerilog enums or parameters

3. **Sequential vs. Combinational Logic**:
   - VHDL 'process(clk)' with 'if rising_edge(clk)' → SystemVerilog 'always_ff @(posedge clk)'
   - VHDL combinational process with sensitivity list → SystemVerilog 'always_comb'
   - Ensure proper use of blocking (=) vs non-blocking (<=) assignments

4. **Common Pitfalls to Avoid**:
   - Don't use VHDL 'out' ports for internal reads (use 'buffer' or signals)
   - Convert VHDL concurrent statements to SystemVerilog 'assign' statements
   - Handle VHDL 'downto' vs SystemVerilog bit ordering carefully
   - Properly convert VHDL generics to SystemVerilog parameters

5. **Verification Requirements**:
   - Create a testbench to verify the converted code
   - Check for simulation equivalence
   - Verify synthesizability of the output
   - Document any conversion assumptions or limitations

## CONVERSION WORKFLOW:

Phase 1 - Analysis:
- Parse and understand the VHDL structure (entities, architectures, processes)
- Identify all signals, ports, types, and dependencies
- Create a conversion plan using sequential_thinking

Phase 2 - Syntax Translation:
- Convert entity declarations to module declarations
- Map VHDL types to SystemVerilog types
- Translate signal declarations
- Convert process blocks to always blocks

Phase 3 - Logic Verification:
- Ensure timing behavior is preserved
- Check for proper blocking/non-blocking usage
- Verify reset and clock handling
- Validate bit vector ordering

Phase 4 - Testbench Creation:
- Generate a SystemVerilog testbench
- Create test cases covering key functionality
- Compare with VHDL simulation if possible

Phase 5 - Documentation:
- Add comments explaining conversion decisions
- Document any behavioral differences
- Note synthesis implications

Phase 6 - Final Review:
- Review the entire conversion
- Verify all functionality is preserved
- Check for any remaining issues
- Ensure proper test coverage

## EXAMPLE CONVERSIONS:

VHDL:
```vhdl
entity counter is
    port(clk, reset : in std_logic;
         count : out std_logic_vector(7 downto 0));
end entity;

architecture rtl of counter is
    signal count_reg : std_logic_vector(7 downto 0);
begin
    process(clk, reset)
    begin
        if reset = '1' then
            count_reg <= (others => '0');
        elsif rising_edge(clk) then
            count_reg <= count_reg + 1;
        end if;
    end process;
    count <= count_reg;
end architecture;
```

SystemVerilog:
```systemverilog
module counter (
    input  logic       clk,
    input  logic       reset,
    output logic [7:0] count
);
    logic [7:0] count_reg;
    
    always_ff @(posedge clk or posedge reset) begin
        if (reset)
            count_reg <= 8'b0;
        else
            count_reg <= count_reg + 1'b1;
    end
    
    assign count = count_reg;
endmodule
```
## VALIDATION STRATEGY:
1. After conversion, do a validation if there is a description in the custom_instructions.md file
    - If there is a description, follow the instructions in the custom_instructions.md file
2. if there are no descriptions, skip this step otherwise fix any issues iteratively
3. if there are no descriptions, skip this step otherwise document the validation results

# Current task context:

Project Path: {project_path}
Task: {task}
"#).to_string();
}

pub struct AlanAgent {
    base: BaseAgentImpl,
    project_path: String,
    base_commit: Option<String>,
    must_patch: String,
    patch_path: Option<String>,
    mcp_servers_config: Option<HashMap<String, MCPServerConfig>>,
    pub allow_mcp_servers: Vec<String>,
    mcp_clients: Vec<Arc<TokioMutex<MCPClient>>>,
    mcp_tools: Vec<Arc<dyn Tool>>,
}

impl AlanAgent {
    pub fn new(
        config: AgentConfig,
        trajectory_recorder: Option<Arc<Mutex<TrajectoryRecorder>>>,
        cli_console: Option<Arc<dyn CLIConsole>>,
    ) -> Result<Self> {
        // Create LLM client
        let model_config = config.model_config.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Model configuration required for AlanAgent"))?;

        let llm_client = create_llm_client(model_config)?;

        // Create base agent
        let base = BaseAgentImpl::new(
            "AlanAgent".to_string(),
            config.clone(),
            llm_client,
            trajectory_recorder,
            cli_console,
        )?;

        // Extract allow_mcp_servers from config
        let allow_mcp_servers = config.allow_mcp_servers.clone();

        Ok(Self {
            base,
            project_path: String::new(),
            base_commit: None,
            must_patch: "false".to_string(),
            patch_path: None,
            mcp_servers_config: config.mcp_servers_config.clone(),
            allow_mcp_servers,
            mcp_clients: Vec::new(),
            mcp_tools: Vec::new(),
        })
    }

    pub async fn initialize_mcp(&mut self) -> Result<()> {
        if let Some(servers_config) = &self.mcp_servers_config {
            for (server_name, server_config) in servers_config {
                // Check if this server is allowed (similar to Python)
                if !self.allow_mcp_servers.is_empty() && !self.allow_mcp_servers.contains(server_name) {
                    tracing::info!("Skipping MCP server '{}' (not in allow list)", server_name);
                    continue;
                }

                if let Some(console) = &self.base.cli_console {
                    console.print_info(&format!("Initializing MCP server: {}", server_name));
                }

                let mut client = MCPClient::new(server_name.clone());

                // Try to connect
                match client.connect(server_config).await {
                    Ok(_) => {
                        // Discover tools
                        match client.list_tools().await {
                            Ok(tools) => {
                                let tool_count = tools.len();
                                let client_arc = Arc::new(TokioMutex::new(client));

                                // Create MCP tool wrappers
                                for tool_def in tools {
                                    let mcp_tool = Arc::new(MCPTool::new(client_arc.clone(), tool_def));
                                    self.mcp_tools.push(mcp_tool);
                                }

                                if let Some(console) = &self.base.cli_console {
                                    console.print_success(&format!(
                                        "MCP server '{}' initialized with {} tools",
                                        server_name, tool_count
                                    ));
                                }

                                self.mcp_clients.push(client_arc);
                            }
                            Err(e) => {
                                if let Some(console) = &self.base.cli_console {
                                    console.print_error(&format!(
                                        "Failed to list tools for MCP server '{}': {}",
                                        server_name, e
                                    ));
                                }
                                // Continue with other servers
                            }
                        }
                    }
                    Err(e) => {
                        if let Some(console) = &self.base.cli_console {
                            console.print_error(&format!(
                                "Failed to connect to MCP server '{}': {}",
                                server_name, e
                            ));
                        }
                        // Continue with other servers
                    }
                }
            }

            // Add MCP tools to the agent's tool collection
            if !self.mcp_tools.is_empty() {
                self.base.tools.extend(self.mcp_tools.clone());
                // Rebuild tool executor with all tools
                self.base.tool_executor = Arc::new(ToolExecutor::new(self.base.tools.clone()));

                if let Some(console) = &self.base.cli_console {
                    console.print_info(&format!(
                        "Total MCP tools registered: {}",
                        self.mcp_tools.len()
                    ));
                }
            }
        }

        Ok(())
    }

    async fn cleanup_mcp_clients(&mut self) -> Result<()> {
        for client in &self.mcp_clients {
            let mut client = client.lock().await;
            if let Err(e) = client.shutdown().await {
                tracing::warn!("Error shutting down MCP client '{}': {}", client.name(), e);
            }
        }

        self.mcp_clients.clear();
        self.mcp_tools.clear();

        Ok(())
    }

    fn parse_task_args(&mut self, task_args: &serde_json::Value) -> Result<()> {
        if let Some(project_path) = task_args.get("project_path").and_then(|v| v.as_str()) {
            self.project_path = project_path.to_string();
        }

        if let Some(base_commit) = task_args.get("base_commit").and_then(|v| v.as_str()) {
            self.base_commit = Some(base_commit.to_string());
        }

        if let Some(must_patch) = task_args.get("must_patch").and_then(|v| v.as_str()) {
            self.must_patch = must_patch.to_string();
        }

        if let Some(patch_path) = task_args.get("patch_path").and_then(|v| v.as_str()) {
            self.patch_path = Some(patch_path.to_string());
        }

        Ok(())
    }
}


impl BaseAgent for AlanAgent {
    fn get_name(&self) -> &str {
        self.base.get_name()
    }

    fn get_max_steps(&self) -> u32 {
        self.base.get_max_steps()
    }

    fn get_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.base.get_tools()
    }

    fn get_tool_executor(&self) -> Arc<ToolExecutor> {
        self.base.get_tool_executor()
    }

    fn get_llm_client(&self) -> Arc<dyn LLMClient> {
        self.base.get_llm_client()
    }

    fn get_trajectory_recorder(&self) -> Option<Arc<Mutex<TrajectoryRecorder>>> {
        self.base.get_trajectory_recorder()
    }

    fn get_cli_console(&self) -> Option<Arc<dyn CLIConsole>> {
        self.base.get_cli_console()
    }

    fn initialize(&mut self) -> Result<()> {
        self.base.initialize()?;
        // MCP initialization needs to be done asynchronously after this
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.base.shutdown()?;

        // Cleanup MCP clients using tokio runtime
        if !self.mcp_clients.is_empty() {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(self.cleanup_mcp_clients())?;
        }

        Ok(())
    }

    fn prepare_system_message(&self, task: &str, task_args: &serde_json::Value) -> String {
        let project_path = task_args.get("project_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let system_prompt = ALAN_AGENT_SYSTEM_PROMPT
            .replace("{project_path}", project_path)
            .replace("{task}", task);

        tracing::debug!("AlanAgent::prepare_system_message called");
        tracing::debug!("Task: {}", task);
        tracing::debug!("Project path: {}", project_path);
        tracing::debug!("System prompt length: {} chars", system_prompt.len());
        tracing::trace!("System prompt first 200 chars: {}", &system_prompt.chars().take(200).collect::<String>());

        system_prompt
    }

    fn run(
        &self,
        task: String,
        task_args: serde_json::Value,
        cancel_flag: Arc<AtomicBool>,
    ) -> Result<String> {
        let mut execution = AgentExecution::new(task.clone());
        execution.start();

        // Record task start
        if let Some(recorder) = self.get_trajectory_recorder() {
            let mut recorder = recorder.lock().unwrap();
            recorder.record_task(&task)?;
        }

        // Prepare initial message - THIS IS THE KEY FIX
        // We call our own prepare_system_message directly instead of relying on trait dispatch
        let system_message = self.prepare_system_message(&task, &task_args);
        let mut messages = vec![LLMMessage::system(system_message)];
        messages.push(LLMMessage::user(task.clone()));

        // Main execution loop
        let max_steps = self.get_max_steps();
        for step_num in 0..max_steps {
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                execution.stop();
                return Err(AgentError::Cancelled.into());
            }

            let done = self.run_step(&mut messages, &mut execution, cancel_flag.clone(), step_num + 1)?;

            if done {
                break;
            }
        }

        // Check if we exceeded max steps
        if execution.step_count() >= max_steps as usize && execution.state != AgentState::Finished {
            execution.finish_with_error(format!("Maximum steps ({}) exceeded", max_steps));
            return Err(AgentError::MaxStepsExceeded(max_steps).into());
        }

        // Return result
        match execution.state {
            AgentState::Finished => Ok(execution.result.unwrap_or_default()),
            AgentState::Error => Err(AgentError::Other(execution.error.unwrap_or_default()).into()),
            AgentState::Stopped => Err(AgentError::Cancelled.into()),
            _ => Err(AgentError::Other("Unexpected agent state".to_string()).into()),
        }
    }

    fn process_response(
        &self,
        response: &LLMResponse,
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

// Manual Clone implementation since we have Arc fields
impl Clone for AlanAgent {
    fn clone(&self) -> Self {
        Self {
            base: BaseAgentImpl {
                name: self.base.name.clone(),
                config: self.base.config.clone(),
                llm_client: self.base.llm_client.clone(),
                tools: self.base.tools.clone(),
                tool_executor: self.base.tool_executor.clone(),
                trajectory_recorder: self.base.trajectory_recorder.clone(),
                cli_console: self.base.cli_console.clone(),
            },
            project_path: self.project_path.clone(),
            base_commit: self.base_commit.clone(),
            must_patch: self.must_patch.clone(),
            patch_path: self.patch_path.clone(),
            mcp_servers_config: self.mcp_servers_config.clone(),
            allow_mcp_servers: self.allow_mcp_servers.clone(),
            mcp_clients: self.mcp_clients.clone(),
            mcp_tools: self.mcp_tools.clone(),
        }
    }
}