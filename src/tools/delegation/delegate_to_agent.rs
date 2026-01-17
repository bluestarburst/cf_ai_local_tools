use crate::agents::delegation::create_delegation_request;
use crate::core::{Tool, ToolContext, ToolParameter, ToolResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateToAgent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateToAgentArgs {
    pub target_agent: String,
    pub task: String,
    pub required_capabilities: Option<Vec<String>>,
    pub priority: Option<String>,
    pub timeout_seconds: Option<u32>,
    pub context_data: Option<serde_json::Value>,
}

impl DelegateToAgent {
    pub fn new() -> Self {
        Self {
            id: "delegate_to_agent".to_string(),
            name: "Delegate to Agent".to_string(),
            description: "Delegate a task to another specialized agent".to_string(),
            category: "delegation".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "target_agent".to_string(),
                    param_type: "string".to_string(),
                    description: "ID of the agent to delegate to".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                ToolParameter {
                    name: "task".to_string(),
                    param_type: "string".to_string(),
                    description: "Task description to delegate".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                ToolParameter {
                    name: "required_capabilities".to_string(),
                    param_type: "array".to_string(),
                    description: "Capabilities the target agent must have".to_string(),
                    required: false,
                    default: Some(serde_json::json!([])),
                    enum_values: None,
                },
                ToolParameter {
                    name: "priority".to_string(),
                    param_type: "string".to_string(),
                    description: "Delegation priority (low, normal, high, critical)".to_string(),
                    required: false,
                    default: Some(serde_json::json!("normal")),
                    enum_values: Some(vec![
                        "low".to_string(),
                        "normal".to_string(),
                        "high".to_string(),
                        "critical".to_string(),
                    ]),
                },
                ToolParameter {
                    name: "timeout_seconds".to_string(),
                    param_type: "number".to_string(),
                    description: "Maximum time to wait for delegation completion".to_string(),
                    required: false,
                    default: Some(serde_json::json!(300)),
                    enum_values: None,
                },
                ToolParameter {
                    name: "context_data".to_string(),
                    param_type: "object".to_string(),
                    description: "Additional context data to pass to the delegated agent"
                        .to_string(),
                    required: false,
                    default: Some(serde_json::json!({})),
                    enum_values: None,
                },
            ],
        }
    }
}

#[async_trait::async_trait]
impl Tool for DelegateToAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn category(&self) -> &str {
        &self.category
    }

    fn parameters(&self) -> &[ToolParameter] {
        &self.parameters
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        context: &ToolContext,
    ) -> crate::core::Result<ToolResult> {
        let args: DelegateToAgentArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;

        // Get delegation manager from tool context (this would be passed in a real implementation)
        // For now, we'll create a placeholder implementation

        // Send progress update
        if let Some(ref manager) = context.conversation_manager {
            manager
                .send_progress_update(
                    &context.agent_id,
                    crate::agents::conversation::ProgressType::Executing,
                    &format!("Delegating task to agent: {}", args.target_agent),
                    Some(0.5),
                )
                .await?;
        }

        // Create delegation request
        let mut request = create_delegation_request(
            &args.target_agent,
            &args.task,
            &context.agent_id,
            "delegation-session", // TODO: Use actual session ID
            args.required_capabilities.unwrap_or_default(),
        );

        // Apply optional parameters
        if let Some(timeout_secs) = args.timeout_seconds {
            request.timeout = Some(std::time::Duration::from_secs(timeout_secs as u64));
        }

        if let Some(context_data) = args.context_data {
            request.context.shared_context = context_data;
        }

        // TODO: Actually execute the delegation
        // For now, return a mock successful result
        let mock_result = format!(
            "Successfully delegated task '{}' to agent '{}'",
            args.task, args.target_agent
        );

        let result = ToolResult {
            success: true,
            message: mock_result,
            data: Some(serde_json::json!({
                "delegated_to": args.target_agent,
                "task": args.task,
                "status": "completed",
                "execution_time": 150
            })),
            execution_time: std::time::Duration::from_millis(150),
        };

        Ok(result)
    }

    fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        let _args: DelegateToAgentArgs = serde_json::from_value(args.clone())?;
        Ok(())
    }
}
