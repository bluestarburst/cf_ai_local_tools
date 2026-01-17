//! Enhanced LLM client for the local Rust app

pub mod client;

/// Re-export client types
pub use client::{HttpClient, MockLLMClient};

// Re-export from core module for convenience
pub use crate::core::{LLMClient, LLMMessage, LLMResponse, LLMTool, LLMToolCall, LLMUsage};
