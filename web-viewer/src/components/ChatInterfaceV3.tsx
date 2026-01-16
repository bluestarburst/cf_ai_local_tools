/**
 * Chat Interface Component - WebSocket Version (v3.0)
 *
 * This is the new switchboard architecture where:
 * - Web viewer connects via WebSocket
 * - Messages relay through Worker to Rust app
 * - Rust app runs ReAct loop and executes tools locally
 */

import React, { useState, useRef, useEffect } from 'react';
import { useAgentStore } from '../store/agentStore';
import { useExecutionStore } from '../store/executionStore';
import { usePromptStore, handlePromptMessage } from '../store/promptStore';
import { useWebSocket } from '../hooks/useWebSocket';
import { fetchBackendPresets, hasPresetsLoaded } from '../utils/backendPresets';
import { ExecutionLogger } from './ExecutionLogger';

const WORKER_URL = import.meta.env.VITE_WORKER_URL || 'http://localhost:8787';
const WS_URL = WORKER_URL.replace(/^http/, 'ws') + '/connect?device=web-viewer';

interface ChatMessage {
    role: 'user' | 'assistant';
    content: string;
}

export const ChatInterfaceV3: React.FC = () => {
    const { currentAgent, setBackendAgents } = useAgentStore();
    const { startExecution, addIteration, updateFinalResponse, finishExecution } = useExecutionStore();
    const { setWebSocket: setPromptWebSocket } = usePromptStore();
    const { ws, connected, send } = useWebSocket(WS_URL);
    const [message, setMessage] = useState('');
    const [conversationHistory, setConversationHistory] = useState<ChatMessage[]>([]);
    const [isWaitingForResponse, setIsWaitingForResponse] = useState(false);
    const messagesEndRef = useRef<HTMLDivElement>(null);
    const executionStartTime = useRef<number>(0);

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    };

    useEffect(() => {
        scrollToBottom();
    }, [conversationHistory]);

    // Pass WebSocket to prompt store when connected
    useEffect(() => {
        if (ws && connected) {
            setPromptWebSocket(ws);
        }
    }, [ws, connected, setPromptWebSocket]);

    // Fetch backend presets on connection
    useEffect(() => {
        if (!ws || !connected) return;

        // Only fetch if not already cached
        if (!hasPresetsLoaded()) {
            console.log('[Chat] Fetching backend presets...');
            fetchBackendPresets(ws)
                .then((presets) => {
                    console.log('[Chat] Backend presets loaded:', presets);
                    // Update agent store with backend agents
                    if (presets.agents && presets.agents.length > 0) {
                        setBackendAgents(presets.agents);
                    }
                })
                .catch((error) => {
                    console.error('[Chat] Error fetching backend presets:', error);
                });
        }
    }, [ws, connected, setBackendAgents]);

    // Listen for responses from WebSocket
    useEffect(() => {
        if (!ws) return;

        const handleMessage = (event: MessageEvent) => {
            try {
                const data = JSON.parse(event.data);

                // Handle prompt-related messages
                if (
                    data.type === 'prompts' ||
                    data.type === 'prompt_created' ||
                    data.type === 'prompt_updated' ||
                    data.type === 'prompt_deleted' ||
                    data.type === 'prompts_reset' ||
                    data.type === 'prompt_error'
                ) {
                    handlePromptMessage(data);
                } else if (data.type === 'execution_step') {
                    console.log('[Chat] Received execution step:', data.step);
                    addIteration(data.step);
                } else if (data.type === 'chat_response') {
                    console.log('[Chat] Received response:', data);

                    // Update final response in execution store
                    updateFinalResponse(data.content);

                    // Add assistant response to conversation
                    setConversationHistory((prev) => [
                        ...prev,
                        {
                            role: 'assistant',
                            content: data.content,
                        },
                    ]);

                    // Finish execution
                    const executionTime = Date.now() - executionStartTime.current;
                    finishExecution({
                        agentId: currentAgent?.id || '',
                        userMessage: message,
                        iterations: [],
                        finalResponse: data.content,
                        toolCallsCount: 0,
                        executionTime,
                        completedAt: new Date().toISOString(),
                        status: 'success',
                    });

                    setIsWaitingForResponse(false);
                }
            } catch (error) {
                console.error('[Chat] Error parsing message:', error);
            }
        };

        ws.addEventListener('message', handleMessage);

        return () => {
            ws.removeEventListener('message', handleMessage);
        };
    }, [ws, currentAgent, message, addIteration, updateFinalResponse, finishExecution]);

    const handleSend = async () => {
        if (!message.trim() || !currentAgent || isWaitingForResponse || !connected) {
            return;
        }

        const userMessage = message.trim();
        setMessage('');

        // Add user message to conversation
        setConversationHistory((prev) => [
            ...prev,
            { role: 'user', content: userMessage },
        ]);

        // Start tracking execution
        executionStartTime.current = Date.now();
        startExecution(userMessage, currentAgent?.id);

        // Send chat request via WebSocket
        setIsWaitingForResponse(true);

        send({
            type: 'chat_request',
            message: userMessage,
            agent: {
                systemPrompt: currentAgent.systemPrompt,
                modelId: currentAgent.modelId,
                maxIterations: currentAgent.maxIterations,
                tools: currentAgent.tools
                    .filter((t) => t.enabled)
                    .map((t) => t.toolId),
            },
        });
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSend();
        }
    };

    return (
        <div className="flex flex-col h-full bg-gray-50">
            {/* Header */}
            <div className="px-6 py-4 bg-white border-b border-gray-200">
                <div className="flex items-center justify-between">
                    <div>
                        <h2 className="text-lg font-semibold text-gray-900">
                            {currentAgent?.name || 'Select an agent'}
                        </h2>
                        {currentAgent && (
                            <p className="text-sm text-gray-500">
                                Model: {currentAgent.modelId.split('/').pop()}
                            </p>
                        )}
                    </div>
                    <div className="flex items-center gap-2">
                        <div
                            className={`px-3 py-1 rounded-full text-sm font-medium ${
                                connected
                                    ? 'bg-green-100 text-green-800'
                                    : 'bg-red-100 text-red-800'
                            }`}
                        >
                            {connected ? 'Connected' : 'Disconnected'}
                        </div>
                    </div>
                </div>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
                {/* Execution Logger - shows intermediate steps */}
                <ExecutionLogger />

                {conversationHistory.length === 0 && (
                    <div className="text-center text-gray-500 mt-8">
                        <p className="text-lg mb-2">Start a conversation</p>
                        <p className="text-sm">
                            Type a message below to chat with your agent
                        </p>
                    </div>
                )}

                {conversationHistory.map((msg, index) => (
                    <div
                        key={index}
                        className={`flex ${
                            msg.role === 'user' ? 'justify-end' : 'justify-start'
                        }`}
                    >
                        <div
                            className={`max-w-2xl rounded-lg px-4 py-3 ${
                                msg.role === 'user'
                                    ? 'bg-blue-600 text-white'
                                    : 'bg-white text-gray-900 border border-gray-200'
                            }`}
                        >
                            <p className="whitespace-pre-wrap">{msg.content}</p>
                        </div>
                    </div>
                ))}

                {isWaitingForResponse && (
                    <div className="flex justify-start">
                        <div className="max-w-2xl rounded-lg px-4 py-3 bg-white text-gray-900 border border-gray-200">
                            <div className="flex items-center gap-2">
                                <div className="animate-pulse">Thinking...</div>
                            </div>
                        </div>
                    </div>
                )}

                <div ref={messagesEndRef} />
            </div>

            {/* Input */}
            <div className="px-6 py-4 bg-white border-t border-gray-200">
                <div className="flex gap-2">
                    <textarea
                        value={message}
                        onChange={(e) => setMessage(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder={
                            !currentAgent
                                ? 'Select an agent first...'
                                : !connected
                                ? 'Connecting...'
                                : 'Type your message (Enter to send, Shift+Enter for new line)'
                        }
                        disabled={!currentAgent || !connected || isWaitingForResponse}
                        className="flex-1 px-4 py-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed resize-none"
                        rows={3}
                    />
                    <button
                        onClick={handleSend}
                        disabled={
                            !message.trim() ||
                            !currentAgent ||
                            !connected ||
                            isWaitingForResponse
                        }
                        className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors font-medium"
                    >
                        Send
                    </button>
                </div>
                <p className="text-xs text-gray-500 mt-2">
                    {currentAgent
                        ? `Using ${currentAgent.tools.filter((t) => t.enabled).length} tools`
                        : 'No agent selected'}
                </p>
            </div>
        </div>
    );
};
