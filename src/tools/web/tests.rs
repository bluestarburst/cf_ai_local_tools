use super::*;
use crate::core::mock::MockToolContext;

#[tokio::test]
async fn test_web_search_tool() {
    let tool = WebSearch::new();
    let context = MockToolContext::new();

    let args = serde_json::json!({
        "query": "test query",
        "max_results": 5,
        "include_content": false
    });

    let result = tool.execute(&args, &context).await;
    assert!(result.is_ok());

    let tool_result = result.unwrap();
    assert!(tool_result.success);
    assert!(tool_result.message.contains("test query"));

    if let Some(data) = tool_result.data {
        assert_eq!(data["query"], "test query");
        assert!(data["results"].as_array().unwrap().len() <= 5);
    }
}

#[tokio::test]
async fn test_web_search_validation() {
    let tool = WebSearch::new();

    // Valid arguments
    let valid_args = serde_json::json!({
        "query": "valid query"
    });
    assert!(tool.validate_args(&valid_args).is_ok());

    // Invalid arguments (missing required field)
    let invalid_args = serde_json::json!({
        "max_results": 10
    });
    assert!(tool.validate_args(&invalid_args).is_err());

    // Empty query
    let empty_query_args = serde_json::json!({
        "query": ""
    });
    assert!(tool.validate_args(&empty_query_args).is_ok()); // JSON validation passes
}

#[tokio::test]
async fn test_fetch_url_tool() {
    let tool = FetchUrl::new();
    let context = MockToolContext::new();

    let args = serde_json::json!({
        "url": "https://example.com",
        "include_html": false,
        "max_content_length": 1000
    });

    let result = tool.execute(&args, &context).await;
    assert!(result.is_ok());

    let tool_result = result.unwrap();
    assert!(tool_result.success);
    assert!(tool_result.message.contains("https://example.com"));

    if let Some(data) = tool_result.data {
        assert_eq!(data["url"], "https://example.com");
        assert!(data["title"]
            .as_str()
            .unwrap()
            .contains("https://example.com"));
        assert!(data["content"].as_str().unwrap().len() <= 1000);
    }
}

#[tokio::test]
async fn test_fetch_url_validation() {
    let tool = FetchUrl::new();

    // Valid arguments
    let valid_args = serde_json::json!({
        "url": "https://example.com"
    });
    assert!(tool.validate_args(&valid_args).is_ok());

    // Invalid arguments (missing required field)
    let invalid_args = serde_json::json!({
        "include_html": true
    });
    assert!(tool.validate_args(&invalid_args).is_err());

    // Invalid URL
    let invalid_url_args = serde_json::json!({
        "url": "not-a-url"
    });
    assert!(tool.validate_args(&invalid_url_args).is_ok()); // JSON validation passes
}

#[tokio::test]
async fn test_fetch_url_with_html() {
    let tool = FetchUrl::new();
    let context = MockToolContext::new();

    let args = serde_json::json!({
        "url": "https://example.com",
        "include_html": true
    });

    let result = tool.execute(&args, &context).await.unwrap();

    if let Some(data) = result.data {
        assert!(data["html"].is_string());
        assert!(data["html"].as_str().unwrap().contains("<html>"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_web_workflow() {
        let context = MockToolContext::new();

        // 1. Search for information
        let search_tool = WebSearch::new();
        let search_args = serde_json::json!({
            "query": "artificial intelligence",
            "max_results": 3
        });
        let search_result = search_tool.execute(&search_args, &context).await.unwrap();
        assert!(search_result.success);

        // 2. Fetch content from a URL
        let fetch_tool = FetchUrl::new();
        let fetch_args = serde_json::json!({
            "url": "https://example.com/ai-info"
        });
        let fetch_result = fetch_tool.execute(&fetch_args, &context).await.unwrap();
        assert!(fetch_result.success);
    }
}
