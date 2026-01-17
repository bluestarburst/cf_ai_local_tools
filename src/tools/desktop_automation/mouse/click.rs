use crate::core::{Tool, ToolContext, ToolParameter, ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Click {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickArgs {
    pub button: String,
    pub double_click: Option<bool>,
    pub delay_ms: Option<u64>,
}

impl Click {
    pub fn new() -> Self {
        Self {
            id: "mouse_click".to_string(),
            name: "Mouse Click".to_string(),
            description: "Clicks a mouse button at the current cursor position".to_string(),
            category: "desktop_automation".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "button".to_string(),
                    param_type: "string".to_string(),
                    description: "Mouse button to click (left, right, middle)".to_string(),
                    required: true,
                    default: Some(serde_json::json!("left")),
                    enum_values: Some(vec![
                        "left".to_string(),
                        "right".to_string(),
                        "middle".to_string(),
                    ]),
                },
                ToolParameter {
                    name: "double_click".to_string(),
                    param_type: "boolean".to_string(),
                    description: "Perform a double-click".to_string(),
                    required: false,
                    default: Some(serde_json::json!(false)),
                    enum_values: None,
                },
                ToolParameter {
                    name: "delay_ms".to_string(),
                    param_type: "number".to_string(),
                    description: "Delay between clicks in milliseconds (for double-click)"
                        .to_string(),
                    required: false,
                    default: Some(serde_json::json!(50)),
                    enum_values: None,
                },
            ],
        }
    }
}

#[async_trait::async_trait]
impl Tool for Click {
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
        let args: ClickArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;

        // Validate button
        let valid_buttons = ["left", "right", "middle"];
        if !valid_buttons.contains(&args.button.as_str()) {
            return Err(crate::core::AppError::Tool(format!(
                "Invalid button '{}'. Must be one of: {}",
                args.button,
                valid_buttons.join(", ")
            )));
        }

        let double_click = args.double_click.unwrap_or(false);
        let delay_ms = args.delay_ms.unwrap_or(50);

        // Send progress update
        if let Some(ref manager) = context.conversation_manager {
            let click_type = if double_click {
                "double-click"
            } else {
                "click"
            };
            manager
                .send_progress_update(
                    &context.agent_id,
                    crate::agents::conversation::ProgressType::Executing,
                    &format!("Performing {} with {} button", click_type, args.button),
                    Some(0.5),
                )
                .await?;
        }

        // Execute real click using rustautogui
        let gui = rustautogui::RustAutoGui::new(false).map_err(|e| {
            crate::core::AppError::Tool(format!("Failed to init automation: {}", e))
        })?;

        let get_button = || match args.button.as_str() {
            "left" => rustautogui::MouseClick::LEFT,
            "right" => rustautogui::MouseClick::RIGHT,
            "middle" => rustautogui::MouseClick::MIDDLE,
            _ => rustautogui::MouseClick::LEFT,
        };

        let start = std::time::Instant::now();

        if double_click {
            gui.click(get_button())
                .map_err(|e| crate::core::AppError::Tool(format!("First click failed: {}", e)))?;
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            gui.click(get_button())
                .map_err(|e| crate::core::AppError::Tool(format!("Second click failed: {}", e)))?;
        } else {
            gui.click(get_button())
                .map_err(|e| crate::core::AppError::Tool(format!("Click failed: {}", e)))?;
        }

        let elapsed = start.elapsed();

        let click_type = if double_click {
            "double-clicked"
        } else {
            "clicked"
        };
        let result = ToolResult {
            success: true,
            message: format!("Successfully {} with {} button", click_type, args.button),
            data: Some(serde_json::json!({
                "button": args.button,
                "double_click": double_click,
                "delay_ms": delay_ms
            })),
            execution_time: elapsed,
        };

        Ok(result)
    }

    fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        let _args: ClickArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;
        Ok(())
    }
}
