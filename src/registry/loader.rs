//! Component loader for dynamic loading of agents and tools

use crate::core::{Agent, Tool};
use crate::registry::core::CentralRegistry;
use std::path::PathBuf;

/// Component loader for discovering and loading agents and tools
pub struct ComponentLoader {
    agents_path: PathBuf,
    tools_path: PathBuf,
}

impl ComponentLoader {
    /// Create a new component loader
    pub fn new() -> Self {
        let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

        Self {
            agents_path: base_path.join("agents"),
            tools_path: base_path.join("tools"),
        }
    }

    /// Load all built-in agents
    pub async fn load_builtin_agents(&self) -> crate::core::Result<Vec<Box<dyn Agent>>> {
        let mut agents: Vec<Box<dyn Agent>> = Vec::new();

        // Load desktop automation agent
        if self.agents_path.join("desktop_automation").exists() {
            agents.push(Box::new(crate::agents::DesktopAutomationAgent::new()));
        }

        // Load web research agent
        if self.agents_path.join("web_research").exists() {
            agents.push(Box::new(crate::agents::WebResearchAgent::new()));
        }

        Ok(agents)
    }

    /// Load all built-in tools
    pub async fn load_builtin_tools(&self) -> crate::core::Result<Vec<Box<dyn Tool>>> {
        let mut tools: Vec<Box<dyn Tool>> = Vec::new();

        // Load desktop automation tools
        if self.tools_path.join("desktop_automation").exists() {
            // Mouse tools
            tools.push(Box::new(
                crate::tools::desktop_automation::mouse::MoveCursor::new(),
            ));
            tools.push(Box::new(
                crate::tools::desktop_automation::mouse::Click::new(),
            ));
            tools.push(Box::new(
                crate::tools::desktop_automation::mouse::Scroll::new(),
            ));

            // Keyboard tools
            tools.push(Box::new(
                crate::tools::desktop_automation::keyboard::TypeText::new(),
            ));
            tools.push(Box::new(
                crate::tools::desktop_automation::keyboard::Hotkey::new(),
            ));

            // Screen tools
            tools.push(Box::new(
                crate::tools::desktop_automation::screen::Screenshot::new(),
            ));
            tools.push(Box::new(
                crate::tools::desktop_automation::screen::GetPosition::new(),
            ));
        }

        Ok(tools)
    }

    /// Load all built-in components into the registry
    pub async fn load_all_into_registry(
        &self,
        registry: &mut CentralRegistry,
    ) -> crate::core::Result<()> {
        // Load and register agents
        let agents = self.load_builtin_agents().await?;
        for agent in agents {
            registry.agents.register(agent).await?;
        }

        // Load and register tools
        let tools = self.load_builtin_tools().await?;
        for tool in tools {
            registry.tools.register(tool).await?;
        }

        Ok(())
    }

    /// Discover available agent directories
    pub fn discover_agent_directories(&self) -> crate::core::Result<Vec<PathBuf>> {
        self.discover_component_directories(&self.agents_path)
    }

    /// Discover available tool directories
    pub fn discover_tool_directories(&self) -> crate::core::Result<Vec<PathBuf>> {
        self.discover_component_directories(&self.tools_path)
    }

    /// Helper method to discover component directories
    fn discover_component_directories(
        &self,
        base_path: &PathBuf,
    ) -> crate::core::Result<Vec<PathBuf>> {
        let mut directories = Vec::new();

        if !base_path.exists() {
            return Ok(directories);
        }

        for entry in std::fs::read_dir(base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Check if it has a mod.rs file (indicating it's a valid component)
                if path.join("mod.rs").exists() {
                    directories.push(path);
                }
            }
        }

        Ok(directories)
    }

    /// Validate a component directory structure
    pub fn validate_component_directory(&self, dir_path: &PathBuf) -> crate::core::Result<()> {
        // Check for required mod.rs file
        if !dir_path.join("mod.rs").exists() {
            return Err(crate::core::AppError::Registry(format!(
                "Component directory '{}' missing mod.rs file",
                dir_path.display()
            )));
        }

        // Additional validation could be added here
        // - Check for required functions/structs
        // - Validate configuration files
        // - Check dependencies

        Ok(())
    }

    /// Get component information from directory
    pub fn get_component_info(&self, dir_path: &PathBuf) -> crate::core::Result<ComponentInfo> {
        let mod_file = dir_path.join("mod.rs");

        // Read mod.rs to extract component information
        // This is a simplified implementation - in a real system,
        // you might parse the Rust code or use metadata files
        let component_name = dir_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                crate::core::AppError::Registry(format!(
                    "Invalid component directory name: {}",
                    dir_path.display()
                ))
            })?;

        // Determine component type based on parent directory
        let component_type = if dir_path.starts_with(&self.agents_path) {
            ComponentType::Agent
        } else if dir_path.starts_with(&self.tools_path) {
            ComponentType::Tool
        } else {
            ComponentType::Unknown
        };

        Ok(ComponentInfo {
            name: component_name.to_string(),
            path: dir_path.clone(),
            component_type,
        })
    }
}

/// Information about a discovered component
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub name: String,
    pub path: PathBuf,
    pub component_type: ComponentType,
}

/// Type of component
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Agent,
    Tool,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_loader_creation() {
        let loader = ComponentLoader::new();

        assert!(loader.agents_path.ends_with("agents/builtin"));
        assert!(loader.tools_path.ends_with("tools/builtin"));
    }

    #[tokio::test]
    async fn test_load_builtin_agents() {
        let loader = ComponentLoader::new();

        let agents = loader.load_builtin_agents().await.unwrap();

        // Should load at least the desktop automation agent
        assert!(!agents.is_empty());
        assert!(agents.iter().any(|a| a.id() == "desktop-automation-agent"));
    }

    #[tokio::test]
    async fn test_load_builtin_tools() {
        let loader = ComponentLoader::new();

        let tools = loader.load_builtin_tools().await.unwrap();

        // Should load at least the mouse tools
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.id() == "mouse_move"));
        assert!(tools.iter().any(|t| t.id() == "mouse_click"));
    }

    #[test]
    fn test_discover_agent_directories() {
        let loader = ComponentLoader::new();

        let directories = loader.discover_agent_directories().unwrap();

        // Should find at least the desktop_automation directory
        assert!(!directories.is_empty());
        assert!(directories
            .iter()
            .any(|d| d.ends_with("desktop_automation")));
    }

    #[test]
    fn test_discover_tool_directories() {
        let loader = ComponentLoader::new();

        let directories = loader.discover_tool_directories().unwrap();

        // Should find at least the desktop_automation directory
        assert!(!directories.is_empty());
        assert!(directories
            .iter()
            .any(|d| d.ends_with("desktop_automation")));
    }

    #[test]
    fn test_validate_component_directory() {
        let loader = ComponentLoader::new();

        let desktop_dir = loader.agents_path.join("desktop_automation");

        // Should validate successfully
        assert!(loader.validate_component_directory(&desktop_dir).is_ok());
    }

    #[test]
    fn test_get_component_info() {
        let loader = ComponentLoader::new();

        let desktop_dir = loader.agents_path.join("desktop_automation");

        let info = loader.get_component_info(&desktop_dir).unwrap();

        assert_eq!(info.name, "desktop_automation");
        assert_eq!(info.component_type, ComponentType::Agent);
    }
}
