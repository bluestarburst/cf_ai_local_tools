//! Built-in agents for enhanced local Rpub mod conversational;
pub mod desktop_automation;
pub mod web_research;

pub mod conversation;
pub mod conversational;
pub mod delegation;
pub mod registry;
pub mod thinking;

// Re-export all built-in agents
pub use conversational::ConversationalAgent;
pub use desktop_automation::DesktopAutomationAgent;
pub use web_research::WebResearchAgent;
