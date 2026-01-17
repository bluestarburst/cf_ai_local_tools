use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::info;

/// Agent configuration that matches the web viewer format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub purpose: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: String,
    pub tools: Vec<String>,
    #[serde(rename = "modelId")]
    pub model_id: String,
    #[serde(rename = "maxIterations")]
    pub max_iterations: usize,
    #[serde(rename = "isLocked")]
    pub is_locked: bool,
    #[serde(rename = "separateReasoningModel", default)]
    pub separate_reasoning_model: bool,
    #[serde(rename = "reasoningModelId", default)]
    pub reasoning_model_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// Agent storage manager
pub struct AgentStorage {
    storage_path: PathBuf,
    agents: HashMap<String, Agent>,
}

impl AgentStorage {
    /// Create a new agent storage instance
    pub fn new() -> Result<Self> {
        let storage_path = Self::get_storage_path()?;

        // Ensure directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent).context("Failed to create storage directory")?;
        }

        let mut storage = Self {
            storage_path,
            agents: HashMap::new(),
        };

        // Load existing agents or create defaults
        if storage.storage_path.exists() {
            storage.load()?;
        } else {
            storage.agents = Self::create_default_agents();
            storage.save()?;
        }

        Ok(storage)
    }

    /// Get the storage file path
    fn get_storage_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Could not determine home directory")?;

        Ok(home_dir
            .join(".config")
            .join("cf_ai_local_tools")
            .join("agents.json"))
    }

    /// Load agents from disk
    fn load(&mut self) -> Result<()> {
        let contents =
            fs::read_to_string(&self.storage_path).context("Failed to read agents file")?;

        self.agents = serde_json::from_str(&contents).context("Failed to parse agents JSON")?;

        info!("[AgentStorage] Loaded {} agents", self.agents.len());
        Ok(())
    }

    /// Save agents to disk
    fn save(&self) -> Result<()> {
        let json =
            serde_json::to_string_pretty(&self.agents).context("Failed to serialize agents")?;

        fs::write(&self.storage_path, json).context("Failed to write agents file")?;

        info!("[AgentStorage] Saved {} agents", self.agents.len());
        Ok(())
    }

    /// Get all agents
    pub fn get_all(&self) -> Vec<Agent> {
        self.agents.values().cloned().collect()
    }

    /// Get agent by ID
    pub fn get(&self, id: &str) -> Option<Agent> {
        self.agents.get(id).cloned()
    }

    /// Create a new agent
    pub fn create(&mut self, agent: Agent) -> Result<Agent> {
        if self.agents.contains_key(&agent.id) {
            anyhow::bail!("Agent with id '{}' already exists", agent.id);
        }

        self.agents.insert(agent.id.clone(), agent.clone());
        self.save()?;

        info!("[AgentStorage] Created agent: {}", agent.id);
        Ok(agent)
    }

    /// Update an existing agent
    pub fn update(&mut self, id: &str, agent: Agent) -> Result<Agent> {
        let existing = self
            .agents
            .get(id)
            .context(format!("Agent '{}' not found", id))?;

        if existing.is_locked {
            anyhow::bail!("Cannot update locked agent '{}'", id);
        }

        self.agents.insert(id.to_string(), agent.clone());
        self.save()?;

        info!("[AgentStorage] Updated agent: {}", id);
        Ok(agent)
    }

    /// Delete an agent
    pub fn delete(&mut self, id: &str) -> Result<()> {
        let existing = self
            .agents
            .get(id)
            .context(format!("Agent '{}' not found", id))?;

        if existing.is_locked {
            anyhow::bail!("Cannot delete locked agent '{}'", id);
        }

        self.agents.remove(id);
        self.save()?;

        info!("[AgentStorage] Deleted agent: {}", id);
        Ok(())
    }

    /// Clear all agents from storage
    pub fn clear(&mut self) -> Result<()> {
        self.agents.clear();
        self.save()?;
        info!("[AgentStorage] Cleared all agents");
        Ok(())
    }

    /// Validate that all tools in an agent exist
    pub fn validate_tools(&self, agent: &Agent, available_tools: &[String]) -> Result<()> {
        let available_set: std::collections::HashSet<_> = available_tools.iter().collect();

        for tool in &agent.tools {
            if !available_set.contains(tool) {
                anyhow::bail!(
                    "Agent '{}' references unknown tool '{}'. Available tools: {:?}",
                    agent.id,
                    tool,
                    available_tools
                );
            }
        }

        Ok(())
    }

    /// Create default agents that only use available tools
    fn create_default_agents() -> HashMap<String, Agent> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut agents = HashMap::new();

        // Desktop Automation Agent
        let desktop_agent = Agent {
            id: "desktop-automation-agent".to_string(),
            name: "Desktop Automation Agent".to_string(),
            purpose: "Control mouse and keyboard for desktop automation tasks".to_string(),
            system_prompt: r#"You are a desktop automation agent. Your purpose is to control mouse and keyboard for desktop automation tasks.

You have access to the following tools:
- mouse_move: Move the mouse cursor to coordinates
- mouse_click: Click the mouse at current position
- mouse_scroll: Scroll the mouse wheel
- keyboard_input: Type text on the keyboard
- keyboard_command: Execute keyboard shortcuts
- get_mouse_position: Get current mouse coordinates
- take_screenshot: Capture the screen

When given a task:
1. Think about what actions are needed
2. Execute the appropriate tool calls
3. Observe the results
4. Continue until the task is complete

Always be precise with coordinates and timing."#.to_string(),
            tools: vec![
                "mouse_move".to_string(),
                "mouse_click".to_string(),
                "mouse_scroll".to_string(),
                "keyboard_input".to_string(),
                "keyboard_command".to_string(),
                "get_mouse_position".to_string(),
                "take_screenshot".to_string(),
            ],
            model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
            max_iterations: 5,
            is_locked: true,
            separate_reasoning_model: false,
            reasoning_model_id: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        agents.insert(desktop_agent.id.clone(), desktop_agent);

        // Web Research Agent
        let web_agent = Agent {
            id: "web-research-agent".to_string(),
            name: "Web Research Agent".to_string(),
            purpose: "Search the web and fetch information from URLs".to_string(),
            system_prompt: r#"You are a web research agent. Your purpose is to search the web and fetch information from URLs.

You have access to the following tools:
- web_search: Search the web for information
- fetch_url: Fetch content from a specific URL

When given a research task:
1. Think about what information is needed
2. Use web_search to find relevant sources
3. Use fetch_url to get detailed information from promising URLs
4. Synthesize the information to answer the question

Always cite your sources and be accurate."#.to_string(),
            tools: vec![
                "web_search".to_string(),
                "fetch_url".to_string(),
            ],
            model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
            max_iterations: 5,
            is_locked: true,
            separate_reasoning_model: false,
            reasoning_model_id: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        agents.insert(web_agent.id.clone(), web_agent);

        // General Assistant
        let general_agent = Agent {
            id: "general-assistant".to_string(),
            name: "General Assistant".to_string(),
            purpose: "Help with various tasks using all available tools".to_string(),
            system_prompt: r#"You are a general-purpose AI assistant with access to desktop automation and web research tools.

You have access to the following tools:
- mouse_move, mouse_click, mouse_scroll: Mouse control
- keyboard_input, keyboard_command: Keyboard control
- get_mouse_position, take_screenshot: System information
- web_search, fetch_url: Web research

When given a task:
1. Analyze what the user needs
2. Determine which tools are appropriate
3. Execute a plan using the available tools
4. Iterate until the task is complete

Be helpful, accurate, and efficient."#.to_string(),
            tools: vec![
                "mouse_move".to_string(),
                "mouse_click".to_string(),
                "mouse_scroll".to_string(),
                "keyboard_input".to_string(),
                "keyboard_command".to_string(),
                "get_mouse_position".to_string(),
                "take_screenshot".to_string(),
                "web_search".to_string(),
                "fetch_url".to_string(),
            ],
            model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
            max_iterations: 5,
            is_locked: true,
            separate_reasoning_model: false,
            reasoning_model_id: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        agents.insert(general_agent.id.clone(), general_agent);

        agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tools() {
        let storage = AgentStorage::new().unwrap();
        let agent = Agent {
            id: "test".to_string(),
            name: "Test".to_string(),
            purpose: "Test agent".to_string(),
            system_prompt: "Test prompt".to_string(),
            tools: vec!["mouse_move".to_string(), "invalid_tool".to_string()],
            model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
            max_iterations: 5,
            is_locked: false,
            separate_reasoning_model: false,
            reasoning_model_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let available_tools = vec!["mouse_move".to_string(), "mouse_click".to_string()];

        let result = storage.validate_tools(&agent, &available_tools);
        assert!(result.is_err());
    }
}
