//! Built-in tools for the enhanced local Rust app

pub mod delegation;
pub mod desktop_automation;
pub mod registry;
pub mod web;

// Re-export all built-in tools
pub use delegation::*;
pub use desktop_automation::*;
pub use web::*;
