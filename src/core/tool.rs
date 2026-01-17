//! Tool trait and types for the enhanced local Rust app

use crate::core::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub enum_values: Option<Vec<String>>,
}

/// Core trait that all tools must implement
use dyn_clone::DynClone;

/// Core trait that all tools must implement
#[async_trait]
pub trait Tool: DynClone + Send + Sync {
    /// Get the unique identifier for this tool
    fn id(&self) -> &str;

    /// Get the human-readable name for this tool
    fn name(&self) -> &str;

    /// Get a description of what this tool does
    fn description(&self) -> &str;

    /// Get the category this tool belongs to
    fn category(&self) -> &str;

    /// Get the parameters this tool accepts
    fn parameters(&self) -> &[ToolParameter];

    /// Execute the tool with given arguments
    async fn execute(&self, args: &serde_json::Value, context: &ToolContext) -> Result<ToolResult>;

    /// Validate tool arguments before execution
    fn validate_args(&self, args: &serde_json::Value) -> Result<()>;
}

dyn_clone::clone_trait_object!(Tool);

/// Tool context for execution
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Agent ID executing the tool
    pub agent_id: String,
    /// Conversation manager for updates
    pub conversation_manager:
        Option<std::sync::Arc<dyn crate::agents::conversation::ConversationManager>>,
    /// Tool execution state
    pub execution_state: std::sync::Arc<tokio::sync::RwLock<ToolExecutionState>>,
}

/// Tool execution state
#[derive(Debug, Clone, Default)]
pub struct ToolExecutionState {
    /// Tools executed so far
    pub executed_tools: Vec<String>,
    /// Current iteration
    pub current_iteration: usize,
    /// Loop detection data
    pub loop_detection: HashMap<String, usize>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether execution was successful
    pub success: bool,
    /// Result message
    pub message: String,
    /// Result data
    pub data: Option<serde_json::Value>,
    /// Execution time
    pub execution_time: std::time::Duration,
}

/// Loop detection for tool calls
pub struct LoopDetector {
    recent_calls: std::collections::VecDeque<(String, serde_json::Value)>,
    max_history: usize,
}

impl LoopDetector {
    pub fn new(max_history: usize) -> Self {
        Self {
            recent_calls: std::collections::VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn check_loop(&mut self, tool_name: &str, args: &serde_json::Value) -> bool {
        let call_signature = (tool_name.to_string(), args.clone());

        // Count occurrences in recent history
        let count = self
            .recent_calls
            .iter()
            .filter(|(name, args)| name == tool_name && args == args)
            .count();

        // Add current call to history
        if self.recent_calls.len() >= self.max_history {
            self.recent_calls.pop_front();
        }
        self.recent_calls.push_back(call_signature);

        // Detect loop if same call appears 3+ times
        count >= 2
    }
}
