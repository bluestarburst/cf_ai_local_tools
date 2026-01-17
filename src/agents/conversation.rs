//! Conversation management for real-time agent communication

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Progress types for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressType {
    /// Agent is thinking
    Thinking,
    /// Agent is planning
    Planning,
    /// Agent is executing a tool
    Executing,
    /// Agent is observing results
    Observing,
    /// Agent is reflecting
    Reflecting,
    /// Task is completing
    Completing,
}

/// Trait for conversation managers that handle real-time updates
#[async_trait]
pub trait ConversationManager: std::fmt::Debug + Send + Sync {
    /// Send thinking update
    async fn send_thinking_update(
        &self,
        agent_id: &str,
        step_number: usize,
        thought: &str,
    ) -> crate::core::Result<()>;

    /// Send progress update
    async fn send_progress_update(
        &self,
        agent_id: &str,
        progress_type: ProgressType,
        message: &str,
        percentage: Option<f32>,
    ) -> crate::core::Result<()>;

    /// Send error update
    async fn send_error_update(
        &self,
        agent_id: &str,
        error: &str,
        recovery_suggestions: Vec<String>,
    ) -> crate::core::Result<()>;

    /// Send completion update
    async fn send_completion_update(
        &self,
        agent_id: &str,
        final_response: &str,
        success: bool,
    ) -> crate::core::Result<()>;
}
