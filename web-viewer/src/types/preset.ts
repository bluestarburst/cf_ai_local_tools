/**
 * Preset Types (Web Viewer)
 */

import { Metadata } from './agent';

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

export interface AgentPreset extends Preset<any> {
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
