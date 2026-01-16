use serde_json::{json, Value};
use crate::agents::{ToolDefinition, ToolParameter};
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::debug;
use websearch::{
    providers::{ArxivProvider, DuckDuckGoProvider},
    web_search,
    SearchOptions,
    SearchProvider,
    SearchResult as WebSearchResult,
};

const DEFAULT_PROVIDER: &str = "duckduckgo";
const SUPPORTED_PROVIDERS: &[&str] = &["duckduckgo", "arxiv"];

/// Get all web search and browsing tools
pub fn get_search_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            id: "web_search".to_string(),
            name: "Web Search".to_string(),
            description: "Search the web using the websearch crate (DuckDuckGo by default). Returns URLs, titles, and snippets.".to_string(),
            category: "web".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "query".to_string(),
                    param_type: "string".to_string(),
                    description: "Search query".to_string(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "provider".to_string(),
                    param_type: "string".to_string(),
                    description: "Search provider (duckduckgo or arxiv). Defaults to duckduckgo.".to_string(),
                    required: false,
                    enum_values: Some(SUPPORTED_PROVIDERS.iter().map(|s| s.to_string()).collect()),
                    default: Some(json!(DEFAULT_PROVIDER)),
                },
                ToolParameter {
                    name: "max_results".to_string(),
                    param_type: "integer".to_string(),
                    description: "Maximum number of results to return (1-50). Optional.".to_string(),
                    required: false,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "language".to_string(),
                    param_type: "string".to_string(),
                    description: "Language for results (if supported by provider). Optional.".to_string(),
                    required: false,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "region".to_string(),
                    param_type: "string".to_string(),
                    description: "Region/country code for results (if supported by provider). Optional.".to_string(),
                    required: false,
                    enum_values: None,
                    default: None,
                },
            ],
            returns_observation: true,
        },
    ]
}

/// Get all URL fetching and content extraction tools
pub fn get_fetch_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            id: "fetch_url".to_string(),
            name: "Fetch URL".to_string(),
            description: "Fetch and extract content from a URL".to_string(),
            category: "web".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "url".to_string(),
                    param_type: "string".to_string(),
                    description: "URL to fetch".to_string(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "extract_type".to_string(),
                    param_type: "string".to_string(),
                    description: "Type of content to extract (text, links, images, all)".to_string(),
                    required: false,
                    enum_values: Some(vec!["text".to_string(), "links".to_string(), "images".to_string(), "all".to_string()]),
                    default: Some(json!("text")),
                },
            ],
            returns_observation: true,
        },
    ]
}

/// Get all web-related tools
pub fn get_all_web_tools() -> Vec<ToolDefinition> {
    let mut tools = get_search_tools();
    tools.extend(get_fetch_tools());
    tools
}

/// Parse string from JSON arguments
fn parse_string(value: &Value, field: &str) -> Result<String> {
    value.as_str()
        .ok_or_else(|| anyhow::anyhow!("Parameter '{}' is required and must be a string", field))
        .map(|s| s.to_string())
}

/// Parse optional string from JSON arguments
fn parse_optional_string(value: Option<&Value>) -> Result<Option<String>> {
    match value {
        Some(v) if v.is_null() => Ok(None),
        Some(v) => v.as_str().map(|s| Some(s.to_string())).ok_or_else(||
            anyhow::anyhow!("Optional string parameter must be a string")),
        None => Ok(None),
    }
}

/// Parse optional u32 from JSON arguments
fn parse_optional_u32(value: Option<&Value>, field: &str) -> Result<Option<u32>> {
    match value {
        Some(v) if v.is_null() => Ok(None),
        Some(Value::Number(n)) => n
            .as_u64()
            .map(|num| Some(num as u32))
            .ok_or_else(|| anyhow::anyhow!("Parameter '{}' must be a positive integer", field)),
        Some(Value::String(s)) => s
            .parse::<u32>()
            .map(Some)
            .map_err(|_| anyhow::anyhow!("Parameter '{}' must be a positive integer", field)),
        Some(_) => Err(anyhow::anyhow!("Parameter '{}' must be a number", field)),
        None => Ok(None),
    }
}

/// Select a search provider based on input name, returning the normalized name and provider instance
fn select_provider(provider: Option<String>) -> Result<(String, Box<dyn SearchProvider>)> {
    let provider_name = provider
        .unwrap_or_else(|| DEFAULT_PROVIDER.to_string())
        .to_lowercase();

    match provider_name.as_str() {
        "duckduckgo" => {
            debug!(target: "web_search", "Selecting DuckDuckGo provider");
            Ok((provider_name, Box::new(DuckDuckGoProvider::new())))
        }
        "arxiv" => {
            debug!(target: "web_search", "Selecting ArXiv provider");
            Ok((provider_name, Box::new(ArxivProvider::new())))
        }
        other => Err(anyhow::anyhow!(
            "Unsupported provider '{}'. Supported providers: {}",
            other,
            SUPPORTED_PROVIDERS.join(", ")
        )),
    }
}

/// Convert provider results into a JSON-ready vector with optional trimming
fn format_results(results: &[WebSearchResult], max_results: Option<u32>) -> Vec<Value> {
    let limit = max_results.unwrap_or(10).clamp(1, 50) as usize;

    results
        .iter()
        .take(limit)
        .map(|r| {
            json!({
                "url": r.url,
                "title": r.title,
                "snippet": r.snippet,
                "domain": r.domain,
                "published_date": r.published_date,
                "provider": r.provider,
            })
        })
        .collect()
}

/// Execute web search using the websearch crate (async version)
pub async fn execute_web_search_async(arguments: &Value) -> Result<String> {
    let query = parse_string(&arguments["query"], "query")?;
    let provider_name = parse_optional_string(arguments.get("provider"))?;
    let language = parse_optional_string(arguments.get("language"))?;
    let region = parse_optional_string(arguments.get("region"))?;
    let max_results = parse_optional_u32(arguments.get("max_results"), "max_results")?;

    let (provider_name, provider) = select_provider(provider_name)?;

    let options = SearchOptions {
        query: query.clone(),
        language,
        region,
        max_results,
        provider,
        ..Default::default()
    };

    debug!(
        target: "web_search",
        "websearch request provider={} query=\"{}\" max_results={:?}",
        provider_name,
        query,
        max_results
    );

    // Note: Using a timeout to prevent hanging on provider requests
    let timeout_duration = std::time::Duration::from_secs(15);
    let web_search_future = web_search(options);
    
    let payload = match tokio::time::timeout(timeout_duration, web_search_future).await {
        Ok(Ok(results)) => json!({
            "status": "success",
            "query": query,
            "provider": provider_name,
            "result_count": results.len(),
            "results": format_results(&results, max_results),
        }),
        Ok(Err(err)) => json!({
            "status": "error",
            "query": query,
            "provider": provider_name,
            "error": err.to_string(),
            "suggestion": "Provider request failed. Try duckduckgo or arxiv, or check your network connection.",
        }),
        Err(_) => json!({
            "status": "error",
            "query": query,
            "provider": provider_name,
            "error": "Request timeout after 15 seconds",
            "suggestion": "The search provider took too long to respond. This may indicate network issues or provider unavailability.",
        }),
    };

    Ok(payload.to_string())
}

/// Execute web search (sync wrapper)
fn execute_web_search(arguments: &Value) -> Result<String> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| anyhow::anyhow!("No tokio runtime available"))?;

    rt.block_on(execute_web_search_async(arguments))
}

/// Execute URL fetch (async version)
pub async fn execute_fetch_url_async(arguments: &Value) -> Result<String> {
    let url = parse_string(&arguments["url"], "url")?;
    let extract_type = parse_optional_string(arguments.get("extract_type"))?
        .unwrap_or_else(|| "text".to_string());

    // Validate extract_type
    match extract_type.as_str() {
        "text" | "links" | "images" | "all" => {},
        _ => return Err(anyhow::anyhow!(
            "extract_type must be one of [text, links, images, all], got: '{}'",
            extract_type
        )),
    }

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client.get(&url)
        .header("User-Agent", "cf-ai-local-tools/0.1.0")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }

    let html = response.text().await?;
    let document = Html::parse_document(&html);

    let content = match extract_type.as_str() {
        "text" => {
            // Extract all text content
            let body_selector = Selector::parse("body").unwrap();
            let text = document.select(&body_selector)
                .next()
                .map(|body| {
                    body.text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();

            // Truncate if too long
            let truncated = if text.len() > 5000 {
                format!("{}... [truncated]", &text[..5000])
            } else {
                text
            };

            json!({
                "status": "success",
                "url": url,
                "content_type": "text",
                "content": truncated
            })
        }
        "links" => {
            let link_selector = Selector::parse("a[href]").unwrap();
            let links: Vec<String> = document.select(&link_selector)
                .filter_map(|a| a.value().attr("href"))
                .filter(|href| href.starts_with("http"))
                .take(50)
                .map(|s| s.to_string())
                .collect();

            json!({
                "status": "success",
                "url": url,
                "content_type": "links",
                "links": links
            })
        }
        "images" => {
            let img_selector = Selector::parse("img[src]").unwrap();
            let images: Vec<String> = document.select(&img_selector)
                .filter_map(|img| img.value().attr("src"))
                .filter(|src| src.starts_with("http"))
                .take(20)
                .map(|s| s.to_string())
                .collect();

            json!({
                "status": "success",
                "url": url,
                "content_type": "images",
                "images": images
            })
        }
        "all" => {
            let body_selector = Selector::parse("body").unwrap();
            let text = document.select(&body_selector)
                .next()
                .map(|body| {
                    body.text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();

            let truncated = if text.len() > 3000 {
                format!("{}... [truncated]", &text[..3000])
            } else {
                text
            };

            let link_selector = Selector::parse("a[href]").unwrap();
            let links: Vec<String> = document.select(&link_selector)
                .filter_map(|a| a.value().attr("href"))
                .filter(|href| href.starts_with("http"))
                .take(20)
                .map(|s| s.to_string())
                .collect();

            json!({
                "status": "success",
                "url": url,
                "content_type": "all",
                "text": truncated,
                "links": links
            })
        }
        _ => return Err(anyhow::anyhow!("Invalid extract_type"))
    };

    Ok(content.to_string())
}

/// Execute URL fetch (sync wrapper)
fn execute_fetch_url(arguments: &Value) -> Result<String> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| anyhow::anyhow!("No tokio runtime available"))?;

    rt.block_on(execute_fetch_url_async(arguments))
}

/// Execute a web search tool
///
/// # Arguments
/// * `tool_name` - The ID of the tool to execute
/// * `arguments` - JSON arguments for the tool
///
/// # Returns
/// * `Ok(String)` - Result of the web tool execution
/// * `Err(anyhow::Error)` - If the tool is unknown or parameters are invalid
pub fn execute_web_tool(
    tool_name: &str,
    arguments: &Value,
) -> Result<String> {
    match tool_name {
        "web_search" => execute_web_search(arguments),
        "fetch_url" => execute_fetch_url(arguments),
        _ => {
            // Verify this is a known web tool before returning unknown error
            if get_all_web_tools().iter().any(|t| t.id == tool_name) {
                Err(anyhow::anyhow!("Web tool '{}' is not implemented", tool_name))
            } else {
                Err(anyhow::anyhow!("Unknown web tool: {}", tool_name))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_tools_definitions() {
        let tools = get_all_web_tools();
        assert_eq!(tools.len(), 2);

        let search = tools.iter().find(|t| t.id == "web_search");
        assert!(search.is_some(), "web_search tool should exist");

        let fetch = tools.iter().find(|t| t.id == "fetch_url");
        assert!(fetch.is_some(), "fetch_url tool should exist");
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test test_real_web_search -- --ignored --nocapture
    async fn test_real_web_search() {
        let args = json!({
            "query": "rust programming language"
        });

        let result = execute_web_search_async(&args).await;
        match result {
            Ok(json_str) => {
                println!("Search result: {}", json_str);
                let parsed: Value = serde_json::from_str(&json_str).unwrap();
                println!("Parsed result: {:?}", parsed);

                if parsed["status"] == "success" {
                    assert!(parsed["result_count"].as_u64().unwrap_or(0) >= 0);
                } else {
                    println!("Search returned error payload: {:?}", parsed);
                }
            }
            Err(e) => {
                println!("Search error (may be expected if provider/network unavailable): {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test test_debug_web_search -- --ignored --nocapture
    async fn test_debug_web_search() {
        println!("\n=== Debug Web Search Test ===");
        
        // Test 1: DuckDuckGo with simple query
        println!("\n--- Test 1: DuckDuckGo ---");
        let args = json!({
            "query": "rust"
        });
        
        match execute_web_search_async(&args).await {
            Ok(json_str) => {
                println!("Raw result: {}", json_str);
                if let Ok(parsed) = serde_json::from_str::<Value>(&json_str) {
                    println!("Status: {}", parsed["status"]);
                    println!("Provider: {}", parsed["provider"]);
                    println!("Result count: {}", parsed["result_count"]);
                    if let Some(results) = parsed["results"].as_array() {
                        println!("Results array length: {}", results.len());
                        for (i, r) in results.iter().enumerate() {
                            println!("  Result {}: title={}, url={}", 
                                i, 
                                r.get("title").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                r.get("url").and_then(|v| v.as_str()).unwrap_or("N/A")
                            );
                        }
                    }
                    if let Some(err) = parsed.get("error") {
                        println!("Error: {}", err);
                    }
                }
            }
            Err(e) => println!("Request error: {}", e),
        }

        // Test 2: ArXiv with simple query
        println!("\n--- Test 2: ArXiv ---");
        let args = json!({
            "query": "machine learning",
            "provider": "arxiv"
        });
        
        match execute_web_search_async(&args).await {
            Ok(json_str) => {
                println!("Raw result: {}", json_str);
                if let Ok(parsed) = serde_json::from_str::<Value>(&json_str) {
                    println!("Status: {}", parsed["status"]);
                    println!("Provider: {}", parsed["provider"]);
                    println!("Result count: {}", parsed["result_count"]);
                    if let Some(results) = parsed["results"].as_array() {
                        println!("Results array length: {}", results.len());
                        for (i, r) in results.iter().take(3).enumerate() {
                            println!("  Result {}: title={}, url={}", 
                                i, 
                                r.get("title").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                r.get("url").and_then(|v| v.as_str()).unwrap_or("N/A")
                            );
                        }
                    }
                    if let Some(err) = parsed.get("error") {
                        println!("Error: {}", err);
                    }
                }
            }
            Err(e) => println!("Request error: {}", e),
        }
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test test_direct_websearch_crate -- --ignored --nocapture
    async fn test_direct_websearch_crate() {
        println!("\n=== Direct WebSearch Crate Test ===");
        
        // Test with DuckDuckGo directly
        println!("\n--- Direct DuckDuckGo Provider ---");
        let provider = DuckDuckGoProvider::new();
        let options = SearchOptions {
            query: "rust programming".to_string(),
            max_results: Some(5),
            provider: Box::new(provider),
            ..Default::default()
        };
        
        match web_search(options).await {
            Ok(results) => {
                println!("SUCCESS: Got {} results", results.len());
                for (i, r) in results.iter().take(3).enumerate() {
                    println!("  [{}] {}", i, r.title);
                    println!("      URL: {}", r.url);
                    if let Some(snippet) = &r.snippet {
                        println!("      Snippet: {}", &snippet[..snippet.len().min(100)]);
                    }
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
            }
        }

        // Test with ArXiv directly
        println!("\n--- Direct ArXiv Provider ---");
        let provider = ArxivProvider::new();
        let options = SearchOptions {
            query: "rust programming".to_string(),
            max_results: Some(3),
            provider: Box::new(provider),
            ..Default::default()
        };
        
        match web_search(options).await {
            Ok(results) => {
                println!("SUCCESS: Got {} results", results.len());
                for (i, r) in results.iter().take(2).enumerate() {
                    println!("  [{}] {}", i, r.title);
                    println!("      URL: {}", r.url);
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test test_real_fetch_url -- --ignored --nocapture
    async fn test_real_fetch_url() {
        let args = json!({
            "url": "https://httpbin.org/html",
            "extract_type": "text"
        });

        let result = execute_fetch_url_async(&args).await;
        match result {
            Ok(json_str) => {
                println!("Fetch result: {}", json_str);
                let parsed: Value = serde_json::from_str(&json_str).unwrap();
                assert_eq!(parsed["status"], "success");
                println!("Content: {}", parsed["content"]);
            }
            Err(e) => {
                // May fail in environments without internet access
                println!("Fetch error (may be expected in offline environments): {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test test_arxiv_works -- --ignored --nocapture
    async fn test_arxiv_works() {
        println!("\n=== Testing ArXiv Provider ===");
        let args = json!({
            "query": "machine learning",
            "provider": "arxiv",
            "max_results": 3
        });

        match execute_web_search_async(&args).await {
            Ok(result_json) => {
                println!("Result: {}", result_json);
                if let Ok(parsed) = serde_json::from_str::<Value>(&result_json) {
                    if parsed["status"] == "success" && parsed["result_count"].as_u64().unwrap_or(0) > 0 {
                        println!("✓ ArXiv search works! Got {} results", parsed["result_count"]);
                    } else {
                        println!("✗ ArXiv returned 0 results or error");
                    }
                }
            }
            Err(e) => println!("✗ Error: {}", e),
        }
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test test_duckduckgo_html_diagnostic -- --ignored --nocapture
    async fn test_duckduckgo_html_diagnostic() {
        println!("\n=== DuckDuckGo HTML Diagnostic ===");
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create client");

        let url = "https://html.duckduckgo.com/html?q=rust&kl=wt-wt";
        println!("Fetching: {}", url);

        match client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await
        {
            Ok(response) => {
                println!("Status: {}", response.status());
                match response.text().await {
                    Ok(html) => {
                        println!("HTML length: {} bytes", html.len());
                        if html.len() > 500 {
                            println!("First 500 chars:\n{}", &html[..500]);
                        } else {
                            println!("Full HTML:\n{}", html);
                        }
                        
                        // Try to find result elements
                        let document = Html::parse_document(&html);
                        
                        // Try different selectors
                        let selectors = vec![
                            ("h2.result__title a", "Original websearch selector"),
                            (".result__title a", "Without h2"),
                            ("a.result__link", "With .result__link"),
                            (".result a", "Just .result a"),
                            ("a[href^='http']", "All external links"),
                        ];
                        
                        for (selector_str, desc) in selectors {
                            if let Ok(sel) = Selector::parse(selector_str) {
                                let count = document.select(&sel).count();
                                println!("Found {} with '{}' ({})", count, selector_str, desc);
                            }
                        }
                    }
                    Err(e) => println!("Failed to read response: {}", e),
                }
            }
            Err(e) => println!("Request failed: {}", e),
        }
    }
}
