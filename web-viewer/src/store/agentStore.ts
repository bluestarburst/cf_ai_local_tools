/**
 * Agent Store - Zustand State Management
 */

import { create } from 'zustand';
import { Agent } from '../types/agent';
import {
  getAllAgentPresets,
  saveAgentPreset,
  deleteAgentPreset,
  getCurrentAgent,
  setCurrentAgent as setCurrentAgentStorage,
  getRecentAgents,
  generateId,
} from './presetStorage';
import { DEFAULT_AGENTS } from '../types/agent';
import { AgentPreset } from '../types/preset';

interface AgentState {
  // Current agent being edited or used
  currentAgent: Agent | null;
  
  // All available agents (built-in + user-created)
  agents: Agent[];
  
  // Recently used agent IDs
  recentAgentIds: string[];
  
  // Unsaved changes flag
  hasUnsavedChanges: boolean;
  
  // Loading state
  isLoading: boolean;
  
  // Actions
  loadAgents: () => void;
  setCurrentAgent: (agent: Agent) => void;
  createAgent: (config: Partial<Agent>) => Agent;
  updateCurrentAgent: (updates: Partial<Agent>) => void;
  saveCurrentAgent: () => void;
  deleteAgent: (id: string) => void;
  duplicateAgent: (id: string) => Agent;
  resetToDefault: (id: string) => void;
  markUnsaved: () => void;
  clearUnsaved: () => void;
}

export const useAgentStore = create<AgentState>((set, get) => ({
  currentAgent: null,
  agents: [],
  recentAgentIds: [],
  hasUnsavedChanges: false,
  isLoading: false,

  loadAgents: () => {
    set({ isLoading: true });
    
    // Load built-in agents from defaults
    const builtInAgents = Object.values(DEFAULT_AGENTS).map((p) => ({
      ...p.content,
      isDefault: true, // Ensure built-in agents are marked as default
    }));
    
    // Load modified built-in agents from localStorage and merge
    const modifiedBuiltInsStr = localStorage.getItem('modified-built-in-agents');
    if (modifiedBuiltInsStr) {
      try {
        const modifiedBuiltIns = JSON.parse(modifiedBuiltInsStr);
        Object.keys(modifiedBuiltIns).forEach((id) => {
          const index = builtInAgents.findIndex((a) => a.id === id);
          if (index >= 0) {
            builtInAgents[index] = {
              ...modifiedBuiltIns[id],
              isDefault: true, // Keep isDefault flag
            };
          }
        });
      } catch (e) {
        console.error('Failed to load modified built-in agents:', e);
      }
    }
    
    // Load user-created agents only
    const userPresets = getAllAgentPresets();
    const userAgents = Object.values(userPresets)
      .filter((p) => p.category === 'user-created')
      .map((p) => ({
        ...p.content,
        isDefault: false, // Ensure user agents are marked as user-created
      }));
    
    // Get recent agents
    const recentIds = getRecentAgents();
    
    // Get current agent from storage
    const current = getCurrentAgent();
    
    // Find the current agent in the loaded list to ensure reference consistency
    const allAgents = [...builtInAgents, ...userAgents];
    const currentAgent = current 
      ? allAgents.find((a) => a.id === current.id) || current
      : allAgents[0] || null;
    
    set({
      agents: allAgents,
      recentAgentIds: recentIds,
      currentAgent,
      isLoading: false,
    });
  },

  setCurrentAgent: (agent: Agent) => {
    // Use the agent from the agents list to ensure reference consistency
    const { agents } = get();
    const agentInList = agents.find((a) => a.id === agent.id);
    const agentToSet = agentInList || agent;
    
    setCurrentAgentStorage(agentToSet);
    set({ currentAgent: agentToSet, hasUnsavedChanges: false });
  },

  createAgent: (config: Partial<Agent>) => {
    const newAgent: Agent = {
      id: generateId('agent'),
      name: config.name || 'New Agent',
      purpose: config.purpose || '',
      systemPrompt: config.systemPrompt || '',
      tools: config.tools || [],
      modelId: config.modelId || '@cf/meta/llama-3.3-70b-instruct-fp8-fast',
      maxIterations: config.maxIterations || 5,
      metadata: {
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        version: '1.0.0',
      },
    };

    // Save as preset
    const preset: AgentPreset = {
      id: newAgent.id,
      name: newAgent.name,
      description: newAgent.purpose,
      type: 'agent',
      category: 'user-created',
      content: newAgent,
      metadata: newAgent.metadata,
    };

    saveAgentPreset(preset);
    
    const { agents } = get();
    set({ agents: [...agents, newAgent], currentAgent: newAgent });
    
    return newAgent;
  },

  updateCurrentAgent: (updates: Partial<Agent>) => {
    const { currentAgent } = get();
    if (!currentAgent) return;

    const updated = {
      ...currentAgent,
      ...updates,
      metadata: {
        ...currentAgent.metadata,
        updatedAt: new Date().toISOString(),
      },
    };

    set({ currentAgent: updated, hasUnsavedChanges: true });
  },

  saveCurrentAgent: () => {
    const { currentAgent, agents } = get();
    if (!currentAgent) return;

    let agentToSave = currentAgent;
    let updatedAgents = agents;
    
    if (currentAgent.isDefault) {
      // Save modified built-in agent to localStorage
      const modifiedBuiltInsStr = localStorage.getItem('modified-built-in-agents');
      const modifiedBuiltIns = modifiedBuiltInsStr ? JSON.parse(modifiedBuiltInsStr) : {};
      
      // Update modified agent with new timestamp
      const updatedAgent = {
        ...currentAgent,
        metadata: {
          ...currentAgent.metadata,
          updatedAt: new Date().toISOString(),
        },
      };
      
      modifiedBuiltIns[currentAgent.id] = updatedAgent;
      localStorage.setItem('modified-built-in-agents', JSON.stringify(modifiedBuiltIns));
      
      // Update in-memory agents list
      updatedAgents = agents.map((a) =>
        a.id === currentAgent.id ? updatedAgent : a
      );
      
      set({ agents: updatedAgents, currentAgent: updatedAgent, hasUnsavedChanges: false });
      setCurrentAgentStorage(updatedAgent);
      return;
    } else {
      // Update metadata version for user agents
      const [major, minor, patch] = agentToSave.metadata.version.split('.').map(Number);
      agentToSave.metadata.version = `${major}.${minor}.${patch + 1}`;
      agentToSave.metadata.updatedAt = new Date().toISOString();

      // Update in agents list
      updatedAgents = agents.map((a) =>
        a.id === agentToSave.id ? agentToSave : a
      );
      set({ agents: updatedAgents });
    }

    // Save as preset
    const preset: AgentPreset = {
      id: agentToSave.id,
      name: agentToSave.name,
      description: agentToSave.purpose,
      type: 'agent',
      category: 'user-created',
      content: agentToSave,
      metadata: agentToSave.metadata,
    };

    saveAgentPreset(preset);
    setCurrentAgentStorage(agentToSave);
    set({ hasUnsavedChanges: false });
  },

  deleteAgent: (id: string) => {
    deleteAgentPreset(id);
    const { agents, currentAgent } = get();
    const filtered = agents.filter((a) => a.id !== id);
    
    // If deleted current agent, set to first available
    const newCurrent = currentAgent?.id === id ? filtered[0] || null : currentAgent;
    
    set({ agents: filtered, currentAgent: newCurrent });
  },

  duplicateAgent: (id: string) => {
    const { agents } = get();
    const original = agents.find((a) => a.id === id);
    if (!original) throw new Error('Agent not found');

    const duplicate: Agent = {
      ...original,
      id: generateId('agent'),
      name: `${original.name} (Copy)`,
      metadata: {
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        version: '1.0.0',
      },
      isDefault: false,
    };

    const preset: AgentPreset = {
      id: duplicate.id,
      name: duplicate.name,
      description: duplicate.purpose,
      type: 'agent',
      category: 'user-created',
      content: duplicate,
      metadata: duplicate.metadata,
    };

    saveAgentPreset(preset);
    set({ agents: [...agents, duplicate] });
    
    return duplicate;
  },

  resetToDefault: (id: string) => {
    const defaultPreset = DEFAULT_AGENTS[id];
    if (!defaultPreset) return;

    const defaultAgent = {
      ...defaultPreset.content,
      isDefault: true,
    };
    
    // Remove from modified built-in agents in localStorage
    const modifiedBuiltInsStr = localStorage.getItem('modified-built-in-agents');
    if (modifiedBuiltInsStr) {
      try {
        const modifiedBuiltIns = JSON.parse(modifiedBuiltInsStr);
        delete modifiedBuiltIns[id];
        localStorage.setItem('modified-built-in-agents', JSON.stringify(modifiedBuiltIns));
      } catch (e) {
        console.error('Failed to update modified built-in agents:', e);
      }
    }
    
    // Update in-memory agents list
    const { agents } = get();
    const updatedAgents = agents.map((a) =>
      a.id === id ? defaultAgent : a
    );
    
    set({ agents: updatedAgents, currentAgent: defaultAgent, hasUnsavedChanges: false });
    setCurrentAgentStorage(defaultAgent);
  },

  markUnsaved: () => set({ hasUnsavedChanges: true }),
  
  clearUnsaved: () => set({ hasUnsavedChanges: false }),
}));
