//! Error handling types for the enhanced local Rust app

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("LLM error: {0}")]
    LLM(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
