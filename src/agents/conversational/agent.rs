use crate::agents::conversation::ProgressType;
use crate::{
    Agent, AgentContext, AgentResult, ExecutionStep, LLMClient, LLMMessage, LLMTool,
    ReasoningConfig, StepType, ToolCall, ToolObservation,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationalAgent {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub reasoning_config: ReasoningConfig,
    pub capabilities: Vec<String>,
    pub tool_dependencies: Vec<String>,
}

impl ConversationalAgent {
    pub fn new() -> Self {
        Self {
            id: "conversational-agent".to_string(),
            name: "Conversational Agent".to_string(),
            system_prompt: include_str!("prompt.txt").to_string(),
            reasoning_config: ReasoningConfig::default(),
            capabilities: vec!["conversation".to_string(), "general_knowledge".to_string()],
            tool_dependencies: vec![],
        }
    }

    fn to_llm_tools(&self, tools: &[Box<dyn crate::core::Tool>]) -> Vec<LLMTool> {
        tools
            .iter()
            .map(|t| LLMTool {
                name: t.name().to_string(),
                description: t.description().to_string(),
                parameters: self.convert_params_to_schema(t.parameters()),
            })
            .collect()
    }

    fn convert_params_to_schema(&self, params: &[crate::core::ToolParameter]) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in params {
            let mut param_schema = serde_json::Map::new();
            param_schema.insert("type".to_string(), serde_json::json!(param.param_type));
            param_schema.insert(
                "description".to_string(),
                serde_json::json!(param.description),
            );

            if let Some(enums) = &param.enum_values {
                param_schema.insert("enum".to_string(), serde_json::json!(enums));
            }

            properties.insert(param.name.clone(), serde_json::Value::Object(param_schema));

            if param.required {
                required.push(serde_json::Value::String(param.name.clone()));
            }
        }

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    }
}

#[async_trait]
impl Agent for ConversationalAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Handles general conversation and chat."
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    fn tool_dependencies(&self) -> &[String] {
        &self.tool_dependencies
    }

    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    fn reasoning_config(&self) -> &ReasoningConfig {
        &self.reasoning_config
    }

    async fn execute(
        &self,
        task: &str,
        context: &AgentContext,
        llm: &dyn LLMClient,
        conversation_manager: Option<
            std::sync::Arc<dyn crate::agents::conversation::ConversationManager>,
        >,
        available_tools: &[Box<dyn crate::core::Tool>],
    ) -> crate::core::Result<AgentResult> {
        let mut steps = Vec::new();
        let start_time = std::time::Instant::now();

        // 1. Convert tools to LLM format
        let llm_tools = self.to_llm_tools(available_tools);

        // 2. Prepare messages
        let mut messages = Vec::new();
        // Add system prompt
        messages.push(LLMMessage {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
            tool_calls: None,
        });

        // Add history from context (simplified)
        for msg in &context.messages {
            messages.push(LLMMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                tool_calls: None,
            });
        }

        // Add current task
        messages.push(LLMMessage {
            role: "user".to_string(),
            content: task.to_string(),
            tool_calls: None,
        });

        // 3. Call LLM
        let response = llm
            .chat_with_tools(&messages, &self.reasoning_config.model_id, Some(llm_tools))
            .await?;

        // 4. Process tool calls
        if let Some(tool_calls) = response.tool_calls {
            for call in tool_calls {
                // Determine step number
                let step_num = steps.len() + 1;

                // Send thinking/action update
                if let Some(manager) = &conversation_manager {
                    let _ = manager
                        .send_progress_update(
                            &self.id,
                            ProgressType::Executing,
                            &format!("Calling tool: {}", call.name),
                            None,
                        )
                        .await;
                }

                // Record thinking/action step
                steps.push(ExecutionStep {
                    step_number: step_num,
                    step_type: StepType::Action,
                    content: format!("Calling tool: {}", call.name),
                    tool_call: Some(ToolCall {
                        tool_name: call.name.clone(),
                        arguments: call.arguments.clone(),
                        execution_time: std::time::Duration::from_millis(0),
                    }),
                    tool_observation: None,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });

                // Find and execute tool
                println!(
                    "DEBUG: Looking for tool with name '{}', available tools:",
                    call.name
                );
                for t in available_tools.iter() {
                    println!("DEBUG:   - id='{}', name='{}'", t.id(), t.name());
                }
                if let Some(tool) = available_tools
                    .iter()
                    .find(|t| t.name() == call.name || t.id() == call.name)
                {
                    println!("DEBUG: Found tool: {}", tool.id());
                    println!("DEBUG: Executing tool with args: {:?}", call.arguments);
                    let tool_start = std::time::Instant::now();
                    let result = tool
                        .execute(
                            &call.arguments,
                            &crate::core::ToolContext {
                                agent_id: self.id.clone(),
                                conversation_manager: conversation_manager.clone(),
                                execution_state: std::sync::Arc::new(tokio::sync::RwLock::new(
                                    crate::core::ToolExecutionState::default(),
                                )),
                            },
                        )
                        .await;

                    println!("DEBUG: Tool execution result: {:?}", result);

                    match result {
                        Ok(tool_result) => {
                            println!("DEBUG: Tool succeeded: {}", tool_result.message);
                            // Send observation update
                            if let Some(manager) = &conversation_manager {
                                let _ = manager
                                    .send_progress_update(
                                        &self.id,
                                        ProgressType::Observing,
                                        &tool_result.message,
                                        None,
                                    )
                                    .await;
                            }

                            steps.push(ExecutionStep {
                                step_number: steps.len() + 1,
                                step_type: StepType::Observation,
                                content: tool_result.message.clone(),
                                tool_call: None,
                                tool_observation: Some(ToolObservation {
                                    success: tool_result.success,
                                    message: tool_result.message,
                                    data: tool_result.data,
                                    error: None,
                                }),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            });
                        }
                        Err(e) => {
                            println!("DEBUG: Tool execution error: {}", e);
                            // Send error update
                            if let Some(manager) = &conversation_manager {
                                let _ = manager
                                    .send_error_update(
                                        &self.id,
                                        &format!("Tool execution failed: {}", e),
                                        vec![],
                                    )
                                    .await;
                            }

                            steps.push(ExecutionStep {
                                step_number: steps.len() + 1,
                                step_type: StepType::Observation,
                                content: format!("Tool execution failed: {}", e),
                                tool_call: None,
                                tool_observation: Some(ToolObservation {
                                    success: false,
                                    message: format!("Error: {}", e),
                                    data: None,
                                    error: Some(e.to_string()),
                                }),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            });
                        }
                    }
                } else {
                    if let Some(manager) = &conversation_manager {
                        let _ = manager
                            .send_error_update(
                                &self.id,
                                &format!("Tool not found: {}", call.name),
                                vec![],
                            )
                            .await;
                    }

                    steps.push(ExecutionStep {
                        step_number: steps.len() + 1,
                        step_type: StepType::Observation,
                        content: format!("Tool not found: {}", call.name),
                        tool_call: None,
                        tool_observation: Some(ToolObservation {
                            success: false,
                            message: "Tool not found".to_string(),
                            data: None,
                            error: Some("Tool not found".to_string()),
                        }),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
            }
        }

        Ok(AgentResult {
            success: true,
            response: response.response,
            steps,
            execution_time: start_time.elapsed(),
            final_context: context.clone(),
        })
    }

    fn can_handle_task(&self, _task: &str) -> f32 {
        0.5 // Default fallback
    }
}
