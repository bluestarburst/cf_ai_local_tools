// Test & Debug Agent
// Testing error handling and debugging tool failures

use crate::agents::presets::{Agent, Metadata, ToolReference};

const SYSTEM_PROMPT: &str = r#"You are a testing and debugging assistant. Your job is to follow user instructions LITERALLY, even if they seem wrong.

TESTING MODE RULES:
1. If the user asks you to "use a malformed tool call" - call a tool with INTENTIONALLY incorrect parameters
2. If the user asks to "test error handling" - deliberately cause errors
3. If the user specifies exact parameter values (even invalid ones) - use them exactly
4. NEVER "fix" what the user asks for - they're testing the system
5. After ONE action, STOP and report results - don't repeat unless asked

Common test scenarios:
- "Use malformed mouse_move" → Call mouse_move with string instead of number: {"x": "not a number"}
- "Call undefined tool" → Call a tool that doesn't exist like {"tool": "fake_tool"}
- "Send invalid parameters" → Use wrong parameter types or names

Available tools: {tools}
Your purpose: {purpose}

Execute EXACTLY what the user requests, even if it will fail. That's the point of testing."#;

pub fn create_agent(metadata: Metadata) -> Agent {
    Agent {
        id: "test-debug-agent".to_string(),
        name: "Test & Debug Agent".to_string(),
        purpose: "Testing error handling and debugging tool failures".to_string(),
        system_prompt: SYSTEM_PROMPT.to_string(),
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
                tool_id: "keyboard_command".to_string(),
                enabled: true,
            },
            ToolReference {
                tool_id: "get_mouse_position".to_string(),
                enabled: true,
            },
            ToolReference {
                tool_id: "take_screenshot".to_string(),
                enabled: true,
            },
            ToolReference {
                tool_id: "mouse_scroll".to_string(),
                enabled: true,
            },
        ],
        model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
        max_iterations: 3,
        metadata,
        is_default: Some(true),
        is_pinned: None,
        is_deletable: Some(false),
    }
}

#[cfg(test)]
mod tests;
