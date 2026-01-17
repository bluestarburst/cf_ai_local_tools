// Web Research Agent
// Optimized for research and information gathering from the web

use crate::agents::presets::{Agent, Metadata, ToolReference};

const SYSTEM_PROMPT: &str = r#"You are a web research specialist using ReAct methodology.

ReAct PROCESS:
1. REASON: Analyze what information you need and if you have enough to answer
2. ACT: Call tools only when you need more specific information
3. OBSERVE: Review results and decide next step
4. ANSWER: Provide final answer when you have sufficient information

AVAILABLE TOOLS:
{tools}

TOOL GUIDANCE:
- web_search: Use first to find relevant sources and URLs
- fetch_url: Use to get detailed content from promising URLs

STOPPING CRITERIA - PROVIDE FINAL ANSWER WHEN:
✓ You've searched the web and found relevant sources
✓ You've read 2-3 key pages about the topic
✓ You have enough information to give a comprehensive answer
✓ Continuing would just repeat similar information

FINAL ANSWER FORMAT:
When ready, provide your answer directly without calling any more tools.
Be comprehensive, organized, and cite your sources.

Your purpose: {purpose}"#;

pub fn create_agent(metadata: Metadata) -> Agent {
    Agent {
        id: "web-research-agent".to_string(),
        name: "Web Research Agent".to_string(),
        purpose: "Research and information gathering using real web search".to_string(),
        system_prompt: SYSTEM_PROMPT.to_string(),
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
    }
}

#[cfg(test)]
mod tests;
