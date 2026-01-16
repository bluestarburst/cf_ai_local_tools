/**
 * Default Presets (Web Viewer)
 * Mirror of backend presets for client-side availability
 */

export interface Metadata {
  createdAt: string;
  updatedAt: string;
  version: string;
  author?: string;
  tags?: string[];
}

export interface ToolReference {
  toolId: string;
  enabled: boolean;
}

export interface Agent {
  id: string;
  name: string;
  purpose: string;
  systemPrompt: string;
  tools: ToolReference[];
  modelId: string;
  maxIterations: number;
  metadata: Metadata;
  isDefault?: boolean;
  isPinned?: boolean;
  isDeletable?: boolean; // false = cannot be deleted (default true)
  availableAgentIds?: string[]; // Agent IDs available for delegation
}

export type PresetType = 'agent' | 'systemPrompt' | 'toolConfig' | 'workspace';
export type PresetCategory = 'built-in' | 'user-created' | 'imported';

export interface Preset<T = any> {
  id: string;
  name: string;
  description: string;
  type: PresetType;
  category: PresetCategory;
  content: T;
  metadata: Metadata;
  isLocked?: boolean;
}

export interface AgentPreset extends Preset<Agent> {
  type: 'agent';
}

export interface SystemPromptPreset extends Preset<string> {
  type: 'systemPrompt';
}

/**
 * ⚠️ DEPRECATED: Default agents and prompts are now provided by the Rust backend
 * 
 * The web viewer no longer maintains hardcoded defaults. All agents are loaded
 * from the Rust local app via WebSocket on connect:
 * 1. Web viewer connects to WebSocket
 * 2. Rust backend sends handshake with all available agents
 * 3. Web viewer loads agents from backend only
 * 
 * To add/modify agents: Edit src/agents/presets.rs in the Rust backend
 */

// These exports are deprecated but kept for backward compatibility during migration
export const DEFAULT_PROMPTS: Record<string, SystemPromptPreset> = {};
export const DEFAULT_AGENTS: Record<string, AgentPreset> = {};

/**
 * @deprecated Load agents from Rust backend via WebSocket instead
 */
export function getDefaultAgent(agentId: string): AgentPreset | undefined {
  console.warn(
    'getDefaultAgent() is deprecated. Load agents from Rust backend via WebSocket.'
  );
  return undefined;
}

/**
 * @deprecated Load agents from Rust backend via WebSocket instead
 */
export function getAllDefaultAgents(): AgentPreset[] {
  console.warn(
    'getAllDefaultAgents() is deprecated. Load agents from Rust backend via WebSocket.'
  );
  return [];
}

/**
 * @deprecated Load prompts from Rust backend via WebSocket instead
 */
export function getDefaultPrompt(
  promptId: string
): SystemPromptPreset | undefined {
  console.warn(
    'getDefaultPrompt() is deprecated. Load agents from Rust backend via WebSocket.'
  );
  return undefined;
}

/**
 * @deprecated Load prompts from Rust backend via WebSocket instead
 */
export function getAllDefaultPrompts(): SystemPromptPreset[] {
  console.warn(
    'getAllDefaultPrompts() is deprecated. Load agents from Rust backend via WebSocket.'
  );
  return [];
}

export function interpolatePrompt(
  template: string,
  variables: Record<string, string>
): string {
  let result = template;
  for (const [key, value] of Object.entries(variables)) {
    result = result.replace(new RegExp(`\\{${key}\\}`, 'g'), value);
  }
  return result;
}
