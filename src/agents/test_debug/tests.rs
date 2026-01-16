// Integration tests for Test & Debug Agent

use crate::agents::presets::Metadata;

/// Helper function to create a test agent
fn create_test_agent() -> super::super::presets::Agent {
    let metadata = Metadata {
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
        author: Some("test".to_string()),
        tags: None,
    };
    super::create_agent(metadata)
}

#[tokio::test]
#[ignore]
async fn test_debug_agent_follows_literal_instructions() {
    // Placeholder test - verify agent follows literal instructions
    let agent = create_test_agent();
    assert!(agent.system_prompt.contains("LITERALLY"));
}
