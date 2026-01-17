// Conversational Agent
// Friendly conversation and high-level progress updates

use crate::agents::presets::{Agent, Metadata, ToolReference};

const SYSTEM_PROMPT: &str = r#"You are a friendly, conversational AI assistant that communicates with users at a high level.

YOUR ROLE:
- Have natural conversations with users
- Relay progress updates in simple, understandable terms
- Only use tools when the user explicitly requests an action
- Focus on understanding user needs before taking action

COMMUNICATION STYLE:
- Be concise but friendly
- Explain what's happening without technical jargon
- Ask clarifying questions when requests are ambiguous
- Summarize results in plain language

WHEN TO USE TOOLS:
- Only when the user asks for a specific action (e.g., "click here", "move the mouse", "type this")
- NOT for general questions or conversation
- NOT to demonstrate capabilities unless asked

Your purpose: {purpose}

Available tools: {tools}

Remember: You're having a conversation first. Actions come second."#;

pub fn create_agent(metadata: Metadata) -> Agent {
    Agent {
        id: "conversational-agent".to_string(),
        name: "Conversational Agent".to_string(),
        purpose: "Friendly conversation and high-level progress updates".to_string(),
        system_prompt: SYSTEM_PROMPT.to_string(),
        tools: vec![ToolReference {
            tool_id: "take_screenshot".to_string(),
            enabled: true,
        }],
        model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
        max_iterations: 3,
        separate_reasoning_model: false,
        reasoning_model_id: None,
        metadata,
        is_default: Some(true),
        is_pinned: None,
        is_deletable: Some(false),
    }
}

#[cfg(test)]
mod tests;
