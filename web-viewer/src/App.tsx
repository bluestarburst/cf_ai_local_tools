import { useState, useEffect, useRef } from 'react';
import { Send, Activity, Terminal, Cpu, AlertCircle, CheckCircle2, Loader2 } from 'lucide-react';

// Configure your worker URL
const WORKER_URL = import.meta.env.VITE_WORKER_URL || 'http://localhost:8787';

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  toolCalls?: ToolCall[];
  executedTools?: ExecutedTool[];
  thinking?: any;
}

interface ToolCall {
  name: string;
  arguments: any;
}

interface ExecutedTool {
  tool: string;
  arguments: any;
  result?: any;
  error?: string;
}

interface ConnectionStatus {
  connected: boolean;
  sessions?: Array<{
    clientId: string;
    connectedAt: string;
    uptime: number;
  }>;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Check connection status periodically
  useEffect(() => {
    const checkStatus = async () => {
      try {
        const response = await fetch(`${WORKER_URL}/api/status`);
        const status = await response.json();
        setConnectionStatus(status);
      } catch (error) {
        console.error('Failed to fetch status:', error);
        setConnectionStatus({ connected: false });
      }
    };

    checkStatus();
    const interval = setInterval(checkStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const sendMessage = async () => {
    if (!input.trim() || loading) return;

    const userMessage: Message = {
      id: `msg_${Date.now()}`,
      role: 'user',
      content: input,
      timestamp: Date.now(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setLoading(true);

    try {
      const response = await fetch(`${WORKER_URL}/api/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: input,
          conversationHistory: messages.slice(-10).map((m) => ({
            role: m.role,
            content: m.content,
          })),
        }),
      });

      const data = await response.json();

      const assistantMessage: Message = {
        id: `msg_${Date.now()}`,
        role: 'assistant',
        content: data.response || 'Command executed',
        timestamp: Date.now(),
        toolCalls: data.toolCalls,
        executedTools: data.executedTools,
        thinking: data.thinking,
      };

      setMessages((prev) => [...prev, assistantMessage]);
    } catch (error: any) {
      const errorMessage: Message = {
        id: `msg_${Date.now()}`,
        role: 'system',
        content: `Error: ${error.message}`,
        timestamp: Date.now(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 text-white">
      {/* Header */}
      <header className="border-b border-gray-700 bg-gray-900/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Cpu className="w-8 h-8 text-blue-400" />
            <div>
              <h1 className="text-xl font-bold">AI Local Tools</h1>
              <p className="text-sm text-gray-400">Control your computer with AI</p>
            </div>
          </div>
          
          {/* Connection Status */}
          <div className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800/50">
            {connectionStatus?.connected ? (
              <>
                <Activity className="w-4 h-4 text-green-400 animate-pulse" />
                <span className="text-sm font-medium text-green-400">Connected</span>
              </>
            ) : (
              <>
                <AlertCircle className="w-4 h-4 text-red-400" />
                <span className="text-sm font-medium text-red-400">Disconnected</span>
              </>
            )}
          </div>
        </div>
      </header>

      <div className="max-w-7xl mx-auto px-4 py-6 h-[calc(100vh-88px)] flex flex-col">
        {/* Messages Area */}
        <div className="flex-1 overflow-y-auto mb-4 space-y-4 pr-2">
          {messages.length === 0 && (
            <div className="text-center py-20 text-gray-500">
              <Terminal className="w-16 h-16 mx-auto mb-4 opacity-50" />
              <p className="text-lg">Start a conversation to control your computer</p>
              <p className="text-sm mt-2">Try: "Move the mouse to 500, 500" or "Type hello world"</p>
            </div>
          )}

          {messages.map((message) => (
            <div
              key={message.id}
              className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
            >
              <div
                className={`max-w-3xl rounded-lg px-4 py-3 ${
                  message.role === 'user'
                    ? 'bg-blue-600'
                    : message.role === 'system'
                    ? 'bg-red-900/50 border border-red-700'
                    : 'bg-gray-800 border border-gray-700'
                }`}
              >
                <p className="text-sm font-medium mb-1 opacity-70">
                  {message.role === 'user' ? 'You' : message.role === 'system' ? 'System' : 'AI Assistant'}
                </p>
                <p className="whitespace-pre-wrap">{message.content}</p>

                {/* Tool Calls */}
                {message.executedTools && message.executedTools.length > 0 && (
                  <div className="mt-3 space-y-2">
                    <p className="text-xs font-semibold text-blue-300 uppercase tracking-wider">
                      Tool Executions:
                    </p>
                    {message.executedTools.map((tool, idx) => (
                      <div
                        key={idx}
                        className="bg-gray-900/50 rounded p-3 text-sm border border-gray-700"
                      >
                        <div className="flex items-start justify-between gap-2 mb-2">
                          <code className="text-blue-300 font-mono text-xs">{tool.tool}</code>
                          {tool.error ? (
                            <AlertCircle className="w-4 h-4 text-red-400 flex-shrink-0" />
                          ) : (
                            <CheckCircle2 className="w-4 h-4 text-green-400 flex-shrink-0" />
                          )}
                        </div>
                        <pre className="text-xs text-gray-400 overflow-x-auto">
                          {JSON.stringify(tool.arguments, null, 2)}
                        </pre>
                        {tool.result && (
                          <div className="mt-2 pt-2 border-t border-gray-700">
                            <p className="text-xs text-gray-500 mb-1">Result:</p>
                            <pre className="text-xs text-green-400">
                              {JSON.stringify(tool.result, null, 2)}
                            </pre>
                          </div>
                        )}
                        {tool.error && (
                          <div className="mt-2 pt-2 border-t border-red-900">
                            <p className="text-xs text-red-400">{tool.error}</p>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}

                {/* Thinking / Raw Response */}
                {message.thinking && (
                  <details className="mt-3">
                    <summary className="text-xs text-gray-500 cursor-pointer hover:text-gray-400">
                      Show raw thinking
                    </summary>
                    <pre className="text-xs text-gray-500 mt-2 overflow-x-auto bg-gray-900/30 p-2 rounded">
                      {JSON.stringify(message.thinking, null, 2)}
                    </pre>
                  </details>
                )}

                <p className="text-xs text-gray-500 mt-2">
                  {new Date(message.timestamp).toLocaleTimeString()}
                </p>
              </div>
            </div>
          ))}

          {loading && (
            <div className="flex justify-start">
              <div className="bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 flex items-center gap-2">
                <Loader2 className="w-4 h-4 animate-spin text-blue-400" />
                <span className="text-sm text-gray-400">Processing...</span>
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        {/* Input Area */}
        <div className="border-t border-gray-700 pt-4">
          <div className="flex gap-2">
            <textarea
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder={
                connectionStatus?.connected
                  ? 'Type a command... (e.g., "move mouse to center" or "type hello")'
                  : 'Waiting for local app connection...'
              }
              disabled={!connectionStatus?.connected || loading}
              className="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none disabled:opacity-50 disabled:cursor-not-allowed"
              rows={3}
            />
            <button
              onClick={sendMessage}
              disabled={!input.trim() || !connectionStatus?.connected || loading}
              className="px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded-lg font-medium transition-colors flex items-center gap-2"
            >
              <Send className="w-4 h-4" />
              Send
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
