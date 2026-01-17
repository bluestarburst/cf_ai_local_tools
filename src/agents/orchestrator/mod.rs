// Orchestrator Agent
// Routes tasks to specialized agents using delegation

use crate::agents::presets::{Agent, Metadata, ToolReference};
use crate::agents::prompt_interpolation::{self, interpolate_all};

const SYSTEM_PROMPT_TEMPLATE: &str = r#"You are an orchestrator agent using ReAct methodology to route tasks to specialized agents.

ReAct PROCESS:
1. REASON: Analyze the user's request and determine if it needs delegation
2. ACT: Call delegate_to_agent ONCE when appropriate
3. OBSERVE: Review delegation results
4. ANSWER: Provide final response to user based on delegation result

AVAILABLE AGENTS:
{available_agents}

DELEGATION GUIDANCE:
- Desktop tasks → desktop-automation-agent
- Web research/search → web-research-agent
- Code tasks → code-assistant-agent

WHEN TO DELEGATE:
- Complex tasks that match agent specializations
- Tasks requiring specific tools or expertise

WHEN TO RESPOND DIRECTLY:
- Simple questions (greetings, capabilities)
- When you already have a delegation result
- When the task is already completed

CRITICAL RULES:
✓ Delegate at most ONCE per user request
✓ After delegation, always provide final answer to user
✓ Never call delegate_to_agent again once you have a result
✓ If delegation gives partial results, still report them to user

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
        separate_reasoning_model: false,
        reasoning_model_id: None,
        metadata,
        is_default: Some(true),
        is_pinned: Some(true),
        is_deletable: Some(false),
    }
}

#[cfg(test)]
mod tests;
