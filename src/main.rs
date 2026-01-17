//! Enhanced Local Rust App - Main Application
//!
//! A modular, dynamic agent system with plug-and-play tools and thinking capabilities.
//! This application runs as a backend client, connecting to a Cloudflare Worker relay
//! to receive instructions from the frontend and execute agents locally.

use cf_ai_local_tools::llm::client::HttpClient;
use cf_ai_local_tools::registry::CentralRegistry;
use cf_ai_local_tools::websocket::WebSocketRelayClient;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🚀 Starting Enhanced Local Rust App...");

    // Initialize components
    println!("📦 Initializing components...");

    // Create central registry
    let mut registry = CentralRegistry::new();
    registry.initialize().await?;
    let registry = Arc::new(registry);

    // Create LLM client
    // We configure it to point to the Worker's LLM proxy endpoint
    let llm = Arc::new(HttpClient::new("http://localhost:8787".to_string()));

    // Create WebSocket Client
    // Connects to the Worker relay as the 'desktop' device
    let ws_url = "ws://localhost:8787/connect?device=desktop";
    let client = WebSocketRelayClient::new(ws_url.to_string(), registry.clone(), llm.clone());

    println!("🌐 Connecting to relay at {}...", ws_url);

    // Run the client loop
    if let Err(e) = client.run().await {
        eprintln!("❌ Application error: {}", e);
    }

    println!("👋 Enhanced Local Rust App shutting down...");
    Ok(())
}
