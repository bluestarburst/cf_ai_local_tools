use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
struct LLMRequest {
    messages: Vec<Message>,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LLMResponse {
    pub response: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    #[allow(dead_code)]
    pub usage: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

pub struct LLMClient {
    client: Client,
    endpoint: String,
}

impl LLMClient {
    pub fn new(worker_url: &str) -> Self {
        Self {
            client: Client::new(),
            endpoint: format!("{}/api/llm", worker_url),
        }
    }

    /// Call LLM with conversation history and optional tools
    /// Returns full response including tool_calls if present
    pub async fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        model: &str,
        tools: Option<Vec<Value>>,
    ) -> Result<LLMResponse> {
        let request = LLMRequest {
            messages,
            model: model.to_string(),
            tools,
        };

        let resp = self
            .client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            anyhow::bail!("LLM API error: {}", error_text);
        }

        let llm_response = resp.json::<LLMResponse>().await?;
        Ok(llm_response)
    }

    /// Call LLM without tools (backward compatible)
    #[allow(dead_code)]
    pub async fn chat(&self, messages: Vec<Message>, model: &str) -> Result<String> {
        let response = self.chat_with_tools(messages, model, None).await?;
        Ok(response.response)
    }

    /// Helper: Create a message
    #[allow(dead_code)]
    pub fn message(role: &str, content: &str) -> Message {
        Message {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}
