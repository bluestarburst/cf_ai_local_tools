use crate::core::{Tool, ToolContext, ToolParameter, ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPosition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionArgs {}

impl GetPosition {
    pub fn new() -> Self {
        Self {
            id: "screen_get_position".to_string(),
            name: "Get Mouse Position".to_string(),
            description: "Gets the current mouse cursor position on screen".to_string(),
            category: "desktop_automation".to_string(),
            parameters: vec![], // No parameters needed
        }
    }
}

#[async_trait::async_trait]
impl Tool for GetPosition {
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
        let _args: GetPositionArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;

        // Send progress update
        if let Some(ref manager) = context.conversation_manager {
            manager
                .send_progress_update(
                    &context.agent_id,
                    crate::agents::conversation::ProgressType::Executing,
                    "Getting current mouse position",
                    Some(0.5),
                )
                .await?;
        }

        // Get mouse position (placeholder - would use platform-specific code)
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Placeholder coordinates
        let x = 500.0;
        let y = 300.0;

        let result = ToolResult {
            success: true,
            message: format!("Mouse position: ({}, {})", x, y),
            data: Some(serde_json::json!({
                "x": x,
                "y": y,
                "screen_coordinates": true
            })),
            execution_time: std::time::Duration::from_millis(50),
        };

        Ok(result)
    }

    fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        let _args: GetPositionArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;
        Ok(())
    }
}
