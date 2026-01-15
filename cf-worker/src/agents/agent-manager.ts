/**
 * Agent Manager Service
 * Handles CRUD operations for agents
 */

import { Agent, AgentConfig, Metadata } from '../types/agent';
import { ToolReference } from '../types/agent';
import { validateToolCall } from '../tools/tool-registry';

// In-memory agent registry (could be extended to use KV storage)
const agentRegistry = new Map<string, Agent>();

/**
 * Generate a unique agent ID
 */
function generateId(): string {
  return `agent-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Create metadata for an agent
 */
function createMetadata(version = '1.0.0'): Metadata {
  const now = new Date().toISOString();
  return {
    createdAt: now,
    updatedAt: now,
    version,
  };
}

/**
 * Create a new agent
 */
export function createAgent(config: AgentConfig): Agent {
  const agent: Agent = {
    id: generateId(),
    name: config.name,
    purpose: config.purpose,
    systemPrompt: config.systemPrompt,
    tools: config.tools,
    modelId: config.modelId,
    maxIterations: config.maxIterations,
    metadata: createMetadata(),
  };

  agentRegistry.set(agent.id, agent);
  return agent;
}

/**
 * Get an agent by ID
 */
export function getAgent(agentId: string): Agent | undefined {
  return agentRegistry.get(agentId);
}

/**
 * Get all agents
 */
export function getAllAgents(): Agent[] {
  return Array.from(agentRegistry.values());
}

/**
 * Update an existing agent
 */
export function updateAgent(
  agentId: string,
  updates: Partial<AgentConfig>
): Agent | undefined {
  const agent = agentRegistry.get(agentId);
  if (!agent) {
    return undefined;
  }

  // Update fields
  if (updates.name) agent.name = updates.name;
  if (updates.purpose) agent.purpose = updates.purpose;
  if (updates.systemPrompt) agent.systemPrompt = updates.systemPrompt;
  if (updates.tools) agent.tools = updates.tools;
  if (updates.modelId) agent.modelId = updates.modelId;
  if (updates.maxIterations !== undefined)
    agent.maxIterations = updates.maxIterations;

  // Update metadata
  agent.metadata.updatedAt = new Date().toISOString();
  const [major, minor, patch] = agent.metadata.version.split('.').map(Number);
  agent.metadata.version = `${major}.${minor}.${patch + 1}`;

  agentRegistry.set(agentId, agent);
  return agent;
}

/**
 * Delete an agent
 */
export function deleteAgent(agentId: string): boolean {
  const agent = agentRegistry.get(agentId);
  if (!agent) {
    return false;
  }

  // Don't allow deleting default agents
  if (agent.isDefault) {
    throw new Error('Cannot delete default agent');
  }

  return agentRegistry.delete(agentId);
}

/**
 * Duplicate/clone an agent
 */
export function duplicateAgent(agentId: string, newName?: string): Agent | undefined {
  const original = agentRegistry.get(agentId);
  if (!original) {
    return undefined;
  }

  const duplicate: Agent = {
    id: generateId(),
    name: newName || `${original.name} (Copy)`,
    purpose: original.purpose,
    systemPrompt: original.systemPrompt,
    tools: [...original.tools], // Copy array
    modelId: original.modelId,
    maxIterations: original.maxIterations,
    metadata: createMetadata(),
  };

  agentRegistry.set(duplicate.id, duplicate);
  return duplicate;
}

/**
 * Register a default agent (from presets)
 */
export function registerDefaultAgent(agent: Agent): void {
  agent.isDefault = true;
  agentRegistry.set(agent.id, agent);
}

/**
 * Validate agent configuration
 */
export function validateAgent(config: AgentConfig): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  if (!config.name || config.name.trim().length === 0) {
    errors.push('Agent name is required');
  }

  if (!config.purpose || config.purpose.trim().length === 0) {
    errors.push('Agent purpose is required');
  }

  if (!config.systemPrompt || config.systemPrompt.trim().length === 0) {
    errors.push('System prompt is required');
  }

  if (!config.tools || config.tools.length === 0) {
    errors.push('At least one tool must be enabled');
  }

  if (!config.modelId || config.modelId.trim().length === 0) {
    errors.push('Model ID is required');
  }

  if (
    config.maxIterations === undefined ||
    config.maxIterations < 1 ||
    config.maxIterations > 20
  ) {
    errors.push('Max iterations must be between 1 and 20');
  }

  return { valid: errors.length === 0, errors };
}

/**
 * Search agents by name or purpose
 */
export function searchAgents(query: string): Agent[] {
  const lowerQuery = query.toLowerCase();
  return Array.from(agentRegistry.values()).filter(
    (agent) =>
      agent.name.toLowerCase().includes(lowerQuery) ||
      agent.purpose.toLowerCase().includes(lowerQuery)
  );
}

/**
 * Get agents by tag (if metadata includes tags)
 */
export function getAgentsByTag(tag: string): Agent[] {
  return Array.from(agentRegistry.values()).filter(
    (agent) => agent.metadata.tags && agent.metadata.tags.includes(tag)
  );
}

/**
 * Clear all non-default agents
 */
export function clearUserAgents(): void {
  for (const [id, agent] of agentRegistry.entries()) {
    if (!agent.isDefault) {
      agentRegistry.delete(id);
    }
  }
}
