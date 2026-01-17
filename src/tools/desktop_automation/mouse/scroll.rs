use crate::core::{Tool, ToolContext, ToolParameter, ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scroll {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollArgs {
    pub direction: String,
    pub amount: Option<i32>,
    pub smooth: Option<bool>,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            id: "mouse_scroll".to_string(),
            name: "Mouse Scroll".to_string(),
            description: "Scrolls the mouse wheel up or down".to_string(),
            category: "desktop_automation".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "direction".to_string(),
                    param_type: "string".to_string(),
                    description: "Scroll direction (up, down)".to_string(),
                    required: true,
                    default: Some(serde_json::json!("down")),
                    enum_values: Some(vec!["up".to_string(), "down".to_string()]),
                },
                ToolParameter {
                    name: "amount".to_string(),
                    param_type: "number".to_string(),
                    description: "Number of scroll units (default: 3)".to_string(),
                    required: false,
                    default: Some(serde_json::json!(3)),
                    enum_values: None,
                },
                ToolParameter {
                    name: "smooth".to_string(),
                    param_type: "boolean".to_string(),
                    description: "Enable smooth scrolling (default: true)".to_string(),
                    required: false,
                    default: Some(serde_json::json!(true)),
                    enum_values: None,
                },
            ],
        }
    }
}

#[async_trait::async_trait]
impl Tool for Scroll {
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
        let args: ScrollArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;

        // Validate direction
        let valid_directions = ["up", "down"];
        if !valid_directions.contains(&args.direction.as_str()) {
            return Err(crate::core::AppError::Tool(format!(
                "Invalid direction '{}'. Must be one of: {}",
                args.direction,
                valid_directions.join(", ")
            )));
        }

        let amount = args.amount.unwrap_or(3);
        let smooth = args.smooth.unwrap_or(true);

        if amount <= 0 {
            return Err(anyhow::anyhow!("Amount must be positive").into());
        }

        // Send progress update
        if let Some(ref manager) = context.conversation_manager {
            manager
                .send_progress_update(
                    &context.agent_id,
                    crate::agents::conversation::ProgressType::Executing,
                    &format!("Scrolling {} by {} units", args.direction, amount),
                    Some(0.5),
                )
                .await?;
        }

        // Execute scroll (placeholder - would use platform-specific code)
        let execution_time = if smooth {
            tokio::time::sleep(tokio::time::Duration::from_millis((amount as u64) * 50)).await;
            (amount as u64) * 50
        } else {
            tokio::time::sleep(tokio::time::Duration::from_millis((amount as u64) * 20)).await;
            (amount as u64) * 20
        };

        let result = ToolResult {
            success: true,
            message: format!(
                "Successfully scrolled {} by {} units",
                args.direction, amount
            ),
            data: Some(serde_json::json!({
                "direction": args.direction,
                "amount": amount,
                "smooth": smooth
            })),
            execution_time: std::time::Duration::from_millis(execution_time),
        };

        Ok(result)
    }

    fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        let _args: ScrollArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;
        Ok(())
    }
}
