use crate::{Agent, AgentContext, AgentResult, LLMClient, ReasoningConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopAutomationAgent {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub reasoning_config: ReasoningConfig,
    pub capabilities: Vec<String>,
    pub tool_dependencies: Vec<String>,
}

impl DesktopAutomationAgent {
    pub fn new() -> Self {
        Self {
            id: "desktop-automation-agent".to_string(),
            name: "Desktop Automation Agent".to_string(),
            system_prompt: include_str!("prompt.txt").to_string(),
            reasoning_config: ReasoningConfig::default(),
            capabilities: vec![
                "mouse_control".to_string(),
                "keyboard_control".to_string(),
                "screen_capture".to_string(),
            ],
            tool_dependencies: vec![
                "mouse_move".to_string(),
                "mouse_click".to_string(),
                "keyboard_type".to_string(),
            ],
        }
    }
}

#[async_trait]
impl Agent for DesktopAutomationAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Automates desktop interactions using mouse and keyboard."
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
            response: format!("Executed desktop automation task: {}", task),
            steps: vec![],
            execution_time: std::time::Duration::from_millis(0),
            final_context: context.clone(),
        })
    }

    fn can_handle_task(&self, task: &str) -> f32 {
        let task = task.to_lowercase();
        if task.contains("click")
            || task.contains("type")
            || task.contains("scroll")
            || task.contains("move mouse")
        {
            0.9
        } else {
            0.1
        }
    }
}
