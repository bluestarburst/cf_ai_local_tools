/// Dynamic prompt template interpolation
///
/// This module provides utilities for interpolating system prompts with runtime values
/// like available tools, agents, and agent purposes.
use crate::agents::ToolDefinition;
use crate::tools;

/// Interpolate a prompt template with available tools
///
/// Replaces {tools} placeholder with formatted list of available tools
pub fn interpolate_tools(template: &str, tool_filter: Option<&[&str]>) -> String {
    let available_tools = tools::get_all_tools();

    // Filter tools if specified (e.g., only delegation tools for orchestrator)
    let filtered_tools: Vec<_> = if let Some(filter) = tool_filter {
        available_tools
            .iter()
            .filter(|t| filter.contains(&t.id.as_str()))
            .collect()
    } else {
        available_tools.iter().collect()
    };

    // Format tools as a readable list
    let tools_text = format_tools_for_prompt(&filtered_tools);
    template.replace("{tools}", &tools_text)
}

/// Interpolate a prompt template with available agents
///
/// Replaces {available_agents} placeholder with formatted list of agents
/// that can be delegated to
pub fn interpolate_agents(template: &str, agent_list: &[(String, String)]) -> String {
    let agents_text = format_agents_for_prompt(agent_list);
    template.replace("{available_agents}", &agents_text)
}

/// Interpolate a prompt template with purpose
///
/// Replaces {purpose} placeholder with the agent's purpose
pub fn interpolate_purpose(template: &str, purpose: &str) -> String {
    template.replace("{purpose}", purpose)
}

/// Interpolate all common placeholders at once
pub fn interpolate_all(
    template: &str,
    purpose: &str,
    tool_filter: Option<&[&str]>,
    agent_list: Option<&[(String, String)]>,
) -> String {
    let mut result = template.to_string();

    // Interpolate purpose first
    result = interpolate_purpose(&result, purpose);

    // Interpolate tools
    result = interpolate_tools(&result, tool_filter);

    // Interpolate agents if provided
    if let Some(agents) = agent_list {
        result = interpolate_agents(&result, agents);
    }

    result
}

/// Format tools for display in a prompt
///
/// Creates a readable list of tools with descriptions
fn format_tools_for_prompt(tools: &[&ToolDefinition]) -> String {
    if tools.is_empty() {
        "No tools available".to_string()
    } else {
        tools
            .iter()
            .map(|tool| format!("- {} ({}): {}", tool.name, tool.id, tool.description))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Format agents for display in a prompt
///
/// Creates a readable list of agents that can be delegated to
fn format_agents_for_prompt(agents: &[(String, String)]) -> String {
    if agents.is_empty() {
        "No agents available for delegation".to_string()
    } else {
        agents
            .iter()
            .map(|(id, description)| format!("- {}: {}", id, description))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Get available agent descriptions for delegation
///
/// Returns a list of (agent_id, description) tuples for agents that can be delegated to
pub fn get_delegatable_agents() -> Vec<(String, String)> {
    vec![
        (
            "desktop-automation-agent".to_string(),
            "Mouse/keyboard control, clicking, typing, GUI automation".to_string(),
        ),
        (
            "web-research-agent".to_string(),
            "Browsing, searching, information gathering from the web".to_string(),
        ),
        (
            "code-assistant-agent".to_string(),
            "Code analysis, writing, debugging, and programming tasks".to_string(),
        ),
        (
            "general-assistant".to_string(),
            "Multi-step tasks requiring multiple tools and coordination".to_string(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_purpose() {
        let template = "Your purpose: {purpose}";
        let result = interpolate_purpose(template, "Testing");
        assert_eq!(result, "Your purpose: Testing");
    }

    #[test]
    fn test_interpolate_tools() {
        let template = "Available tools: {tools}";
        let result = interpolate_tools(template, None);
        assert!(result.contains("Available tools:"));
        assert!(result.contains("mouse") || result.contains("web"));
    }

    #[test]
    fn test_interpolate_agents() {
        let agents = vec![
            ("agent1".to_string(), "Does something".to_string()),
            ("agent2".to_string(), "Does something else".to_string()),
        ];
        let template = "AVAILABLE AGENTS:\n{available_agents}";
        let result = interpolate_agents(template, &agents);
        assert!(result.contains("agent1"));
        assert!(result.contains("agent2"));
        assert!(!result.contains("{available_agents}"));
    }

    #[test]
    fn test_interpolate_all() {
        let template = "Purpose: {purpose}\nTools: {tools}";
        let result = interpolate_all(template, "Testing", None, None);
        assert!(result.contains("Purpose: Testing"));
        assert!(!result.contains("{purpose}"));
        assert!(!result.contains("{tools}"));
    }

    #[test]
    fn test_filter_tools() {
        let template = "Tools: {tools}";
        let result = interpolate_tools(template, Some(&["delegate_to_agent"]));
        assert!(result.contains("Tools:"));
        // Should contain delegation tool
        assert!(result.contains("delegate") || result.is_empty());
    }

    #[test]
    fn test_delegatable_agents() {
        let agents = get_delegatable_agents();
        assert!(agents.len() >= 3);
        assert!(agents
            .iter()
            .any(|(id, _)| id == "desktop-automation-agent"));
        assert!(agents.iter().any(|(id, _)| id == "web-research-agent"));
        assert!(agents.iter().any(|(id, _)| id == "code-assistant-agent"));
    }
}
