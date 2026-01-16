use websearch::{
    providers::DuckDuckGoProvider,
    web_search,
    SearchOptions,
};

#[tokio::main]
async fn main() {
    println!("Testing DuckDuckGo provider directly...");
    
    let provider = DuckDuckGoProvider::new();
    let options = SearchOptions {
        query: "rust".to_string(),
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
                    println!("      Snippet: {}", &snippet[..snippet.len().min(80)]);
                }
            }
        }
        Err(e) => {
            println!("ERROR: {}", e);
            println!("Error type: {:?}", e);
        }
    }
}
