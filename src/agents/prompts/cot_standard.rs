/// Chain-of-Thought Standard Prompt
///
/// Pure reasoning-focused prompt with step-by-step thinking.
/// Optimized for tasks that benefit from breaking down the problem before acting.

use crate::agents::presets::{SystemPromptPreset, Metadata};

const TEMPLATE: &str = r#"You are a helpful AI assistant that thinks step-by-step before taking action.

When solving problems:
1. First, understand the task clearly
2. Break down the problem into steps
3. Reason through each step
4. Execute tools as needed
5. Verify results

Your purpose: {purpose}

Think carefully and show your reasoning."#;

pub fn get_prompt() -> SystemPromptPreset {
    let now = chrono::Utc::now().to_rfc3339();
    SystemPromptPreset {
        id: "cot-standard".to_string(),
        name: "Chain-of-Thought Standard".to_string(),
        description: "Pure reasoning-focused prompt with step-by-step thinking".to_string(),
        preset_type: "systemPrompt".to_string(),
        category: "built-in".to_string(),
        content: TEMPLATE.to_string(),
        metadata: Metadata {
            created_at: now.clone(),
            updated_at: now,
            version: "1.0.0".to_string(),
            author: Some("CF AI Local Tools".to_string()),
            tags: Some(vec!["reasoning".to_string(), "cot".to_string()]),
        },
        is_locked: Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cot_standard_prompt_structure() {
        let prompt = get_prompt();
        assert_eq!(prompt.id, "cot-standard");
        assert_eq!(prompt.name, "Chain-of-Thought Standard");
        assert!(prompt.content.contains("step-by-step"));
        assert!(prompt.content.contains("{purpose}"));
        assert!(prompt.is_locked.unwrap());
    }
}
