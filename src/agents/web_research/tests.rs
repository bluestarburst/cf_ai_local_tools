// Integration tests for Web Research Agent

use crate::agents::presets::Metadata;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// Helper function to create a test agent
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
async fn test_web_research_search_query() {
    // Test that web research agent uses web_search with proper parameters
    let ws_url = "ws://localhost:8787/connect";

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    let chat_request = json!({
        "type": "chat_request",
        "message": "Search for latest AI news",
        "agent": {
            "systemPrompt": create_test_agent().system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 8,
            "tools": ["web_search", "fetch_url"]
        }
    });

    write
        .send(Message::Text(chat_request.to_string()))
        .await
        .expect("Failed to send message");

    let mut responses = Vec::new();
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
            responses.push(parsed.clone());

            if parsed["type"] == "final_response" {
                break;
            }
        }
    }

    // Find web_search tool call
    let tool_call = responses
        .iter()
        .find(|r| r["type"] == "tool_call" && r["tool_name"] == "web_search")
        .expect("Should have called web_search");

    // Verify query parameter exists and is a string
    let args = &tool_call["arguments"];
    assert!(args["query"].is_string(), "query should be a string");

    let query = args["query"].as_str().unwrap().to_lowercase();
    assert!(
        query.contains("ai") || query.contains("artificial intelligence"),
        "Query should be relevant to AI: {}",
        query
    );

    // Verify optional parameters are either valid or omitted
    if let Some(time_range) = args["time_range"].as_str() {
        assert!(
            ["day", "week", "month", "year"].contains(&time_range),
            "time_range should be valid enum value: {}",
            time_range
        );
    }

    if let Some(language) = args["language"].as_str() {
        assert!(!language.is_empty(), "language should not be empty string");
    }
}

#[tokio::test]
#[ignore]
async fn test_web_research_fetch_url() {
    // Test that agent can fetch URL content
    let ws_url = "ws://localhost:8787/connect";

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    let chat_request = json!({
        "type": "chat_request",
        "message": "Get the content from https://example.com",
        "agent": {
            "systemPrompt": create_test_agent().system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 8,
            "tools": ["web_search", "fetch_url"]
        }
    });

    write
        .send(Message::Text(chat_request.to_string()))
        .await
        .expect("Failed to send message");

    let mut responses = Vec::new();
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
            responses.push(parsed.clone());

            if parsed["type"] == "final_response" {
                break;
            }
        }
    }

    // Find fetch_url tool call
    let tool_call = responses
        .iter()
        .find(|r| r["type"] == "tool_call" && r["tool_name"] == "fetch_url")
        .expect("Should have called fetch_url");

    // Verify URL parameter
    let args = &tool_call["arguments"];
    let url = args["url"].as_str().unwrap();
    assert!(url.contains("example.com"), "URL should match request");

    // Verify extract_type if provided
    if let Some(extract_type) = args["extract_type"].as_str() {
        assert!(
            ["text", "links", "images", "all"].contains(&extract_type),
            "extract_type should be valid enum: {}",
            extract_type
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_web_research_no_empty_optional_params() {
    // Test that agent doesn't send empty strings for optional parameters
    let ws_url = "ws://localhost:8787/connect";

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    let chat_request = json!({
        "type": "chat_request",
        "message": "Search for rust programming language",
        "agent": {
            "systemPrompt": create_test_agent().system_prompt,
            "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            "maxIterations": 8,
            "tools": ["web_search", "fetch_url"]
        }
    });

    write
        .send(Message::Text(chat_request.to_string()))
        .await
        .expect("Failed to send message");

    let mut responses = Vec::new();
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
            responses.push(parsed.clone());

            if parsed["type"] == "final_response" {
                break;
            }
        }
    }

    // Find all web_search calls
    let search_calls: Vec<_> = responses
        .iter()
        .filter(|r| r["type"] == "tool_call" && r["tool_name"] == "web_search")
        .collect();

    for call in search_calls {
        let args = &call["arguments"];

        // Check time_range if present
        if let Some(time_range) = args.get("time_range") {
            if let Some(tr_str) = time_range.as_str() {
                assert!(!tr_str.is_empty(), "time_range should not be empty string");
            }
        }

        // Check language if present
        if let Some(language) = args.get("language") {
            if let Some(lang_str) = language.as_str() {
                assert!(!lang_str.is_empty(), "language should not be empty string");
            }
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_web_research_retry_on_error() {
    // Test that agent retries with corrected parameters after validation error
    // This test would need to simulate a validation error response
    // For now, we verify the agent has retry logic in its prompt

    let agent = create_test_agent();
    let prompt = &agent.system_prompt;
    assert!(
        prompt.contains("retry") && prompt.contains("corrected parameters"),
        "Agent prompt should include retry logic"
    );
    assert!(
        prompt.contains("Don't give up after one error"),
        "Agent should be instructed to persist"
    );
}
