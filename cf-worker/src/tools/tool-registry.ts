/**
 * Dynamic Tool Registry
 * Fetches tools from connected Rust client + Worker-side tools
 */

import { ToolDefinition } from '../types/tool';

/**
 * Worker-side only tools (not executed by Rust client)
 */
const WORKER_TOOLS: Record<string, ToolDefinition> = {
  delegate_to_agent: {
    id: 'delegate_to_agent',
    name: 'Delegate to Agent',
    description: 'Delegate a task to another specialized agent. The agent will execute the task and return the result.',
    category: 'orchestration',
    parameters: [
      {
        name: 'agent_id',
        type: 'string',
        description: 'ID of the agent to delegate to (e.g., desktop-automation-agent, web-research-agent, code-assistant-agent, conversational-agent)',
        required: true,
      },
      {
        name: 'task',
        type: 'string',
        description: 'The task description to send to the agent',
        required: true,
      },
    ],
    returnsObservation: true,
  },
};

// Cache for dynamic tools from Rust client
let cachedClientTools: Record<string, ToolDefinition> = {};
let cacheTimestamp = 0;
const CACHE_TTL_MS = 5000; // 5 second cache

/**
 * Fetch tools from the connected Rust client via Durable Object
 */
export async function fetchClientTools(env: any): Promise<Record<string, ToolDefinition>> {
  try {
    const id = env.CONNECTIONS.idFromName('default');
    const stub = env.CONNECTIONS.get(id);

    const response = await stub.fetch(new Request('http://internal/tools'));
    const data = await response.json() as { connected: boolean; tools: ToolDefinition[] };

    if (!data.connected || !data.tools) {
      return {};
    }

    const toolsMap: Record<string, ToolDefinition> = {};
    for (const tool of data.tools) {
      toolsMap[tool.id] = tool;
    }

    // Update cache
    cachedClientTools = toolsMap;
    cacheTimestamp = Date.now();

    return toolsMap;
  } catch (e) {
    console.error('Failed to fetch client tools:', e);
    return cachedClientTools; // Return cached on error
  }
}

/**
 * Get all available tools (Worker + Client)
 * Uses cache if recent, otherwise fetches fresh
 */
export async function getAllToolsAsync(env: any): Promise<ToolDefinition[]> {
  // Check cache
  if (Date.now() - cacheTimestamp < CACHE_TTL_MS && Object.keys(cachedClientTools).length > 0) {
    return [...Object.values(WORKER_TOOLS), ...Object.values(cachedClientTools)];
  }

  const clientTools = await fetchClientTools(env);
  return [...Object.values(WORKER_TOOLS), ...Object.values(clientTools)];
}

/**
 * Get a tool definition by ID (sync, uses cache)
 * For validation during execution - assumes tools were fetched at request start
 */
export function getTool(toolId: string): ToolDefinition | undefined {
  // Check Worker tools first
  if (WORKER_TOOLS[toolId]) {
    return WORKER_TOOLS[toolId];
  }
  // Then check cached client tools
  return cachedClientTools[toolId];
}

/**
 * Get all tools as array (sync, uses cache)
 */
export function getAllTools(): ToolDefinition[] {
  return [...Object.values(WORKER_TOOLS), ...Object.values(cachedClientTools)];
}

/**
 * Check if a tool is a Worker-side tool (not sent to Rust client)
 */
export function isWorkerTool(toolId: string): boolean {
  // Worker handles: delegation, web_search, fetch_url
  // Client handles: mouse, keyboard, screenshot
  return toolId === 'delegate_to_agent' ||
         toolId === 'web_search' ||
         toolId === 'fetch_url';
}

/**
 * Get tools by category
 */
export function getToolsByCategory(
  category: ToolDefinition['category']
): ToolDefinition[] {
  return getAllTools().filter((t) => t.category === category);
}

/**
 * Validate tool call parameters
 */
export function validateToolCall(
  toolId: string,
  parameters: Record<string, any>
): { valid: boolean; errors: string[] } {
  const tool = getTool(toolId);
  const errors: string[] = [];

  if (!tool) {
    return { valid: false, errors: [`Tool not found: ${toolId}`] };
  }

  for (const param of tool.parameters) {
    if (param.required && !(param.name in parameters)) {
      errors.push(`Missing required parameter: ${param.name}`);
    }

    if (param.name in parameters) {
      const value = parameters[param.name];
      const expectedType = param.type;

      if (expectedType === 'number' && typeof value !== 'number') {
        errors.push(`Parameter ${param.name} must be a number`);
      }
      if (expectedType === 'string' && typeof value !== 'string') {
        errors.push(`Parameter ${param.name} must be a string`);
      }
      if (expectedType === 'boolean' && typeof value !== 'boolean') {
        errors.push(`Parameter ${param.name} must be a boolean`);
      }

      if (param.enum && param.enum.length > 0 && !(param.enum as any[]).includes(value)) {
        errors.push(
          `Parameter ${param.name} must be one of: ${param.enum.join(', ')}`
        );
      }
    }
  }

  return { valid: errors.length === 0, errors };
}

/**
 * Refresh the tool cache (call at start of request)
 */
export async function refreshToolCache(env: any): Promise<void> {
  await fetchClientTools(env);
}
