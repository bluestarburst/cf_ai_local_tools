use cf_ai_local_tools::agents::conversational::ConversationalAgent;
use cf_ai_local_tools::core::{Agent, AgentContext, LLMToolCall, Tool};
use cf_ai_local_tools::llm::MockLLMClient;
use cf_ai_local_tools::tools::delegation::DelegateToAgent;

#[tokio::test]
async fn test_single_delegation() {
    // 1. Setup Orchestrator Agent
    let orchestrator = ConversationalAgent::new();
    let context = AgentContext::new(orchestrator.id().to_string());

    // 2. Setup Tools
    let delegation_tool = Box::new(DelegateToAgent::new());
    let tools: Vec<Box<dyn Tool>> = vec![delegation_tool];

    // 3. Setup Mock LLM
    let mut mock_llm = MockLLMClient::new();

    // Program the LLM to call the delegation tool
    let tool_call = LLMToolCall {
        name: "delegate_to_agent".to_string(), // Must match tool ID/name
        arguments: serde_json::json!({
            "target_agent": "desktop-automation-agent",
            "task": "Move mouse to (100, 100)"
        }),
        id: Some("call_1".to_string()),
    };

    mock_llm.add_tool_response(
        "I will delegate this task to the desktop automation agent.".to_string(),
        vec![tool_call],
    );

    // 4. Execute Orchestrator
    let result = orchestrator
        .execute("Please move the mouse.", &context, &mock_llm, None, &tools)
        .await
        .expect("Agent execution failed");

    // 5. Verify Results
    assert!(result.success);

    // Check if tool was called
    let action_step = result
        .steps
        .iter()
        .find(|s| matches!(s.step_type, cf_ai_local_tools::core::StepType::Action));
    assert!(action_step.is_some(), "No action step found");

    // Check tool call details
    let tool_call_step = action_step.unwrap().tool_call.as_ref().unwrap();
    assert_eq!(tool_call_step.tool_name, "delegate_to_agent"); // ID of DelegateToAgent is "delegate_to_agent"

    // Check observation (result of tool execution)
    let observation_step = result
        .steps
        .iter()
        .find(|s| matches!(s.step_type, cf_ai_local_tools::core::StepType::Observation));
    assert!(observation_step.is_some(), "No observation step found");

    let observation = observation_step.unwrap().tool_observation.as_ref().unwrap();
    assert!(observation.success);
    assert!(observation.message.contains("Successfully delegated task"));
    assert!(observation.message.contains("desktop-automation-agent"));
}

#[tokio::test]
async fn test_multi_agent_delegation() {
    // 1. Setup Orchestrator Agent
    let orchestrator = ConversationalAgent::new();
    let context = AgentContext::new(orchestrator.id().to_string());

    // 2. Setup Tools
    let delegation_tool = Box::new(DelegateToAgent::new());
    let tools: Vec<Box<dyn Tool>> = vec![delegation_tool];

    // 3. Setup Mock LLM to delegate to two different agents
    let mut mock_llm = MockLLMClient::new();

    let tool_call_1 = LLMToolCall {
        name: "delegate_to_agent".to_string(),
        arguments: serde_json::json!({
            "target_agent": "web-research-agent",
            "task": "Find information about Rust Async"
        }),
        id: Some("call_1".to_string()),
    };

    let tool_call_2 = LLMToolCall {
        name: "delegate_to_agent".to_string(),
        arguments: serde_json::json!({
            "target_agent": "desktop-automation-agent",
            "task": "Write the findings to a file"
        }),
        id: Some("call_2".to_string()),
    };

    mock_llm.add_tool_response(
        "I will research the topic and then write the results.".to_string(),
        vec![tool_call_1, tool_call_2],
    );

    // 4. Execute Orchestrator
    let result = orchestrator
        .execute(
            "Research Rust Async and write a report.",
            &context,
            &mock_llm,
            None,
            &tools,
        )
        .await
        .expect("Agent execution failed");

    // 5. Verify Results
    assert!(result.success);

    // Check for two action steps
    let action_steps: Vec<_> = result
        .steps
        .iter()
        .filter(|s| matches!(s.step_type, cf_ai_local_tools::core::StepType::Action))
        .collect();

    assert_eq!(action_steps.len(), 2, "Expected 2 action steps");

    // Verify first delegation
    let call_1 = action_steps[0].tool_call.as_ref().unwrap();
    assert_eq!(call_1.tool_name, "delegate_to_agent");
    assert_eq!(call_1.arguments["target_agent"], "web-research-agent");

    // Verify second delegation
    let call_2 = action_steps[1].tool_call.as_ref().unwrap();
    assert_eq!(call_2.tool_name, "delegate_to_agent");
    assert_eq!(call_2.arguments["target_agent"], "desktop-automation-agent");

    // Check for two observation steps
    let observation_steps: Vec<_> = result
        .steps
        .iter()
        .filter(|s| matches!(s.step_type, cf_ai_local_tools::core::StepType::Observation))
        .collect();

    assert_eq!(observation_steps.len(), 2, "Expected 2 observation steps");
    assert!(
        observation_steps[0]
            .tool_observation
            .as_ref()
            .unwrap()
            .success
    );
    assert!(
        observation_steps[1]
            .tool_observation
            .as_ref()
            .unwrap()
            .success
    );
}
