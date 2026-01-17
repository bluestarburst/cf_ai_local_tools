#[cfg(test)]
mod test_duckduckgo_fixed {
    use crate::tools::web_search::execute_web_search_async;
    use serde_json::{json, Value};

    #[tokio::test]
    #[ignore] // Run with: cargo test test_duckduckgo_fixed -- --ignored --nocapture
    async fn test_duckduckgo_fixed() {
        println!("\n=== DuckDuckGo Fix Verification ===\n");
        
        let args = json!({
            "query": "rust programming language",
            "provider": "duckduckgo",
            "max_results": 5
        });

        match execute_web_search_async(&args).await {
            Ok(result_json) => {
                println!("Raw JSON response:\n{}\n", result_json);
                
                match serde_json::from_str::<Value>(&result_json) {
                    Ok(parsed) => {
                        let status = &parsed["status"];
                        let result_count = parsed["result_count"].as_u64().unwrap_or(0);
                        let provider = &parsed["provider"];
                        
                        println!("Status: {}", status);
                        println!("Provider: {}", provider);
                        println!("Result Count: {}\n", result_count);
                        
                        if status == "success" && result_count > 0 {
                            println!("✅ SUCCESS: DuckDuckGo is returning {} results!", result_count);
                            
                            if let Some(results) = parsed["results"].as_array() {
                                println!("\nFirst 3 results:");
                                for (i, result) in results.iter().take(3).enumerate() {
                                    if let (Some(title), Some(url)) = (
                                        result["title"].as_str(),
                                        result["url"].as_str(),
                                    ) {
                                        println!("  {}. {}", i + 1, title);
                                        println!("     URL: {}", url);
                                        
                                        // Verify URL is not a duckduckgo proxy
                                        if url.contains("duckduckgo.com") {
                                            println!("     ❌ WARNING: URL still contains duckduckgo.com");
                                        } else if url.starts_with("http") {
                                            println!("     ✓ Valid external URL");
                                        }
                                    }
                                }
                            }
                        } else if status == "error" {
                            println!("❌ ERROR: {}", parsed.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error"));
                        } else {
                            println!("⚠️ Unexpected status or 0 results");
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to parse JSON response: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Error executing search: {}", e);
            }
        }
    }
}
