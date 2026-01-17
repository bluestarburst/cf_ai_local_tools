use crate::{Agent, AgentContext, AgentResult, LLMClient, ReasoningConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebResearchAgent {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub reasoning_config: ReasoningConfig,
    pub capabilities: Vec<String>,
    pub tool_dependencies: Vec<String>,
}

impl WebResearchAgent {
    pub fn new() -> Self {
        Self {
            id: "web-research-agent".to_string(),
            name: "Web Research Agent".to_string(),
            system_prompt: include_str!("prompt.txt").to_string(),
            reasoning_config: ReasoningConfig::default(),
            capabilities: vec!["web_search".to_string(), "content_extraction".to_string()],
            tool_dependencies: vec!["web_search".to_string(), "fetch_url".to_string()],
        }
    }
}

#[async_trait]
impl Agent for WebResearchAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Searches the web and extracts information."
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    fn tool_dependencies(&self) -> &[String] {
        &self.tool_dependencies
    }

    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    fn reasoning_config(&self) -> &ReasoningConfig {
        &self.reasoning_config
    }

    async fn execute(
        &self,
        task: &str,
        context: &AgentContext,
        llm: &dyn LLMClient,
        conversation_manager: Option<
            std::sync::Arc<dyn crate::agents::conversation::ConversationManager>,
        >,
        available_tools: &[Box<dyn crate::core::Tool>],
    ) -> crate::core::Result<AgentResult> {
        Ok(AgentResult {
            success: true,
            response: format!("Executed web research task: {}", task),
            steps: vec![],
            execution_time: std::time::Duration::from_millis(0),
            final_context: context.clone(),
        })
    }

    fn can_handle_task(&self, task: &str) -> f32 {
        let task = task.to_lowercase();
        if task.contains("search")
            || task.contains("find")
            || task.contains("lookup")
            || task.contains("google")
        {
            0.9
        } else {
            0.1
        }
    }
}
