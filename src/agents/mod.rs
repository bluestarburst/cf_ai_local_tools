// Agent module - handles agent definitions, storage, and execution
pub mod presets;
pub mod prompt_interpolation;
pub mod prompt_storage;
pub mod prompts;
pub mod react_loop;
pub mod storage;

// Individual agent modules
pub mod code_assistant;
pub mod conversational;
pub mod desktop_automation;
pub mod orchestrator;
pub mod test_debug;
pub mod web_research;

// Public exports - only what main.rs and tools module need
pub use presets::{get_all_default_agents, get_all_default_prompts};
pub use prompt_storage::{Prompt, PromptStorage};
pub use react_loop::{
    execute, AgentConfig, ExecutionStep, StepSender, ToolDefinition, ToolParameter,
};
pub use storage::{Agent, AgentStorage};
