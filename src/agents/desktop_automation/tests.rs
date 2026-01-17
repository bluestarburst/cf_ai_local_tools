// Integration tests for Desktop Automation Agent
// These tests require the full stack to be running:
// 1. Cloudflare Worker: cd cf-worker && wrangler dev
// 2. Desktop App: cargo run

use crate::agents::presets::Metadata;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// Helper function to create a test agent with proper configuration
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

#[tokio::test]
#[ignore]
async fn test_desktop_agent_mouse_move() {
    println!("\n🧪 Starting desktop agent mouse move test...");
    println!("⚠️  Requirements:");
    println!("   1. Cloudflare Worker running: cd cf-worker && wrangler dev");
    println!("   2. Desktop App running: cargo run");
    println!();

    // Test that desktop agent correctly uses mouse_move tool with proper parameters
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
        "message": "Move the mouse to x=500, y=600",
        "agent": {
            "systemPrompt": agent.system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 3,
            "tools": ["mouse_move", "mouse_click", "keyboard_input", "get_mouse_position"]
        }
    });

    println!("📤 Sending chat request: \"Move the mouse to x=500, y=600\"");
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

                // Debug: print full message details
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
                            if let Some(observation) = step.get("observation") {
                                println!("   👁️  Result: {}", observation["result"]);
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

    match result {
        Ok(_) => {
            println!("\n📊 Test Results:");
            println!("   Total messages received: {}", responses.len());
        }
        Err(_) => {
            println!("\n❌ Test timed out after 30 seconds");
            println!("📊 Received {} messages before timeout:", responses.len());
            for (i, resp) in responses.iter().enumerate() {
                println!("   {}. Type: {}", i + 1, resp["type"]);
            }
            println!("\n💡 Troubleshooting:");
            println!("   1. Is the desktop app running? (cargo run)");
            println!("   2. Is the CF worker running? (cd cf-worker && wrangler dev)");
            println!("   3. Check logs in both terminals");
            println!("   4. Check if LLM is responding with tool calls");
            panic!("Test timed out - no final response received");
        }
    }

    // Find mouse_move action in execution_step messages
    let mut found_mouse_move = false;
    let mut mouse_x = 0;
    let mut mouse_y = 0;

    for resp in &responses {
        if resp["type"] == "execution_step" {
            if let Some(step) = resp.get("step") {
                if let Some(action) = step.get("action") {
                    if action["tool"] == "mouse_move" {
                        println!("\n✅ Found mouse_move action!");
                        // Check both 'parameters' and 'input' fields for compatibility
                        let params = action
                            .get("parameters")
                            .or_else(|| action.get("input"))
                            .and_then(|v| v.as_object());

                        if let Some(params) = params {
                            // Coordinates might be strings or numbers
                            if let Some(x_val) = params.get("x") {
                                mouse_x = x_val
                                    .as_i64()
                                    .or_else(|| x_val.as_str().and_then(|s| s.parse().ok()))
                                    .unwrap_or(0);
                                println!("   X coordinate: {}", mouse_x);
                            }
                            if let Some(y_val) = params.get("y") {
                                mouse_y = y_val
                                    .as_i64()
                                    .or_else(|| y_val.as_str().and_then(|s| s.parse().ok()))
                                    .unwrap_or(0);
                                println!("   Y coordinate: {}", mouse_y);
                            }
                        }
                        found_mouse_move = true;
                        break;
                    }
                }
            }
        }
    }

    if !found_mouse_move {
        println!("\n❌ No mouse_move action found");
        println!("📋 Messages received:");
        for (i, resp) in responses.iter().enumerate() {
            println!("   {}. Type: {}", i + 1, resp["type"]);
            if resp["type"] == "execution_step" {
                if let Some(step) = resp.get("step") {
                    if let Some(action) = step.get("action") {
                        println!("      Tool: {}", action["tool"]);
                    }
                }
            }
        }
        panic!("Should have called mouse_move tool");
    }

    // Verify coordinates
    assert_eq!(mouse_x, 500, "x coordinate should be 500, got {}", mouse_x);
    assert_eq!(mouse_y, 600, "y coordinate should be 600, got {}", mouse_y);

    println!("\n🎉 Test passed! Mouse move tool was called with correct parameters");
}

#[tokio::test]
#[ignore]
async fn test_desktop_agent_mouse_click() {
    println!("\n🧪 Testing desktop agent mouse click...");
    println!("⚠️  Requirements:");
    println!("   1. Cloudflare Worker running: cd cf-worker && wrangler dev");
    println!("   2. Desktop App running: cargo run");
    println!();

    let ws_url = "ws://localhost:8787/connect?device=web-viewer";

    println!("📡 Connecting to WebSocket: {}", ws_url);
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");
    println!("✅ WebSocket connected\n");

    let (mut write, mut read) = ws_stream.split();

    let agent = create_test_agent();
    let chat_request = json!({
        "type": "chat_request",
        "message": "Click the left mouse button",
        "agent": {
            "systemPrompt": agent.system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 3,
            "tools": ["mouse_move", "mouse_click", "keyboard_input", "get_mouse_position"]
        }
    });

    println!("📤 Sending chat request: \"Click the left mouse button\"");
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

                // Debug output
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
                            if let Some(observation) = step.get("observation") {
                                println!("   👁️  Result: {}", observation["result"]);
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
        println!("\n❌ Test timed out after 30 seconds");
        println!("📊 Received {} messages before timeout:", responses.len());
        for (i, resp) in responses.iter().enumerate() {
            println!("   {}. Type: {}", i + 1, resp["type"]);
        }
        println!("\n💡 Troubleshooting:");
        println!("   1. Is the desktop app running? (cargo run)");
        println!("   2. Is the CF worker running? (cd cf-worker && wrangler dev)");
        println!("   3. Check logs in both terminals");
        panic!("Test timed out - no final response received");
    }

    println!("\n📊 Test Results:");
    println!("   Total messages received: {}", responses.len());

    // Find mouse_click action in execution_step messages
    let mut found_click = false;
    for resp in &responses {
        if resp["type"] == "execution_step" {
            if let Some(step) = resp.get("step") {
                if let Some(action) = step.get("action") {
                    if action["tool"] == "mouse_click" {
                        found_click = true;

                        let params = action
                            .get("parameters")
                            .or_else(|| action.get("input"))
                            .and_then(|v| v.as_object());

                        if let Some(params) = params {
                            if let Some(button) = params.get("button").and_then(|v| v.as_str()) {
                                assert_eq!(button, "left", "Should click left button");
                                println!("✅ Mouse click test passed!");
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    assert!(found_click, "Should have called mouse_click tool");
}

#[tokio::test]
#[ignore]
async fn test_desktop_agent_keyboard_input() {
    println!("\n🧪 Testing desktop agent keyboard input...");
    println!("⚠️  Requirements:");
    println!("   1. Cloudflare Worker running: cd cf-worker && wrangler dev");
    println!("   2. Desktop App running: cargo run");
    println!();

    let ws_url = "ws://localhost:8787/connect?device=web-viewer";

    println!("📡 Connecting to WebSocket: {}", ws_url);
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");
    println!("✅ WebSocket connected\n");

    let (mut write, mut read) = ws_stream.split();

    let agent = create_test_agent();
    let chat_request = json!({
        "type": "chat_request",
        "message": "Type 'hello world'",
        "agent": {
            "systemPrompt": agent.system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 3,
            "tools": ["mouse_move", "mouse_click", "keyboard_input", "get_mouse_position"]
        }
    });

    println!("📤 Sending chat request: \"Type 'hello world'\"");
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

                // Debug output
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
                            if let Some(observation) = step.get("observation") {
                                println!("   👁️  Result: {}", observation["result"]);
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
        println!("\n❌ Test timed out after 30 seconds");
        println!("📊 Received {} messages before timeout:", responses.len());
        for (i, resp) in responses.iter().enumerate() {
            println!("   {}. Type: {}", i + 1, resp["type"]);
        }
        println!("\n💡 Troubleshooting:");
        println!("   1. Is the desktop app running? (cargo run)");
        println!("   2. Is the CF worker running? (cd cf-worker && wrangler dev)");
        println!("   3. Check logs in both terminals");
        panic!("Test timed out - no final response received");
    }

    println!("\n📊 Test Results:");
    println!("   Total messages received: {}", responses.len());

    // Find keyboard_input action in execution_step messages
    let mut found_keyboard = false;
    for resp in &responses {
        if resp["type"] == "execution_step" {
            if let Some(step) = resp.get("step") {
                if let Some(action) = step.get("action") {
                    if action["tool"] == "keyboard_input" {
                        found_keyboard = true;

                        let params = action
                            .get("parameters")
                            .or_else(|| action.get("input"))
                            .and_then(|v| v.as_object());

                        if let Some(params) = params {
                            if let Some(text) = params.get("text").and_then(|v| v.as_str()) {
                                assert_eq!(text, "hello world", "Should type exact text");
                                println!("✅ Keyboard input test passed!");
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    assert!(found_keyboard, "Should have called keyboard_input tool");
}

#[tokio::test]
#[ignore]
async fn test_desktop_agent_no_repetition() {
    println!("\n🧪 Testing agent doesn't repeat successful actions...");
    println!("⚠️  Requirements:");
    println!("   1. Cloudflare Worker running: cd cf-worker && wrangler dev");
    println!("   2. Desktop App running: cargo run");
    println!();

    let ws_url = "ws://localhost:8787/connect?device=web-viewer";

    println!("📡 Connecting to WebSocket: {}", ws_url);
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");
    println!("✅ WebSocket connected\n");

    let (mut write, mut read) = ws_stream.split();

    let agent = create_test_agent();
    let chat_request = json!({
        "type": "chat_request",
        "message": "Move mouse to 100, 100",
        "agent": {
            "systemPrompt": agent.system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 3,
            "tools": ["mouse_move", "mouse_click", "keyboard_input", "get_mouse_position"]
        }
    });

    println!("📤 Sending chat request: \"Move mouse to 100, 100\"");
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

                // Debug output
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
                            if let Some(observation) = step.get("observation") {
                                println!("   👁️  Result: {}", observation["result"]);
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
        println!("\n❌ Test timed out after 30 seconds");
        println!("📊 Received {} messages before timeout:", responses.len());
        for (i, resp) in responses.iter().enumerate() {
            println!("   {}. Type: {}", i + 1, resp["type"]);
        }
        println!("\n💡 Troubleshooting:");
        println!("   1. Is the desktop app running? (cargo run)");
        println!("   2. Is the CF worker running? (cd cf-worker && wrangler dev)");
        println!("   3. Check logs in both terminals");
        panic!("Test timed out - no final response received");
    }

    println!("\n📊 Test Results:");
    println!("   Total messages received: {}", responses.len());

    // Count mouse_move calls
    let move_count = responses
        .iter()
        .filter(|r| {
            r["type"] == "execution_step"
                && r.get("step")
                    .and_then(|s| s.get("action"))
                    .and_then(|a| a.get("tool"))
                    .map(|t| t == "mouse_move")
                    .unwrap_or(false)
        })
        .count();

    assert_eq!(
        move_count, 1,
        "Should only call mouse_move once for single instruction, got {} calls",
        move_count
    );

    println!("✅ No repetition test passed!");
}

#[test]
fn test_agent_configuration() {
    println!("\n🧪 Testing agent configuration...");

    let agent = create_test_agent();

    assert_eq!(agent.id, "desktop-automation-agent");
    assert_eq!(agent.name, "Desktop Automation Agent");
    assert_eq!(agent.max_iterations, 3);
    assert_eq!(agent.model_id, "@cf/meta/llama-3.3-70b-instruct-fp8-fast");

    // Check tools are configured
    let tool_ids: Vec<_> = agent.tools.iter().map(|t| t.tool_id.as_str()).collect();
    assert!(
        tool_ids.contains(&"mouse_move"),
        "Should have mouse_move tool"
    );
    assert!(
        tool_ids.contains(&"mouse_click"),
        "Should have mouse_click tool"
    );
    assert!(
        tool_ids.contains(&"keyboard_input"),
        "Should have keyboard_input tool"
    );
    assert!(
        tool_ids.contains(&"get_mouse_position"),
        "Should have get_mouse_position tool"
    );

    // Check prompt contains key phrases
    assert!(
        agent.system_prompt.contains("precise") || agent.system_prompt.contains("exactly"),
        "Prompt should emphasize precision"
    );

    println!("✅ Agent configuration is correct");
    println!("   Agent ID: {}", agent.id);
    println!("   Tools: {}", tool_ids.len());
    println!("   Max iterations: {}", agent.max_iterations);
}
