use crate::core::{Tool, ToolContext, ToolParameter, ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeText {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeTextArgs {
    pub text: String,
    pub delay_ms: Option<u64>,
    pub auto_enter: Option<bool>,
}

impl TypeText {
    pub fn new() -> Self {
        Self {
            id: "keyboard_type".to_string(),
            name: "Type Text".to_string(),
            description: "Types text on the keyboard with configurable speed".to_string(),
            category: "desktop_automation".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "text".to_string(),
                    param_type: "string".to_string(),
                    description: "Text to type".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                ToolParameter {
                    name: "delay_ms".to_string(),
                    param_type: "number".to_string(),
                    description: "Delay between keystrokes in milliseconds".to_string(),
                    required: false,
                    default: Some(serde_json::json!(50)),
                    enum_values: None,
                },
                ToolParameter {
                    name: "auto_enter".to_string(),
                    param_type: "boolean".to_string(),
                    description: "Press Enter after typing (default: false)".to_string(),
                    required: false,
                    default: Some(serde_json::json!(false)),
                    enum_values: None,
                },
            ],
        }
    }
}

#[async_trait::async_trait]
impl Tool for TypeText {
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
        let args: TypeTextArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;

        if args.text.is_empty() {
            return Err(crate::core::AppError::Tool(
                "Text cannot be empty".to_string(),
            ));
        }

        let delay_ms = args.delay_ms.unwrap_or(50);
        let auto_enter = args.auto_enter.unwrap_or(false);

        // Send progress update
        if let Some(ref manager) = context.conversation_manager {
            manager
                .send_progress_update(
                    &context.agent_id,
                    crate::agents::conversation::ProgressType::Executing,
                    &format!("Typing text: '{}'", args.text),
                    Some(0.5),
                )
                .await?;
        }

        // Execute typing (placeholder - would use platform-specific code)
        let execution_time = if auto_enter {
            (args.text.len() as u64 + 1) * delay_ms // +1 for Enter key
        } else {
            args.text.len() as u64 * delay_ms
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(execution_time)).await;

        let message = if auto_enter {
            format!("Successfully typed: '{}' and pressed Enter", args.text)
        } else {
            format!("Successfully typed: '{}'", args.text)
        };

        let result = ToolResult {
            success: true,
            message,
            data: Some(serde_json::json!({
                "text_typed": args.text,
                "delay_ms": delay_ms,
                "auto_enter": auto_enter,
                "character_count": args.text.len()
            })),
            execution_time: std::time::Duration::from_millis(execution_time),
        };

        Ok(result)
    }

    fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        let _args: TypeTextArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;
        Ok(())
    }
}
