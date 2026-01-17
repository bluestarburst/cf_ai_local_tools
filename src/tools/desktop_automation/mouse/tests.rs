use super::*;
use crate::tools::execution::mock::MockToolContext;

#[tokio::test]
async fn test_move_cursor_tool() {
    let tool = MoveCursor::new();
    let context = MockToolContext::new();
    
    let args = serde_json::json!({
        "x": 100.0,
        "y": 200.0
    });
    
    let result = tool.execute(&args, &context).await;
    assert!(result.is_ok());
    
    let tool_result = result.unwrap();
    assert!(tool_result.success);
    assert!(tool_result.message.contains("Successfully moved cursor"));
}

#[tokio::test]
async fn test_move_cursor_validation() {
    let tool = MoveCursor::new();
    
    // Valid arguments
    let valid_args = serde_json::json!({
        "x": 100.0,
        "y": 200.0
    });
    assert!(tool.validate_args(&valid_args).is_ok());
    
    // Invalid arguments (missing required fields)
    let invalid_args = serde_json::json!({
        "x": 100.0
    });
    assert!(tool.validate_args(&invalid_args).is_err());
}

#[tokio::test]
async fn test_move_cursor_with_optional_params() {
    let tool = MoveCursor::new();
    let context = MockToolContext::new();
    
    let args = serde_json::json!({
        "x": 100.0,
        "y": 200.0,
        "speed": 0.8,
        "smooth": false
    });
    
    let result = tool.execute(&args, &context).await;
    assert!(result.is_ok());
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_move_cursor_workflow() {
        let tool = MoveCursor::new();
        let context = MockToolContext::new();
        
        // Test movement to different positions
        let positions = vec![(0.0, 0.0), (100.0, 100.0), (500.0, 300.0)];
        
        for (x, y) in positions {
            let args = serde_json::json!({"x": x, "y": y});
            let result = tool.execute(&args, &context).await.unwrap();
            assert!(result.success);
        }
    }
}