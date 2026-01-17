//! Desktop Automation Tools
//!
//! This module provides tools for controlling mouse, keyboard, and screen interactions.
//! These tools enable agents to perform desktop automation tasks.

pub mod keyboard;
pub mod mouse;
pub mod screen;

// Re-export all tools for registry
pub use keyboard::{Hotkey, TypeText};
pub use mouse::{Click, MoveCursor, Scroll};
pub use screen::{GetPosition, Screenshot};

// Tool category metadata
pub const CATEGORY_ID: &str = "desktop_automation";
pub const CATEGORY_NAME: &str = "Desktop Automation";
pub const CATEGORY_DESCRIPTION: &str =
    "Tools for controlling mouse, keyboard, and screen interactions";
