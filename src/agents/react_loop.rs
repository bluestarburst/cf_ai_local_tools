use crate::llm::{LLMClient, Message};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::future::Future;
use tokio::sync::mpsc;
use tracing::{debug, warn};

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    #[serde(rename = "systemPrompt")]
    pub system_prompt: String,
    #[serde(rename = "modelId")]
    pub model_id: String,
    #[serde(rename = "maxIterations")]
    pub max_iterations: usize,
    pub tools: Vec<String>, // enabled tool IDs
    #[serde(rename = "separateReasoningModel", default)]
    pub separate_reasoning_model: bool,
    #[serde(rename = "reasoningModelId", default)]
    pub reasoning_model_id: Option<String>,
}

/// Represents a single ReAct step to be sent to the client
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionStep {
    #[serde(rename = "stepNumber")]
    pub step_number: usize,
    pub thought: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ToolAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation: Option<ToolObservation>,
    /// ID of the agent that generated this step (for tracking delegated agent steps)
    #[serde(rename = "agentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

/// Channel-based step sender for real-time streaming
pub type StepSender = mpsc::UnboundedSender<ExecutionStep>;

/// Track recent tool calls for loop detection
#[derive(Debug, Clone, PartialEq)]
struct ToolCallSignature {
    tool_name: String,
    arguments_hash: String,
}

impl ToolCallSignature {
    fn new(tool_name: &str, arguments: &Value) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            arguments_hash: arguments.to_string(),
        }
    }
}

/// Detect if we're in a loop (same tool called 3+ times with same args)
fn is_loop_detected(history: &VecDeque<ToolCallSignature>, current: &ToolCallSignature) -> bool {
    let count = history.iter().filter(|h| *h == current).count();
    count >= 2 // If we've seen this exact call twice already (3 total), we're looping
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolAction {
    pub tool: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolObservation {
    pub result: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Tool definition from Rust (matches main.rs ToolDefinition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameter>,
    #[serde(rename = "returnsObservation")]
    pub returns_observation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

/// Convert Rust tool definitions to Cloudflare Workers AI tool schema
fn convert_tools_to_cf_schema(tools: &[ToolDefinition]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            // Build parameters schema
            let mut properties = serde_json::Map::new();
            let mut required_params = Vec::new();

            for param in &tool.parameters {
                let mut param_schema = json!({
                    "type": param.param_type,
                    "description": param.description,
                });

                // Add enum values if present
                if let Some(ref enum_vals) = param.enum_values {
                    param_schema["enum"] = json!(enum_vals);
                }

                properties.insert(param.name.clone(), param_schema);

                if param.required {
                    required_params.push(param.name.clone());
                }
            }

            // Cloudflare Workers AI tool schema format
            json!({
                "name": tool.id,
                "description": tool.description,
                "parameters": {
                    "type": "object",
                    "properties": properties,
                    "required": required_params,
                }
            })
        })
        .collect()
}

/// Execute ReAct loop using Cloudflare Workers AI native tool calling
///
/// The `on_step` callback is called whenever a new execution step is completed.
/// This allows streaming intermediate steps to the client in real-time.
///
/// The `tool_executor` callback executes a tool and returns the observation result.
///
/// Arguments:
/// - `config`: Agent configuration
/// - `user_message`: User's input message
/// - `llm`: LLM client for API calls
/// - `available_tools`: All available tools
/// - `on_step`: Optional callback for step notifications (legacy)
/// - `tool_executor`: Function to execute tools
/// - `step_sender`: Optional channel to send steps for real-time streaming
/// - `agent_id`: Optional agent ID to tag steps with
pub async fn execute<F, E, Fut>(
    config: &AgentConfig,
    user_message: &str,
    llm: &LLMClient,
    available_tools: &[ToolDefinition],
    on_step: Option<F>,
    tool_executor: E,
    step_sender: Option<StepSender>,
    agent_id: Option<String>,
) -> Result<String>
where
    F: Fn(ExecutionStep) -> Result<()>,
    E: Fn(&str, &Value) -> Fut,
    Fut: Future<Output = Result<String>>,
{
    // Filter tools to only enabled ones
    let enabled_tools: Vec<ToolDefinition> = available_tools
        .iter()
        .filter(|t| config.tools.contains(&t.id))
        .cloned()
        .collect();

    debug!("[ReAct] Enabled tools: {:?}", config.tools);

    // Convert to Cloudflare schema
    let cf_tools = convert_tools_to_cf_schema(&enabled_tools);
    debug!("[ReAct] Converted {} tools to CF schema", cf_tools.len());

    let tools_option = if cf_tools.is_empty() {
        None
    } else {
        debug!("[ReAct] Sending tools to LLM: {:?}", cf_tools);
        Some(cf_tools)
    };

    // Interpolate system prompt placeholders
    let tools_list = enabled_tools
        .iter()
        .map(|t| format!("- {} ({}): {}", t.name, t.id, t.description))
        .collect::<Vec<_>>()
        .join("\n");

    let interpolated_prompt = config
        .system_prompt
        .replace("{tools}", &tools_list)
        .replace("{purpose}", "Execute user tasks using available tools");

    let mut messages = vec![
        Message {
            role: "system".to_string(),
            content: interpolated_prompt,
        },
        Message {
            role: "user".to_string(),
            content: user_message.to_string(),
        },
    ];

    // Track recent tool calls for loop detection
    let mut tool_call_history: VecDeque<ToolCallSignature> = VecDeque::with_capacity(10);

    for iteration in 1..=config.max_iterations {
        debug!("[ReAct] Iteration {}/{}", iteration, config.max_iterations);

        // PHASE 1: Get reasoning/thought (without tools)
        // This forces Cloudflare AI to provide reasoning before tool selection
        let reasoning_prompt = "Before taking action, think step-by-step and reflect:\n\
            1. What is the user's overall goal from the conversation history?\n\
            2. Review the most recent observations (if any) and summarize key insights or changes they introduce.\n\
            3. What specific action should you take next to progress toward the goal? Explain why this action differs from previous ones if applicable.\n\
            4. Will this action complete the goal? If yes, end your thought with 'GOAL_COMPLETE'.\n\
            \n\
            Provide concise reasoning (2-3 sentences max). Do NOT call tools or suggest actions here - focus on thinking only.".to_string();

        let mut reasoning_messages = messages.clone();
        reasoning_messages.push(Message {
            role: "user".to_string(),
            content: reasoning_prompt,
        });

        let reasoning_model_id = if config.separate_reasoning_model {
            config
                .reasoning_model_id
                .as_ref()
                .unwrap_or(&config.model_id)
        } else {
            &config.model_id
        };
        let reasoning_response = llm
            .chat_with_tools(reasoning_messages.clone(), reasoning_model_id, None)
            .await?;

        let thought = reasoning_response.response.trim().to_string();
        debug!(
            "[ReAct] Phase 1 (reasoning) - Raw response: '{}'",
            reasoning_response.response
        );
        debug!(
            "[ReAct] Phase 1 (reasoning) - Thought extracted: '{}'",
            thought
        );

        // Check if goal is complete based on reasoning
        if thought.to_uppercase().contains("GOAL_COMPLETE") {
            debug!("[ReAct] Goal marked as complete in reasoning phase");
            return Ok(format!("Task completed: {}", thought));
        }

        // If reasoning phase returned empty, use a default thought based on the action
        let thought = if thought.is_empty() {
            debug!("[ReAct] Warning: Reasoning phase returned empty response");
            format!("Processing user request: {}", user_message)
        } else {
            thought
        };

        // PHASE 2: Get tool calls (with tools)
        // Now ask the LLM to execute based on its reasoning
        let action_prompt = "Based on your reasoning above, execute the next action. \
            You MUST call exactly one available tool to make progress toward the goal. \
            Do not explain, describe, or add text - just call the tool with the appropriate parameters. \
            If your reasoning indicated 'GOAL_COMPLETE', do not call any tools.".to_string();

        let mut action_messages = messages.clone();
        action_messages.push(Message {
            role: "assistant".to_string(),
            content: thought.clone(),
        });
        action_messages.push(Message {
            role: "user".to_string(),
            content: action_prompt,
        });

        debug!(
            "[ReAct] Phase 2 - Sending {} tools to LLM",
            tools_option.as_ref().map(|t| t.len()).unwrap_or(0)
        );

        let response = llm
            .chat_with_tools(action_messages, &config.model_id, tools_option.clone())
            .await?;

        debug!("[ReAct] LLM response: {}", response.response);
        debug!("[ReAct] LLM tool_calls: {:?}", response.tool_calls);

        // Check if LLM wants to call tools
        if let Some(tool_calls) = response.tool_calls {
            if !tool_calls.is_empty() {
                debug!("[ReAct] Tool calls detected: {} calls", tool_calls.len());

                // Send step with action (tool call) to client
                let first_tool_call = &tool_calls[0];

                // Check for loop - same tool with same arguments called repeatedly
                let call_signature =
                    ToolCallSignature::new(&first_tool_call.name, &first_tool_call.arguments);
                if is_loop_detected(&tool_call_history, &call_signature) {
                    warn!("[ReAct] Loop detected! Tool '{}' called with same arguments 3+ times. Breaking loop.", first_tool_call.name);
                    return Ok(format!(
                        "I attempted to call {} multiple times with the same parameters but couldn't make progress. \
                        The task may require different tools or a different approach. Last attempted: {} with {}",
                        first_tool_call.name,
                        first_tool_call.name,
                        first_tool_call.arguments
                    ));
                }

                // Track this call
                tool_call_history.push_back(call_signature);
                if tool_call_history.len() > 10 {
                    tool_call_history.pop_front();
                }
                let step = ExecutionStep {
                    step_number: iteration,
                    thought, // Use the reasoning from Phase 1
                    action: Some(ToolAction {
                        tool: first_tool_call.name.clone(),
                        parameters: first_tool_call.arguments.clone(),
                    }),
                    observation: None,
                    agent_id: agent_id.clone(),
                };

                // Send via channel for real-time streaming (preferred)
                if let Some(ref sender) = step_sender {
                    let _ = sender.send(step.clone());
                }

                // Also call legacy callback if provided
                if let Some(ref callback) = on_step {
                    callback(step)?;
                }

                // Add assistant's response (with tool calls)
                messages.push(Message {
                    role: "assistant".to_string(),
                    content: response.response.clone(),
                });

                // Execute each tool call and collect observations
                let mut observations = Vec::new();
                for (tool_idx, tool_call) in tool_calls.iter().enumerate() {
                    debug!(
                        "[ReAct] Executing tool: {} with args: {}",
                        tool_call.name, tool_call.arguments
                    );

                    // Execute the tool using the provided executor
                    let (observation, error) =
                        match tool_executor(&tool_call.name, &tool_call.arguments).await {
                            Ok(result) => (result, None),
                            Err(e) => {
                                let err_msg =
                                    format!("Error executing tool '{}': {}", tool_call.name, e);
                                (err_msg.clone(), Some(err_msg))
                            }
                        };

                    debug!("[ReAct] Tool observation: {}", observation);

                    // Format observation with status
                    let status = if error.is_some() { "FAILED" } else { "SUCCESS" };
                    let formatted_observation = format!(
                        "[{}] Tool '{}': {}\nDetails: {}",
                        status,
                        tool_call.name,
                        if error.is_some() {
                            "Failed"
                        } else {
                            "Succeeded"
                        },
                        observation
                    );

                    // Send observation step for real-time streaming
                    let obs_step = ExecutionStep {
                        step_number: iteration,
                        thought: format!(
                            "Executed {} (tool {}/{})",
                            tool_call.name,
                            tool_idx + 1,
                            tool_calls.len()
                        ),
                        action: Some(ToolAction {
                            tool: tool_call.name.clone(),
                            parameters: tool_call.arguments.clone(),
                        }),
                        observation: Some(ToolObservation {
                            result: serde_json::Value::String(formatted_observation.clone()),
                            error,
                        }),
                        agent_id: agent_id.clone(),
                    };

                    // Send via channel for real-time streaming
                    if let Some(ref sender) = step_sender {
                        let _ = sender.send(obs_step);
                    }

                    observations.push(formatted_observation);
                }

                // Add observations as user message
                let observations_text = observations.join("\n\n");
                messages.push(Message {
                    role: "user".to_string(),
                    content: format!(
                        "Latest Observations:\n{}\n\nReflect on these results and decide the next action to progress toward the goal. If errors occurred, adapt your approach.",
                        observations_text
                    ),
                });

                continue; // Next iteration
            }
        }

        // No tool calls = final answer
        debug!("[ReAct] No tool calls, returning final answer");

        // Combine thought and final response for better context
        let final_response = if !thought.is_empty() && !response.response.is_empty() {
            format!("{}\n\n{}", thought, response.response)
        } else if !response.response.is_empty() {
            response.response
        } else {
            thought
        };

        return Ok(final_response);
    }

    Ok(format!(
        "Max iterations ({}) reached without completing the goal. The task may require a different approach or additional tools. Last thought: '{}'",
        config.max_iterations,
        messages.last().map(|m| m.content.as_str()).unwrap_or("None")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_tools_to_cf_schema() {
        let tools = vec![ToolDefinition {
            id: "mouse_move".to_string(),
            name: "Mouse Move".to_string(),
            description: "Move mouse to coordinates".to_string(),
            category: "mouse".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "x".to_string(),
                    param_type: "number".to_string(),
                    description: "X coordinate".to_string(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "y".to_string(),
                    param_type: "number".to_string(),
                    description: "Y coordinate".to_string(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
            ],
            returns_observation: true,
        }];

        let cf_schema = convert_tools_to_cf_schema(&tools);
        assert_eq!(cf_schema.len(), 1);

        let tool = &cf_schema[0];
        assert_eq!(tool["name"], "mouse_move");
        assert!(tool["parameters"]["properties"].is_object());
    }
}
