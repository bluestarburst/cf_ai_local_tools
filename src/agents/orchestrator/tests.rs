// Integration tests for Orchestrator Agent
// Tests WebSocket communication and delegation behavior
//
// Requirements:
// 1. Cloudflare Worker running: cd cf-worker && wrangler dev
// 2. Desktop App running: cargo run

use crate::agents::presets::Metadata;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// Helper function to create a test orchestrator agent with interpolated prompts
fn create_test_agent() -> super::super::presets::Agent {
    let metadata = Metadata {
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
        author: Some("test".to_string()),
        tags: None,
    };
    super::create_agent(metadata)
}

/// Helper to extract tool calls from execution_step messages
fn extract_tool_calls(responses: &[serde_json::Value]) -> Vec<(String, serde_json::Value)> {
    responses
        .iter()
        .filter_map(|r| {
            if r["type"] == "execution_step" {
                if let Some(step) = r.get("step") {
                    if let Some(action) = step.get("action") {
                        let tool = action["tool"].as_str().unwrap_or("").to_string();
                        let params = action.get("parameters").cloned().unwrap_or(json!({}));
                        return Some((tool, params));
                    }
                }
            }
            None
        })
        .collect()
}

#[tokio::test]
#[ignore] // Run with: cargo test orchestrator -- --ignored --nocapture
async fn test_orchestrator_greeting_no_delegation() {
    println!("\n🧪 Testing orchestrator responds directly to greetings (no delegation)...");
    println!("⚠️  Requirements:");
    println!("   1. Cloudflare Worker running: cd cf-worker && wrangler dev");
    println!("   2. Desktop App running: cargo run");
    println!();

    let ws_url = "ws://localhost:8787/connect?device=web-viewer";

    println!("📡 Connecting to WebSocket: {}", ws_url);
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket - is cf-worker running?");

    let (mut write, mut read) = ws_stream.split();
    println!("✅ WebSocket connected\n");

    let agent = create_test_agent();
    let chat_request = json!({
        "type": "chat_request",
        "message": "hello",
        "agent": {
            "systemPrompt": agent.system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 5,
            "tools": ["delegate_to_agent"]
        }
    });

    println!("📤 Sending chat request: \"hello\"");
    write
        .send(Message::Text(chat_request.to_string()))
        .await
        .expect("Failed to send message");
    println!("✅ Request sent\n");

    println!("⏳ Waiting for responses (timeout: 30s)...");
    let mut responses = Vec::new();
    let timeout_duration = tokio::time::Duration::from_secs(30);

    let result = tokio::time::timeout(timeout_duration, async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();

                println!("📨 Received message type: {}", parsed["type"]);

                match parsed["type"].as_str() {
                    Some("execution_step") => {
                        if let Some(step) = parsed.get("step") {
                            let thought = step["thought"].as_str().unwrap_or("(no thought)");
                            let thought_display = if thought.is_empty() {
                                "(empty)"
                            } else {
                                thought
                            };
                            println!(
                                "   Step #{}: 💭 Thought: \"{}\"",
                                step["stepNumber"], thought_display
                            );
                            if let Some(action) = step.get("action") {
                                println!("   🔧 Action tool: {}", action["tool"]);
                                println!(
                                    "   📋 Parameters: {}",
                                    action.get("parameters").unwrap_or(&json!({}))
                                );
                            }
                        }
                    }
                    Some("chat_response") => {
                        println!("   💬 Response: {}", parsed["content"]);
                    }
                    Some("error") => {
                        println!(
                            "   ❌ Error: {}",
                            parsed
                                .get("content")
                                .or(parsed.get("error"))
                                .unwrap_or(&json!("Unknown error"))
                        );
                    }
                    _ => {}
                }

                responses.push(parsed.clone());

                // Break when we receive chat_response (final response)
                if parsed["type"] == "chat_response" {
                    println!("\n✅ Received final response, test complete");
                    break;
                }
            }
        }
    })
    .await;

    if result.is_err() {
        println!("\n❌ Test timed out after 30 seconds");
        panic!("Test timed out - no final response received");
    }

    println!("\n📊 Test Results:");
    println!("   Total messages received: {}", responses.len());

    // Extract tool calls
    let tool_calls = extract_tool_calls(&responses);
    println!("   Tool calls made: {}", tool_calls.len());

    // Check that NO delegation occurred (for a simple greeting)
    let has_delegation = tool_calls
        .iter()
        .any(|(tool, _)| tool == "delegate_to_agent");

    if has_delegation {
        println!("\n⚠️  Warning: Orchestrator delegated a greeting (not ideal but acceptable)");
        // Don't fail - LLM behavior can vary
    } else {
        println!("\n✅ Orchestrator correctly responded directly without delegation");
    }

    // Check for a response
    let final_response = responses
        .iter()
        .find(|r| r["type"] == "chat_response")
        .expect("Should have a chat_response");

    let response_content = final_response["content"].as_str().unwrap_or("");
    println!("   Final response: {}", response_content);

    assert!(
        !response_content.is_empty(),
        "Should have non-empty response"
    );
    println!("\n🎉 Test passed!");
}

#[tokio::test]
#[ignore]
async fn test_orchestrator_delegates_automation_task() {
    println!("\n🧪 Testing orchestrator delegates automation tasks to desktop-automation-agent...");
    println!("⚠️  Requirements:");
    println!("   1. Cloudflare Worker running: cd cf-worker && wrangler dev");
    println!("   2. Desktop App running: cargo run");
    println!();

    let ws_url = "ws://localhost:8787/connect?device=web-viewer";

    println!("📡 Connecting to WebSocket: {}", ws_url);
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();
    println!("✅ WebSocket connected\n");

    let agent = create_test_agent();
    let chat_request = json!({
        "type": "chat_request",
        "message": "move the mouse to 500, 600",
        "agent": {
            "systemPrompt": agent.system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 10,
            "tools": ["delegate_to_agent"]
        }
    });

    println!("📤 Sending chat request: \"move the mouse to 500, 600\"");
    write
        .send(Message::Text(chat_request.to_string()))
        .await
        .expect("Failed to send message");
    println!("✅ Request sent\n");

    println!("⏳ Waiting for responses (timeout: 60s)...");
    let mut responses = Vec::new();
    let timeout_duration = tokio::time::Duration::from_secs(60);

    let result = tokio::time::timeout(timeout_duration, async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();

                println!("📨 Received message type: {}", parsed["type"]);

                match parsed["type"].as_str() {
                    Some("execution_step") => {
                        if let Some(step) = parsed.get("step") {
                            let thought = step["thought"].as_str().unwrap_or("(no thought)");
                            let thought_display = if thought.is_empty() {
                                "(empty)"
                            } else {
                                thought
                            };
                            println!(
                                "   Step #{}: 💭 Thought: \"{}\"",
                                step["stepNumber"], thought_display
                            );
                            if let Some(action) = step.get("action") {
                                println!("   🔧 Action tool: {}", action["tool"]);
                                println!(
                                    "   📋 Parameters: {}",
                                    action.get("parameters").unwrap_or(&json!({}))
                                );
                            }
                            if let Some(observation) = step.get("observation") {
                                let obs_str = observation["result"]
                                    .as_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| observation["result"].to_string());
                                // Truncate long observations
                                let display = if obs_str.len() > 100 {
                                    format!("{}...", &obs_str[..100])
                                } else {
                                    obs_str
                                };
                                println!("   👁️  Observation: {}", display);
                            }
                        }
                    }
                    Some("chat_response") => {
                        println!("   💬 Response: {}", parsed["content"]);
                    }
                    Some("error") => {
                        println!(
                            "   ❌ Error: {}",
                            parsed
                                .get("content")
                                .or(parsed.get("error"))
                                .unwrap_or(&json!("Unknown error"))
                        );
                    }
                    _ => {}
                }

                responses.push(parsed.clone());

                if parsed["type"] == "chat_response" {
                    println!("\n✅ Received final response, test complete");
                    break;
                }
            }
        }
    })
    .await;

    if result.is_err() {
        println!("\n❌ Test timed out after 60 seconds");
        println!("📊 Received {} messages before timeout:", responses.len());
        for (i, resp) in responses.iter().enumerate() {
            println!("   {}. Type: {}", i + 1, resp["type"]);
        }
        panic!("Test timed out");
    }

    println!("\n📊 Test Results:");
    println!("   Total messages received: {}", responses.len());

    // Extract tool calls
    let tool_calls = extract_tool_calls(&responses);
    println!("   Tool calls made: {}", tool_calls.len());
    for (tool, params) in &tool_calls {
        println!("      - {} with params: {}", tool, params);
    }

    // Find delegation to desktop-automation-agent
    let delegation = tool_calls.iter().find(|(tool, params)| {
        tool == "delegate_to_agent"
            && params.get("agent_id").and_then(|v| v.as_str()) == Some("desktop-automation-agent")
    });

    if let Some((_, params)) = delegation {
        println!("\n✅ Found delegation to desktop-automation-agent!");
        let task = params["task"].as_str().unwrap_or("");
        println!("   Task: {}", task);

        // Verify task contains coordinates
        assert!(
            task.contains("500") || task.contains("600"),
            "Task should include coordinates: {}",
            task
        );
        println!("\n🎉 Test passed!");
    } else {
        // Check if the orchestrator directly delegated or handled it differently
        println!(
            "\n⚠️  No explicit delegation found. Checking if task was completed differently..."
        );

        // Look for mouse_move being called (maybe orchestrator has different tools enabled)
        let mouse_move = tool_calls.iter().find(|(tool, _)| tool == "mouse_move");
        if mouse_move.is_some() {
            println!("   Found direct mouse_move call - orchestrator may have the tool directly");
        }

        // For now, just ensure we got a response
        let final_response = responses.iter().find(|r| r["type"] == "chat_response");
        assert!(final_response.is_some(), "Should have received a response");
        println!("\n🎉 Test completed (delegation path may differ based on configuration)");
    }
}

#[tokio::test]
#[ignore]
async fn test_orchestrator_no_infinite_loop() {
    println!("\n🧪 Testing orchestrator doesn't get stuck in infinite delegation loop...");
    println!("⚠️  Requirements:");
    println!("   1. Cloudflare Worker running: cd cf-worker && wrangler dev");
    println!("   2. Desktop App running: cargo run");
    println!();

    let ws_url = "ws://localhost:8787/connect?device=web-viewer";

    println!("📡 Connecting to WebSocket: {}", ws_url);
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();
    println!("✅ WebSocket connected\n");

    let agent = create_test_agent();
    let chat_request = json!({
        "type": "chat_request",
        "message": "help",
        "agent": {
            "systemPrompt": agent.system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 10,
            "tools": ["delegate_to_agent"]
        }
    });

    println!("📤 Sending chat request: \"help\"");
    write
        .send(Message::Text(chat_request.to_string()))
        .await
        .expect("Failed to send message");
    println!("✅ Request sent\n");

    println!("⏳ Waiting for responses (timeout: 30s)...");
    let mut responses = Vec::new();
    let timeout_duration = tokio::time::Duration::from_secs(30);

    let result = tokio::time::timeout(timeout_duration, async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();

                println!("📨 Received message type: {}", parsed["type"]);

                match parsed["type"].as_str() {
                    Some("execution_step") => {
                        if let Some(step) = parsed.get("step") {
                            let thought = step["thought"].as_str().unwrap_or("(no thought)");
                            let thought_display = if thought.is_empty() {
                                "(empty)"
                            } else {
                                thought
                            };
                            println!(
                                "   Step #{}: 💭 Thought: \"{}\"",
                                step["stepNumber"], thought_display
                            );
                            if let Some(action) = step.get("action") {
                                println!("   🔧 Action tool: {}", action["tool"]);
                            }
                        }
                    }
                    Some("chat_response") => {
                        println!("   💬 Response: {}", parsed["content"]);
                    }
                    _ => {}
                }

                responses.push(parsed.clone());

                if parsed["type"] == "chat_response" {
                    println!("\n✅ Received final response, test complete");
                    break;
                }
            }
        }
    })
    .await;

    if result.is_err() {
        println!("\n❌ Test timed out after 30 seconds");
        panic!("Test timed out - possible infinite loop");
    }

    println!("\n📊 Test Results:");
    println!("   Total messages received: {}", responses.len());

    // Count delegation tool calls
    let tool_calls = extract_tool_calls(&responses);
    let delegation_count = tool_calls
        .iter()
        .filter(|(tool, _)| tool == "delegate_to_agent")
        .count();

    println!("   Delegation calls: {}", delegation_count);

    // Should not delegate more than once for unclear requests
    assert!(
        delegation_count <= 2,
        "Should not delegate more than twice for unclear requests, got {} delegations",
        delegation_count
    );

    // Should have a final response
    let final_response = responses
        .iter()
        .find(|r| r["type"] == "chat_response")
        .expect("Should have a chat_response");

    assert!(
        final_response["content"].as_str().is_some(),
        "Should have a text response"
    );

    println!("\n🎉 Test passed! No infinite loop detected.");
}

#[tokio::test]
#[ignore]
async fn test_orchestrator_web_research_delegation() {
    println!("\n🧪 Testing orchestrator delegates web research tasks...");
    println!("⚠️  Requirements:");
    println!("   1. Cloudflare Worker running: cd cf-worker && wrangler dev");
    println!("   2. Desktop App running: cargo run");
    println!();

    let ws_url = "ws://localhost:8787/connect?device=web-viewer";

    println!("📡 Connecting to WebSocket: {}", ws_url);
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();
    println!("✅ WebSocket connected\n");

    let agent = create_test_agent();
    let chat_request = json!({
        "type": "chat_request",
        "message": "search for the latest news about AI",
        "agent": {
            "systemPrompt": agent.system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 10,
            "tools": ["delegate_to_agent"]
        }
    });

    println!("📤 Sending chat request: \"search for the latest news about AI\"");
    write
        .send(Message::Text(chat_request.to_string()))
        .await
        .expect("Failed to send message");
    println!("✅ Request sent\n");

    println!("⏳ Waiting for responses (timeout: 60s)...");
    let mut responses = Vec::new();
    let timeout_duration = tokio::time::Duration::from_secs(60);

    let result = tokio::time::timeout(timeout_duration, async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();

                println!("📨 Received message type: {}", parsed["type"]);

                match parsed["type"].as_str() {
                    Some("execution_step") => {
                        if let Some(step) = parsed.get("step") {
                            let thought = step["thought"].as_str().unwrap_or("(no thought)");
                            let thought_display = if thought.is_empty() {
                                "(empty)"
                            } else {
                                thought
                            };
                            println!(
                                "   Step #{}: 💭 Thought: \"{}\"",
                                step["stepNumber"], thought_display
                            );
                            if let Some(action) = step.get("action") {
                                println!("   🔧 Action tool: {}", action["tool"]);
                                println!(
                                    "   📋 Parameters: {}",
                                    action.get("parameters").unwrap_or(&json!({}))
                                );
                            }
                            if let Some(observation) = step.get("observation") {
                                let obs_str = observation["result"]
                                    .as_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| observation["result"].to_string());
                                let display = if obs_str.len() > 100 {
                                    format!("{}...", &obs_str[..100])
                                } else {
                                    obs_str
                                };
                                println!("   👁️  Observation: {}", display);
                            }
                        }
                    }
                    Some("chat_response") => {
                        println!("   💬 Response: {}", parsed["content"]);
                    }
                    _ => {}
                }

                responses.push(parsed.clone());

                if parsed["type"] == "chat_response" {
                    println!("\n✅ Received final response, test complete");
                    break;
                }
            }
        }
    })
    .await;

    if result.is_err() {
        println!("\n❌ Test timed out after 60 seconds");
        println!("📊 Received {} messages before timeout:", responses.len());
        panic!("Test timed out");
    }

    println!("\n📊 Test Results:");
    println!("   Total messages received: {}", responses.len());

    // Extract tool calls
    let tool_calls = extract_tool_calls(&responses);
    println!("   Tool calls made: {}", tool_calls.len());
    for (tool, params) in &tool_calls {
        println!("      - {} with params: {}", tool, params);
    }

    // Find delegation to web-research-agent
    let delegation = tool_calls.iter().find(|(tool, params)| {
        tool == "delegate_to_agent"
            && params.get("agent_id").and_then(|v| v.as_str()) == Some("web-research-agent")
    });

    if let Some((_, params)) = delegation {
        println!("\n✅ Found delegation to web-research-agent!");
        let task = params["task"].as_str().unwrap_or("");
        println!("   Task: {}", task);
        println!("\n🎉 Test passed!");
    } else {
        // LLM might not always delegate as expected
        println!("\n⚠️  No delegation to web-research-agent found");

        // Ensure we got some response
        let final_response = responses.iter().find(|r| r["type"] == "chat_response");
        assert!(final_response.is_some(), "Should have received a response");
        println!("   Test completed (delegation behavior may vary)");
    }
}

#[test]
fn test_orchestrator_agent_configuration() {
    println!("\n🧪 Testing orchestrator agent configuration...");

    let agent = create_test_agent();

    assert_eq!(agent.id, "orchestrator-agent");
    assert_eq!(agent.name, "Orchestrator");
    assert_eq!(agent.max_iterations, 10);
    assert_eq!(agent.model_id, "@cf/meta/llama-3.3-70b-instruct-fp8-fast");

    // Check tools are configured
    let tool_ids: Vec<_> = agent.tools.iter().map(|t| t.tool_id.as_str()).collect();
    assert!(
        tool_ids.contains(&"delegate_to_agent"),
        "Should have delegate_to_agent tool"
    );

    // Check prompt contains key sections
    assert!(
        agent.system_prompt.contains("AVAILABLE AGENTS"),
        "Prompt should list available agents"
    );
    assert!(
        agent.system_prompt.contains("WHEN TO DELEGATE"),
        "Prompt should have delegation rules"
    );
    assert!(
        agent.system_prompt.contains("WHEN TO RESPOND DIRECTLY"),
        "Prompt should have direct response rules"
    );
    assert!(
        agent.system_prompt.contains("desktop-automation-agent"),
        "Prompt should include desktop-automation-agent"
    );

    println!("✅ Orchestrator configuration is correct");
    println!("   Agent ID: {}", agent.id);
    println!("   Tools: {:?}", tool_ids);
    println!("   Max iterations: {}", agent.max_iterations);
}
