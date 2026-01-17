//! Enhanced Local Rust App
//!
//! A modular, dynamic agent system with plug-and-play tools and thinking capabilities.

pub mod agents;
pub mod config;
pub mod core;
pub mod llm;
pub mod registry;
pub mod tools;
pub mod utils;
pub mod websocket;

// Re-export key types for convenience
pub use agents::conversation::{ConversationManager, ProgressType};
pub use agents::registry::AgentRegistry;
pub use agents::{ConversationalAgent, DesktopAutomationAgent, WebResearchAgent};
pub use core::agent::{
    ConversationMessage, ExecutionStep, LLMClient, LLMMessage, LLMResponse, LLMTool, LLMToolCall,
    LLMUsage, ReasoningConfig, StepType, ToolCall, ToolObservation,
};
pub use core::{
    Agent, AgentContext, AgentResult, Tool, ToolContext, ToolExecutionState, ToolResult,
};
pub use llm::{HttpClient, MockLLMClient};
pub use tools::registry::{
    DefaultToolRegistry, ToolRegistry, ToolRegistry as RegistryToolRegistry,
};
