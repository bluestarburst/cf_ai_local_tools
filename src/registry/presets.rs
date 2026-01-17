use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetMetadata {
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
pub struct PresetAgent {
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
    #[serde(rename = "separateReasoningModel", default)]
    pub separate_reasoning_model: bool,
    #[serde(rename = "reasoningModelId", skip_serializing_if = "Option::is_none")]
    pub reasoning_model_id: Option<String>,
    pub metadata: PresetMetadata,
    #[serde(rename = "isDefault", skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(rename = "isPinned", skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
    #[serde(rename = "isDeletable", skip_serializing_if = "Option::is_none")]
    pub is_deletable: Option<bool>,
}

fn create_metadata() -> PresetMetadata {
    let now = chrono::Utc::now().to_rfc3339();
    PresetMetadata {
        created_at: now.clone(),
        updated_at: now,
        version: "1.0.0".to_string(),
        author: Some("CF AI Local Tools".to_string()),
        tags: None,
    }
}

/// Get default agent presets
pub fn get_default_presets() -> Vec<PresetAgent> {
    let metadata = create_metadata();

    vec![
        // Conversational Agent
        PresetAgent {
            id: "conversational-agent".to_string(),
            name: "Conversational Agent".to_string(),
            purpose: "Friendly conversation and high-level progress updates".to_string(),
            system_prompt: include_str!("../agents/conversational/prompt.txt").to_string(),
            tools: vec![],
            model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
            max_iterations: 10,
            separate_reasoning_model: false,
            reasoning_model_id: None,
            metadata: metadata.clone(),
            is_default: Some(true),
            is_pinned: None,
            is_deletable: Some(false),
        },
        // Desktop Automation Agent
        PresetAgent {
            id: "desktop-automation-agent".to_string(),
            name: "Desktop Automation Agent".to_string(),
            purpose: "Precise desktop task automation with mouse and keyboard control".to_string(),
            system_prompt: include_str!("../agents/desktop_automation/prompt.txt").to_string(),
            tools: vec![
                ToolReference {
                    tool_id: "mouse_move".to_string(),
                    enabled: true,
                },
                ToolReference {
                    tool_id: "mouse_click".to_string(),
                    enabled: true,
                },
                ToolReference {
                    tool_id: "keyboard_input".to_string(),
                    enabled: true,
                },
                ToolReference {
                    tool_id: "get_mouse_position".to_string(),
                    enabled: true,
                },
            ],
            model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
            max_iterations: 3,
            separate_reasoning_model: false,
            reasoning_model_id: None,
            metadata: metadata.clone(),
            is_default: Some(true),
            is_pinned: None,
            is_deletable: Some(false),
        },
        // Web Research Agent
        PresetAgent {
            id: "web-research-agent".to_string(),
            name: "Web Research Agent".to_string(),
            purpose: "Research and information gathering using real web search".to_string(),
            system_prompt: include_str!("../agents/web_research/prompt.txt").to_string(),
            tools: vec![
                ToolReference {
                    tool_id: "web_search".to_string(),
                    enabled: true,
                },
                ToolReference {
                    tool_id: "fetch_url".to_string(),
                    enabled: true,
                },
            ],
            model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
            max_iterations: 8,
            separate_reasoning_model: false,
            reasoning_model_id: None,
            metadata,
            is_default: Some(true),
            is_pinned: None,
            is_deletable: Some(false),
        },
    ]
}
