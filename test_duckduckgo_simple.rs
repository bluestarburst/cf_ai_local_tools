// Simple standalone test to debug DuckDuckGo search
use reqwest::Client;
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DuckDuckGo Test ===\n");
    
    let client = Client::new();
    let mut form_data = std::collections::HashMap::new();
    form_data.insert("q", "rust");
    form_data.insert("b", "");
    form_data.insert("kl", "wt-wt");
    
    let response = client
        .post("https://html.duckduckgo.com/html")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .header("Referer", "https://html.duckduckgo.com/")
        .form(&form_data)
        .send()
        .await?;
    
    let status = response.status();
    let html = response.text().await?;
    
    println!("Status: {}", status);
    println!("HTML length: {}", html.len());
    println!();
    
    // Parse HTML
    let document = Html::parse_document(&html);
    
    // Try the original websearch selectors
    let link_selector = Selector::parse("h2.result__title a").unwrap();
    let links: Vec<_> = document.select(&link_selector).collect();
    
    println!("Found {} links with 'h2.result__title a'", links.len());
    
    for (i, link) in links.iter().take(5).enumerate() {
        if let Some(href) = link.value().attr("href") {
            let title = link.inner_html();
            println!("{}. href={}", i + 1, href);
            println!("   title={}", title);
            
            // Check if it's a valid URL
            if href.starts_with("http") {
                println!("   ✓ Valid URL");
            } else if href.starts_with("//duckduckgo.com") {
                println!("   ✗ DuckDuckGo internal link");
            } else {
                println!("   ? href={}", href);
            }
        }
    }
    
    // Try alternative selectors
    println!("\n=== Alternative Selectors ===");
    
    let test_selectors = vec![
        "a.result__a",
        ".result__title a",
        ".result a",
        "a[href]",
    ];
    
    for selector_str in test_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let count = document.select(&selector).count();
            println!("{}: {} matches", selector_str, count);
        }
    }
    
    // Check for snippet selector
    println!("\n=== Snippet Selector ===");
    let snippet_selector = Selector::parse(".result__snippet").unwrap();
    let snippets: Vec<_> = document.select(&snippet_selector).collect();
    println!("Found {} snippets with '.result__snippet'", snippets.len());
    
    Ok(())
}
