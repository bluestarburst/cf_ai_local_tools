/**
 * Agent List Component
 * Displays all available agents with filtering and selection
 */

import React, { useEffect, useState } from 'react';
import { useAgentStore } from '../store/agentStore';

export const AgentList: React.FC = () => {
  const {
    agents,
    currentAgent,
    setCurrentAgent,
    deleteAgent,
    duplicateAgent,
    loadAgents,
  } = useAgentStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [filterCategory, setFilterCategory] = useState<'all' | 'built-in' | 'user'>('all');

  useEffect(() => {
    loadAgents();
  }, [loadAgents]);

  const filteredAgents = agents.filter((agent) => {
    const matchesSearch =
      agent.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      agent.purpose.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesFilter =
      filterCategory === 'all' ||
      (filterCategory === 'built-in' && agent.isDefault) ||
      (filterCategory === 'user' && !agent.isDefault);

    return matchesSearch && matchesFilter;
  });

  // Sort agents: pinned first, then alphabetically
  const sortedAgents = [...filteredAgents].sort((a, b) => {
    // Pinned agents always come first
    if (a.isPinned && !b.isPinned) return -1;
    if (!a.isPinned && b.isPinned) return 1;
    // Then sort alphabetically
    return a.name.localeCompare(b.name);
  });

  const handleDelete = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm('Delete this agent?')) {
      deleteAgent(id);
    }
  };

  const handleDuplicate = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    duplicateAgent(id);
  };

  return (
    <div className="agent-list h-full flex flex-col bg-gray-50 border-r border-gray-200">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 bg-white">
        <h2 className="text-lg font-semibold text-gray-800 mb-3">Agents</h2>
        
        {/* Search */}
        <input
          type="text"
          placeholder="Search agents..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />

        {/* Filter */}
        <div className="flex gap-2 mt-2">
          <button
            onClick={() => setFilterCategory('all')}
            className={`px-3 py-1 text-sm rounded ${
              filterCategory === 'all'
                ? 'bg-blue-500 text-white'
                : 'bg-gray-200 text-gray-700'
            }`}
          >
            All
          </button>
          <button
            onClick={() => setFilterCategory('built-in')}
            className={`px-3 py-1 text-sm rounded ${
              filterCategory === 'built-in'
                ? 'bg-blue-500 text-white'
                : 'bg-gray-200 text-gray-700'
            }`}
          >
            Built-in
          </button>
          <button
            onClick={() => setFilterCategory('user')}
            className={`px-3 py-1 text-sm rounded ${
              filterCategory === 'user'
                ? 'bg-blue-500 text-white'
                : 'bg-gray-200 text-gray-700'
            }`}
          >
            My Agents
          </button>
        </div>
      </div>

      {/* Agent List */}
      <div className="flex-1 overflow-y-auto">
        {sortedAgents.length === 0 ? (
          <div className="p-4 text-center text-gray-500">
            No agents found
          </div>
        ) : (
          sortedAgents.map((agent) => (
            <div
              key={agent.id}
              onClick={() => setCurrentAgent(agent)}
              className={`p-3 border-b border-gray-200 cursor-pointer hover:bg-gray-100 transition-colors ${
                currentAgent?.id === agent.id ? 'bg-blue-50 border-l-4 border-l-blue-500' : ''
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1 min-w-0">
                  <h3 className="font-medium text-gray-900 truncate flex items-center gap-2">
                    {agent.isPinned && (
                      <svg className="w-4 h-4 text-amber-500 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M10 2a1 1 0 011 1v1.323l3.954 1.582 1.599-.8a1 1 0 01.894 1.79l-1.233.616 1.738 5.42a1 1 0 01-.285 1.05A3.989 3.989 0 0115 15a3.989 3.989 0 01-2.667-1.019 1 1 0 01-.285-1.05l1.715-5.349L10 6.477l-3.763 1.105 1.715 5.349a1 1 0 01-.285 1.05A3.989 3.989 0 015 15a3.989 3.989 0 01-2.667-1.019 1 1 0 01-.285-1.05l1.738-5.42-1.233-.617a1 1 0 01.894-1.788l1.599.799L9 4.323V3a1 1 0 011-1zm-5 8.274l-.818 2.552c.25.112.526.174.818.174.292 0 .569-.062.818-.174L5 10.274zm10 0l-.818 2.552c.25.112.526.174.818.174.292 0 .569-.062.818-.174L15 10.274z" />
                      </svg>
                    )}
                    <span className="truncate">{agent.name}</span>
                    {agent.isDefault && (
                      <span className="px-2 py-0.5 text-xs bg-gray-200 text-gray-700 rounded flex-shrink-0">
                        Built-in
                      </span>
                    )}
                  </h3>
                  <p className="text-sm text-gray-600 truncate mt-1">
                    {agent.purpose}
                  </p>
                  <div className="flex items-center gap-2 mt-2 text-xs text-gray-500">
                    <span>{agent.tools.filter((t) => t.enabled).length} tools</span>
                    <span>•</span>
                    <span>{agent.maxIterations} iterations</span>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex gap-1 ml-2">
                  <button
                    onClick={(e) => handleDuplicate(agent.id, e)}
                    className="p-1 hover:bg-gray-200 rounded"
                    title="Duplicate"
                  >
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                  </button>
                  {/* Only show delete for non-built-in agents */}
                  {!agent.isDefault && (
                    <button
                      onClick={(e) => handleDelete(agent.id, e)}
                      className="p-1 hover:bg-red-100 text-red-600 rounded"
                      title="Delete"
                    >
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  )}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
