/**
 * Preset Types - for saving agent configs, prompts, and tool configs
 */

import { Metadata, Agent } from './agent';

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
  isLocked?: boolean; // Built-in presets are locked
}

export interface AgentPreset extends Preset<Agent> {
  type: 'agent';
}

export interface SystemPromptPreset extends Preset<string> {
  type: 'systemPrompt';
}

export interface ToolConfigPreset extends Preset<Record<string, boolean>> {
  type: 'toolConfig';
}

export interface WorkspacePreset extends Preset<WorkspaceContent> {
  type: 'workspace';
}

export interface WorkspaceContent {
  agentIds: string[];
  promptIds: string[];
  description?: string;
}

export type AnyPreset =
  | AgentPreset
  | SystemPromptPreset
  | ToolConfigPreset
  | WorkspacePreset;

export interface PresetStorage {
  agents: Map<string, AgentPreset>;
  systemPrompts: Map<string, SystemPromptPreset>;
  toolConfigs: Map<string, ToolConfigPreset>;
  workspaces: Map<string, WorkspacePreset>;
}

export interface PresetExport {
  version: string;
  exportedAt: string;
  presets: AnyPreset[];
  metadata: {
    totalCount: number;
    byType: Record<PresetType, number>;
  };
}
