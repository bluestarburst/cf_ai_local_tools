use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCursorArgs {
    pub x: f64,
    pub y: f64,
    pub speed: Option<f64>,
    pub smooth: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCursor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<crate::core::ToolParameter>,
}

impl MoveCursor {
    pub fn new() -> Self {
        Self {
            id: "mouse_move".to_string(),
            name: "Move Cursor".to_string(),
            description: "Moves the mouse cursor to the specified coordinates".to_string(),
            category: "desktop_automation".to_string(),
            parameters: vec![
                crate::core::ToolParameter {
                    name: "x".to_string(),
                    param_type: "number".to_string(),
                    description: "X coordinate on screen".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                crate::core::ToolParameter {
                    name: "y".to_string(),
                    param_type: "number".to_string(),
                    description: "Y coordinate on screen".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                crate::core::ToolParameter {
                    name: "speed".to_string(),
                    param_type: "number".to_string(),
                    description: "Movement speed (0.0-1.0)".to_string(),
                    required: false,
                    default: Some(serde_json::json!(0.5)),
                    enum_values: None,
                },
                crate::core::ToolParameter {
                    name: "smooth".to_string(),
                    param_type: "boolean".to_string(),
                    description: "Enable smooth movement animation".to_string(),
                    required: false,
                    default: Some(serde_json::json!(true)),
                    enum_values: None,
                },
            ],
        }
    }

    pub async fn execute(
        &self,
        args: &serde_json::Value,
        context: &crate::core::ToolContext,
    ) -> crate::core::Result<crate::core::ToolResult> {
        // Parse arguments
        let args: MoveCursorArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;

        // Validate coordinates
        if args.x < 0.0 || args.y < 0.0 {
            return Err(crate::core::AppError::Tool(
                "Coordinates must be non-negative".to_string(),
            ));
        }

        // Send progress update if conversation manager is available
        if let Some(ref manager) = context.conversation_manager {
            manager
                .send_progress_update(
                    &context.agent_id,
                    crate::agents::conversation::ProgressType::Executing,
                    &format!("Moving mouse to ({}, {})", args.x, args.y),
                    Some(0.5),
                )
                .await?;
        }

        // Initialize RustAutoGui
        let gui = rustautogui::RustAutoGui::new(false).map_err(|e| {
            crate::core::AppError::Tool(format!("Failed to init automation: {}", e))
        })?;

        // Calculate duration based on speed (inverse relationship)
        let speed = args.speed.unwrap_or(0.5);
        let duration = if args.smooth.unwrap_or(true) {
            (1.0 - speed) * 2.0 // 0.5 speed = 1.0 second duration
        } else {
            0.0 // Instant move
        };

        // Execute real mouse movement
        let start = std::time::Instant::now();
        gui.move_mouse_to_pos(args.x as u32, args.y as u32, duration as f32)
            .map_err(|e| crate::core::AppError::Tool(format!("Mouse move failed: {}", e)))?;

        let elapsed = start.elapsed();

        let result = crate::core::ToolResult {
            success: true,
            message: format!("Successfully moved cursor to ({}, {})", args.x, args.y),
            data: Some(serde_json::json!({
                "final_position": {"x": args.x, "y": args.y},
                "speed": speed,
                "duration_ms": elapsed.as_millis()
            })),
            execution_time: elapsed,
        };

        Ok(result)
    }

    pub fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        // Validate JSON structure
        let _args: MoveCursorArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::core::Tool for MoveCursor {
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

    fn parameters(&self) -> &[crate::core::ToolParameter] {
        &self.parameters
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        context: &crate::core::ToolContext,
    ) -> crate::core::Result<crate::core::ToolResult> {
        self.execute(args, context).await
    }

    fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        self.validate_args(args)
    }
}
