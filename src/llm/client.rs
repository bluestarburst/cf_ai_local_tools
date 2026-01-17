//! HTTP-based LLM client implementation for Cloudflare Workers AI

use crate::core::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// HTTP client for Cloudflare Workers AI
pub struct HttpClient {
    base_url: String,
    client: Client,
    api_token: Option<String>,
}

#[derive(Debug, Serialize)]
struct LLMRequest {
    model: String,
    messages: Vec<crate::llm::LLMMessage>,
    tools: Option<Vec<crate::llm::LLMTool>>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct LLMResponse {
    response: String,
    tool_calls: Option<Vec<crate::llm::LLMToolCall>>,
    model: String,
    usage: Option<LLMUsage>,
    response_time: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct LLMUsage {
    #[serde(alias = "input_tokens")]
    prompt_tokens: u32,
    #[serde(alias = "output_tokens")]
    completion_tokens: u32,
    total_tokens: u32,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
            api_token: std::env::var("CF_API_TOKEN").ok(),
        }
    }

    /// Create client with API token
    pub fn with_token(base_url: String, api_token: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
            api_token: Some(api_token),
        }
    }

    /// Set API token
    pub fn set_token(&mut self, token: String) {
        self.api_token = Some(token);
    }

    /// Get base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Test connection to LLM service
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::core::AppError::LLM(format!("Health check failed: {}", e)))?;

        Ok(response.status().is_success())
    }

    /// Get available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::core::AppError::LLM(format!("Failed to list models: {}", e)))?;

        if !response.status().is_success() {
            return Err(crate::core::AppError::LLM(format!(
                "API error: {}",
                response.status()
            )));
        }

        let models: Vec<String> = response.json().await.map_err(|e| {
            crate::core::AppError::LLM(format!("Failed to parse models response: {}", e))
        })?;

        Ok(models)
    }

    /// Make the actual HTTP request
    async fn make_request(&self, request: LLMRequest) -> Result<LLMResponse> {
        let url = format!("{}/api/llm", self.base_url);

        let mut req_builder = self.client.post(&url);

        // Add API token if available
        if let Some(token) = &self.api_token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = req_builder
            .json(&request)
            .send()
            .await
            .map_err(|e| crate::core::AppError::LLM(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::core::AppError::LLM(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        // Get raw response text for debugging
        let response_text = response.text().await.map_err(|e| {
            crate::core::AppError::LLM(format!("Failed to read response body: {}", e))
        })?;

        println!("DEBUG: Raw LLM response body: {}", response_text);

        let llm_response: LLMResponse = serde_json::from_str(&response_text)
            .map_err(|e| crate::core::AppError::LLM(format!("Failed to parse response: {}", e)))?;

        Ok(llm_response)
    }

    /// Convert internal messages to HTTP format
    fn convert_messages(messages: &[crate::llm::LLMMessage]) -> Vec<crate::llm::LLMMessage> {
        messages.to_vec()
    }

    /// Convert internal tools to HTTP format
    fn convert_tools(tools: &[crate::llm::LLMTool]) -> Vec<crate::llm::LLMTool> {
        tools.to_vec()
    }
}

#[async_trait]
impl crate::llm::LLMClient for HttpClient {
    async fn chat(
        &self,
        messages: &[crate::llm::LLMMessage],
        model_id: &str,
    ) -> Result<crate::llm::LLMResponse> {
        let request = LLMRequest {
            model: model_id.to_string(),
            messages: Self::convert_messages(messages),
            tools: None,
            max_tokens: Some(4096),
            temperature: Some(0.7),
            stream: Some(false),
        };

        let start_time = std::time::Instant::now();
        let response = self.make_request(request).await?;
        let response_time = start_time.elapsed();

        Ok(crate::llm::LLMResponse {
            response: response.response,
            tool_calls: response.tool_calls,
            model: response.model,
            usage: response.usage.map(|u| crate::llm::LLMUsage {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
            response_time,
        })
    }

    async fn chat_with_tools(
        &self,
        messages: &[crate::llm::LLMMessage],
        model_id: &str,
        tools: Option<Vec<crate::llm::LLMTool>>,
    ) -> Result<crate::llm::LLMResponse> {
        let request = LLMRequest {
            model: model_id.to_string(),
            messages: Self::convert_messages(messages),
            tools: tools.map(|t| Self::convert_tools(&t)),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            stream: Some(false),
        };

        let start_time = std::time::Instant::now();
        let response = self.make_request(request).await?;
        let response_time = start_time.elapsed();

        Ok(crate::llm::LLMResponse {
            response: response.response,
            tool_calls: response.tool_calls,
            model: response.model,
            usage: response.usage.map(|u| crate::llm::LLMUsage {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
            response_time,
        })
    }
}

/// Mock LLM client for testing
pub struct MockLLMClient {
    responses: std::collections::VecDeque<String>,
    tool_calls: std::collections::VecDeque<Option<Vec<crate::llm::LLMToolCall>>>,
}

impl MockLLMClient {
    pub fn new() -> Self {
        Self {
            responses: std::collections::VecDeque::new(),
            tool_calls: std::collections::VecDeque::new(),
        }
    }

    pub fn add_response(&mut self, response: String) {
        self.responses.push_back(response);
        self.tool_calls.push_back(None);
    }

    pub fn add_tool_response(
        &mut self,
        response: String,
        tool_calls: Vec<crate::llm::LLMToolCall>,
    ) {
        self.responses.push_back(response);
        self.tool_calls.push_back(Some(tool_calls));
    }
}

#[async_trait]
impl crate::llm::LLMClient for MockLLMClient {
    async fn chat(
        &self,
        _messages: &[crate::llm::LLMMessage],
        _model_id: &str,
    ) -> Result<crate::llm::LLMResponse> {
        let response = self
            .responses
            .front()
            .cloned()
            .unwrap_or_else(|| "Mock response".to_string());

        Ok(crate::llm::LLMResponse {
            response,
            tool_calls: None,
            model: "mock-model".to_string(),
            usage: Some(crate::llm::LLMUsage {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
            }),
            response_time: std::time::Duration::from_millis(100),
        })
    }

    async fn chat_with_tools(
        &self,
        _messages: &[crate::llm::LLMMessage],
        _model_id: &str,
        _tools: Option<Vec<crate::llm::LLMTool>>,
    ) -> Result<crate::llm::LLMResponse> {
        let response = self
            .responses
            .front()
            .cloned()
            .unwrap_or_else(|| "Mock tool response".to_string());

        let tool_calls = self.tool_calls.front().cloned().flatten();

        Ok(crate::llm::LLMResponse {
            response,
            tool_calls,
            model: "mock-model".to_string(),
            usage: Some(crate::llm::LLMUsage {
                input_tokens: 15,
                output_tokens: 25,
                total_tokens: 40,
            }),
            response_time: std::time::Duration::from_millis(150),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::LLMClient;

    #[tokio::test]
    async fn test_http_client_creation() {
        let client = HttpClient::new("http://localhost:8787".to_string());
        assert_eq!(client.base_url(), "http://localhost:8787");
    }

    #[tokio::test]
    async fn test_mock_client() {
        let mut mock = MockLLMClient::new();
        mock.add_response("Hello from mock!".to_string());

        let messages = vec![crate::llm::LLMMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tool_calls: None,
        }];

        let response = mock.chat(&messages, "test-model").await.unwrap();
        assert_eq!(response.response, "Hello from mock!");
        assert_eq!(response.model, "mock-model");
        assert!(response.usage.is_some());
    }

    #[tokio::test]
    async fn test_mock_client_with_tools() {
        let mut mock = MockLLMClient::new();
        mock.add_tool_response(
            "I'll use the tool".to_string(),
            vec![crate::llm::LLMToolCall {
                name: "test_tool".to_string(),
                arguments: serde_json::json!({"param": "value"}),
                id: Some("call_1".to_string()),
            }],
        );

        let messages = vec![crate::llm::LLMMessage {
            role: "user".to_string(),
            content: "Use a tool".to_string(),
            tool_calls: None,
        }];

        let tools = vec![crate::llm::LLMTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "param": {"type": "string"}
                }
            }),
        }];

        let response = mock
            .chat_with_tools(&messages, "test-model", Some(tools))
            .await
            .unwrap();
        assert_eq!(response.response, "I'll use the tool");
        assert!(response.tool_calls.is_some());
        assert_eq!(response.tool_calls.as_ref().unwrap().len(), 1);
    }
}
