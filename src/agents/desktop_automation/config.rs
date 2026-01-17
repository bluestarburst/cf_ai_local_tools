use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopAutomationConfig {
    pub model_id: Option<String>,
    pub max_iterations: Option<usize>,
    pub custom_prompt: Option<String>,
}

impl Default for DesktopAutomationConfig {
    fn default() -> Self {
        Self {
            model_id: None,
            max_iterations: None,
            custom_prompt: None,
        }
    }
}