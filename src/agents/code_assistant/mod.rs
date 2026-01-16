// Code Assistant Agent
// Code analysis, generation, and debugging assistance

use crate::agents::presets::{Agent, Metadata, ToolReference};

const SYSTEM_PROMPT: &str = r#"You are a helpful AI assistant that thinks step-by-step before taking action.

When solving problems:
1. First, understand the task clearly
2. Break down the problem into steps
3. Reason through each step
4. Execute tools as needed
5. Verify results

Your purpose: {purpose}

Think carefully and show your reasoning."#;

pub fn create_agent(metadata: Metadata) -> Agent {
    Agent {
        id: "code-assistant-agent".to_string(),
        name: "Code Assistant Agent".to_string(),
        purpose: "Code analysis, generation, and debugging assistance".to_string(),
        system_prompt: SYSTEM_PROMPT.to_string(),
        tools: vec![
            ToolReference {
                tool_id: "keyboard_input".to_string(),
                enabled: true,
            },
            ToolReference {
                tool_id: "take_screenshot".to_string(),
                enabled: true,
            },
            ToolReference {
                tool_id: "mouse_move".to_string(),
                enabled: true,
            },
            ToolReference {
                tool_id: "mouse_click".to_string(),
                enabled: true,
            },
        ],
        model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
        max_iterations: 4,
        metadata,
        is_default: Some(true),
        is_pinned: None,
        is_deletable: Some(false),
    }
}

#[cfg(test)]
mod tests;
