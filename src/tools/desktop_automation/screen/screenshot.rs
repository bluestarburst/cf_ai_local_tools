use crate::core::{Tool, ToolContext, ToolParameter, ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotArgs {
    pub region: Option<ScreenshotRegion>,
    pub format: Option<String>,
    pub save_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Screenshot {
    pub fn new() -> Self {
        Self {
            id: "screen_screenshot".to_string(),
            name: "Take Screenshot".to_string(),
            description: "Captures screenshot of entire screen or specific region".to_string(),
            category: "desktop_automation".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "region".to_string(),
                    param_type: "object".to_string(),
                    description: "Optional region to capture (x, y, width, height)".to_string(),
                    required: false,
                    default: None,
                    enum_values: None,
                },
                ToolParameter {
                    name: "format".to_string(),
                    param_type: "string".to_string(),
                    description: "Image format (png, jpg, default: png)".to_string(),
                    required: false,
                    default: Some(serde_json::json!("png")),
                    enum_values: Some(vec![
                        "png".to_string(),
                        "jpg".to_string(),
                        "jpeg".to_string(),
                    ]),
                },
                ToolParameter {
                    name: "save_path".to_string(),
                    param_type: "string".to_string(),
                    description: "Optional path to save screenshot".to_string(),
                    required: false,
                    default: None,
                    enum_values: None,
                },
            ],
        }
    }
}

#[async_trait::async_trait]
impl Tool for Screenshot {
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
        let args: ScreenshotArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;

        let format = args.format.unwrap_or_else(|| "png".to_string());

        // Validate format
        let valid_formats = ["png", "jpg", "jpeg"];
        if !valid_formats.contains(&format.to_lowercase().as_str()) {
            return Err(crate::core::AppError::Tool(format!(
                "Invalid format '{}'. Must be one of: {}",
                format,
                valid_formats.join(", ")
            )));
        }

        // Validate region if provided
        if let Some(ref region) = args.region {
            if region.width == 0 || region.height == 0 {
                return Err(crate::core::AppError::Tool(
                    "Region width and height must be greater than 0".to_string(),
                ));
            }
        }

        // Send progress update
        if let Some(ref manager) = context.conversation_manager {
            let region_type = if args.region.is_some() {
                "regional"
            } else {
                "full screen"
            };
            manager
                .send_progress_update(
                    &context.agent_id,
                    crate::agents::conversation::ProgressType::Executing,
                    &format!("Taking {} screenshot", region_type),
                    Some(0.5),
                )
                .await?;
        }

        // Capture screenshot (placeholder - would use platform-specific code)
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let screenshot_size = 1024000; // Placeholder size in bytes

        let result_data = if let Some(save_path) = &args.save_path {
            // Would save to file and return file info
            serde_json::json!({
                "saved_to": save_path,
                "format": format,
                "size": screenshot_size,
                "region": args.region
            })
        } else {
            // Would return base64 encoded image
            serde_json::json!({
                "data_base64": "placeholder_base64_data", // Placeholder
                "format": format,
                "size": screenshot_size,
                "region": args.region
            })
        };

        let region_type = if args.region.is_some() {
            "regional"
        } else {
            "full screen"
        };
        let result = ToolResult {
            success: true,
            message: format!("Successfully captured {} screenshot", region_type),
            data: Some(result_data),
            execution_time: std::time::Duration::from_millis(200),
        };

        Ok(result)
    }

    fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        let _args: ScreenshotArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;
        Ok(())
    }
}
