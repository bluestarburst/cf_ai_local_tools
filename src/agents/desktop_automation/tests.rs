use crate::agents::builtin::desktop_automation::{DesktopAutomationAgent, DesktopAutomationConfig};
use crate::agents::runtime::{Agent, AgentContext};
use crate::llm::mock::MockLLMClient;
use tokio_test;

#[tokio::test]
async fn test_desktop_automation_agent_creation() {
    let agent = DesktopAutomationAgent::new();
    
    assert_eq!(agent.id(), "desktop-automation-agent");
    assert_eq!(agent.name(), "Desktop Automation Agent");
    assert!(!agent.capabilities().is_empty());
    assert!(!agent.tool_dependencies().is_empty());
}

#[tokio::test]
async fn test_desktop_automation_agent_with_config() {
    let config = DesktopAutomationConfig {
        model_id: Some("@cf/test/model".to_string()),
        max_iterations: Some(5),
        custom_prompt: Some("Custom test prompt".to_string()),
    };
    
    let agent = DesktopAutomationAgent::with_config(config);
    
    assert_eq!(agent.reasoning_config.model_id, "@cf/test/model");
    assert_eq!(agent.reasoning_config.max_iterations, 5);
    assert_eq!(agent.system_prompt, "Custom test prompt");
}

#[tokio::test]
async fn test_task_scoring() {
    let agent = DesktopAutomationAgent::new();
    
    // High confidence for desktop automation tasks
    assert!(agent.can_handle_task("Click the mouse at coordinates 100, 200") > 0.5);
    assert!(agent.can_handle_task("Type 'Hello World' in the active window") > 0.5);
    assert!(agent.can_handle_task("Take a screenshot of the desktop") > 0.5);
    
    // Low confidence for unrelated tasks
    assert!(agent.can_handle_task("Search the web for information") < 0.2);
    assert!(agent.can_handle_task("Write Python code for data analysis") < 0.2);
}

#[tokio::test]
async fn test_agent_execution() {
    let agent = DesktopAutomationAgent::new();
    let context = AgentContext::new();
    let llm = MockLLMClient::new();
    let thinking_engine = crate::agents::thinking::ThinkingEngine::new();
    
    let result = agent
        .execute(
            "Move mouse to position (100, 200)",
            &context,
            &llm,
            &thinking_engine,
        )
        .await;
    
    assert!(result.is_ok());
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_desktop_automation_workflow() {
        let agent = DesktopAutomationAgent::new();
        let context = AgentContext::new();
        let llm = MockLLMClient::new();
        let thinking_engine = crate::agents::thinking::ThinkingEngine::new();
        
        // Test a complete desktop automation workflow
        let task = "Open calculator by clicking the Start button, then typing 'calculator'";
        
        let result = agent
            .execute(task, &context, &llm, &thinking_engine)
            .await;
        
        assert!(result.is_ok());
        
        let agent_result = result.unwrap();
        assert!(agent_result.success);
        assert!(!agent_result.steps.is_empty());
        
        // Verify that appropriate tools were called
        let tool_calls: Vec<String> = agent_result
            .steps
            .iter()
            .filter_map(|step| step.tool_call.as_ref())
            .map(|call| call.tool_name.clone())
            .collect();
        
        assert!(tool_calls.contains(&"mouse_click".to_string()));
        assert!(tool_calls.contains(&"keyboard_type".to_string()));
    }
}