// Orchestrator Agent
// Routes tasks to specialized agents using delegation

use crate::agents::presets::{Agent, Metadata, ToolReference};
use crate::agents::prompt_interpolation::{self, interpolate_all};

const SYSTEM_PROMPT_TEMPLATE: &str = r#"You are an orchestrator agent that routes tasks to specialized agents.

AVAILABLE AGENTS:
{available_agents}

DELEGATION RULES:
1. Delegate ONCE per task - never call the same agent twice for the same task
2. After delegation, report the result to the user (success or failure)
3. If delegation fails or returns incomplete, tell the user and ask for clarification
4. DO NOT loop - if you already delegated to an agent, do not delegate again

WHEN TO DELEGATE:
- Desktop automation (mouse, keyboard, clicks) → desktop-automation-agent
- Web search or research → web-research-agent
- Code tasks → code-assistant-agent

WHEN TO RESPOND DIRECTLY (NO delegation):
- Greetings (hello, hi)
- Questions about capabilities
- When user message is unclear

CRITICAL: After receiving a delegation result, you MUST respond to the user with the result. Do not call delegate_to_agent again.

Available tools: {tools}

Your purpose: {purpose}"#;

pub fn create_agent(metadata: Metadata) -> Agent {
    let purpose = "Planning complex tasks and coordinating specialized agents";

    // Interpolate the system prompt with available agents and tools
    let delegatable_agents = prompt_interpolation::get_delegatable_agents();
    let system_prompt = interpolate_all(
        SYSTEM_PROMPT_TEMPLATE,
        purpose,
        Some(&["delegate_to_agent"]), // Only show delegation tool
        Some(&delegatable_agents),
    );

    Agent {
        id: "orchestrator-agent".to_string(),
        name: "Orchestrator".to_string(),
        purpose: purpose.to_string(),
        system_prompt,
        tools: vec![ToolReference {
            tool_id: "delegate_to_agent".to_string(),
            enabled: true,
        }],
        model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
        max_iterations: 10,
        metadata,
        is_default: Some(true),
        is_pinned: Some(true),
        is_deletable: Some(false),
    }
}

#[cfg(test)]
mod tests;
