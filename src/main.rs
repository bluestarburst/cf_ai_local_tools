use anyhow::{Context, Result};
use rustautogui::{MouseClick, RustAutoGui};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tracing::{info, error, warn};

/// Tool parameter definition
#[derive(Debug, Serialize, Clone)]
struct ToolParameter {
    name: String,
    #[serde(rename = "type")]
    param_type: String,
    description: String,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enum")]
    enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<serde_json::Value>,
}

/// Tool definition sent to worker
#[derive(Debug, Serialize, Clone)]
struct ToolDefinition {
    id: String,
    name: String,
    description: String,
    category: String,
    parameters: Vec<ToolParameter>,
    #[serde(rename = "returnsObservation")]
    returns_observation: bool,
}

/// Get all available tools this client can execute
fn get_available_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            id: "mouse_move".into(),
            name: "Mouse Move".into(),
            description: "Move the mouse cursor to specified coordinates".into(),
            category: "mouse".into(),
            parameters: vec![
                ToolParameter {
                    name: "x".into(),
                    param_type: "number".into(),
                    description: "X coordinate to move to".into(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "y".into(),
                    param_type: "number".into(),
                    description: "Y coordinate to move to".into(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "duration".into(),
                    param_type: "number".into(),
                    description: "Duration of movement in seconds".into(),
                    required: false,
                    enum_values: None,
                    default: Some(json!(1.0)),
                },
            ],
            returns_observation: true,
        },
        ToolDefinition {
            id: "mouse_click".into(),
            name: "Mouse Click".into(),
            description: "Click a mouse button at current position".into(),
            category: "mouse".into(),
            parameters: vec![
                ToolParameter {
                    name: "button".into(),
                    param_type: "string".into(),
                    description: "Which button to click".into(),
                    required: true,
                    enum_values: Some(vec!["left".into(), "right".into(), "middle".into()]),
                    default: None,
                },
            ],
            returns_observation: true,
        },
        ToolDefinition {
            id: "mouse_scroll".into(),
            name: "Mouse Scroll".into(),
            description: "Scroll the mouse wheel in a direction".into(),
            category: "mouse".into(),
            parameters: vec![
                ToolParameter {
                    name: "direction".into(),
                    param_type: "string".into(),
                    description: "Direction to scroll".into(),
                    required: true,
                    enum_values: Some(vec!["up".into(), "down".into(), "left".into(), "right".into()]),
                    default: None,
                },
                ToolParameter {
                    name: "intensity".into(),
                    param_type: "number".into(),
                    description: "How much to scroll (1-10)".into(),
                    required: false,
                    enum_values: None,
                    default: Some(json!(3)),
                },
            ],
            returns_observation: true,
        },
        ToolDefinition {
            id: "keyboard_input".into(),
            name: "Keyboard Input".into(),
            description: "Type text using the keyboard".into(),
            category: "keyboard".into(),
            parameters: vec![
                ToolParameter {
                    name: "text".into(),
                    param_type: "string".into(),
                    description: "Text to type".into(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
            ],
            returns_observation: true,
        },
        ToolDefinition {
            id: "keyboard_command".into(),
            name: "Keyboard Command".into(),
            description: "Execute a keyboard shortcut (e.g., 'ctrl+c', 'cmd+v')".into(),
            category: "keyboard".into(),
            parameters: vec![
                ToolParameter {
                    name: "command".into(),
                    param_type: "string".into(),
                    description: "Keyboard shortcut to execute".into(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
            ],
            returns_observation: true,
        },
        ToolDefinition {
            id: "get_mouse_position".into(),
            name: "Get Mouse Position".into(),
            description: "Get the current mouse cursor position".into(),
            category: "system".into(),
            parameters: vec![],
            returns_observation: true,
        },
        ToolDefinition {
            id: "take_screenshot".into(),
            name: "Take Screenshot".into(),
            description: "Capture a screenshot of the screen".into(),
            category: "system".into(),
            parameters: vec![
                ToolParameter {
                    name: "region".into(),
                    param_type: "string".into(),
                    description: "Region to capture (full, window, custom)".into(),
                    required: false,
                    enum_values: Some(vec!["full".into(), "window".into(), "custom".into()]),
                    default: Some(json!("full")),
                },
            ],
            returns_observation: true,
        },
        ToolDefinition {
            id: "web_search".into(),
            name: "Web Search".into(),
            description: "Search the web using SearXNG. Returns search results with URLs and summaries.".into(),
            category: "search".into(),
            parameters: vec![
                ToolParameter {
                    name: "query".into(),
                    param_type: "string".into(),
                    description: "The search query".into(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "time_range".into(),
                    param_type: "string".into(),
                    description: "Time range filter".into(),
                    required: false,
                    enum_values: Some(vec!["day".into(), "week".into(), "month".into(), "year".into()]),
                    default: None,
                },
                ToolParameter {
                    name: "language".into(),
                    param_type: "string".into(),
                    description: "Language code (e.g., en, es, fr)".into(),
                    required: false,
                    enum_values: None,
                    default: None,
                },
            ],
            returns_observation: true,
        },
        ToolDefinition {
            id: "fetch_url".into(),
            name: "Fetch URL".into(),
            description: "Fetch and parse content from a URL".into(),
            category: "utility".into(),
            parameters: vec![
                ToolParameter {
                    name: "url".into(),
                    param_type: "string".into(),
                    description: "URL to fetch".into(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "extract_type".into(),
                    param_type: "string".into(),
                    description: "Type of content to extract".into(),
                    required: false,
                    enum_values: Some(vec!["text".into(), "links".into(), "images".into(), "all".into()]),
                    default: Some(json!("text")),
                },
            ],
            returns_observation: true,
        },
    ]
}

/// Command received from Cloudflare Worker
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Command {
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

/// Response sent back to Worker
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Response {
    Success { message: String },
    Error { error: String },
    MousePosition { x: i32, y: i32 },
    Screenshot { data: String }, // base64 encoded
}

struct AutomationHandler {
    gui: RustAutoGui,
}

impl AutomationHandler {
    fn new() -> Result<Self> {
        let gui = RustAutoGui::new(false)
            .context("Failed to initialize RustAutoGui")?;
        Ok(Self { gui })
    }

    fn handle_command(&self, cmd: Command) -> Response {
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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // WebSocket URL - update this with your actual Cloudflare Worker URL
    let ws_url = std::env::var("WORKER_WS_URL")
        .unwrap_or_else(|_| "ws://localhost:8787/connect".to_string());

    info!("Starting automation client...");
    info!("Will connect to: {}", ws_url);

    let handler = AutomationHandler::new()?;

    // Connection retry loop
    loop {
        match connect_and_run(&ws_url, &handler).await {
            Ok(_) => {
                warn!("Connection closed normally");
            }
            Err(e) => {
                error!("Connection error: {}", e);
            }
        }

        info!("Reconnecting in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}

async fn connect_and_run(url: &str, handler: &AutomationHandler) -> Result<()> {
    info!("Connecting to WebSocket...");
    
    let (ws_stream, _) = connect_async(url)
        .await
        .context("Failed to connect to WebSocket")?;

    info!("Connected successfully!");

    let (mut write, mut read) = ws_stream.split();

    // Send initial handshake with available tools
    let tools = get_available_tools();
    let handshake = serde_json::json!({
        "type": "handshake",
        "client": "rust-automation",
        "version": env!("CARGO_PKG_VERSION"),
        "tools": tools
    });

    info!("Registering {} tools with server", tools.len());
    write
        .send(Message::Text(handshake.to_string()))
        .await
        .context("Failed to send handshake")?;

    // Process incoming messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received command: {}", text);
                
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(mut value) => {
                        // Handle protocol messages (don't try to parse as commands)
                        if let Some(msg_type) = value.get("type").and_then(|v| v.as_str()) {
                            match msg_type {
                                "handshake_ack" => {
                                    info!("Server handshake acknowledged");
                                    continue;
                                }
                                "pong" => {
                                    // Respond to pings with pongs
                                    continue;
                                }
                                _ => {} // Continue to parse as command
                            }
                        }
                        
                        // Extract commandId if present
                        let command_id = value.get("commandId")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        // Remove commandId before parsing as Command
                        if value.is_object() {
                            value.as_object_mut().unwrap().remove("commandId");
                        }
                        
                        match serde_json::from_value::<Command>(value) {
                            Ok(cmd) => {
                                let mut response = handler.handle_command(cmd);
                                
                                // Add commandId back to response
                                let mut response_value = serde_json::to_value(&response)?;
                                if let Some(id) = command_id {
                                    if let Some(obj) = response_value.as_object_mut() {
                                        obj.insert("commandId".to_string(), serde_json::Value::String(id));
                                    }
                                }
                                
                                let response_json = serde_json::to_string(&response_value)?;
                                
                                write
                                    .send(Message::Text(response_json))
                                    .await
                                    .context("Failed to send response")?;
                            }
                            Err(e) => {
                                error!("Failed to parse command after removing commandId: {}", e);
                                let error_response = Response::Error {
                                    error: format!("Invalid command format: {}", e),
                                };
                                let mut response_json = serde_json::to_value(&error_response)?;
                                if let Some(id) = command_id {
                                    if let Some(obj) = response_json.as_object_mut() {
                                        obj.insert("commandId".to_string(), serde_json::Value::String(id));
                                    }
                                }
                                write.send(Message::Text(serde_json::to_string(&response_json)?)).await?;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse command: {}", e);
                        let error_response = Response::Error {
                            error: format!("Invalid command format: {}", e),
                        };
                        let response_json = serde_json::to_string(&error_response)?;
                        write.send(Message::Text(response_json)).await?;
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                write.send(Message::Pong(data)).await?;
            }
            Ok(Message::Close(_)) => {
                info!("Server closed connection");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
