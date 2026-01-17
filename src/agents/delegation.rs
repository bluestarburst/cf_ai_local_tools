use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRequest {
    pub target_agent_id: String,
    pub task: String,
    pub source_agent_id: String,
    pub session_id: String,
    pub required_capabilities: Vec<String>,
    pub context: DelegationContext,
    pub timeout: Option<std::time::Duration>,
    pub priority: DelegationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DelegationContext {
    pub shared_context: serde_json::Value,
    pub depth: usize,
    pub history: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DelegationPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for DelegationPriority {
    fn default() -> Self {
        Self::Normal
    }
}

pub fn create_delegation_request(
    target_agent_id: &str,
    task: &str,
    source_agent_id: &str,
    session_id: &str,
    required_capabilities: Vec<String>,
) -> DelegationRequest {
    DelegationRequest {
        target_agent_id: target_agent_id.to_string(),
        task: task.to_string(),
        source_agent_id: source_agent_id.to_string(),
        session_id: session_id.to_string(),
        required_capabilities,
        context: DelegationContext::default(),
        timeout: None,
        priority: DelegationPriority::Normal,
    }
}
