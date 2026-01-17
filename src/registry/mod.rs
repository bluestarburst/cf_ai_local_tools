//! Component registry system for managing agents and tools

pub mod core;
pub mod loader;
pub mod presets;

// Re-export main types
pub use core::{CentralRegistry, ComponentMetadata, Registry};
pub use loader::{ComponentInfo, ComponentLoader, ComponentType};
