use crate::core::{Tool, ToolContext, ToolParameter, ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchUrl {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchUrlArgs {
    pub url: String,
    pub include_html: Option<bool>,
    pub max_content_length: Option<u32>,
    pub timeout_seconds: Option<u32>,
}

impl FetchUrl {
    pub fn new() -> Self {
        Self {
            id: "fetch_url".to_string(),
            name: "Fetch URL".to_string(),
            description: "Fetch content from a specific URL".to_string(),
            category: "web".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "url".to_string(),
                    param_type: "string".to_string(),
                    description: "URL to fetch content from".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                ToolParameter {
                    name: "include_html".to_string(),
                    param_type: "boolean".to_string(),
                    description: "Include raw HTML in response (default: false)".to_string(),
                    required: false,
                    default: Some(serde_json::json!(false)),
                    enum_values: None,
                },
                ToolParameter {
                    name: "max_content_length".to_string(),
                    param_type: "number".to_string(),
                    description: "Maximum content length to return in characters (default: 5000)"
                        .to_string(),
                    required: false,
                    default: Some(serde_json::json!(5000)),
                    enum_values: None,
                },
                ToolParameter {
                    name: "timeout_seconds".to_string(),
                    param_type: "number".to_string(),
                    description: "Request timeout in seconds (default: 30)".to_string(),
                    required: false,
                    default: Some(serde_json::json!(30)),
                    enum_values: None,
                },
            ],
        }
    }
}

#[async_trait::async_trait]
impl Tool for FetchUrl {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn category(&self) -> &str {
        &self.category
    }

    fn parameters(&self) -> &[ToolParameter] {
        &self.parameters
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        context: &ToolContext,
    ) -> crate::core::Result<ToolResult> {
        let args: FetchUrlArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;

        // Validate URL
        if !self.is_valid_url(&args.url) {
            return Err(crate::core::AppError::Tool(format!(
                "Invalid URL format: {}",
                args.url
            )));
        }

        let include_html = args.include_html.unwrap_or(false);
        let max_content_length = args.max_content_length.unwrap_or(5000) as usize;
        let timeout_seconds = args.timeout_seconds.unwrap_or(30);

        // Send progress update
        if let Some(ref manager) = context.conversation_manager {
            manager
                .send_progress_update(
                    &context.agent_id,
                    crate::agents::conversation::ProgressType::Executing,
                    &format!("Fetching content from: {}", args.url),
                    Some(0.5),
                )
                .await?;
        }

        // Execute URL fetch (placeholder - would use actual HTTP client)
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        // Mock fetch result
        let mock_content = self.generate_mock_content(&args.url, max_content_length, include_html);

        let result = ToolResult {
            success: true,
            message: format!("Successfully fetched content from: {}", args.url),
            data: Some(mock_content),
            execution_time: std::time::Duration::from_millis(300),
        };

        Ok(result)
    }

    fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        let _args: FetchUrlArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;
        Ok(())
    }
}

impl FetchUrl {
    /// Basic URL validation
    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Generate mock content for testing
    fn generate_mock_content(
        &self,
        url: &str,
        max_length: usize,
        include_html: bool,
    ) -> serde_json::Value {
        let title = format!("Page Title for {}", url);
        let description = format!("This is sample content fetched from {}. It contains information about the URL and demonstrates the fetch functionality.", url);

        let content = if description.len() > max_length {
            description.chars().take(max_length).collect::<String>() + "..."
        } else {
            description
        };

        let mut result = serde_json::json!({
            "url": url,
            "title": title,
            "content": content,
            "content_length": content.len(),
            "status_code": 200,
            "headers": {
                "content-type": "text/html",
                "server": "mock-server"
            }
        });

        if include_html {
            result["html"] = serde_json::json!(format!(
                "<html><head><title>{}</title></head><body><h1>{}</h1><p>{}</p></body></html>",
                title, title, content
            ));
        }

        result
    }
}
