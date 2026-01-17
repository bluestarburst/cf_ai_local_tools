use crate::core::{Tool, ToolContext, ToolParameter, ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearch {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchArgs {
    pub query: String,
    pub max_results: Option<u32>,
    pub include_content: Option<bool>,
}

impl WebSearch {
    pub fn new() -> Self {
        Self {
            id: "web_search".to_string(),
            name: "Web Search".to_string(),
            description: "Search the web for information and return relevant results".to_string(),
            category: "web".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "query".to_string(),
                    param_type: "string".to_string(),
                    description: "Search query to execute".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                ToolParameter {
                    name: "max_results".to_string(),
                    param_type: "number".to_string(),
                    description: "Maximum number of results to return (default: 10)".to_string(),
                    required: false,
                    default: Some(serde_json::json!(10)),
                    enum_values: None,
                },
                ToolParameter {
                    name: "include_content".to_string(),
                    param_type: "boolean".to_string(),
                    description: "Include full content snippets (default: false)".to_string(),
                    required: false,
                    default: Some(serde_json::json!(false)),
                    enum_values: None,
                },
            ],
        }
    }
}

#[async_trait::async_trait]
impl Tool for WebSearch {
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
        let args: WebSearchArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;

        if args.query.trim().is_empty() {
            return Err(crate::core::AppError::Tool(
                "Search query cannot be empty".to_string(),
            ));
        }

        let max_results = args.max_results.unwrap_or(10).min(20); // Cap at 20
        let include_content = args.include_content.unwrap_or(false);

        // Send progress update
        if let Some(ref manager) = context.conversation_manager {
            manager
                .send_progress_update(
                    &context.agent_id,
                    crate::agents::conversation::ProgressType::Executing,
                    &format!("Searching web for: '{}'", args.query),
                    Some(0.5),
                )
                .await?;
        }

        // Execute real web search using websearch crate
        let start = std::time::Instant::now();

        let provider = websearch::providers::DuckDuckGoProvider::new();
        let options = websearch::SearchOptions {
            query: args.query.clone(),
            max_results: Some(max_results),
            provider: Box::new(provider),
            ..Default::default()
        };

        // Execute search with timeout
        let timeout_duration = std::time::Duration::from_secs(15);
        let search_result =
            tokio::time::timeout(timeout_duration, websearch::web_search(options)).await;

        let elapsed = start.elapsed();

        let result = match search_result {
            Ok(Ok(results)) => {
                let formatted: Vec<serde_json::Value> = results
                    .iter()
                    .take(max_results as usize)
                    .map(|r| {
                        serde_json::json!({
                            "title": r.title,
                            "url": r.url,
                            "snippet": r.snippet,
                            "domain": r.domain,
                        })
                    })
                    .collect();

                ToolResult {
                    success: true,
                    message: format!(
                        "Found {} results for query: '{}'",
                        formatted.len(),
                        args.query
                    ),
                    data: Some(serde_json::json!({
                        "status": "success",
                        "query": args.query,
                        "total_results": formatted.len(),
                        "results": formatted
                    })),
                    execution_time: elapsed,
                }
            }
            Ok(Err(e)) => ToolResult {
                success: false,
                message: format!("Search failed: {}", e),
                data: Some(serde_json::json!({
                    "status": "error",
                    "query": args.query,
                    "error": e.to_string()
                })),
                execution_time: elapsed,
            },
            Err(_) => ToolResult {
                success: false,
                message: "Search timed out after 15 seconds".to_string(),
                data: Some(serde_json::json!({
                    "status": "timeout",
                    "query": args.query,
                    "error": "Request timed out"
                })),
                execution_time: elapsed,
            },
        };

        Ok(result)
    }

    fn validate_args(&self, args: &serde_json::Value) -> crate::core::Result<()> {
        let _args: WebSearchArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::core::AppError::Tool(format!("Invalid arguments: {}", e)))?;
        Ok(())
    }
}

impl WebSearch {
    /// Generate mock search results for testing
    fn generate_mock_results(
        &self,
        query: &str,
        max_results: u32,
        include_content: bool,
    ) -> Vec<serde_json::Value> {
        let mut results = Vec::new();

        for i in 1..=max_results.min(10) {
            let mut result = serde_json::json!({
                "title": format!("Result {} for '{}'", i, query),
                "url": format!("https://example.com/result{}", i),
                "snippet": format!("This is a sample snippet for search result {} about {}.", i, query),
                "rank": i
            });

            if include_content {
                result["content"] =
                    serde_json::json!(format!("Full content for result {}: {}", i, query));
            }

            results.push(result);
        }

        results
    }
}
