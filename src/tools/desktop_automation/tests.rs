use super::*;
use crate::tools::execution::mock::MockToolContext;

#[tokio::test]
async fn test_mouse_tools() {
    let context = MockToolContext::new();
    
    // Test MoveCursor
    let move_tool = MoveCursor::new();
    let move_args = serde_json::json!({
        "x": 100.0,
        "y": 200.0,
        "speed": 0.8
    });
    let move_result = move_tool.execute(&move_args, &context).await.unwrap();
    assert!(move_result.success);
    
    // Test Click
    let click_tool = Click::new();
    let click_args = serde_json::json!({
        "button": "left",
        "double_click": false
    });
    let click_result = click_tool.execute(&click_args, &context).await.unwrap();
    assert!(click_result.success);
    
    // Test Scroll
    let scroll_tool = Scroll::new();
    let scroll_args = serde_json::json!({
        "direction": "down",
        "amount": 3
    });
    let scroll_result = scroll_tool.execute(&scroll_args, &context).await.unwrap();
    assert!(scroll_result.success);
}

#[tokio::test]
async fn test_keyboard_tools() {
    let context = MockToolContext::new();
    
    // Test TypeText
    let type_tool = TypeText::new();
    let type_args = serde_json::json!({
        "text": "Hello World",
        "delay_ms": 100,
        "auto_enter": true
    });
    let type_result = type_tool.execute(&type_args, &context).await.unwrap();
    assert!(type_result.success);
    
    // Test Hotkey
    let hotkey_tool = Hotkey::new();
    let hotkey_args = serde_json::json!({
        "keys": ["ctrl", "c"],
        "hold_ms": 100
    });
    let hotkey_result = hotkey_tool.execute(&hotkey_args, &context).await.unwrap();
    assert!(hotkey_result.success);
}

#[tokio::test]
async fn test_screen_tools() {
    let context = MockToolContext::new();
    
    // Test Screenshot
    let screenshot_tool = Screenshot::new();
    let screenshot_args = serde_json::json!({
        "format": "png",
        "region": {
            "x": 0,
            "y": 0,
            "width": 800,
            "height": 600
        }
    });
    let screenshot_result = screenshot_tool.execute(&screenshot_args, &context).await.unwrap();
    assert!(screenshot_result.success);
    
    // Test GetPosition
    let position_tool = GetPosition::new();
    let position_args = serde_json::json!({});
    let position_result = position_tool.execute(&position_args, &context).await.unwrap();
    assert!(position_result.success);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_complete_automation_workflow() {
        let context = MockToolContext::new();
        
        // 1. Get current position
        let position_tool = GetPosition::new();
        let position_result = position_tool.execute(&serde_json::json!({}), &context).await.unwrap();
        assert!(position_result.success);
        
        // 2. Move to specific position
        let move_tool = MoveCursor::new();
        let move_result = move_tool.execute(
            &serde_json::json!({"x": 100.0, "y": 200.0}), 
            &context
        ).await.unwrap();
        assert!(move_result.success);
        
        // 3. Click at position
        let click_tool = Click::new();
        let click_result = click_tool.execute(
            &serde_json::json!({"button": "left"}), 
            &context
        ).await.unwrap();
        assert!(click_result.success);
        
        // 4. Type text
        let type_tool = TypeText::new();
        let type_result = type_tool.execute(
            &serde_json::json!({"text": "Hello World"}), 
            &context
        ).await.unwrap();
        assert!(type_result.success);
        
        // 5. Take screenshot to verify
        let screenshot_tool = Screenshot::new();
        let screenshot_result = screenshot_tool.execute(
            &serde_json::json!({"format": "png"}), 
            &context
        ).await.unwrap();
        assert!(screenshot_result.success);
    }
}