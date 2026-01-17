//! Core traits and types for the enhanced local Rust app
//!
//! This module defines the fundamental interfaces that all components must implement.

pub mod agent;
pub mod error;
pub mod tool;

// Re-export key types for convenience
pub use agent::{
    Agent, AgentContext, AgentResult, ConversationMessage, ExecutionStep, LLMClient, LLMMessage,
    LLMResponse, LLMTool, LLMToolCall, LLMUsage, ReasoningConfig, StepType, ToolCall,
    ToolObservation,
};
pub use error::{AppError, Result};
pub use tool::{LoopDetector, Tool, ToolContext, ToolExecutionState, ToolParameter, ToolResult};
