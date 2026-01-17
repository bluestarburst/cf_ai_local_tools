//! Agent registry for managing agent components

use crate::core::Agent;
use async_trait::async_trait;

/// Trait for agent registries
#[async_trait]
pub trait AgentRegistry: Send + Sync {
    /// Register an agent
    async fn register(&mut self, agent: Box<dyn Agent>) -> crate::core::Result<()>;

    /// Unregister an agent by ID
    async fn unregister(&mut self, id: &str) -> crate::core::Result<()>;

    /// Get an agent by ID
    async fn get(&self, id: &str) -> crate::core::Result<Option<Box<dyn Agent>>>;

    /// Get all registered agents
    async fn list(&self) -> crate::core::Result<Vec<Box<dyn Agent>>>;

    /// Get the count of registered agents
    async fn count(&self) -> crate::core::Result<usize>;

    /// Find agents by capability
    async fn find_by_capability(
        &self,
        capability: &str,
    ) -> crate::core::Result<Vec<Box<dyn Agent>>>;

    /// Find agents that depend on a specific tool
    async fn find_by_tool_dependency(
        &self,
        tool_id: &str,
    ) -> crate::core::Result<Vec<Box<dyn Agent>>>;

    /// Validate that an agent's dependencies are satisfied
    async fn validate_dependencies(
        &self,
        agent: &dyn Agent,
        available_tools: &[String],
    ) -> crate::core::Result<()>;

    /// Get agent metadata
    async fn get_metadata(
        &self,
        agent_id: &str,
    ) -> crate::core::Result<Option<crate::registry::ComponentMetadata>>;
}

/// Default implementation of AgentRegistry
pub struct DefaultAgentRegistry {
    agents: std::collections::HashMap<String, Box<dyn Agent>>,
    capability_index: std::collections::HashMap<String, Vec<String>>,
    tool_dependency_index: std::collections::HashMap<String, Vec<String>>,
}

impl DefaultAgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: std::collections::HashMap::new(),
            capability_index: std::collections::HashMap::new(),
            tool_dependency_index: std::collections::HashMap::new(),
        }
    }

    /// Rebuild capability and dependency indexes
    fn rebuild_indexes(&mut self) {
        self.capability_index.clear();
        self.tool_dependency_index.clear();

        for (agent_id, agent) in &self.agents {
            // Build capability index
            for capability in agent.capabilities() {
                self.capability_index
                    .entry(capability.clone())
                    .or_insert_with(Vec::new)
                    .push(agent_id.clone());
            }

            // Build tool dependency index
            for tool_id in agent.tool_dependencies() {
                self.tool_dependency_index
                    .entry(tool_id.clone())
                    .or_insert_with(Vec::new)
                    .push(agent_id.clone());
            }
        }
    }
}

#[async_trait]
impl AgentRegistry for DefaultAgentRegistry {
    async fn register(&mut self, agent: Box<dyn Agent>) -> crate::core::Result<()> {
        let agent_id = agent.id().to_string();

        if self.agents.contains_key(&agent_id) {
            return Err(crate::core::AppError::Registry(format!(
                "Agent '{}' already registered",
                agent_id
            )));
        }

        self.agents.insert(agent_id, agent);
        self.rebuild_indexes();

        Ok(())
    }

    async fn unregister(&mut self, id: &str) -> crate::core::Result<()> {
        if self.agents.remove(id).is_none() {
            return Err(crate::core::AppError::Registry(format!(
                "Agent '{}' not found",
                id
            )));
        }

        self.rebuild_indexes();
        Ok(())
    }

    async fn get(&self, id: &str) -> crate::core::Result<Option<Box<dyn Agent>>> {
        Ok(self
            .agents
            .get(id)
            .map(|a| dyn_clone::clone_box(a.as_ref())))
    }

    async fn list(&self) -> crate::core::Result<Vec<Box<dyn Agent>>> {
        Ok(self
            .agents
            .values()
            .map(|a| dyn_clone::clone_box(a.as_ref()))
            .collect())
    }

    async fn count(&self) -> crate::core::Result<usize> {
        Ok(self.agents.len())
    }

    async fn find_by_capability(
        &self,
        capability: &str,
    ) -> crate::core::Result<Vec<Box<dyn Agent>>> {
        let agent_ids = self
            .capability_index
            .get(capability)
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        for agent_id in agent_ids {
            if let Some(agent) = self.agents.get(&agent_id) {
                results.push(dyn_clone::clone_box(agent.as_ref()));
            }
        }

        Ok(results)
    }

    async fn find_by_tool_dependency(
        &self,
        tool_id: &str,
    ) -> crate::core::Result<Vec<Box<dyn Agent>>> {
        let agent_ids = self
            .tool_dependency_index
            .get(tool_id)
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        for agent_id in agent_ids {
            if let Some(agent) = self.agents.get(&agent_id) {
                results.push(dyn_clone::clone_box(agent.as_ref()));
            }
        }

        Ok(results)
    }

    async fn validate_dependencies(
        &self,
        agent: &dyn Agent,
        available_tools: &[String],
    ) -> crate::core::Result<()> {
        let required_tools = agent.tool_dependencies();
        let available_set: std::collections::HashSet<_> = available_tools.iter().collect();

        for tool_id in required_tools {
            if !available_set.contains(tool_id) {
                return Err(crate::core::AppError::Registry(format!(
                    "Agent '{}' requires tool '{}' which is not available",
                    agent.id(),
                    tool_id
                )));
            }
        }

        Ok(())
    }

    async fn get_metadata(
        &self,
        agent_id: &str,
    ) -> crate::core::Result<Option<crate::registry::ComponentMetadata>> {
        if let Some(agent) = self.agents.get(agent_id) {
            Ok(Some(crate::registry::ComponentMetadata {
                id: agent.id().to_string(),
                name: agent.name().to_string(),
                version: agent.version().to_string(),
                description: agent.description().to_string(),
                category: "agent".to_string(),
                dependencies: agent.tool_dependencies().iter().cloned().collect(),
                capabilities: agent.capabilities().iter().cloned().collect(),
            }))
        } else {
            Ok(None)
        }
    }
}
