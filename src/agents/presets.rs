// Agent and Prompt Presets
// This module aggregates default agents and system prompts from their respective modules.
// - Prompts are defined in src/agents/prompts/ submodules
// - Agents are defined in src/agents/{agent_type}/ submodules
// - Tools are managed in src/tools/ module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::prompts;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolReference {
    #[serde(rename = "toolId")]
    pub tool_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub purpose: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: String,
    pub tools: Vec<ToolReference>,
    #[serde(rename = "modelId")]
    pub model_id: String,
    #[serde(rename = "maxIterations")]
    pub max_iterations: usize,
    pub metadata: Metadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isPinned")]
    pub is_pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isDeletable")]
    pub is_deletable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPromptPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub preset_type: String, // "systemPrompt"
    pub category: String,    // "built-in"
    pub content: String,
    pub metadata: Metadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isLocked")]
    pub is_locked: Option<bool>,
}

fn create_metadata() -> Metadata {
    let now = chrono::Utc::now().to_rfc3339();
    Metadata {
        created_at: now.clone(),
        updated_at: now,
        version: "1.0.0".to_string(),
        author: Some("CF AI Local Tools".to_string()),
        tags: None,
    }
}

/// Get all default prompts
///
/// This function aggregates prompts from the prompts submodule.
/// Each prompt is defined in its own module for better organization and maintainability.
pub fn get_default_prompts() -> HashMap<String, SystemPromptPreset> {
    prompts::get_all_prompts()
}

pub fn get_default_agents() -> HashMap<String, Agent> {
    let mut agents = HashMap::new();
    let metadata = create_metadata();

    // Use individual agent modules
    agents.insert(
        "orchestrator-agent".to_string(),
        super::orchestrator::create_agent(metadata.clone()),
    );

    agents.insert(
        "conversational-agent".to_string(),
        super::conversational::create_agent(metadata.clone()),
    );

    agents.insert(
        "desktop-automation-agent".to_string(),
        super::desktop_automation::create_agent(metadata.clone()),
    );

    agents.insert(
        "web-research-agent".to_string(),
        super::web_research::create_agent(metadata.clone()),
    );

    agents.insert(
        "code-assistant-agent".to_string(),
        super::code_assistant::create_agent(metadata.clone()),
    );

    agents.insert(
        "test-debug-agent".to_string(),
        super::test_debug::create_agent(metadata.clone()),
    );

    agents
}

// Public API for getting default presets
pub fn get_all_default_agents() -> Vec<Agent> {
    get_default_agents().into_values().collect()
}

pub fn get_all_default_prompts() -> Vec<SystemPromptPreset> {
    get_default_prompts().into_values().collect()
}
