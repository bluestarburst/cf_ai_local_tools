//! Agent trait and types for the enhanced local Rust app

use crate::core::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core trait that all agents must implement
use dyn_clone::DynClone;

/// Core trait that all agents must implement
#[async_trait]
pub trait Agent: DynClone + Send + Sync {
    /// Get the unique identifier for this agent
    fn id(&self) -> &str;

    /// Get the human-readable name for this agent
    fn name(&self) -> &str;

    /// Get a description of what this agent does
    fn description(&self) -> &str;

    /// Get the version of this agent
    fn version(&self) -> &str;

    /// Get the capabilities this agent provides
    fn capabilities(&self) -> &[String];

    /// Get the tools this agent depends on
    fn tool_dependencies(&self) -> &[String];

    /// Get the system prompt for this agent
    fn system_prompt(&self) -> &str;

    /// Get the reasoning model configuration
    fn reasoning_config(&self) -> &ReasoningConfig;

    /// Execute a task with this agent
    async fn execute(
        &self,
        task: &str,
        context: &AgentContext,
        llm: &dyn LLMClient,
        conversation_manager: Option<
            std::sync::Arc<dyn crate::agents::conversation::ConversationManager>,
        >,
        available_tools: &[Box<dyn crate::core::Tool>],
    ) -> Result<AgentResult>;

    /// Calculate confidence score for handling a specific task (0.0-1.0)
    fn can_handle_task(&self, task: &str) -> f32;
}

dyn_clone::clone_trait_object!(Agent);

/// Configuration for agent reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    /// Model ID to use for reasoning
    pub model_id: String,
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Whether to use separate reasoning model
    pub separate_reasoning_model: bool,
    /// Reasoning model ID (if different from main model)
    pub reasoning_model_id: Option<String>,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            model_id: "@cf/meta/llama-3.1-8b-instruct".to_string(),
            max_iterations: 10,
            separate_reasoning_model: false,
            reasoning_model_id: None,
        }
    }
}

/// Context for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Agent ID
    pub agent_id: String,
    /// Conversation history
    pub messages: Vec<ConversationMessage>,
    /// Shared state
    pub shared_state: HashMap<String, serde_json::Value>,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

impl AgentContext {
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            messages: Vec::new(),
            shared_state: HashMap::new(),
            metadata: ExecutionMetadata::default(),
        }
    }
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Message role (user, assistant, system)
    pub role: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: String,
}

/// Execution metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionMetadata {
    /// Start time
    pub start_time: Option<String>,
    /// Current iteration
    pub current_iteration: usize,
    /// Goal completion progress
    pub goal_progress: f32,
}

/// Result of agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// Whether execution was successful
    pub success: bool,
    /// Final response message
    pub response: String,
    /// Execution steps taken
    pub steps: Vec<ExecutionStep>,
    /// Total execution time
    pub execution_time: std::time::Duration,
    /// Final context state
    pub final_context: AgentContext,
}

/// A single execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number
    pub step_number: usize,
    /// Type of step
    pub step_type: StepType,
    /// Content of the step
    pub content: String,
    /// Tool call if this step involves a tool
    pub tool_call: Option<ToolCall>,
    /// Tool observation if this step is a tool result
    pub tool_observation: Option<ToolObservation>,
    /// Timestamp
    pub timestamp: String,
}

/// Type of execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    /// Thinking/reasoning phase
    Thinking,
    /// Planning phase
    Planning,
    /// Tool execution phase
    Action,
    /// Observation phase
    Observation,
    /// Reflection phase
    Reflection,
    /// Completion phase
    Completion,
}

/// Tool call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool name
    pub tool_name: String,
    /// Tool arguments
    pub arguments: serde_json::Value,
    /// Execution time
    pub execution_time: std::time::Duration,
}

/// Tool observation/result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolObservation {
    /// Whether tool execution was successful
    pub success: bool,
    /// Tool result message
    pub message: String,
    /// Tool result data
    pub data: Option<serde_json::Value>,
    /// Error if any
    pub error: Option<String>,
}

// LLM Types (moved from llm module to avoid circular dependencies)

/// Core trait for LLM clients
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Chat with the LLM (without tools)
    async fn chat(&self, messages: &[LLMMessage], model_id: &str) -> Result<LLMResponse>;

    /// Chat with the LLM (with tools)
    async fn chat_with_tools(
        &self,
        messages: &[LLMMessage],
        model_id: &str,
        tools: Option<Vec<LLMTool>>,
    ) -> Result<LLMResponse>;
}

/// A message in LLM conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMMessage {
    /// Message role (system, user, assistant)
    pub role: String,
    /// Message content
    pub content: String,
    /// Optional tool calls (for assistant messages)
    pub tool_calls: Option<Vec<LLMToolCall>>,
}

/// LLM tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool parameters schema
    pub parameters: serde_json::Value,
}

/// LLM tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMToolCall {
    /// Tool name
    pub name: String,
    /// Tool arguments
    pub arguments: serde_json::Value,
    /// Tool call ID (if provided by LLM)
    pub id: Option<String>,
}

/// LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    /// Response content
    pub response: String,
    /// Tool calls made by the LLM
    pub tool_calls: Option<Vec<LLMToolCall>>,
    /// Model used
    pub model: String,
    /// Usage information
    pub usage: Option<LLMUsage>,
    /// Response time
    pub response_time: std::time::Duration,
}

/// LLM token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMUsage {
    /// Input tokens used
    pub input_tokens: u32,
    /// Output tokens used
    pub output_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}
