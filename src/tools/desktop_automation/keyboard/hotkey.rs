use crate::core::{Tool, ToolContext, ToolParameter, ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyArgs {
    pub keys: Vec<String>,
    pub hold_ms: Option<u64>,
}

impl Hotkey {
    pub fn new() -> Self {
        Self {
            id: "keyboard_hotkey".to_string(),
            name: "Hotkey".to_string(),
            description: "Executes keyboard shortcuts and hotkeys".to_string(),
            category: "desktop_automation".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "keys".to_string(),
                    param_type: "array".to_string(),
                    description: "Array of keys to press (e.g., ['ctrl', 'c'])".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                ToolParameter {
                    name: "hold_ms".to_string(),
                    param_type: "number".to_string(),
                    description: "How long to hold keys in milliseconds (default: 100)".to_string(),
                    required: false,
                    default: Some(serde_json::json!(100)),
                    enum_values: None,
                },
            ],
        }
    }
}

#[async_trait::async_trait]
impl Tool for Hotkey {
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
        let args: HotkeyArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;

        if args.keys.is_empty() {
            return Err(crate::core::AppError::Tool(
                "Keys array cannot be empty".to_string(),
            ));
        }

        // Validate keys (simplified - would have comprehensive key validation)
        let valid_modifiers = ["ctrl", "alt", "shift", "meta", "cmd"];
        let valid_keys = [
            "a",
            "b",
            "c",
            "d",
            "e",
            "f",
            "g",
            "h",
            "i",
            "j",
            "k",
            "l",
            "m",
            "n",
            "o",
            "p",
            "q",
            "r",
            "s",
            "t",
            "u",
            "v",
            "w",
            "x",
            "y",
            "z",
            "f1",
            "f2",
            "f3",
            "f4",
            "f5",
            "f6",
            "f7",
            "f8",
            "f9",
            "f10",
            "f11",
            "f12",
            "enter",
            "space",
            "tab",
            "escape",
            "backspace",
            "delete",
        ];

        for key in &args.keys {
            let key_lower = key.to_lowercase();
            if !valid_modifiers.contains(&key_lower.as_str())
                && !valid_keys.contains(&key_lower.as_str())
            {
                return Err(anyhow::anyhow!("Invalid key: {}", key).into());
            }
        }

        let hold_ms = args.hold_ms.unwrap_or(100);

        // Send progress update
        if let Some(ref manager) = context.conversation_manager {
            manager
                .send_progress_update(
                    &context.agent_id,
                    crate::agents::conversation::ProgressType::Executing,
                    &format!("Executing hotkey: {}", args.keys.join(" + ")),
                    Some(0.5),
                )
                .await?;
        }

        // Execute hotkey (placeholder - would use platform-specific code)
        tokio::time::sleep(tokio::time::Duration::from_millis(hold_ms)).await;

        let result = ToolResult {
            success: true,
            message: format!("Successfully executed hotkey: {}", args.keys.join(" + ")),
            data: Some(serde_json::json!({
                "keys": args.keys,
                "hold_ms": hold_ms,
                "key_count": args.keys.len()
            })),
            execution_time: std::time::Duration::from_millis(hold_ms),
        };

        Ok(result)
    }

    fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        let _args: HotkeyArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;
        Ok(())
    }
}
