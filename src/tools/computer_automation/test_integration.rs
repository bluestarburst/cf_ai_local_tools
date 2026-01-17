/// Integration tests for computer automation tools
/// Run with: cargo test --test computer_automation_integration -- --ignored

#[cfg(test)]
mod tests {
    use crate::tools::computer_automation::{execute_automation_tool, AutomationHandler};
    use serde_json::json;

    #[test]
    #[ignore] // Requires GUI environment
    fn test_mouse_move_basic() {
        let handler = AutomationHandler::new().expect("Failed to create handler");

        // Test moving mouse to a position
        let result = execute_automation_tool(
            "mouse_move",
            &json!({
                "x": 500,
                "y": 600
            }),
            &handler,
        );

        assert!(
            result.is_ok(),
            "mouse_move should succeed: {:?}",
            result.err()
        );
        let result_str = result.unwrap();
        assert!(result_str.contains("500") && result_str.contains("600"));
    }

    #[test]
    #[ignore] // Requires GUI environment
    fn test_get_mouse_position() {
        let handler = AutomationHandler::new().expect("Failed to create handler");

        let result = execute_automation_tool("get_mouse_position", &json!({}), &handler);

        assert!(result.is_ok(), "get_mouse_position should succeed");
        let result_str = result.unwrap();
        // Should contain x and y coordinates
        assert!(result_str.contains("x") || result_str.contains("position"));
    }

    #[test]
    #[ignore] // Requires GUI environment
    fn test_mouse_move_with_duration() {
        let handler = AutomationHandler::new().expect("Failed to create handler");

        let result = execute_automation_tool(
            "mouse_move",
            &json!({
                "x": 100,
                "y": 100,
                "duration": 0.5
            }),
            &handler,
        );

        assert!(result.is_ok(), "mouse_move with duration should succeed");
    }

    #[test]
    #[ignore] // Requires GUI environment
    fn test_mouse_move_sequence() {
        let handler = AutomationHandler::new().expect("Failed to create handler");

        // Move to first position
        let result1 = execute_automation_tool("mouse_move", &json!({"x": 500, "y": 600}), &handler);
        assert!(result1.is_ok());

        std::thread::sleep(std::time::Duration::from_millis(1200));

        // Get position
        let pos1 = execute_automation_tool("get_mouse_position", &json!({}), &handler);
        assert!(pos1.is_ok());

        // Move to second position
        let result2 = execute_automation_tool("mouse_move", &json!({"x": 100, "y": 100}), &handler);
        assert!(result2.is_ok());

        std::thread::sleep(std::time::Duration::from_millis(1200));

        // Get position again
        let pos2 = execute_automation_tool("get_mouse_position", &json!({}), &handler);
        assert!(pos2.is_ok());

        // Positions should be different
        assert_ne!(pos1.unwrap(), pos2.unwrap());
    }

    #[test]
    fn test_invalid_tool_name() {
        let handler = AutomationHandler::new().expect("Failed to create handler");

        let result = execute_automation_tool("invalid_tool", &json!({}), &handler);

        assert!(result.is_err(), "Should reject invalid tool name");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown automation tool"));
    }

    #[test]
    #[ignore] // Requires GUI environment
    fn test_mouse_click() {
        let handler = AutomationHandler::new().expect("Failed to create handler");

        let result = execute_automation_tool("mouse_click", &json!({"button": "left"}), &handler);

        assert!(result.is_ok(), "mouse_click should succeed");
    }

    #[test]
    #[ignore] // Requires GUI environment
    fn test_keyboard_input() {
        let handler = AutomationHandler::new().expect("Failed to create handler");

        let result =
            execute_automation_tool("keyboard_input", &json!({"text": "hello world"}), &handler);

        assert!(result.is_ok(), "keyboard_input should succeed");
    }
}
