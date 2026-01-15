/**
 * Agent Editor Component
 * Edit agent configuration
 */

import React, { useState } from 'react';
import { useAgentStore } from '../store/agentStore';
import { ToolSelector } from './ToolSelector';
import { VariablesPanel } from './VariablesPanel';
import { getAllTools } from '../utils/toolUtils';

type EditorTab = 'config' | 'variables';

export const AgentEditor: React.FC = () => {
  const {
    currentAgent,
    agents,
    updateCurrentAgent,
    saveCurrentAgent,
    resetToDefault,
    hasUnsavedChanges,
  } = useAgentStore();

  const [activeTab, setActiveTab] = useState<EditorTab>('config');

  if (!currentAgent) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500">
        Select an agent to edit
      </div>
    );
  }

  const isBuiltIn = currentAgent.isDefault;
  const tools = getAllTools();

  const handleAgentsChange = (enabledAgentIds: string[]) => {
    updateCurrentAgent({ availableAgentIds: enabledAgentIds });
  };

  const handleResetToDefault = () => {
    if (confirm('Reset this agent to default settings? All changes will be lost.')) {
      resetToDefault(currentAgent.id);
    }
  };

  return (
    <div className="agent-editor h-full flex flex-col bg-white">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 flex items-center justify-between">
        <h2 className="text-lg font-semibold text-gray-800">
          Edit "{currentAgent.name}"
          {hasUnsavedChanges && (
            <span className="ml-2 text-sm text-orange-600">(Unsaved changes)</span>
          )}
          {isBuiltIn && (
            <span className="ml-2 text-xs text-gray-500">(Built-in)</span>
          )}
        </h2>
        <div className="flex gap-2">
          {isBuiltIn && (
            <button
              onClick={handleResetToDefault}
              className="px-4 py-2 rounded bg-gray-200 text-gray-700 hover:bg-gray-300"
            >
              Reset to Default
            </button>
          )}
          {hasUnsavedChanges && (
            <button
              onClick={saveCurrentAgent}
              disabled={!hasUnsavedChanges}
              className={`px-4 py-2 rounded ${
                hasUnsavedChanges
                  ? 'bg-blue-600 text-white hover:bg-blue-700'
                  : 'bg-gray-200 text-gray-500 cursor-not-allowed'
              }`}
            >
              Save Changes
            </button>
          )}
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200 bg-gray-50">
        <div className="flex">
          <button
            onClick={() => setActiveTab('config')}
            className={`px-6 py-3 font-medium text-sm transition-colors ${activeTab === 'config'
              ? 'text-blue-600 border-b-2 border-blue-600 bg-white'
              : 'text-gray-600 hover:text-gray-800 hover:bg-gray-100'
              }`}
          >
            Configuration
          </button>
          <button
            onClick={() => setActiveTab('variables')}
            className={`px-6 py-3 font-medium text-sm transition-colors ${activeTab === 'variables'
              ? 'text-blue-600 border-b-2 border-blue-600 bg-white'
              : 'text-gray-600 hover:text-gray-800 hover:bg-gray-100'
              }`}
          >
            Template Variables
          </button>
        </div>
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-y-auto">
        {activeTab === 'config' && (
          <div className="p-4 space-y-6">
            {isBuiltIn && (
              <div className="bg-blue-50 border border-blue-200 rounded-md p-3 text-sm text-blue-800">
                This is a built-in agent. You can edit it, but use "Reset to Default" to restore original settings.
              </div>
            )}

            {/* Name */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Name
              </label>
              <input
                type="text"
                value={currentAgent.name}
                onChange={(e) => updateCurrentAgent({ name: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* Purpose */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Purpose
              </label>
              <textarea
                value={currentAgent.purpose}
                onChange={(e) => updateCurrentAgent({ purpose: e.target.value })}
                rows={2}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* Model */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Model
              </label>
              <select
                value={currentAgent.modelId}
                onChange={(e) => updateCurrentAgent({ modelId: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="@cf/meta/llama-3.3-70b-instruct-fp8-fast">
                  Llama 3.3 70B (Fast)
                </option>
                <option value="@cf/meta/llama-3.1-70b-instruct">
                  Llama 3.1 70B
                </option>
                <option value="@cf/meta/llama-2-7b-chat-int8">
                  Llama 2 7B
                </option>
              </select>
            </div>

            {/* Max Iterations */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Max Iterations: {currentAgent.maxIterations}
              </label>
              <input
                type="range"
                min="1"
                max="20"
                value={currentAgent.maxIterations}
                onChange={(e) =>
                  updateCurrentAgent({ maxIterations: parseInt(e.target.value) })
                }
                className="w-full"
              />
              <div className="flex justify-between text-xs text-gray-500 mt-1">
                <span>1</span>
                <span>20</span>
              </div>
            </div>

            {/* Tools */}
            <div>
              <ToolSelector
                selectedTools={currentAgent.tools}
                onChange={(tools) => updateCurrentAgent({ tools })}
                disabled={false}
              />
            </div>

            {/* System Prompt */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                System Prompt
              </label>
              <textarea
                value={currentAgent.systemPrompt}
                onChange={(e) => updateCurrentAgent({ systemPrompt: e.target.value })}
                rows={10}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
                placeholder="Enter system prompt..."
              />
            </div>
          </div>
        )}

        {activeTab === 'variables' && (
          <div className="p-4">
            <VariablesPanel
              agent={currentAgent}
              tools={tools}
              allAgents={agents}
              onAgentsChange={handleAgentsChange}
              isReadOnly={false}
            />
          </div>
        )}
      </div>
    </div>
  );
};
