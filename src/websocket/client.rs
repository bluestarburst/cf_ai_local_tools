use crate::agents::conversation::{ConversationManager, ProgressType};
use crate::core::{Agent, AgentContext, ExecutionStep, ToolContext};
use crate::registry::{CentralRegistry, Registry as RegistryTrait};
use crate::websocket::protocol::{
    AgentConfig, IncomingMessage, OutgoingMessage, PresetAgent, PresetMetadata, ToolDefinition,
    ToolReference,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

/// Client that connects to the Cloudflare Worker Relay
pub struct WebSocketRelayClient {
    url: String,
    registry: Arc<CentralRegistry>,
    llm: Arc<dyn crate::core::LLMClient>,
}

impl WebSocketRelayClient {
    pub fn new(
        url: String,
        registry: Arc<CentralRegistry>,
        llm: Arc<dyn crate::core::LLMClient>,
    ) -> Self {
        Self { url, registry, llm }
    }

    /// Connect and run the main event loop
    pub async fn run(&self) -> crate::core::Result<()> {
        println!("Connecting to relay at {}...", self.url);

        let (ws_stream, _) = connect_async(&self.url).await.map_err(|e| {
            crate::core::AppError::Network(format!("WebSocket connection failed: {}", e))
        })?;

        println!("Connected to WebSocket relay.");

        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<OutgoingMessage>();

        // Spawn writer task
        let write_handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match serde_json::to_string(&msg) {
                    Ok(text) => {
                        if let Err(e) = write.send(Message::Text(text)).await {
                            eprintln!("Failed to send message: {}", e);
                            break;
                        }
                    }
                    Err(e) => eprintln!("Failed to serialize message: {}", e),
                }
            }
        });

        // Main read loop
        let registry = self.registry.clone();
        let llm = self.llm.clone();
        let tx_clone = tx.clone(); // Keep for cloning into handlers

        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    println!("DEBUG: Received RAW WebSocket message: {}", text);
                    match serde_json::from_str::<IncomingMessage>(&text) {
                        Ok(msg) => {
                            let tx = tx_clone.clone();
                            let registry = registry.clone();
                            let llm = llm.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_message(msg, tx, registry, llm).await {
                                    eprintln!("Error handling message: {}", e);
                                }
                            });
                        }
                        Err(e) => eprintln!(
                            "Failed to parse incoming message: {}\nRaw text: {}",
                            e, text
                        ),
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("WebSocket connection closed.");
                    break;
                }
                Err(e) => {
                    eprintln!("WebSocket stream error: {}", e);
                    break;
                }
                _ => {} // Ignore
            }
        }

        // Ensure writer closes
        drop(tx_clone);
        let _ = write_handle.await;

        Ok(())
    }

    async fn handle_message(
        msg: IncomingMessage,
        tx: mpsc::UnboundedSender<OutgoingMessage>,
        registry: Arc<CentralRegistry>,
        llm: Arc<dyn crate::core::LLMClient>,
    ) -> crate::core::Result<()> {
        match msg {
            IncomingMessage::ChatRequest {
                message,
                agent: agent_config,
            } => {
                // Not using agent_config fully yet, ensuring we get the conversational agent
                let agent = registry
                    .agents
                    .get("conversational-agent")
                    .await
                    .map_err(|e| crate::core::AppError::Registry(e.to_string()))?
                    .ok_or(crate::core::AppError::Registry(
                        "Default agent not found".to_string(),
                    ))?;

                // Create manager for streaming updates
                let manager: Arc<dyn ConversationManager> =
                    Arc::new(WebSocketConversationManager { tx: tx.clone() });

                let mut context = AgentContext::new("conversational-agent".to_string());
                context.messages.push(crate::core::ConversationMessage {
                    role: "user".to_string(),
                    content: message.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });

                let mut tools = Vec::new();
                for tool_name in agent_config.tools {
                    if let Some(tool) = registry.tools.get(&tool_name).await? {
                        tools.push(tool);
                    }
                }

                let result = agent
                    .execute(&message, &context, llm.as_ref(), Some(manager), &tools)
                    .await?;

                // Steps are already sent incrementally by the agent via send_thinking_update
                // Send final response only
                let _ = tx.send(OutgoingMessage::ChatResponse {
                    content: result.response,
                });
            }
            IncomingMessage::GetPresets | IncomingMessage::ResetPresets => {
                // Collect Tools
                let mut tools_def = Vec::new();
                // list() returns Vec<Box<dyn Tool>>, so iterate directly
                for tool in registry.tools.list().await.unwrap_or_default() {
                    tools_def.push(crate::websocket::protocol::ToolDefinition {
                        id: tool.id().to_string(),
                        name: tool.name().to_string(),
                        description: tool.description().to_string(),
                        category: "utility".to_string(), // TODO: add category to Tool trait
                        parameters: tool.parameters().to_vec(),
                        returns_observation: true,
                    });
                }

                // Get default agent presets from presets module
                let preset_agents = crate::registry::presets::get_default_presets();

                // Convert to protocol types
                let agents_def: Vec<crate::websocket::protocol::PresetAgent> = preset_agents
                    .into_iter()
                    .map(|preset| crate::websocket::protocol::PresetAgent {
                        id: preset.id,
                        name: preset.name,
                        purpose: preset.purpose,
                        system_prompt: preset.system_prompt,
                        tools: preset
                            .tools
                            .into_iter()
                            .map(|tr| crate::websocket::protocol::ToolReference {
                                tool_id: tr.tool_id,
                                enabled: tr.enabled,
                            })
                            .collect(),
                        model_id: preset.model_id,
                        max_iterations: preset.max_iterations,
                        metadata: crate::websocket::protocol::PresetMetadata {
                            created_at: preset.metadata.created_at,
                            updated_at: preset.metadata.updated_at,
                            version: preset.metadata.version,
                            author: preset.metadata.author,
                        },
                    })
                    .collect();

                let _ = tx.send(OutgoingMessage::PresetsList {
                    agents: agents_def,
                    prompts: vec![],
                    tools: tools_def,
                });
            }
            IncomingMessage::GetPrompts => {
                // TODO: Implement prompts if needed
                let _ = tx.send(OutgoingMessage::PresetsList {
                    agents: vec![],
                    prompts: vec![],
                    tools: vec![],
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct WebSocketConversationManager {
    tx: mpsc::UnboundedSender<OutgoingMessage>,
}

#[async_trait::async_trait]
impl ConversationManager for WebSocketConversationManager {
    async fn send_thinking_update(
        &self,
        _agent_id: &str,
        step_number: usize,
        thought: &str,
    ) -> crate::core::Result<()> {
        let step = ExecutionStep {
            step_number,
            step_type: crate::core::StepType::Thinking,
            content: thought.to_string(),
            tool_call: None,
            tool_observation: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        let _ = self.tx.send(OutgoingMessage::ExecutionStep { step });
        Ok(())
    }

    async fn send_progress_update(
        &self,
        _agent_id: &str,
        progress_type: ProgressType,
        message: &str,
        _progress: Option<f32>,
    ) -> crate::core::Result<()> {
        let step_type = match progress_type {
            ProgressType::Thinking => crate::core::StepType::Thinking,
            ProgressType::Planning => crate::core::StepType::Planning,
            ProgressType::Executing => crate::core::StepType::Action,
            ProgressType::Observing => crate::core::StepType::Observation,
            ProgressType::Reflecting => crate::core::StepType::Reflection,
            ProgressType::Completing => crate::core::StepType::Completion,
        };

        let step = ExecutionStep {
            step_number: 0,
            step_type,
            content: message.to_string(),
            tool_call: None,
            tool_observation: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let _ = self.tx.send(OutgoingMessage::ExecutionStep { step });
        Ok(())
    }

    async fn send_error_update(
        &self,
        _agent_id: &str,
        error: &str,
        _recovery_suggestions: Vec<String>,
    ) -> crate::core::Result<()> {
        let _ = self.tx.send(OutgoingMessage::Error {
            error: error.to_string(),
        });
        Ok(())
    }

    async fn send_completion_update(
        &self,
        _agent_id: &str,
        final_response: &str,
        _success: bool,
    ) -> crate::core::Result<()> {
        let _ = self.tx.send(OutgoingMessage::ChatResponse {
            content: final_response.to_string(),
        });
        Ok(())
    }
}
