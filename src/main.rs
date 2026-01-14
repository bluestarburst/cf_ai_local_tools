use anyhow::{Context, Result};
use rustautogui::{MouseClick, RustAutoGui};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tracing::{info, error, warn};

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

    // Send initial handshake
    let handshake = serde_json::json!({
        "type": "handshake",
        "client": "rust-automation",
        "version": env!("CARGO_PKG_VERSION")
    });
    
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
