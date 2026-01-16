mod executor;

pub use executor::{AutomationHandler, Command, Response};
use executor::create_executor;

#[cfg(test)]
mod test_integration;

use serde_json::json;
use crate::agents::{ToolDefinition, ToolParameter};
use anyhow::Result;

/// Get all mouse/keyboard automation tools
pub fn get_mouse_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            id: "mouse_move".to_string(),
            name: "Mouse Move".to_string(),
            description: "Move the mouse cursor to specified coordinates".to_string(),
            category: "mouse".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "x".to_string(),
                    param_type: "number".to_string(),
                    description: "X coordinate to move to".to_string(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "y".to_string(),
                    param_type: "number".to_string(),
                    description: "Y coordinate to move to".to_string(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
                ToolParameter {
                    name: "duration".to_string(),
                    param_type: "number".to_string(),
                    description: "Duration of movement in seconds".to_string(),
                    required: false,
                    enum_values: None,
                    default: Some(json!(1.0)),
                },
            ],
            returns_observation: true,
        },
        ToolDefinition {
            id: "mouse_click".to_string(),
            name: "Mouse Click".to_string(),
            description: "Click a mouse button at current position".to_string(),
            category: "mouse".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "button".to_string(),
                    param_type: "string".to_string(),
                    description: "Which button to click".to_string(),
                    required: true,
                    enum_values: Some(vec!["left".to_string(), "right".to_string(), "middle".to_string()]),
                    default: None,
                },
            ],
            returns_observation: true,
        },
        ToolDefinition {
            id: "mouse_scroll".to_string(),
            name: "Mouse Scroll".to_string(),
            description: "Scroll the mouse wheel in a direction".to_string(),
            category: "mouse".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "direction".to_string(),
                    param_type: "string".to_string(),
                    description: "Direction to scroll".to_string(),
                    required: true,
                    enum_values: Some(vec!["up".to_string(), "down".to_string(), "left".to_string(), "right".to_string()]),
                    default: None,
                },
                ToolParameter {
                    name: "intensity".to_string(),
                    param_type: "number".to_string(),
                    description: "How much to scroll (1-10)".to_string(),
                    required: false,
                    enum_values: None,
                    default: Some(json!(3)),
                },
            ],
            returns_observation: true,
        },
    ]
}

/// Get all keyboard automation tools
pub fn get_keyboard_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            id: "keyboard_input".to_string(),
            name: "Keyboard Input".to_string(),
            description: "Type text using the keyboard".to_string(),
            category: "keyboard".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "text".to_string(),
                    param_type: "string".to_string(),
                    description: "Text to type".to_string(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
            ],
            returns_observation: true,
        },
        ToolDefinition {
            id: "keyboard_command".to_string(),
            name: "Keyboard Command".to_string(),
            description: "Execute a keyboard command or key combination".to_string(),
            category: "keyboard".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "command".to_string(),
                    param_type: "string".to_string(),
                    description: "Keyboard command to execute (e.g., 'cmd+c', 'ctrl+v', 'Return')".to_string(),
                    required: true,
                    enum_values: None,
                    default: None,
                },
            ],
            returns_observation: true,
        },
    ]
}

/// Get all computer automation tools (mouse, keyboard, system)
pub fn get_all_automation_tools() -> Vec<ToolDefinition> {
    let mut tools = get_mouse_tools();
    tools.extend(get_keyboard_tools());
    
    // Add system tools
    tools.push(ToolDefinition {
        id: "get_mouse_position".to_string(),
        name: "Get Mouse Position".to_string(),
        description: "Get the current position of the mouse cursor".to_string(),
        category: "mouse".to_string(),
        parameters: vec![],
        returns_observation: true,
    });
    
    tools.push(ToolDefinition {
        id: "take_screenshot".to_string(),
        name: "Take Screenshot".to_string(),
        description: "Capture a screenshot of the current screen".to_string(),
        category: "system".to_string(),
        parameters: vec![],
        returns_observation: true,
    });
    
    tools
}

/// Execute a computer automation tool
/// 
/// # Arguments
/// * `tool_name` - The ID of the tool to execute
/// * `arguments` - JSON arguments for the tool
/// * `handler` - The AutomationHandler instance to use for execution
/// 
/// # Returns
/// * `Ok(String)` - Success message or result
/// * `Err(anyhow::Error)` - Execution error
pub fn execute_automation_tool(
    tool_name: &str,
    arguments: &serde_json::Value,
    handler: &AutomationHandler,
) -> Result<String> {
    // Verify this is an automation tool
    if !get_all_automation_tools()
        .iter()
        .any(|t| t.id == tool_name)
    {
        return Err(anyhow::anyhow!("Unknown automation tool: {}", tool_name));
    }
    
    // Use the executor to handle the tool
    let executor = create_executor(handler);
    executor(tool_name, arguments)
}

