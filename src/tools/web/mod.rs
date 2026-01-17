//! Web interaction tools
//!
//! This module provides tools for web searching and URL fetching.

pub mod fetch_url;
pub mod search;

// Re-export web tools
pub use fetch_url::FetchUrl;
pub use search::WebSearch;
