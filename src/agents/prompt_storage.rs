use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::info;

/// System prompt preset that can be stored and modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub prompt_type: String, // "systemPrompt"
    pub category: String,    // "built-in" or "user-created"
    pub content: String,
    pub metadata: PromptMetadata,
    #[serde(rename = "isLocked")]
    pub is_locked: bool, // true for built-in, false for user-created
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Prompt storage manager - handles persistence and CRUD
pub struct PromptStorage {
    storage_path: PathBuf,
    prompts: HashMap<String, Prompt>,
}

impl PromptStorage {
    /// Create a new prompt storage instance
    pub fn new() -> Result<Self> {
        let storage_path = Self::get_storage_path()?;

        // Ensure directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create storage directory")?;
        }

        let mut storage = Self {
            storage_path,
            prompts: HashMap::new(),
        };

        // Load existing prompts from storage
        storage.load()?;

        Ok(storage)
    }

    /// Get the storage file path
    fn get_storage_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?;
        Ok(config_dir.join("cf_ai_local_tools/prompts.json"))
    }

    /// Load prompts from disk
    fn load(&mut self) -> Result<()> {
        if self.storage_path.exists() {
            let content = fs::read_to_string(&self.storage_path)
                .context("Failed to read prompts file")?;
            let prompts: HashMap<String, Prompt> = serde_json::from_str(&content)
                .context("Failed to parse prompts file")?;
            self.prompts = prompts;
            info!("[PromptStorage] Loaded {} prompts from storage", self.prompts.len());
        } else {
            info!("[PromptStorage] No prompts file found, starting with empty storage");
        }
        Ok(())
    }

    /// Save prompts to disk
    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.prompts)
            .context("Failed to serialize prompts")?;
        fs::write(&self.storage_path, json)
            .context("Failed to write prompts file")?;
        info!("[PromptStorage] Saved {} prompts to storage", self.prompts.len());
        Ok(())
    }

    /// Get all prompts (built-in + user-created)
    pub fn get_all(&self) -> Vec<Prompt> {
        self.prompts.values().cloned().collect()
    }

    /// Get a specific prompt
    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Option<Prompt> {
        self.prompts.get(id).cloned()
    }

    /// Create a new user-created prompt
    pub fn create(&mut self, mut prompt: Prompt) -> Result<Prompt> {
        if self.prompts.contains_key(&prompt.id) {
            anyhow::bail!("Prompt with id '{}' already exists", prompt.id);
        }

        // Ensure it's marked as user-created and not locked
        prompt.category = "user-created".to_string();
        prompt.is_locked = false;

        // Update metadata
        let now = chrono::Utc::now().to_rfc3339();
        prompt.metadata.created_at = now.clone();
        prompt.metadata.updated_at = now;
        prompt.metadata.version = "1.0.0".to_string();

        self.prompts.insert(prompt.id.clone(), prompt.clone());
        self.save()?;

        info!("[PromptStorage] Created prompt: {}", prompt.id);
        Ok(prompt)
    }

    /// Update an existing user-created prompt
    pub fn update(&mut self, id: &str, mut updates: Prompt) -> Result<Prompt> {
        let existing = self.prompts.get(id)
            .context(format!("Prompt '{}' not found", id))?;

        if existing.is_locked {
            anyhow::bail!("Cannot modify locked prompt '{}'", id);
        }

        // Preserve creation time, update modification time
        updates.metadata.created_at = existing.metadata.created_at.clone();
        updates.metadata.updated_at = chrono::Utc::now().to_rfc3339();
        updates.is_locked = false;
        updates.category = "user-created".to_string();

        self.prompts.insert(id.to_string(), updates.clone());
        self.save()?;

        info!("[PromptStorage] Updated prompt: {}", id);
        Ok(updates)
    }

    /// Delete a user-created prompt
    pub fn delete(&mut self, id: &str) -> Result<()> {
        let existing = self.prompts.get(id)
            .context(format!("Prompt '{}' not found", id))?;

        if existing.is_locked {
            anyhow::bail!("Cannot delete locked prompt '{}'", id);
        }

        self.prompts.remove(id);
        self.save()?;

        info!("[PromptStorage] Deleted prompt: {}", id);
        Ok(())
    }

    /// Clear all user-created prompts (keep built-in)
    pub fn clear_user_created(&mut self) -> Result<()> {
        self.prompts.retain(|_, p| p.is_locked);
        self.save()?;
        info!("[PromptStorage] Cleared all user-created prompts");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_create_prompt() {
        // Tests would go here
    }
}
