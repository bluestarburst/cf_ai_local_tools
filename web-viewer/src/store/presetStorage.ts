/**
 * LocalStorage Management Utilities
 * Handles preset persistence with versioning and backups
 */

import {
  STORAGE_KEYS,
  parseStorageValue,
  stringifyStorageValue,
  getBackupKey,
  MAX_BACKUPS_PER_PRESET,
} from './storageSchema';
import { AnyPreset, AgentPreset, SystemPromptPreset } from '../types/preset';
import { Agent } from '../types/agent';

/**
 * Save an agent preset
 */
export function saveAgentPreset(preset: AgentPreset): void {
  const existing = getAllAgentPresets();
  existing[preset.id] = preset;
  localStorage.setItem(STORAGE_KEYS.AGENTS, stringifyStorageValue(existing));

  // Create backup
  createBackup(preset);
}

/**
 * Get all agent presets
 */
export function getAllAgentPresets(): Record<string, AgentPreset> {
  const data = localStorage.getItem(STORAGE_KEYS.AGENTS);
  return parseStorageValue(data, {}) || {};
}

/**
 * Get a specific agent preset
 */
export function getAgentPreset(id: string): AgentPreset | undefined {
  const presets = getAllAgentPresets();
  return presets[id];
}

/**
 * Delete an agent preset
 */
export function deleteAgentPreset(id: string): boolean {
  const presets = getAllAgentPresets();
  const preset = presets[id];

  if (!preset) return false;
  if (preset.isLocked) {
    throw new Error('Cannot delete locked preset');
  }

  delete presets[id];
  localStorage.setItem(STORAGE_KEYS.AGENTS, stringifyStorageValue(presets));
  return true;
}

/**
 * Save a system prompt preset
 */
export function saveSystemPromptPreset(preset: SystemPromptPreset): void {
  const existing = getAllSystemPromptPresets();
  existing[preset.id] = preset;
  localStorage.setItem(
    STORAGE_KEYS.SYSTEM_PROMPTS,
    stringifyStorageValue(existing)
  );

  createBackup(preset);
}

/**
 * Get all system prompt presets
 */
export function getAllSystemPromptPresets(): Record<string, SystemPromptPreset> {
  const data = localStorage.getItem(STORAGE_KEYS.SYSTEM_PROMPTS);
  return parseStorageValue(data, {}) || {};
}

/**
 * Get a specific system prompt preset
 */
export function getSystemPromptPreset(id: string): SystemPromptPreset | undefined {
  const presets = getAllSystemPromptPresets();
  return presets[id];
}

/**
 * Delete a system prompt preset
 */
export function deleteSystemPromptPreset(id: string): boolean {
  const presets = getAllSystemPromptPresets();
  const preset = presets[id];

  if (!preset) return false;
  if (preset.isLocked) {
    throw new Error('Cannot delete locked preset');
  }

  delete presets[id];
  localStorage.setItem(
    STORAGE_KEYS.SYSTEM_PROMPTS,
    stringifyStorageValue(presets)
  );
  return true;
}

/**
 * Create a backup of a preset
 */
function createBackup(preset: AnyPreset): void {
  try {
    const backupManifest = getBackupManifest();
    const presetBackups = backupManifest[preset.id] || [];

    // Create new backup
    const version = presetBackups.length + 1;
    const backupKey = getBackupKey(preset.id, version);
    localStorage.setItem(backupKey, stringifyStorageValue(preset));

    // Update manifest
    presetBackups.push({
      version,
      timestamp: new Date().toISOString(),
      key: backupKey,
    });

    // Keep only last N backups
    if (presetBackups.length > MAX_BACKUPS_PER_PRESET) {
      const toDelete = presetBackups.shift();
      if (toDelete) {
        localStorage.removeItem(toDelete.key);
      }
    }

    backupManifest[preset.id] = presetBackups;
    localStorage.setItem(
      STORAGE_KEYS.BACKUP_MANIFEST,
      stringifyStorageValue(backupManifest)
    );
  } catch (e) {
    console.error('Failed to create backup:', e);
  }
}

/**
 * Get backup manifest
 */
function getBackupManifest(): Record<
  string,
  Array<{ version: number; timestamp: string; key: string }>
> {
  const data = localStorage.getItem(STORAGE_KEYS.BACKUP_MANIFEST);
  return parseStorageValue(data, {}) || {};
}

/**
 * Restore from backup
 */
export function restoreFromBackup(
  presetId: string,
  version: number
): AnyPreset | undefined {
  const backupKey = getBackupKey(presetId, version);
  const data = localStorage.getItem(backupKey);
  return parseStorageValue(data);
}

/**
 * Get backup history for a preset
 */
export function getBackupHistory(presetId: string): Array<{
  version: number;
  timestamp: string;
}> {
  const manifest = getBackupManifest();
  return (manifest[presetId] || []).map(({ version, timestamp }) => ({
    version,
    timestamp,
  }));
}

/**
 * Set current agent
 */
export function setCurrentAgent(agent: Agent): void {
  localStorage.setItem(STORAGE_KEYS.CURRENT_AGENT, stringifyStorageValue(agent));

  // Update recent agents
  const recent = getRecentAgents();
  const filtered = recent.filter((id) => id !== agent.id);
  filtered.unshift(agent.id);
  const limited = filtered.slice(0, 10); // Keep last 10

  localStorage.setItem(STORAGE_KEYS.RECENT_AGENTS, stringifyStorageValue(limited));
}

/**
 * Get current agent
 */
export function getCurrentAgent(): Agent | undefined {
  const data = localStorage.getItem(STORAGE_KEYS.CURRENT_AGENT);
  return parseStorageValue(data);
}

/**
 * Get recent agent IDs
 */
export function getRecentAgents(): string[] {
  const data = localStorage.getItem(STORAGE_KEYS.RECENT_AGENTS);
  return parseStorageValue(data, []) || [];
}

/**
 * Save conversation history
 */
export function saveConversationHistory(
  agentId: string,
  messages: Array<{ role: string; content: string }>
): void {
  const history = getAllConversationHistory();
  history[agentId] = messages;
  localStorage.setItem(
    STORAGE_KEYS.CONVERSATION_HISTORY,
    stringifyStorageValue(history)
  );
}

/**
 * Get conversation history for agent
 */
export function getConversationHistory(
  agentId: string
): Array<{ role: string; content: string }> {
  const history = getAllConversationHistory();
  return history[agentId] || [];
}

/**
 * Get all conversation history
 */
function getAllConversationHistory(): Record<
  string,
  Array<{ role: string; content: string }>
> {
  const data = localStorage.getItem(STORAGE_KEYS.CONVERSATION_HISTORY);
  return (parseStorageValue(data, {}) as Record<string, Array<{ role: string; content: string }>>) || {};
}

/**
 * Clear conversation history for agent
 */
export function clearConversationHistory(agentId: string): void {
  const history = getAllConversationHistory();
  delete history[agentId];
  localStorage.setItem(
    STORAGE_KEYS.CONVERSATION_HISTORY,
    stringifyStorageValue(history)
  );
}

/**
 * Generate unique ID
 */
export function generateId(prefix = 'preset'): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Check if preset with name exists
 */
export function presetNameExists(name: string, type: 'agent' | 'systemPrompt'): boolean {
  if (type === 'agent') {
    const presets = getAllAgentPresets();
    return Object.values(presets).some((p) => p.name === name);
  } else {
    const presets = getAllSystemPromptPresets();
    return Object.values(presets).some((p) => p.name === name);
  }
}

/**
 * Get unique preset name
 */
export function getUniquePresetName(
  baseName: string,
  type: 'agent' | 'systemPrompt'
): string {
  let name = baseName;
  let counter = 1;

  while (presetNameExists(name, type)) {
    name = `${baseName} (${counter})`;
    counter++;
  }

  return name;
}
