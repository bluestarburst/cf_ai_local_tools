//! Tool registry for managing tool components

use crate::core::Tool;
use async_trait::async_trait;

/// Trait for tool registries
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    /// Register a tool
    async fn register(&mut self, tool: Box<dyn Tool>) -> crate::core::Result<()>;

    /// Unregister a tool by ID
    async fn unregister(&mut self, id: &str) -> crate::core::Result<()>;

    /// Get a tool by ID
    async fn get(&self, id: &str) -> crate::core::Result<Option<Box<dyn Tool>>>;

    /// Get all registered tools
    async fn list(&self) -> crate::core::Result<Vec<Box<dyn Tool>>>;

    /// Get the count of registered tools
    async fn count(&self) -> crate::core::Result<usize>;

    /// Find tools by category
    async fn find_by_category(&self, category: &str) -> crate::core::Result<Vec<Box<dyn Tool>>>;

    /// Find tools by capability
    async fn find_by_capability(&self, capability: &str)
        -> crate::core::Result<Vec<Box<dyn Tool>>>;

    /// Validate tool dependencies
    async fn validate_dependencies(&self, tool: &dyn Tool) -> crate::core::Result<()>;

    /// Get tool metadata
    async fn get_metadata(
        &self,
        tool_id: &str,
    ) -> crate::core::Result<Option<crate::registry::ComponentMetadata>>;
}

/// Default implementation of ToolRegistry
pub struct DefaultToolRegistry {
    tools: std::collections::HashMap<String, Box<dyn Tool>>,
    category_index: std::collections::HashMap<String, Vec<String>>,
    capability_index: std::collections::HashMap<String, Vec<String>>,
}

impl DefaultToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
            category_index: std::collections::HashMap::new(),
            capability_index: std::collections::HashMap::new(),
        }
    }

    /// Rebuild category and capability indexes
    fn rebuild_indexes(&mut self) {
        self.category_index.clear();
        self.capability_index.clear();

        for (tool_id, tool) in &self.tools {
            // Build category index
            let category = tool.category().to_string();
            self.category_index
                .entry(category)
                .or_insert_with(Vec::new)
                .push(tool_id.clone());

            // Build capability index (tools can provide capabilities)
            // For now, we'll use the tool ID as a capability
            let capability = tool.id().to_string();
            self.capability_index
                .entry(capability)
                .or_insert_with(Vec::new)
                .push(tool_id.clone());
        }
    }
}

#[async_trait]
impl ToolRegistry for DefaultToolRegistry {
    async fn register(&mut self, tool: Box<dyn Tool>) -> crate::core::Result<()> {
        let tool_id = tool.id().to_string();

        if self.tools.contains_key(&tool_id) {
            return Err(crate::core::AppError::Registry(format!(
                "Tool '{}' already registered",
                tool_id
            )));
        }

        self.tools.insert(tool_id, tool);
        self.rebuild_indexes();

        Ok(())
    }

    async fn unregister(&mut self, id: &str) -> crate::core::Result<()> {
        if self.tools.remove(id).is_none() {
            return Err(crate::core::AppError::Registry(format!(
                "Tool '{}' not found",
                id
            )));
        }

        self.rebuild_indexes();
        Ok(())
    }

    async fn get(&self, id: &str) -> crate::core::Result<Option<Box<dyn Tool>>> {
        Ok(self.tools.get(id).map(|t| dyn_clone::clone_box(t.as_ref())))
    }

    async fn list(&self) -> crate::core::Result<Vec<Box<dyn Tool>>> {
        Ok(self
            .tools
            .values()
            .map(|t| dyn_clone::clone_box(t.as_ref()))
            .collect())
    }

    async fn count(&self) -> crate::core::Result<usize> {
        Ok(self.tools.len())
    }

    async fn find_by_category(&self, category: &str) -> crate::core::Result<Vec<Box<dyn Tool>>> {
        let tool_ids = self
            .category_index
            .get(category)
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        for tool_id in tool_ids {
            if let Some(tool) = self.tools.get(&tool_id) {
                results.push(dyn_clone::clone_box(tool.as_ref()));
            }
        }

        Ok(results)
    }

    async fn find_by_capability(
        &self,
        capability: &str,
    ) -> crate::core::Result<Vec<Box<dyn Tool>>> {
        let tool_ids = self
            .capability_index
            .get(capability)
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        for tool_id in tool_ids {
            if let Some(tool) = self.tools.get(&tool_id) {
                results.push(dyn_clone::clone_box(tool.as_ref()));
            }
        }

        Ok(results)
    }

    async fn validate_dependencies(&self, _tool: &dyn Tool) -> crate::core::Result<()> {
        // For now, tools don't have dependencies on other tools
        // This could be extended in the future
        Ok(())
    }

    async fn get_metadata(
        &self,
        tool_id: &str,
    ) -> crate::core::Result<Option<crate::registry::ComponentMetadata>> {
        if let Some(tool) = self.tools.get(tool_id) {
            Ok(Some(crate::registry::ComponentMetadata {
                id: tool.id().to_string(),
                name: tool.name().to_string(),
                version: "1.0.0".to_string(), // Tools don't have versions yet
                description: tool.description().to_string(),
                category: tool.category().to_string(),
                dependencies: Vec::new(), // Tools don't have dependencies yet
                capabilities: vec![tool.id().to_string()], // Tool provides its own capability
            }))
        } else {
            Ok(None)
        }
    }
}
