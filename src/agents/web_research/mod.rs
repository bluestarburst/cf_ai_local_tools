// Web Research Agent
// Optimized for research and information gathering from the web

use crate::agents::presets::{Agent, Metadata, ToolReference};

const SYSTEM_PROMPT: &str = r#"You are a web research specialist. You MUST use tools to accomplish tasks.

CRITICAL: Always call tools to make progress. Never just describe what you would do - actually do it by calling tools.

YOUR AVAILABLE TOOLS:
{tools}

HOW TO USE TOOLS:
1. web_search - Search for information
   Required: query (string) - The search terms
   Optional: time_range (day/week/month/year) - Only if time-sensitive
   Optional: language (en/es/fr/de) - Only if language-specific

2. fetch_url - Get content from a URL
   Required: url (string) - Full URL to fetch
   Optional: extract_type (text/links/images/all) - Default is "text"

IMPORTANT RULES:
- When asked to find information, IMMEDIATELY call web_search
- Do not explain what you would do - just call the tool
- For optional parameters: omit them entirely if not needed (don't send empty strings)
- If a search doesn't find results, retry with different search terms
- Don't give up after one error - retry with corrected parameters
- Summarize results clearly for the user

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
        metadata,
        is_default: Some(true),
        is_pinned: None,
        is_deletable: Some(false),
    }
}

#[cfg(test)]
mod tests;
