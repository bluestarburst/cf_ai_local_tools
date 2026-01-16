/// Reusable system prompt templates for agents
///
/// These are general-purpose prompt patterns that users can select
/// when creating custom agents. Each agent module also defines its
/// own specialized inline prompt.

pub mod cot_standard;
pub mod react_basic;
pub mod enhanced_reasoning;

use crate::agents::presets::SystemPromptPreset;
use std::collections::HashMap;

/// Get all reusable prompt templates
pub fn get_all_prompts() -> HashMap<String, SystemPromptPreset> {
    let mut prompts = HashMap::new();

    prompts.insert("cot-standard".to_string(), cot_standard::get_prompt());
    prompts.insert("react-basic".to_string(), react_basic::get_prompt());
    prompts.insert("enhanced-reasoning".to_string(), enhanced_reasoning::get_prompt());

    prompts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_prompts_load() {
        let prompts = get_all_prompts();
        assert_eq!(prompts.len(), 3, "Should have exactly 3 prompt templates");
        assert!(prompts.contains_key("cot-standard"));
        assert!(prompts.contains_key("react-basic"));
        assert!(prompts.contains_key("enhanced-reasoning"));
    }

    #[test]
    fn test_all_prompts_have_content() {
        let prompts = get_all_prompts();
        for (id, prompt) in prompts.iter() {
            assert!(!prompt.content.is_empty(), "Prompt {} has no content", id);
            assert!(!prompt.name.is_empty(), "Prompt {} has no name", id);
        }
    }
}
