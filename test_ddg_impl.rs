/// Test DuckDuckGo custom implementation
#[tokio::test]
#[ignore]
async fn test_custom_duckduckgo_implementation() {
    use scraper::{Html, Selector};
    use url::Url;
    
    println!("\n=== Testing DuckDuckGo Link Extraction ===\n");
    
    // Fetch real DuckDuckGo HTML
    let client = reqwest::Client::new();
    let response = client
        .post("https://html.duckduckgo.com/html")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .form(&vec![
            ("q", "rust programming"),
            ("b", ""),
            ("kl", "wt-wt"),
        ])
        .send()
        .await
        .expect("Failed to fetch");
    
    let html = response.text().await.expect("Failed to read response");
    let document = Html::parse_document(&html);
    
    let link_selector = Selector::parse("h2.result__title a").unwrap();
    let links: Vec<_> = document.select(&link_selector).collect();
    
    println!("Found {} result links", links.len());
    
    // Check and extract first 5 links
    for (i, link) in links.iter().take(5).enumerate() {
        if let Some(href) = link.value().attr("href") {
            println!("\n[Result {}]", i + 1);
            println!("  Raw href: {}", href);
            
            // Test extraction logic
            let extracted_url = if href.starts_with("//duckduckgo.com/l/?uddg=") {
                match Url::parse(&format!("https:{}", href)) {
                    Ok(parsed) => {
                        parsed
                            .query_pairs()
                            .find(|(k, _)| k == "uddg")
                            .map(|(_, v)| urlencoding::decode(&v).unwrap_or_default().to_string())
                            .unwrap_or_else(|| href.to_string())
                    }
                    Err(e) => {
                        println!("  Parse error: {}", e);
                        href.to_string()
                    }
                }
            } else if href.starts_with("http") {
                href.to_string()
            } else {
                "SKIPPED (invalid format)".to_string()
            };
            
            println!("  Extracted URL: {}", extracted_url);
            
            // Check if URL is valid
            if extracted_url.contains("duckduckgo.com") {
                println!("  ❌ Still contains duckduckgo.com - would be filtered");
            } else if extracted_url.starts_with("http") {
                println!("  ✓ Valid external URL");
            }
        }
    }
}
