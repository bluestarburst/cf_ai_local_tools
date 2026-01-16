use anyhow::{Context, Result};
use rustautogui::{MouseClick, RustAutoGui};
use serde::{Deserialize, Serialize};

/// Command received for computer automation
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    MouseMove { x: u32, y: u32, #[serde(default = "default_duration")] duration: f32 },
    MouseClick { button: String },
    MouseScroll { direction: String, #[serde(default = "default_intensity")] intensity: u32 },
    KeyboardInput { text: String },
    KeyboardCommand { command: String },
    Screenshot,
    GetMousePosition,
}

fn default_duration() -> f32 {
    1.0
}

fn default_intensity() -> u32 {
    3
}

/// Response from computer automation
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Success { message: String },
    Error { error: String },
    MousePosition { x: i32, y: i32 },
    #[allow(dead_code)]
    Screenshot { data: String }, // base64 encoded - reserved for future use
}

/// Handler for computer automation commands
pub struct AutomationHandler {
    gui: RustAutoGui,
}

impl AutomationHandler {
    pub fn new() -> Result<Self> {
        let gui = RustAutoGui::new(false)
            .context("Failed to initialize RustAutoGui")?;
        Ok(Self { gui })
    }

    pub fn handle_command(&self, cmd: Command) -> Response {
        match cmd {
            Command::MouseMove { x, y, duration } => {
                match self.gui.move_mouse_to_pos(x, y, duration) {
                    Ok(_) => Response::Success {
                        message: format!("Moved mouse to ({}, {})", x, y),
                    },
                    Err(e) => Response::Error {
                        error: format!("Mouse move failed: {}", e),
                    },
                }
            }
            Command::MouseClick { button } => {
                let btn = match button.as_str() {
                    "left" => MouseClick::LEFT,
                    "right" => MouseClick::RIGHT,
                    "middle" => MouseClick::MIDDLE,
                    _ => {
                        return Response::Error {
                            error: format!("Invalid button: {}", button),
                        }
                    }
                };
                match self.gui.click(btn) {
                    Ok(_) => Response::Success {
                        message: format!("Clicked {} button", button),
                    },
                    Err(e) => Response::Error {
                        error: format!("Click failed: {}", e),
                    },
                }
            }
            Command::MouseScroll { direction, intensity } => {
                let result = match direction.as_str() {
                    "up" => self.gui.scroll_up(intensity),
                    "down" => self.gui.scroll_down(intensity),
                    "left" => self.gui.scroll_left(intensity),
                    "right" => self.gui.scroll_right(intensity),
                    _ => {
                        return Response::Error {
                            error: format!("Invalid scroll direction: {}", direction),
                        }
                    }
                };
                match result {
                    Ok(_) => Response::Success {
                        message: format!("Scrolled {} with intensity {}", direction, intensity),
                    },
                    Err(e) => Response::Error {
                        error: format!("Scroll failed: {}", e),
                    },
                }
            }
            Command::KeyboardInput { text } => {
                match self.gui.keyboard_input(&text) {
                    Ok(_) => Response::Success {
                        message: format!("Typed: {}", text),
                    },
                    Err(e) => Response::Error {
                        error: format!("Keyboard input failed: {}", e),
                    },
                }
            }
            Command::KeyboardCommand { command } => {
                match self.gui.keyboard_command(&command) {
                    Ok(_) => Response::Success {
                        message: format!("Executed keyboard command: {}", command),
                    },
                    Err(e) => Response::Error {
                        error: format!("Keyboard command failed: {}", e),
                    },
                }
            }
            Command::GetMousePosition => {
                match self.gui.get_mouse_position() {
                    Ok((x, y)) => Response::MousePosition { x, y },
                    Err(e) => Response::Error {
                        error: format!("Failed to get mouse position: {}", e),
                    },
                }
            }
            Command::Screenshot => Response::Error {
                error: "Screenshot not yet implemented".to_string(),
            },
        }
    }
}

/// Helper to parse a number from JSON value (handles both number and string)
pub fn parse_number(value: &serde_json::Value, field_name: &str) -> Result<u32> {
    match value {
        serde_json::Value::Number(n) => n.as_u64()
            .ok_or_else(|| anyhow::anyhow!("Number {} is out of range for u32", field_name))
            .map(|v| v as u32),
        serde_json::Value::String(s) => s.parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Parameter '{}' must be a valid number, got: '{}'", field_name, s)),
        _ => Err(anyhow::anyhow!("Parameter '{}' is required and must be a number", field_name)),
    }
}

/// Helper to parse a float from JSON value (handles both number and string)
pub fn parse_float(value: &serde_json::Value, field_name: &str) -> Result<f32> {
    match value {
        serde_json::Value::Number(n) => n.as_f64()
            .ok_or_else(|| anyhow::anyhow!("Number {} is out of range for f32", field_name))
            .map(|v| v as f32),
        serde_json::Value::String(s) => s.parse::<f32>()
            .map_err(|_| anyhow::anyhow!("Parameter '{}' must be a valid number, got: '{}'", field_name, s)),
        _ => Err(anyhow::anyhow!("Parameter '{}' is required and must be a number", field_name)),
    }
}

/// Helper to validate an enum value against allowed values
pub fn validate_enum(value: &str, field_name: &str, allowed: &[&str]) -> Result<String> {
    if allowed.contains(&value) {
        Ok(value.to_string())
    } else {
        Err(anyhow::anyhow!(
            "Parameter '{}' must be one of [{}], got: '{}'",
            field_name,
            allowed.join(", "),
            value
        ))
    }
}

/// Helper to parse a required string parameter
pub fn parse_string(value: &serde_json::Value, field_name: &str) -> Result<String> {
    value.as_str()
        .ok_or_else(|| anyhow::anyhow!("Parameter '{}' is required and must be a string", field_name))
        .map(|s| s.to_string())
}

/// Convert Response enum to String result
pub fn format_response(response: Response) -> Result<String> {
    match response {
        Response::Success { message } => Ok(message),
        Response::Error { error } => Err(anyhow::anyhow!("Tool execution error: {}", error)),
        Response::MousePosition { x, y } => Ok(format!("Mouse position: ({}, {})", x, y)),
        Response::Screenshot { data } => Ok(format!("Screenshot captured: {} bytes", data.len())),
    }
}

/// Create an automation executor for tool execution
/// 
/// This function creates a callback that executes automation tools by:
/// 1. Parsing the JSON arguments into Command structs
/// 2. Executing the command using the AutomationHandler
/// 3. Converting the Response into a Result<String>
pub fn create_executor(handler: &AutomationHandler) -> impl Fn(&str, &serde_json::Value) -> Result<String> + '_ {
    move |tool_name: &str, arguments: &serde_json::Value| -> Result<String> {
        // Parse tool arguments and dispatch to appropriate command based on tool name
        match tool_name {
            "mouse_move" => {
                let x = parse_number(&arguments["x"], "x")?;
                let y = parse_number(&arguments["y"], "y")?;
                let duration = arguments.get("duration")
                    .map(|v| parse_float(v, "duration"))
                    .transpose()?
                    .unwrap_or(1.0);

                let cmd = Command::MouseMove { x, y, duration };
                let response = handler.handle_command(cmd);
                format_response(response)
            },
            "mouse_click" => {
                let button_str = parse_string(&arguments["button"], "button")?;
                let button = validate_enum(&button_str, "button", &["left", "right", "middle"])?;

                let cmd = Command::MouseClick { button };
                let response = handler.handle_command(cmd);
                format_response(response)
            },
            "mouse_scroll" => {
                let direction_str = parse_string(&arguments["direction"], "direction")?;
                let direction = validate_enum(&direction_str, "direction", &["up", "down", "left", "right"])?;
                let intensity = arguments.get("intensity")
                    .map(|v| parse_number(v, "intensity"))
                    .transpose()?
                    .unwrap_or(3);

                let cmd = Command::MouseScroll { direction, intensity };
                let response = handler.handle_command(cmd);
                format_response(response)
            },
            "keyboard_input" => {
                let text = parse_string(&arguments["text"], "text")?;

                let cmd = Command::KeyboardInput { text };
                let response = handler.handle_command(cmd);
                format_response(response)
            },
            "keyboard_command" => {
                let command = parse_string(&arguments["command"], "command")?;

                let cmd = Command::KeyboardCommand { command };
                let response = handler.handle_command(cmd);
                format_response(response)
            },
            "get_mouse_position" => {
                let cmd = Command::GetMousePosition;
                let response = handler.handle_command(cmd);
                format_response(response)
            },
            "take_screenshot" => {
                let cmd = Command::Screenshot;
                let response = handler.handle_command(cmd);
                format_response(response)
            },
            _ => Err(anyhow::anyhow!("Unknown automation tool: {}", tool_name)),
        }
    }
}
