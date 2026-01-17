//! Mouse Control Tools
//!
//! This module provides tools for controlling mouse movement and clicks.

pub mod click;
pub mod move_cursor;
pub mod scroll;

// Re-export mouse tools
pub use click::Click;
pub use move_cursor::MoveCursor;
pub use scroll::Scroll;
