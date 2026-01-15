/**
 * Variables Panel Component
 * Visualizes and edits template variables used in system prompts
 */

import React, { useState } from 'react';
import { Agent } from '../types/agent';
import { ToolDefinition } from '../types/tool';
import { ChevronDown, ChevronRight } from 'lucide-react';

interface VariablesPanelProps {
  agent: Agent;
  tools: ToolDefinition[];
  allAgents: Agent[];
  onAgentsChange?: (enabledAgentIds: string[]) => void;
  isReadOnly?: boolean;
}

interface EnabledAgent {
  id: string;
  enabled: boolean;
}

export const VariablesPanel: React.FC<VariablesPanelProps> = ({
  agent,
  tools,
  allAgents,
  onAgentsChange,
  isReadOnly = false,
}) => {
  const [expandedSections, setExpandedSections] = useState({
    global: true,
    local: true,
    agents: true,
  });

  const [enabledAgents, setEnabledAgents] = useState<EnabledAgent[]>(() => {
    // Parse existing agents config or default to all except self
    const existingAgentIds = agent.availableAgentIds || [];
    return allAgents
      .filter((a) => a.id !== agent.id) // Exclude self
      .map((a) => ({
        id: a.id,
        enabled: existingAgentIds.length === 0 || existingAgentIds.includes(a.id),
      }));
  });

  const toggleSection = (section: keyof typeof expandedSections) => {
    setExpandedSections((prev) => ({
      ...prev,
      [section]: !prev[section],
    }));
  };

  const toggleAgent = (agentId: string) => {
    if (isReadOnly) return;
    
    const updated = enabledAgents.map((a) =>
      a.id === agentId ? { ...a, enabled: !a.enabled } : a
    );
    setEnabledAgents(updated);
    
    const enabledIds = updated.filter((a) => a.enabled).map((a) => a.id);
    onAgentsChange?.(enabledIds);
  };

  const enabledToolIds = agent.tools.filter((t) => t.enabled).map((t) => t.toolId);
  const enabledToolNames = tools
    .filter((t) => enabledToolIds.includes(t.id))
    .map((t) => t.name)
    .join(', ');

  const toolsPreview = enabledToolNames || '(No tools selected)';

  return (
    <div className="variables-panel space-y-3 bg-blue-50 border border-blue-200 rounded-lg p-4">
      <div className="text-sm font-semibold text-blue-900">Template Variables</div>

      {/* Global Variables Section */}
      <div className="bg-white rounded border border-blue-100">
        <button
          onClick={() => toggleSection('global')}
          className="w-full flex items-center gap-2 px-3 py-2 hover:bg-blue-50 transition-colors"
        >
          {expandedSections.global ? (
            <ChevronDown className="w-4 h-4 text-blue-600" />
          ) : (
            <ChevronRight className="w-4 h-4 text-blue-600" />
          )}
          <span className="font-medium text-sm text-gray-700">Global Variables</span>
        </button>

        {expandedSections.global && (
          <div className="px-3 py-2 space-y-2 border-t border-blue-100">
            {/* Purpose */}
            <div className="bg-blue-50 p-2 rounded text-xs font-mono">
              <div className="text-blue-700 font-semibold">{'{{purpose}}'}</div>
              <div className="text-gray-600 mt-1 break-words">{agent.purpose || '(empty)'}</div>
            </div>

            {/* Tools */}
            <div className="bg-blue-50 p-2 rounded text-xs font-mono">
              <div className="text-blue-700 font-semibold">{'{{tools}}'}</div>
              <div className="text-gray-600 mt-1 break-words">{toolsPreview}</div>
            </div>

            {/* Agents */}
            <div className="bg-blue-50 p-2 rounded text-xs font-mono">
              <div className="text-blue-700 font-semibold">{'{{agents}}'}</div>
              <div className="text-gray-600 mt-1 text-xs">
                {enabledAgents
                  .filter((a) => a.enabled)
                  .map((a) => {
                    const agent = allAgents.find((ag) => ag.id === a.id);
                    return agent?.name;
                  })
                  .filter(Boolean)
                  .join(', ') || '(no agents enabled)'}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Local Variables Section */}
      <div className="bg-white rounded border border-blue-100">
        <button
          onClick={() => toggleSection('local')}
          className="w-full flex items-center gap-2 px-3 py-2 hover:bg-blue-50 transition-colors"
        >
          {expandedSections.local ? (
            <ChevronDown className="w-4 h-4 text-blue-600" />
          ) : (
            <ChevronRight className="w-4 h-4 text-blue-600" />
          )}
          <span className="font-medium text-sm text-gray-700">Agent-Specific Variables</span>
        </button>

        {expandedSections.local && (
          <div className="px-3 py-2 space-y-2 border-t border-blue-100">
            {/* Name */}
            <div className="bg-blue-50 p-2 rounded text-xs font-mono">
              <div className="text-blue-700 font-semibold">name</div>
              <div className="text-gray-600 mt-1">{agent.name}</div>
            </div>

            {/* ID */}
            <div className="bg-blue-50 p-2 rounded text-xs font-mono">
              <div className="text-blue-700 font-semibold">id</div>
              <div className="text-gray-600 mt-1 break-all">{agent.id}</div>
            </div>

            {/* Type */}
            <div className="bg-blue-50 p-2 rounded text-xs font-mono">
              <div className="text-blue-700 font-semibold">type</div>
              <div className="text-gray-600 mt-1">
                {agent.isDefault ? 'built-in' : 'user-created'}
              </div>
            </div>

            {/* Max Iterations */}
            <div className="bg-blue-50 p-2 rounded text-xs font-mono">
              <div className="text-blue-700 font-semibold">maxIterations</div>
              <div className="text-gray-600 mt-1">{agent.maxIterations}</div>
            </div>
          </div>
        )}
      </div>

      {/* Available Agents Section */}
      <div className="bg-white rounded border border-blue-100">
        <button
          onClick={() => toggleSection('agents')}
          className="w-full flex items-center gap-2 px-3 py-2 hover:bg-blue-50 transition-colors"
        >
          {expandedSections.agents ? (
            <ChevronDown className="w-4 h-4 text-blue-600" />
          ) : (
            <ChevronRight className="w-4 h-4 text-blue-600" />
          )}
          <span className="font-medium text-sm text-gray-700">
            Delegation ({enabledAgents.filter((a) => a.enabled).length} enabled)
          </span>
        </button>

        {expandedSections.agents && (
          <div className="px-3 py-2 space-y-2 border-t border-blue-100">
            <div className="text-xs text-gray-600 mb-2">
              Enable agents that can be delegated to via delegate_to_agent tool:
            </div>

            {enabledAgents.length === 0 ? (
              <div className="text-xs text-gray-500 py-2">
                (This agent cannot delegate to others)
              </div>
            ) : (
              enabledAgents.map((ea) => {
                const agentInfo = allAgents.find((a) => a.id === ea.id);
                if (!agentInfo) return null;

                return (
                  <label
                    key={ea.id}
                    className={`flex items-start gap-2 p-2 rounded cursor-pointer ${
                      isReadOnly ? 'opacity-50 cursor-not-allowed' : 'hover:bg-blue-50'
                    }`}
                  >
                    <input
                      type="checkbox"
                      checked={ea.enabled}
                      onChange={() => toggleAgent(ea.id)}
                      disabled={isReadOnly}
                      className="mt-0.5 h-3.5 w-3.5 text-blue-600 rounded"
                    />
                    <div className="flex-1 text-xs">
                      <div className="font-medium text-gray-900">{agentInfo.name}</div>
                      <div className="text-gray-600">{agentInfo.purpose}</div>
                      <div className="text-blue-700 font-mono text-xs mt-0.5">
                        {agentInfo.id}
                      </div>
                    </div>
                  </label>
                );
              })
            )}

            {/* Generated agents list for system prompt */}
            {enabledAgents.filter((a) => a.enabled).length > 0 && (
              <div className="mt-3 pt-3 border-t border-blue-100 bg-amber-50 p-2 rounded text-xs">
                <div className="font-medium text-amber-900 mb-1">Generated for system prompt:</div>
                <div className="font-mono text-amber-800 text-xs whitespace-pre-wrap break-words">
                  AVAILABLE AGENTS (use these exact IDs with delegate_to_agent):
                  {'\n'}
                  {enabledAgents
                    .filter((a) => a.enabled)
                    .map((ea) => {
                      const agentInfo = allAgents.find((a) => a.id === ea.id);
                      return agentInfo
                        ? `- ${agentInfo.id}: ${agentInfo.purpose}`
                        : null;
                    })
                    .filter(Boolean)
                    .join('\n')}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
