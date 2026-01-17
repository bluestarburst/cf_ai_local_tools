use crate::agents::prompt_interpolation;
use crate::agents::{ToolDefinition, ToolParameter};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Delegation request - returned when a tool wants to delegate to another agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRequest {
    pub agent_id: String,
    pub task: String,
}

/// Result of delegation detection - reserved for future use
#[allow(dead_code)]
pub enum DelegationResult {
    /// No delegation requested - just a normal result
    Normal(String),
    /// Delegation requested - caller should handle this
    Delegate(DelegationRequest),
}

/// Get utility and delegation tools
///
/// This function dynamically creates the delegation tool with current available agents
/// The agent list is not hardcoded, so it updates when agents are added/removed
pub fn get_delegation_tools() -> Vec<ToolDefinition> {
    let delegatable_agents = prompt_interpolation::get_delegatable_agents();

    // Build the enum values and description dynamically
    let agent_ids: Vec<String> = delegatable_agents
        .iter()
        .map(|(id, _)| id.clone())
        .collect();

    let agents_description = delegatable_agents
        .iter()
        .map(|(id, desc)| format!("{} ({})", id, desc))
        .collect::<Vec<_>>()
        .join(", ");

    vec![
        ToolDefinition {
            id: "delegate_to_agent".to_string(),
            name: "Delegate to Agent".to_string(),
            description: format!(
                "Delegate a task to a specialized agent. Available agents: {}. \
                Use this tool to execute tasks like moving the mouse, clicking, typing, or searching the web.",
                agents_description
            ),
            category: "delegation".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "agent_id".to_string(),
                    param_type: "string".to_string(),
                    description: format!(
                        "ID of the agent to delegate to. Must be one of: {}",
                        agent_ids.join(", ")
                    ),
                    required: true,
                    enum_values: Some(agent_ids),
                    default: None,
                },
                ToolParameter {
                    name: "task".to_string(),
                    param_type: "string".to_string(),
                    description: "Clear description of the task to delegate (e.g., 'Move the mouse to coordinates x=500, y=500')".to_string(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
            ],
            returns_observation: true,
        },
    ]
}

/// Get all utility tools
pub fn get_all_utility_tools() -> Vec<ToolDefinition> {
    get_delegation_tools()
}

/// Execute a utility tool
///
/// # Arguments
/// * `tool_name` - The ID of the tool to execute
/// * `arguments` - JSON arguments for the tool
///
/// # Returns
/// * `Ok(String)` - Success message or delegation marker
/// * `Err(anyhow::Error)` - If the tool is unknown or arguments are invalid
///
/// Note: For delegation, this returns a special JSON-encoded DelegationRequest
/// that the caller should detect and handle by executing the delegated agent.
pub fn execute_utility_tool(tool_name: &str, arguments: &serde_json::Value) -> Result<String> {
    // Verify this is a utility tool
    if !get_all_utility_tools().iter().any(|t| t.id == tool_name) {
        return Err(anyhow::anyhow!("Unknown utility tool: {}", tool_name));
    }

    match tool_name {
        "delegate_to_agent" => {
            // Parse delegation parameters
            let agent_id = arguments["agent_id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing agent_id"))?
                .to_string();
            let task = arguments["task"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing task"))?
                .to_string();

            // Return a special marker that indicates delegation is requested
            // The caller (main.rs) should detect this and handle delegation
            let delegation = DelegationRequest { agent_id, task };

            // Return as JSON with special prefix for easy detection
            Ok(format!(
                "__DELEGATE__:{}",
                serde_json::to_string(&delegation)?
            ))
        }
        _ => Err(anyhow::anyhow!("Unknown utility tool: {}", tool_name)),
    }
}

/// Check if a tool result is a delegation request
pub fn is_delegation_request(result: &str) -> Option<DelegationRequest> {
    if let Some(json_str) = result.strip_prefix("__DELEGATE__:") {
        serde_json::from_str(json_str).ok()
    } else {
        None
    }
}
