use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

mod agents;
mod llm;
mod tools;

use agents::{
    execute as execute_react_loop, get_all_default_agents, get_all_default_prompts, Agent,
    AgentConfig, AgentStorage, ExecutionStep, Prompt, PromptStorage, StepSender, ToolDefinition,
};
use llm::LLMClient;
use tools::{execute_tool_async, is_delegation_request, AutomationHandler};

// Re-export Command and Response for backward compatibility with WebSocket protocol
// Note: Direct Command/Response handling is deprecated in favor of using tools module
use tools::computer_automation::{Command, Response};

/// Get all available tools from the tools module
fn get_available_tools() -> Vec<ToolDefinition> {
    tools::get_all_tools()
}

/// Context for tool execution with delegation support
struct ToolExecutionContext<'a> {
    handler: &'a AutomationHandler,
    llm: &'a LLMClient,
    agent_storage: &'a AgentStorage,
    available_tools: &'a [ToolDefinition],
    max_delegation_depth: usize,
    step_sender: Option<StepSender>,
}

/// Create a tool executor that supports delegation
///
/// This executor will:
/// 1. Execute regular tools normally
/// 2. Detect delegation requests and recursively execute the delegated agent
fn create_delegating_tool_executor<'a>(
    ctx: &'a ToolExecutionContext<'a>,
    current_depth: usize,
) -> impl Fn(
    &str,
    &serde_json::Value,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + 'a>>
       + 'a {
    move |tool_name: &str, arguments: &serde_json::Value| {
        let tool_name = tool_name.to_string();
        let arguments = arguments.clone();

        Box::pin(async move {
            // Execute the tool (async version)
            let result = execute_tool_async(&tool_name, &arguments, Some(ctx.handler)).await?;

            // Check if this is a delegation request
            if let Some(delegation) = is_delegation_request(&result) {
                info!(
                    "Delegation detected: agent='{}', task='{}'",
                    delegation.agent_id, delegation.task
                );

                // Check delegation depth to prevent infinite recursion
                if current_depth >= ctx.max_delegation_depth {
                    return Err(anyhow::anyhow!(
                        "Maximum delegation depth ({}) reached. Cannot delegate to agent '{}'",
                        ctx.max_delegation_depth,
                        delegation.agent_id
                    ));
                }

                // Look up the delegated agent
                let agent = ctx.agent_storage.get(&delegation.agent_id).ok_or_else(|| {
                    anyhow::anyhow!("Delegated agent '{}' not found", delegation.agent_id)
                })?;

                // Convert Agent to AgentConfig
                let agent_config = AgentConfig {
                    model_id: agent.model_id.clone(),
                    system_prompt: agent.system_prompt.clone(),
                    tools: agent.tools.clone(),
                    max_iterations: agent.max_iterations,
                    separate_reasoning_model: false,
                    reasoning_model_id: None,
                };

                info!(
                    "Executing delegated agent: {} (depth: {})",
                    delegation.agent_id,
                    current_depth + 1
                );

                // Create a new tool executor for the delegated agent with increased depth
                let delegated_executor = create_delegating_tool_executor(ctx, current_depth + 1);

                // Execute the delegated agent asynchronously
                // Pass the step_sender so delegated agent steps are also streamed
                let result = execute_react_loop(
                    &agent_config,
                    &delegation.task,
                    ctx.llm,
                    ctx.available_tools,
                    None::<fn(ExecutionStep) -> Result<()>>,
                    delegated_executor,
                    ctx.step_sender.clone(), // Pass step sender to delegated agent
                    Some(delegation.agent_id.clone()), // Tag steps with delegated agent ID
                )
                .await?;

                info!("Delegation completed: {}", result);
                Ok(format!(
                    "Delegated to agent '{}'. Result:\n{}",
                    delegation.agent_id, result
                ))
            } else {
                // Not a delegation - return normal result
                Ok(result)
            }
        })
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

    // Initialize agent and prompt storage
    let mut agent_storage = AgentStorage::new()?;
    info!(
        "Agent storage initialized with {} agents",
        agent_storage.get_all().len()
    );

    let mut prompt_storage = PromptStorage::new()?;
    info!(
        "Prompt storage initialized with {} prompts",
        prompt_storage.get_all().len()
    );

    // Connection retry loop
    loop {
        match connect_and_run(&ws_url, &handler, &mut agent_storage, &mut prompt_storage).await {
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

async fn connect_and_run(
    url: &str,
    handler: &AutomationHandler,
    agent_storage: &mut AgentStorage,
    prompt_storage: &mut PromptStorage,
) -> Result<()> {
    info!("Connecting to WebSocket...");

    // Add device=desktop query parameter
    let ws_url = if url.contains('?') {
        format!("{}&device=desktop", url)
    } else {
        format!("{}?device=desktop", url)
    };

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .context("Failed to connect to WebSocket")?;

    info!("Connected successfully!");

    let (write, mut read) = ws_stream.split();
    // Wrap write in Arc<Mutex> to share between main loop and step streaming task
    let write = Arc::new(tokio::sync::Mutex::new(write));

    // Send initial handshake with available tools and agents
    let tools = get_available_tools();
    let agents = agent_storage.get_all();
    let handshake = serde_json::json!({
        "type": "handshake",
        "client": "rust-automation",
        "version": env!("CARGO_PKG_VERSION"),
        "tools": tools,
        "agents": agents
    });

    info!(
        "Registering {} tools and {} agents with server",
        tools.len(),
        agents.len()
    );
    write
        .lock()
        .await
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
                                "get_agents" => {
                                    info!("Received get_agents request");
                                    let agents = agent_storage.get_all();
                                    let response = json!({
                                        "type": "agents_list",
                                        "agents": agents
                                    });
                                    write
                                        .lock()
                                        .await
                                        .send(Message::Text(response.to_string()))
                                        .await?;
                                    continue;
                                }
                                "create_agent" => {
                                    info!("Received create_agent request");
                                    match value.get("agent").and_then(|v| {
                                        serde_json::from_value::<Agent>(v.clone()).ok()
                                    }) {
                                        Some(agent) => {
                                            // Validate tools exist
                                            let available_tool_ids: Vec<String> =
                                                get_available_tools()
                                                    .iter()
                                                    .map(|t| t.id.clone())
                                                    .collect();

                                            match agent_storage
                                                .validate_tools(&agent, &available_tool_ids)
                                            {
                                                Ok(_) => match agent_storage.create(agent) {
                                                    Ok(created_agent) => {
                                                        let response = json!({
                                                            "type": "agent_created",
                                                            "agent": created_agent
                                                        });
                                                        write
                                                            .lock()
                                                            .await
                                                            .send(Message::Text(
                                                                response.to_string(),
                                                            ))
                                                            .await?;
                                                    }
                                                    Err(e) => {
                                                        let response = json!({
                                                            "type": "agent_error",
                                                            "error": e.to_string()
                                                        });
                                                        write
                                                            .lock()
                                                            .await
                                                            .send(Message::Text(
                                                                response.to_string(),
                                                            ))
                                                            .await?;
                                                    }
                                                },
                                                Err(e) => {
                                                    let response = json!({
                                                        "type": "agent_error",
                                                        "error": e.to_string()
                                                    });
                                                    write
                                                        .lock()
                                                        .await
                                                        .send(Message::Text(response.to_string()))
                                                        .await?;
                                                }
                                            }
                                        }
                                        None => {
                                            let response = json!({
                                                "type": "agent_error",
                                                "error": "Invalid agent data"
                                            });
                                            write
                                                .lock()
                                                .await
                                                .send(Message::Text(response.to_string()))
                                                .await?;
                                        }
                                    }
                                    continue;
                                }
                                "update_agent" => {
                                    info!("Received update_agent request");
                                    let agent_id = value.get("id").and_then(|v| v.as_str());
                                    let agent_data = value.get("agent").and_then(|v| {
                                        serde_json::from_value::<Agent>(v.clone()).ok()
                                    });

                                    if let (Some(id), Some(agent)) = (agent_id, agent_data) {
                                        // Validate tools exist
                                        let available_tool_ids: Vec<String> = get_available_tools()
                                            .iter()
                                            .map(|t| t.id.clone())
                                            .collect();

                                        match agent_storage
                                            .validate_tools(&agent, &available_tool_ids)
                                        {
                                            Ok(_) => match agent_storage.update(id, agent) {
                                                Ok(updated_agent) => {
                                                    let response = json!({
                                                        "type": "agent_updated",
                                                        "agent": updated_agent
                                                    });
                                                    write
                                                        .lock()
                                                        .await
                                                        .send(Message::Text(response.to_string()))
                                                        .await?;
                                                }
                                                Err(e) => {
                                                    let response = json!({
                                                        "type": "agent_error",
                                                        "error": e.to_string()
                                                    });
                                                    write
                                                        .lock()
                                                        .await
                                                        .send(Message::Text(response.to_string()))
                                                        .await?;
                                                }
                                            },
                                            Err(e) => {
                                                let response = json!({
                                                    "type": "agent_error",
                                                    "error": e.to_string()
                                                });
                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(response.to_string()))
                                                    .await?;
                                            }
                                        }
                                    } else {
                                        let response = json!({
                                            "type": "agent_error",
                                            "error": "Invalid agent id or data"
                                        });
                                        write
                                            .lock()
                                            .await
                                            .send(Message::Text(response.to_string()))
                                            .await?;
                                    }
                                    continue;
                                }
                                "delete_agent" => {
                                    info!("Received delete_agent request");
                                    if let Some(agent_id) = value.get("id").and_then(|v| v.as_str())
                                    {
                                        match agent_storage.delete(agent_id) {
                                            Ok(_) => {
                                                let response = json!({
                                                    "type": "agent_deleted",
                                                    "id": agent_id
                                                });
                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(response.to_string()))
                                                    .await?;
                                            }
                                            Err(e) => {
                                                let response = json!({
                                                    "type": "agent_error",
                                                    "error": e.to_string()
                                                });
                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(response.to_string()))
                                                    .await?;
                                            }
                                        }
                                    } else {
                                        let response = json!({
                                            "type": "agent_error",
                                            "error": "Missing agent id"
                                        });
                                        write
                                            .lock()
                                            .await
                                            .send(Message::Text(response.to_string()))
                                            .await?;
                                    }
                                    continue;
                                }
                                "get_agent" => {
                                    info!("Received get_agent request");
                                    if let Some(agent_id) = value.get("id").and_then(|v| v.as_str())
                                    {
                                        match agent_storage.get(agent_id) {
                                            Some(agent) => {
                                                let response = json!({
                                                    "type": "agent_data",
                                                    "agent": agent
                                                });
                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(response.to_string()))
                                                    .await?;
                                            }
                                            None => {
                                                let response = json!({
                                                    "type": "agent_error",
                                                    "error": format!("Agent '{}' not found", agent_id)
                                                });
                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(response.to_string()))
                                                    .await?;
                                            }
                                        }
                                    } else {
                                        let response = json!({
                                            "type": "agent_error",
                                            "error": "Missing agent id"
                                        });
                                        write
                                            .lock()
                                            .await
                                            .send(Message::Text(response.to_string()))
                                            .await?;
                                    }
                                    continue;
                                }
                                "chat_request" => {
                                    // Handle chat request - run ReAct loop
                                    info!("Received chat request");

                                    let user_message =
                                        value.get("message").and_then(|v| v.as_str()).unwrap_or("");

                                    let agent = value.get("agent").and_then(|v| {
                                        serde_json::from_value::<AgentConfig>(v.clone()).ok()
                                    });

                                    if let Some(agent_config) = agent {
                                        // Get worker URL from environment
                                        let worker_url = std::env::var("WORKER_HTTP_URL")
                                            .unwrap_or_else(|_| {
                                                "http://localhost:8787".to_string()
                                            });

                                        info!(
                                            "Starting ReAct loop with model: {}",
                                            agent_config.model_id
                                        );

                                        // Create LLM client
                                        let llm = LLMClient::new(&worker_url);

                                        // Get available tools
                                        let available_tools = get_available_tools();

                                        // Validate that all requested tools exist
                                        let available_tool_ids: Vec<String> =
                                            available_tools.iter().map(|t| t.id.clone()).collect();
                                        let invalid_tools: Vec<String> = agent_config
                                            .tools
                                            .iter()
                                            .filter(|tool_id| !available_tool_ids.contains(tool_id))
                                            .cloned()
                                            .collect();

                                        if !invalid_tools.is_empty() {
                                            error!(
                                                "Agent references unknown tools: {:?}",
                                                invalid_tools
                                            );
                                            let error_response = json!({
                                                "type": "chat_response",
                                                "content": format!(
                                                    "Error: Agent '{}' references unknown tools: {}. Available tools are: {}",
                                                    agent_config.model_id,
                                                    invalid_tools.join(", "),
                                                    available_tool_ids.join(", ")
                                                ),
                                                "error": true
                                            });
                                            write
                                                .lock()
                                                .await
                                                .send(Message::Text(error_response.to_string()))
                                                .await?;
                                            continue;
                                        }

                                        // Create channel for real-time step streaming
                                        let (step_tx, mut step_rx) =
                                            mpsc::unbounded_channel::<ExecutionStep>();

                                        // Create delegation-aware tool executor with step sender
                                        let exec_ctx = ToolExecutionContext {
                                            handler: &handler,
                                            llm: &llm,
                                            agent_storage: &agent_storage,
                                            available_tools: available_tools.as_slice(),
                                            max_delegation_depth: 3, // Allow up to 3 levels of delegation
                                            step_sender: Some(step_tx.clone()),
                                        };
                                        let tool_executor =
                                            create_delegating_tool_executor(&exec_ctx, 0);

                                        // Get agent ID for tagging steps
                                        let agent_id = value
                                            .get("agentId")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string());

                                        // Spawn task to stream steps to WebSocket in real-time
                                        let write_clone = write.clone();
                                        let step_streamer = tokio::spawn(async move {
                                            while let Some(step) = step_rx.recv().await {
                                                let step_message = json!({
                                                    "type": "execution_step",
                                                    "step": step
                                                });
                                                if let Err(e) = write_clone
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(step_message.to_string()))
                                                    .await
                                                {
                                                    error!("Failed to stream step: {}", e);
                                                }
                                            }
                                        });

                                        // Execute ReAct loop with channel-based step streaming
                                        let result = execute_react_loop(
                                            &agent_config,
                                            user_message,
                                            &llm,
                                            available_tools.as_slice(),
                                            None::<fn(ExecutionStep) -> Result<()>>,
                                            tool_executor,
                                            Some(step_tx),
                                            agent_id,
                                        )
                                        .await;

                                        // Wait for step streamer to finish
                                        let _ = step_streamer.await;

                                        match result {
                                            Ok(response) => {
                                                let chat_response = json!({
                                                    "type": "chat_response",
                                                    "content": response,
                                                });

                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(chat_response.to_string()))
                                                    .await?;
                                                info!("Chat response sent");
                                            }
                                            Err(e) => {
                                                error!("ReAct loop error: {}", e);
                                                let error_response = json!({
                                                    "type": "chat_response",
                                                    "content": format!("Error: {}", e),
                                                    "error": true
                                                });
                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(error_response.to_string()))
                                                    .await?;
                                            }
                                        }
                                    } else {
                                        error!("Invalid agent config in chat request");
                                        let error_response = json!({
                                            "type": "chat_response",
                                            "content": "Error: Invalid agent configuration",
                                            "error": true
                                        });
                                        write
                                            .lock()
                                            .await
                                            .send(Message::Text(error_response.to_string()))
                                            .await?;
                                    }

                                    continue;
                                }
                                "get_presets" => {
                                    info!("Received get_presets request");
                                    let tools = get_available_tools();
                                    let agents = get_all_default_agents();
                                    let prompts = get_all_default_prompts();

                                    let response = json!({
                                        "type": "presets",
                                        "tools": tools,
                                        "agents": agents,
                                        "prompts": prompts
                                    });
                                    write
                                        .lock()
                                        .await
                                        .send(Message::Text(response.to_string()))
                                        .await?;
                                    continue;
                                }
                                "get_tools" => {
                                    info!("Received get_tools request");
                                    let tools = get_available_tools();
                                    let response = json!({
                                        "type": "tools",
                                        "tools": tools
                                    });
                                    write
                                        .lock()
                                        .await
                                        .send(Message::Text(response.to_string()))
                                        .await?;
                                    continue;
                                }
                                "reset_agents" => {
                                    info!("Received reset_agents request");
                                    let default_agents = get_all_default_agents();

                                    // Clear existing agents and restore defaults
                                    agent_storage.clear()?;
                                    for agent in default_agents.iter() {
                                        // Convert PresetAgent to Agent (storage format)
                                        let storage_agent = Agent {
                                            id: agent.id.clone(),
                                            name: agent.name.clone(),
                                            purpose: agent.purpose.clone(),
                                            system_prompt: agent.system_prompt.clone(),
                                            tools: agent
                                                .tools
                                                .iter()
                                                .map(|t| t.tool_id.clone())
                                                .collect(),
                                            model_id: agent.model_id.clone(),
                                            max_iterations: agent.max_iterations,
                                            is_locked: true,
                                            separate_reasoning_model: agent
                                                .separate_reasoning_model,
                                            reasoning_model_id: agent.reasoning_model_id.clone(),
                                            created_at: agent.metadata.created_at.clone(),
                                            updated_at: agent.metadata.updated_at.clone(),
                                        };
                                        agent_storage.create(storage_agent)?;
                                    }

                                    let response = json!({
                                        "type": "agents_reset",
                                        "agents": default_agents
                                    });
                                    write
                                        .lock()
                                        .await
                                        .send(Message::Text(response.to_string()))
                                        .await?;
                                    continue;
                                }
                                "get_prompts" => {
                                    info!("Received get_prompts request");
                                    let prompts = prompt_storage.get_all();
                                    let response = json!({
                                        "type": "prompts",
                                        "prompts": prompts
                                    });
                                    write
                                        .lock()
                                        .await
                                        .send(Message::Text(response.to_string()))
                                        .await?;
                                    continue;
                                }
                                "create_prompt" => {
                                    info!("Received create_prompt request");
                                    if let Ok(prompt) =
                                        serde_json::from_value::<Prompt>(value.clone())
                                    {
                                        match prompt_storage.create(prompt) {
                                            Ok(created) => {
                                                let response = json!({
                                                    "type": "prompt_created",
                                                    "prompt": created
                                                });
                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(response.to_string()))
                                                    .await?;
                                            }
                                            Err(e) => {
                                                let response = json!({
                                                    "type": "prompt_error",
                                                    "error": e.to_string()
                                                });
                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(response.to_string()))
                                                    .await?;
                                            }
                                        }
                                    } else {
                                        let response = json!({
                                            "type": "prompt_error",
                                            "error": "Invalid prompt data"
                                        });
                                        write
                                            .lock()
                                            .await
                                            .send(Message::Text(response.to_string()))
                                            .await?;
                                    }
                                    continue;
                                }
                                "update_prompt" => {
                                    info!("Received update_prompt request");
                                    if let Some(prompt_id) =
                                        value.get("id").and_then(|v| v.as_str())
                                    {
                                        if let Ok(prompt) =
                                            serde_json::from_value::<Prompt>(value.clone())
                                        {
                                            match prompt_storage.update(prompt_id, prompt) {
                                                Ok(updated) => {
                                                    let response = json!({
                                                        "type": "prompt_updated",
                                                        "prompt": updated
                                                    });
                                                    write
                                                        .lock()
                                                        .await
                                                        .send(Message::Text(response.to_string()))
                                                        .await?;
                                                }
                                                Err(e) => {
                                                    let response = json!({
                                                        "type": "prompt_error",
                                                        "error": e.to_string()
                                                    });
                                                    write
                                                        .lock()
                                                        .await
                                                        .send(Message::Text(response.to_string()))
                                                        .await?;
                                                }
                                            }
                                        } else {
                                            let response = json!({
                                                "type": "prompt_error",
                                                "error": "Invalid prompt data"
                                            });
                                            write
                                                .lock()
                                                .await
                                                .send(Message::Text(response.to_string()))
                                                .await?;
                                        }
                                    } else {
                                        let response = json!({
                                            "type": "prompt_error",
                                            "error": "Missing prompt id"
                                        });
                                        write
                                            .lock()
                                            .await
                                            .send(Message::Text(response.to_string()))
                                            .await?;
                                    }
                                    continue;
                                }
                                "delete_prompt" => {
                                    info!("Received delete_prompt request");
                                    if let Some(prompt_id) =
                                        value.get("id").and_then(|v| v.as_str())
                                    {
                                        match prompt_storage.delete(prompt_id) {
                                            Ok(_) => {
                                                let response = json!({
                                                    "type": "prompt_deleted",
                                                    "id": prompt_id
                                                });
                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(response.to_string()))
                                                    .await?;
                                            }
                                            Err(e) => {
                                                let response = json!({
                                                    "type": "prompt_error",
                                                    "error": e.to_string()
                                                });
                                                write
                                                    .lock()
                                                    .await
                                                    .send(Message::Text(response.to_string()))
                                                    .await?;
                                            }
                                        }
                                    } else {
                                        let response = json!({
                                            "type": "prompt_error",
                                            "error": "Missing prompt id"
                                        });
                                        write
                                            .lock()
                                            .await
                                            .send(Message::Text(response.to_string()))
                                            .await?;
                                    }
                                    continue;
                                }
                                "reset_prompts" => {
                                    info!("Received reset_prompts request");
                                    prompt_storage.clear_user_created()?;
                                    let default_prompts = get_all_default_prompts();
                                    let response = json!({
                                        "type": "prompts_reset",
                                        "prompts": default_prompts
                                    });
                                    write
                                        .lock()
                                        .await
                                        .send(Message::Text(response.to_string()))
                                        .await?;
                                    continue;
                                }
                                _ => {} // Continue to parse as command
                            }
                        }

                        // Extract commandId if present
                        let command_id = value
                            .get("commandId")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        // Remove commandId before parsing as Command
                        if let Some(obj) = value.as_object_mut() {
                            obj.remove("commandId");
                        }

                        match serde_json::from_value::<Command>(value) {
                            Ok(cmd) => {
                                let response = handler.handle_command(cmd);

                                // Add commandId back to response
                                let mut response_value = serde_json::to_value(&response)?;
                                if let Some(id) = command_id {
                                    if let Some(obj) = response_value.as_object_mut() {
                                        obj.insert(
                                            "commandId".to_string(),
                                            serde_json::Value::String(id),
                                        );
                                    }
                                }

                                let response_json = serde_json::to_string(&response_value)?;

                                write
                                    .lock()
                                    .await
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
                                        obj.insert(
                                            "commandId".to_string(),
                                            serde_json::Value::String(id),
                                        );
                                    }
                                }
                                write
                                    .lock()
                                    .await
                                    .send(Message::Text(serde_json::to_string(&response_json)?))
                                    .await?;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse command: {}", e);
                        let error_response = Response::Error {
                            error: format!("Invalid command format: {}", e),
                        };
                        let response_json = serde_json::to_string(&error_response)?;
                        write
                            .lock()
                            .await
                            .send(Message::Text(response_json))
                            .await?;
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                write.lock().await.send(Message::Pong(data)).await?;
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
