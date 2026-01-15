/**
 * LocalStorage Schema and Keys
 * Defines the structure for persisting user data
 */

export const STORAGE_PREFIX = 'cf-ai-presets';

export const STORAGE_KEYS = {
  // Presets
  AGENTS: `${STORAGE_PREFIX}:agents`,
  SYSTEM_PROMPTS: `${STORAGE_PREFIX}:systemPrompts`,
  TOOL_CONFIGS: `${STORAGE_PREFIX}:toolConfigs`,
  WORKSPACES: `${STORAGE_PREFIX}:workspaces`,

  // Session/UI State
  CURRENT_AGENT: `${STORAGE_PREFIX}:currentAgent`,
  RECENT_AGENTS: `${STORAGE_PREFIX}:recentAgents`,
  CONVERSATION_HISTORY: `${STORAGE_PREFIX}:conversationHistory`,

  // Backups
  BACKUP_PREFIX: `${STORAGE_PREFIX}:backup`,
  BACKUP_MANIFEST: `${STORAGE_PREFIX}:backupManifest`,

  // Metadata
  METADATA: `${STORAGE_PREFIX}:metadata`,
};

/**
 * Generate backup key for a preset
 */
export function getBackupKey(presetId: string, version: number): string {
  return `${STORAGE_KEYS.BACKUP_PREFIX}:${presetId}:v${version}`;
}

/**
 * Storage quota warning threshold (in bytes)
 */
export const STORAGE_WARNING_THRESHOLD = 0.9; // 90% full

/**
 * Maximum backups to keep per preset
 */
export const MAX_BACKUPS_PER_PRESET = 5;

/**
 * Backup retention period (milliseconds)
 */
export const BACKUP_RETENTION_DAYS = 30;

/**
 * Parse a stored JSON value safely
 */
export function parseStorageValue<T>(
  value: string | null,
  defaultValue?: T
): T | undefined {
  if (!value) return defaultValue;
  try {
    return JSON.parse(value);
  } catch {
    console.error('Failed to parse storage value:', value);
    return defaultValue;
  }
}

/**
 * Stringify a value for storage
 */
export function stringifyStorageValue(value: any): string {
  return JSON.stringify(value);
}

/**
 * Get localStorage size estimate
 */
export function getStorageSize(): number {
  let size = 0;
  for (const key in localStorage) {
    if (key.startsWith(STORAGE_PREFIX)) {
      size += localStorage[key].length + key.length;
    }
  }
  return size; // in bytes
}

/**
 * Get estimated storage quota usage
 */
export function getStorageUsagePercentage(): number {
  // Most browsers: 5-10MB
  const estimatedQuota = 10 * 1024 * 1024; // 10MB
  const used = getStorageSize();
  return used / estimatedQuota;
}

/**
 * Check if storage is running low
 */
export function isStorageLow(): boolean {
  return getStorageUsagePercentage() > STORAGE_WARNING_THRESHOLD;
}

/**
 * Clear all user presets (not built-in defaults)
 */
export function clearUserPresets(): void {
  const keysToDelete = [
    STORAGE_KEYS.AGENTS,
    STORAGE_KEYS.SYSTEM_PROMPTS,
    STORAGE_KEYS.TOOL_CONFIGS,
    STORAGE_KEYS.WORKSPACES,
  ];

  for (const key of keysToDelete) {
    localStorage.removeItem(key);
  }
}

/**
 * Clear everything including backups
 */
export function clearAllPresets(): void {
  const keysToDelete: string[] = [];
  for (const key in localStorage) {
    if (key.startsWith(STORAGE_PREFIX)) {
      keysToDelete.push(key);
    }
  }

  for (const key of keysToDelete) {
    localStorage.removeItem(key);
  }
}

/**
 * Export all presets as JSON
 */
export function exportAllPresets(): string {
  const presets: any = {
    version: '1.0.0',
    exportedAt: new Date().toISOString(),
    data: {
      agents: localStorage.getItem(STORAGE_KEYS.AGENTS),
      systemPrompts: localStorage.getItem(STORAGE_KEYS.SYSTEM_PROMPTS),
      toolConfigs: localStorage.getItem(STORAGE_KEYS.TOOL_CONFIGS),
      workspaces: localStorage.getItem(STORAGE_KEYS.WORKSPACES),
    },
  };

  return stringifyStorageValue(presets);
}

/**
 * Import presets from JSON
 */
export function importPresets(
  jsonData: string,
  overwrite = false
): {
  success: boolean;
  imported: number;
  skipped: number;
  errors: string[];
} {
  const errors: string[] = [];
  let imported = 0;
  let skipped = 0;

  try {
    const data = parseStorageValue<{ data?: any }>(jsonData);
    if (!data || !data.data) {
      throw new Error('Invalid export format');
    }

    const { agents, systemPrompts, toolConfigs, workspaces } = data.data;

    // Import agents
    if (agents) {
      try {
        const existing = localStorage.getItem(STORAGE_KEYS.AGENTS);
        const existingData = parseStorageValue(existing, {});
        const newData = parseStorageValue(agents, {});

        const merged = overwrite
          ? newData
          : { ...existingData, ...newData };

        localStorage.setItem(STORAGE_KEYS.AGENTS, stringifyStorageValue(merged));
        imported += Object.keys(newData || {}).length;
      } catch (e) {
        errors.push(`Failed to import agents: ${e}`);
      }
    }

    // Similar for other preset types...
    if (systemPrompts) {
      try {
        const existing = localStorage.getItem(STORAGE_KEYS.SYSTEM_PROMPTS);
        const existingData = parseStorageValue(existing, {});
        const newData = parseStorageValue(systemPrompts, {});

        const merged = overwrite
          ? newData
          : { ...existingData, ...newData };

        localStorage.setItem(
          STORAGE_KEYS.SYSTEM_PROMPTS,
          stringifyStorageValue(merged)
        );
        imported += Object.keys(newData || {}).length;
      } catch (e) {
        errors.push(`Failed to import system prompts: ${e}`);
      }
    }

    // Note: toolConfigs and workspaces would be imported similarly
    if (toolConfigs || workspaces) {
      // Future implementation
    }

    return { success: errors.length === 0, imported, skipped, errors };
  } catch (e) {
    return {
      success: false,
      imported: 0,
      skipped: 0,
      errors: [String(e)],
    };
  }
}
