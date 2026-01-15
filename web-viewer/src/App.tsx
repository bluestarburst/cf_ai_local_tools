/**
 * Main App Component
 * Layout with sidebar, agent editor, and chat interface
 */

import { useState } from 'react';
import { AgentList } from './components/AgentList';
import { AgentEditor } from './components/AgentEditor';
import { ChatInterface } from './components/ChatInterface';
import { useAgentStore } from './store/agentStore';

type View = 'editor' | 'chat';

function App() {
  const [currentView, setCurrentView] = useState<View>('editor');
  const { currentAgent, createAgent } = useAgentStore();

  const handleCreateAgent = () => {
    createAgent({
      name: 'New Agent',
      purpose: 'Describe the purpose of this agent',
      modelId: '@cf/meta/llama-3.3-70b-instruct-fp8-fast',
      maxIterations: 5,
      tools: [],
      systemPrompt: '',
      isDefault: false,
    });
  };

  return (
    <div className="app h-screen flex flex-col bg-gray-50">
      <header className="bg-white border-b border-gray-200 px-4 py-3 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-bold text-gray-900">CF AI Local Tools</h1>
          <span className="text-sm text-gray-500">Agentic Desktop Automation</span>
        </div>
        
        <div className="flex items-center gap-2">
          <button
            onClick={() => setCurrentView('editor')}
            className={`px-4 py-2 rounded ${
              currentView === 'editor'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            Editor
          </button>
          <button
            onClick={() => setCurrentView('chat')}
            className={`px-4 py-2 rounded ${
              currentView === 'chat'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            Chat
          </button>
        </div>
      </header>

      <div className="flex-1 flex overflow-hidden">
        <div className="w-80 border-r border-gray-200 bg-white flex flex-col">
          <div className="p-3 border-b border-gray-200">
            <button
              onClick={handleCreateAgent}
              className="w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 font-medium"
            >
              + New Agent
            </button>
          </div>
          <div className="flex-1 overflow-hidden">
            <AgentList />
          </div>
        </div>

        <div className="flex-1 overflow-hidden">
          {currentView === 'editor' ? <AgentEditor /> : <ChatInterface />}
        </div>
      </div>

      <footer className="bg-white border-t border-gray-200 px-4 py-2 flex items-center justify-between text-xs text-gray-600">
        <div>
          {currentAgent ? (
            <span>
              Active: <strong>{currentAgent.name}</strong> • {currentAgent.tools.filter((t) => t.enabled).length} tools
            </span>
          ) : (
            <span>No agent selected</span>
          )}
        </div>
        <div>Model: Llama 3.3 70B</div>
      </footer>
    </div>
  );
}

export default App;