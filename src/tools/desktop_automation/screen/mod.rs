//! Screen Interaction Tools
//!
//! This module provides tools for screen capture and position detection.

pub mod get_position;
pub mod screenshot;

// Re-export screen tools
pub use get_position::GetPosition;
pub use screenshot::Screenshot;
