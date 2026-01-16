/**
 * Prompt Store - Zustand State Management
 * Manages system prompts with backend WebSocket synchronization
 */

import { create } from 'zustand';

export interface PromptMetadata {
  created_at: string;
  updated_at: string;
  version: string;
  author?: string;
  tags?: string[];
}

export interface Prompt {
  id: string;
  name: string;
  description: string;
  prompt_type: 'system' | 'user' | 'assistant';
  category: 'built-in' | 'user-created';
  content: string;
  metadata: PromptMetadata;
  is_locked: boolean;
}

interface PromptState {
  // All available prompts (built-in + user-created)
  prompts: Prompt[];
  
  // Currently selected prompt for editing
  currentPrompt: Prompt | null;
  
  // Unsaved changes flag
  hasUnsavedChanges: boolean;
  
  // Loading state
  isLoading: boolean;
  
  // Error messages
  error: string | null;
  
  // WebSocket connection reference
  ws: WebSocket | null;
  
  // Actions
  setWebSocket: (ws: WebSocket) => void;
  loadPrompts: () => void;
  setCurrentPrompt: (prompt: Prompt) => void;
  createPrompt: (prompt: Omit<Prompt, 'id' | 'is_locked' | 'category'>) => void;
  updateCurrentPrompt: (updates: Partial<Prompt>) => void;
  saveCurrentPrompt: () => void;
  deletePrompt: (id: string) => void;
  resetToDefault: () => void;
  clearError: () => void;
}

export const usePromptStore = create<PromptState>((set, get) => ({
  prompts: [],
  currentPrompt: null,
  hasUnsavedChanges: false,
  isLoading: false,
  error: null,
  ws: null,

  setWebSocket: (ws: WebSocket) => {
    set({ ws });
  },

  loadPrompts: () => {
    const { ws } = get();
    if (!ws) {
      set({ error: 'WebSocket not connected' });
      return;
    }

    set({ isLoading: true, error: null });
    
    try {
      ws.send(JSON.stringify({ type: 'get_prompts' }));
    } catch (err) {
      set({ error: `Failed to load prompts: ${err}`, isLoading: false });
    }
  },

  setCurrentPrompt: (prompt: Prompt) => {
    set({ currentPrompt: prompt, hasUnsavedChanges: false });
  },

  createPrompt: (promptData) => {
    const { ws } = get();
    if (!ws) {
      set({ error: 'WebSocket not connected' });
      return;
    }

    const newPrompt: Prompt = {
      id: `prompt_${Date.now()}`,
      ...promptData,
      category: 'user-created',
      is_locked: false,
      metadata: {
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        version: '1.0.0',
      },
    };

    try {
      ws.send(JSON.stringify({ type: 'create_prompt', ...newPrompt }));
    } catch (err) {
      set({ error: `Failed to create prompt: ${err}` });
    }
  },

  updateCurrentPrompt: (updates: Partial<Prompt>) => {
    const { currentPrompt } = get();
    if (!currentPrompt) return;

    // Prevent updating locked prompts
    if (currentPrompt.is_locked) {
      set({ error: 'Cannot modify built-in prompts' });
      return;
    }

    const updated = {
      ...currentPrompt,
      ...updates,
      metadata: {
        ...currentPrompt.metadata,
        updated_at: new Date().toISOString(),
      },
    };

    set({ currentPrompt: updated, hasUnsavedChanges: true, error: null });
  },

  saveCurrentPrompt: () => {
    const { currentPrompt, ws } = get();
    if (!currentPrompt) return;
    
    if (!ws) {
      set({ error: 'WebSocket not connected' });
      return;
    }

    // Prevent saving locked prompts
    if (currentPrompt.is_locked) {
      set({ error: 'Cannot modify built-in prompts' });
      return;
    }

    set({ isLoading: true, error: null });

    try {
      ws.send(JSON.stringify({
        type: 'update_prompt',
        ...currentPrompt,
      }));
    } catch (err) {
      set({ error: `Failed to save prompt: ${err}`, isLoading: false });
    }
  },

  deletePrompt: (id: string) => {
    const { ws, prompts } = get();
    if (!ws) {
      set({ error: 'WebSocket not connected' });
      return;
    }

    // Prevent deleting locked prompts
    const prompt = prompts.find(p => p.id === id);
    if (prompt?.is_locked) {
      set({ error: 'Cannot delete built-in prompts' });
      return;
    }

    set({ isLoading: true, error: null });

    try {
      ws.send(JSON.stringify({
        type: 'delete_prompt',
        id,
      }));
    } catch (err) {
      set({ error: `Failed to delete prompt: ${err}`, isLoading: false });
    }
  },

  resetToDefault: () => {
    const { ws } = get();
    if (!ws) {
      set({ error: 'WebSocket not connected' });
      return;
    }

    set({ isLoading: true, error: null });

    try {
      ws.send(JSON.stringify({ type: 'reset_prompts' }));
    } catch (err) {
      set({ error: `Failed to reset prompts: ${err}`, isLoading: false });
    }
  },

  clearError: () => {
    set({ error: null });
  },
}));

// Helper function to handle WebSocket messages for prompts
export const handlePromptMessage = (data: any) => {
  switch (data.type) {
    case 'prompts': {
      usePromptStore.setState({
        prompts: data.prompts || [],
        isLoading: false,
        error: null,
      });
      break;
    }
    case 'prompt_created': {
      const { prompts } = usePromptStore.getState();
      usePromptStore.setState({
        prompts: [...prompts, data.prompt],
        currentPrompt: data.prompt,
        hasUnsavedChanges: false,
        isLoading: false,
        error: null,
      });
      break;
    }
    case 'prompt_updated': {
      const { prompts, currentPrompt } = usePromptStore.getState();
      const updatedPrompts = prompts.map(p =>
        p.id === data.prompt.id ? data.prompt : p
      );
      usePromptStore.setState({
        prompts: updatedPrompts,
        currentPrompt: currentPrompt?.id === data.prompt.id ? data.prompt : currentPrompt,
        hasUnsavedChanges: false,
        isLoading: false,
        error: null,
      });
      break;
    }
    case 'prompt_deleted': {
      const { prompts, currentPrompt } = usePromptStore.getState();
      const updatedPrompts = prompts.filter(p => p.id !== data.id);
      usePromptStore.setState({
        prompts: updatedPrompts,
        currentPrompt: currentPrompt?.id === data.id ? null : currentPrompt,
        isLoading: false,
        error: null,
      });
      break;
    }
    case 'prompts_reset': {
      usePromptStore.setState({
        prompts: data.prompts || [],
        currentPrompt: null,
        hasUnsavedChanges: false,
        isLoading: false,
        error: null,
      });
      break;
    }
    case 'prompt_error': {
      usePromptStore.setState({
        error: data.error || 'An error occurred',
        isLoading: false,
      });
      break;
    }
  }
};
