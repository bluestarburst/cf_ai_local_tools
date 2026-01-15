/**
 * Tool Utilities
 * Helper functions for tool operations
 */

import { ToolDefinition } from '../types/tool';

const AVAILABLE_TOOLS: ToolDefinition[] = [
  {
    id: 'mouse_move',
    name: 'Move Mouse',
    description: 'Move cursor to position',
    category: 'mouse',
    parameters: [
      { name: 'x', type: 'number', description: 'X coordinate', required: true },
      { name: 'y', type: 'number', description: 'Y coordinate', required: true },
      { name: 'duration', type: 'number', description: 'Time in seconds', required: false, default: 1.0 },
    ],
    returnsObservation: true,
  },
  {
    id: 'mouse_click',
    name: 'Click Mouse',
    description: 'Click mouse button',
    category: 'mouse',
    parameters: [
      { name: 'button', type: 'string', description: 'Which button', required: true },
      { name: 'clicks', type: 'number', description: 'Number of clicks', required: false, default: 1 },
    ],
    returnsObservation: true,
  },
  {
    id: 'mouse_scroll',
    name: 'Scroll Mouse',
    description: 'Scroll in direction',
    category: 'mouse',
    parameters: [
      { name: 'direction', type: 'string', description: 'Direction to scroll', required: true },
      { name: 'intensity', type: 'number', description: 'Scroll intensity', required: false, default: 3 },
    ],
    returnsObservation: true,
  },
  {
    id: 'keyboard_input',
    name: 'Type Text',
    description: 'Type text on keyboard',
    category: 'keyboard',
    parameters: [
      { name: 'text', type: 'string', description: 'Text to type', required: true },
      { name: 'interval', type: 'number', description: 'Delay between keystrokes', required: false, default: 10 },
    ],
    returnsObservation: true,
  },
  {
    id: 'keyboard_command',
    name: 'Key Command',
    description: 'Execute keyboard command',
    category: 'keyboard',
    parameters: [
      { name: 'command', type: 'string', description: 'Keyboard command', required: true },
    ],
    returnsObservation: true,
  },
  {
    id: 'get_mouse_position',
    name: 'Get Mouse Position',
    description: 'Get current cursor position',
    category: 'system',
    parameters: [],
    returnsObservation: true,
  },
  {
    id: 'take_screenshot',
    name: 'Screenshot',
    description: 'Capture screenshot',
    category: 'system',
    parameters: [
      { name: 'region', type: 'string', description: 'Region to capture', required: false, default: 'full' },
    ],
    returnsObservation: true,
  },
  {
    id: 'web_search',
    name: 'Web Search',
    description: 'Search the web using SearXNG',
    category: 'search',
    parameters: [
      { name: 'query', type: 'string', description: 'Search query', required: true },
      { name: 'time_range', type: 'string', description: 'Time range filter', required: false },
      { name: 'language', type: 'string', description: 'Language code', required: false },
      { name: 'detailed', type: 'boolean', description: 'Comprehensive search', required: false, default: false },
    ],
    returnsObservation: true,
  },
  {
    id: 'fetch_url',
    name: 'Fetch URL',
    description: 'Fetch and parse webpage content',
    category: 'search',
    parameters: [
      { name: 'url', type: 'string', description: 'URL to fetch', required: true },
      { name: 'extract_type', type: 'string', description: 'Content to extract', required: false, default: 'text' },
    ],
    returnsObservation: true,
  },
  {
    id: 'delegate_to_agent',
    name: 'Delegate to Agent',
    description: 'Delegate task to another agent',
    category: 'utility',
    parameters: [
      { name: 'agent_id', type: 'string', description: 'Agent ID', required: true },
      { name: 'task', type: 'string', description: 'Task description', required: true },
    ],
    returnsObservation: true,
  },
];

export function getAllTools(): ToolDefinition[] {
  return AVAILABLE_TOOLS;
}

export function getToolById(toolId: string): ToolDefinition | undefined {
  return AVAILABLE_TOOLS.find((t) => t.id === toolId);
}

export function getToolsByCategory(category: string): ToolDefinition[] {
  return AVAILABLE_TOOLS.filter((t) => t.category === category);
}
