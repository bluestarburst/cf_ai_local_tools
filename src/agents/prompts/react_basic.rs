/// ReAct Basic Prompt
///
/// Basic Reasoning + Acting loop with tool usage.
/// Implements the core ReAct pattern for efficient tool execution.
use crate::agents::presets::{Metadata, SystemPromptPreset};

const TEMPLATE: &str = r#"You are an AI agent that can use tools to complete tasks.

Use this format for each step:
Thought: [Your reasoning about what to do next]
Action: [tool_name with parameters]
Observation: [Result from the tool]

Then continue to the next step or conclude if done.

Your purpose: {purpose}

Be precise and efficient in your actions."#;

pub fn get_prompt() -> SystemPromptPreset {
    let now = chrono::Utc::now().to_rfc3339();
    SystemPromptPreset {
        id: "react-basic".to_string(),
        name: "ReAct Basic".to_string(),
        description: "Basic Reasoning + Acting loop with tool usage".to_string(),
        preset_type: "systemPrompt".to_string(),
        category: "built-in".to_string(),
        content: TEMPLATE.to_string(),
        metadata: Metadata {
            created_at: now.clone(),
            updated_at: now,
            version: "1.0.0".to_string(),
            author: Some("CF AI Local Tools".to_string()),
            tags: Some(vec!["react".to_string(), "basic".to_string()]),
        },
        is_locked: Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_react_basic_prompt_structure() {
        let prompt = get_prompt();
        assert_eq!(prompt.id, "react-basic");
        assert!(prompt.content.contains("Thought:"));
        assert!(prompt.content.contains("Action:"));
        assert!(prompt.content.contains("Observation:"));
    }
}
