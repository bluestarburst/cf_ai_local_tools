// Removed ProgressType - steps are now sent directly via send_thinking_update
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
        let mut step_counter = 0usize;

        // Async helper to send step immediately via manager
        async fn send_step_async(
            manager: &Option<std::sync::Arc<dyn crate::agents::conversation::ConversationManager>>,
            step: &ExecutionStep,
        ) {
            if let Some(m) = manager {
                let _ = m
                    .send_thinking_update(
                        "",
                        step.step_number,
                        &serde_json::to_string(step).unwrap_or_default(),
                    )
                    .await;
            }
        }

        // ============================================
        // STEP 0: THINKING - Understand the task
        // ============================================
        let thinking_step = ExecutionStep {
            step_number: step_counter,
            step_type: StepType::Thinking,
            content: format!("Analyzing task: \"{}\"", task),
            tool_call: None,
            tool_observation: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        steps.push(thinking_step.clone());
        send_step_async(&conversation_manager, &thinking_step).await;
        step_counter += 1;

        // 1. Convert tools to LLM format
        let llm_tools = self.to_llm_tools(available_tools);

        // 2. Prepare messages
        let mut messages = Vec::new();
        messages.push(LLMMessage {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
            tool_calls: None,
        });

        for msg in &context.messages {
            messages.push(LLMMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                tool_calls: None,
            });
        }

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
            // ============================================
            // STEP N: PLANNING - Identify tools to use
            // ============================================
            let tool_names: Vec<String> = tool_calls.iter().map(|c| c.name.clone()).collect();
            let planning_step = ExecutionStep {
                step_number: step_counter,
                step_type: StepType::Planning,
                content: format!("Planning to use tool(s): {}", tool_names.join(", ")),
                tool_call: None,
                tool_observation: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            steps.push(planning_step.clone());
            send_step_async(&conversation_manager, &planning_step).await;
            step_counter += 1;

            for call in tool_calls {
                // ============================================
                // STEP N: ACTION - Execute the tool
                // ============================================
                let action_step = ExecutionStep {
                    step_number: step_counter,
                    step_type: StepType::Action,
                    content: format!("Executing tool: {}", call.name),
                    tool_call: Some(ToolCall {
                        tool_name: call.name.clone(),
                        arguments: call.arguments.clone(),
                        execution_time: std::time::Duration::from_millis(0),
                    }),
                    tool_observation: None,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                steps.push(action_step.clone());
                send_step_async(&conversation_manager, &action_step).await;
                step_counter += 1;

                // Find and execute tool
                if let Some(tool) = available_tools
                    .iter()
                    .find(|t| t.name() == call.name || t.id() == call.name)
                {
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

                    let execution_time = tool_start.elapsed();

                    // ============================================
                    // STEP N: OBSERVATION - Record result
                    // ============================================
                    match result {
                        Ok(tool_result) => {
                            let obs_step = ExecutionStep {
                                step_number: step_counter,
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
                            };
                            steps.push(obs_step.clone());
                            send_step_async(&conversation_manager, &obs_step).await;
                            step_counter += 1;
                        }
                        Err(e) => {
                            let obs_step = ExecutionStep {
                                step_number: step_counter,
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
                            };
                            steps.push(obs_step.clone());
                            send_step_async(&conversation_manager, &obs_step).await;
                            step_counter += 1;
                        }
                    }
                } else {
                    let obs_step = ExecutionStep {
                        step_number: step_counter,
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
                    };
                    steps.push(obs_step.clone());
                    send_step_async(&conversation_manager, &obs_step).await;
                    step_counter += 1;
                }
            }

            // ============================================
            // STEP N: REFLECTION - Verify goal completion
            // ============================================
            let reflection_step = ExecutionStep {
                step_number: step_counter,
                step_type: StepType::Reflection,
                content: "Task execution complete. Verifying goal satisfaction.".to_string(),
                tool_call: None,
                tool_observation: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            steps.push(reflection_step.clone());
            send_step_async(&conversation_manager, &reflection_step).await;
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
