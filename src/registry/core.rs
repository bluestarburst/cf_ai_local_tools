//! Core registry traits and central registry system

use crate::agents::registry::{AgentRegistry, DefaultAgentRegistry};
use crate::tools::registry::{DefaultToolRegistry, ToolRegistry};
use async_trait::async_trait;

/// Core trait for all component registries
#[async_trait]
pub trait Registry {
    /// Register a new item
    async fn register(&mut self, item: Box<dyn std::any::Any>) -> crate::core::Result<()>;

    /// Unregister an item by ID
    async fn unregister(&mut self, id: &str) -> crate::core::Result<()>;

    /// Get an item by ID
    async fn get(&self, id: &str) -> crate::core::Result<Option<Box<dyn std::any::Any>>>;

    /// Get all registered items
    async fn list(&self) -> crate::core::Result<Vec<Box<dyn std::any::Any>>>;

    /// Get the count of registered items
    async fn count(&self) -> crate::core::Result<usize>;
}

/// Metadata for registered components
#[derive(Debug, Clone)]
pub struct ComponentMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
}

/// Central registry that manages all component registries
pub struct CentralRegistry {
    pub agents: Box<dyn AgentRegistry>,
    pub tools: Box<dyn ToolRegistry>,
}

impl CentralRegistry {
    pub fn new() -> Self {
        Self {
            agents: Box::new(DefaultAgentRegistry::new()),
            tools: Box::new(DefaultToolRegistry::new()),
        }
    }

    /// Initialize the central registry with built-in components
    pub async fn initialize(&mut self) -> crate::core::Result<()> {
        // Register built-in agents
        self.register_builtin_agents().await?;

        // Register built-in tools
        self.register_builtin_tools().await?;

        Ok(())
    }

    async fn register_builtin_agents(&mut self) -> crate::core::Result<()> {
        // Register desktop automation agent
        let desktop_agent: Box<dyn crate::core::Agent> =
            Box::new(crate::agents::DesktopAutomationAgent::new());
        self.agents.register(desktop_agent).await?;

        // Register web research agent
        let web_agent: Box<dyn crate::core::Agent> =
            Box::new(crate::agents::WebResearchAgent::new());
        self.agents.register(web_agent).await?;

        // Register conversational agent
        let conv_agent: Box<dyn crate::core::Agent> =
            Box::new(crate::agents::ConversationalAgent::new());
        self.agents.register(conv_agent).await?;

        Ok(())
    }

    async fn register_builtin_tools(&mut self) -> crate::core::Result<()> {
        // Register desktop automation tools
        self.tools
            .register(Box::new(
                crate::tools::desktop_automation::mouse::MoveCursor::new(),
            ))
            .await?;
        self.tools
            .register(Box::new(
                crate::tools::desktop_automation::mouse::Click::new(),
            ))
            .await?;
        self.tools
            .register(Box::new(
                crate::tools::desktop_automation::mouse::Scroll::new(),
            ))
            .await?;
        self.tools
            .register(Box::new(
                crate::tools::desktop_automation::keyboard::TypeText::new(),
            ))
            .await?;
        self.tools
            .register(Box::new(
                crate::tools::desktop_automation::keyboard::Hotkey::new(),
            ))
            .await?;
        self.tools
            .register(Box::new(
                crate::tools::desktop_automation::screen::Screenshot::new(),
            ))
            .await?;
        self.tools
            .register(Box::new(
                crate::tools::desktop_automation::screen::GetPosition::new(),
            ))
            .await?;

        // Register web tools
        self.tools
            .register(Box::new(crate::tools::web::WebSearch::new()))
            .await?;
        self.tools
            .register(Box::new(crate::tools::web::FetchUrl::new()))
            .await?;

        // Register delegation tools
        self.tools
            .register(Box::new(crate::tools::delegation::DelegateToAgent::new()))
            .await?;

        Ok(())
    }

    /// Shutdown the central registry and cleanup resources
    pub async fn shutdown(&self) -> crate::core::Result<()> {
        // Any cleanup needed
        Ok(())
    }
}
