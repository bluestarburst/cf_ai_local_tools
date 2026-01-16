/**
 * Backend Presets - Fetches tool and agent definitions from the local app backend
 */

import { Agent } from '../types/agent';
import { ToolDefinition } from '../types/tool';

export interface BackendPresetsResponse {
  tools: ToolDefinition[];
  agents: Agent[];
  prompts: SystemPromptPreset[];
}

export interface SystemPromptPreset {
  id: string;
  name: string;
  description: string;
  type: string;
  category: string;
  content: string;
  metadata: {
    createdAt: string;
    updatedAt: string;
    version: string;
    author?: string;
    tags?: string[];
  };
  isLocked?: boolean;
}

// Global store for backend presets
let cachedPresets: BackendPresetsResponse | null = null;
let isLoading = false;
let loadPromise: Promise<BackendPresetsResponse> | null = null;

/**
 * Fetch all presets from the backend in a single request
 */
export async function fetchBackendPresets(ws: WebSocket): Promise<BackendPresetsResponse> {
  // Return cached presets if available
  if (cachedPresets) {
    return cachedPresets;
  }

  // Return existing promise if already loading
  if (isLoading && loadPromise) {
    return loadPromise;
  }

  // Set loading state
  isLoading = true;

  loadPromise = new Promise((resolve, reject) => {
    // Create a handler for the presets response
    const handler = (event: MessageEvent) => {
      try {
        const message = JSON.parse(event.data);
        
        if (message.type === 'presets') {
          // Remove the handler
          ws.removeEventListener('message', handler);
          
          // Cache the response
          cachedPresets = {
            tools: message.tools || [],
            agents: message.agents || [],
            prompts: message.prompts || [],
          };
          
          isLoading = false;
          loadPromise = null;
          
          resolve(cachedPresets);
        }
      } catch (e) {
        console.error('Error parsing presets response:', e);
      }
    };

    // Add event listener
    ws.addEventListener('message', handler);

    // Send request to backend
    try {
      ws.send(JSON.stringify({ type: 'get_presets' }));
    } catch (e) {
      console.error('Error sending get_presets request:', e);
      ws.removeEventListener('message', handler);
      isLoading = false;
      loadPromise = null;
      reject(e);
    }

    // Timeout after 10 seconds
    setTimeout(() => {
      if (isLoading) {
        ws.removeEventListener('message', handler);
        isLoading = false;
        loadPromise = null;
        reject(new Error('Backend presets request timed out'));
      }
    }, 10000);
  });

  return loadPromise;
}

/**
 * Get cached presets (returns null if not loaded)
 */
export function getCachedPresets(): BackendPresetsResponse | null {
  return cachedPresets;
}

/**
 * Clear cached presets (useful for refresh)
 */
export function clearCachedPresets(): void {
  cachedPresets = null;
  isLoading = false;
  loadPromise = null;
}

/**
 * Check if presets are loaded
 */
export function hasPresetsLoaded(): boolean {
  return cachedPresets !== null;
}
