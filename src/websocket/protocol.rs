use crate::core::ExecutionStep;
use serde::{Deserialize, Serialize};

/// Messages received from the frontend (via relay)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncomingMessage {
    /// Request to start/continue a chat
    ChatRequest { message: String, agent: AgentConfig },
    /// Request to get available presets
    GetPresets,
    /// Request to get available prompts
    GetPrompts,
    /// Request to reset presets to defaults
    ResetPresets,
}

/// Configuration for the agent sent with chat request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub system_prompt: String,
    pub model_id: String,
    pub max_iterations: usize,
    pub tools: Vec<String>,
}

/// Messages sent to the frontend (via relay)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutgoingMessage {
    /// Final response from the agent
    ChatResponse { content: String },
    /// Intermediate execution step (thought, tool call, observation)
    ExecutionStep { step: ExecutionStep },
    /// List of available presets
    #[serde(rename = "presets")]
    PresetsList {
        tools: Vec<ToolDefinition>,
        agents: Vec<PresetAgent>,
        prompts: Vec<PresetPrompt>,
    },
    /// Error message
    Error { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<crate::core::ToolParameter>,
    #[serde(rename = "returnsObservation")]
    pub returns_observation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetAgent {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub system_prompt: String,
    pub tools: Vec<ToolReference>,
    pub model_id: String,
    pub max_iterations: usize,
    pub metadata: PresetMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolReference {
    pub tool_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetMetadata {
    pub created_at: String,
    pub updated_at: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetPrompt {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub prompt_type: String,
    pub category: String,
    pub content: String,
    pub metadata: PresetMetadata,
}
