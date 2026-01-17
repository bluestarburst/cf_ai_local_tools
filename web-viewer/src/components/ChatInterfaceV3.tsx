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
import { usePromptStore, handlePromptMessage } from '../store/promptStore';
import { useWebSocketStore } from '../store/webSocketStore';
import { fetchBackendPresets, hasPresetsLoaded } from '../utils/backendPresets';
import type { ExecutionStep } from '../store/executionStore';

// Note: WebSocket connection is managed by webSocketStore in App.tsx

interface ChatMessage {
    role: 'user' | 'assistant';
    content: string;
    executionSteps?: ExecutionStep[];
    isStreaming?: boolean;
}

const ExecutionSteps: React.FC<{ steps: ExecutionStep[] }> = ({ steps }) => {
    if (!steps || steps.length === 0) return null;

    const [isExpanded, setIsExpanded] = React.useState(true);

    return (
        <div className="mt-3 border border-gray-200 rounded-lg overflow-hidden bg-gray-50">
            <button
                onClick={() => setIsExpanded(!isExpanded)}
                className="w-full px-3 py-2 text-left text-xs font-medium text-gray-700 hover:bg-gray-100 flex items-center justify-between"
            >
                <span>📋 Execution Steps ({steps.length})</span>
                <span>{isExpanded ? '▼' : '▶'}</span>
            </button>
            {isExpanded && (
                <div className="p-3 space-y-3">
                    {steps.map((step, idx) => {
                        const hasError = !!step.observation?.error;
                        const hasAction = !!step.action;
                        const isThinkingOnly = !hasAction;

                        return (
                            <div
                                key={idx}
                                className={`border rounded p-2 ${
                                    hasError ? 'bg-red-50 border-red-200' :
                                    isThinkingOnly ? 'bg-blue-50 border-blue-200' :
                                    'bg-white border-gray-200'
                                }`}
                            >
                                <div className="text-xs font-medium mb-1 flex items-center justify-between">
                                    <span className="text-gray-500">Step {step.stepNumber}</span>
                                    {hasError && (
                                        <span className="text-red-600 font-semibold text-xs">FAILED</span>
                                    )}
                                    {isThinkingOnly && (
                                        <span className="text-blue-600 font-semibold text-xs">REASONING</span>
                                    )}
                                    {!hasError && hasAction && (
                                        <span className="text-green-600 font-semibold text-xs">SUCCESS</span>
                                    )}
                                </div>

                                <div className="mb-2">
                                    <div className="text-xs font-medium text-gray-700 mb-1">💭 Thought</div>
                                    <div className="text-xs text-gray-800 bg-blue-50 p-1.5 rounded">
                                        {step.thought}
                                    </div>
                                </div>

                                {step.action && (
                                    <div className="mb-2">
                                        <div className="text-xs font-medium text-gray-700 mb-1">
                                            {step.observation?.error ? '⚠️ Attempted Action' : '⚡ Action'}
                                        </div>
                                        <div
                                            className={`border rounded p-1.5 text-xs ${
                                                step.observation?.error
                                                    ? 'bg-orange-50 border-orange-200'
                                                    : 'bg-purple-50 border-purple-200'
                                            }`}
                                        >
                                            <div className={`font-mono ${step.observation?.error ? 'text-orange-900' : 'text-purple-900'}`}>
                                                {step.action.tool}
                                            </div>
                                            {step.action.parameters && (
                                                <pre className="text-xs text-gray-600 mt-1 overflow-x-auto">
                                                    {JSON.stringify(step.action.parameters, null, 2)}
                                                </pre>
                                            )}
                                        </div>
                                    </div>
                                )}

                                {step.observation && (
                                    <div>
                                        <div className="text-xs font-medium text-gray-700 mb-1">
                                            {step.observation.error ? '❌ Error' : '✅ Observation'}
                                        </div>
                                        <div
                                            className={`text-xs whitespace-pre-wrap p-1.5 rounded ${
                                                step.observation.error
                                                    ? 'bg-red-50 border border-red-200 text-red-800'
                                                    : 'bg-green-50 border border-green-200 text-gray-700'
                                            }`}
                                        >
                                            {step.observation.error ? (
                                                <div>
                                                    <div className="font-semibold mb-1">Error:</div>
                                                    <div className="text-xs">{step.observation.error}</div>
                                                    {step.observation.result && (
                                                        <div className="mt-2 pt-1 border-t border-red-300">
                                                            <div className="font-semibold mb-1">Partial Result:</div>
                                                            <pre className="text-xs overflow-x-auto">
                                                                {typeof step.observation.result === 'string'
                                                                    ? step.observation.result
                                                                    : JSON.stringify(step.observation.result, null, 2)}
                                                            </pre>
                                                        </div>
                                                    )}
                                                </div>
                                            ) : (
                                                <div>
                                                    {typeof step.observation.result === 'string'
                                                        ? step.observation.result
                                                        : JSON.stringify(step.observation.result, null, 2)}
                                                </div>
                                            )}
                                        </div>
                                    </div>
                                )}
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
};

export const ChatInterfaceV3: React.FC = () => {
    const { currentAgent, setBackendAgents } = useAgentStore();
    const { setWebSocket: setPromptWebSocket } = usePromptStore();
    const { ws, connected, send } = useWebSocketStore();
    const [message, setMessage] = useState('');
    const [conversationHistory, setConversationHistory] = useState<ChatMessage[]>([]);
    const [isWaitingForResponse, setIsWaitingForResponse] = useState(false);
    const messagesEndRef = useRef<HTMLDivElement>(null);
    const currentAssistantIndex = useRef<number | null>(null);

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    };

    useEffect(() => {
        scrollToBottom();
    }, [conversationHistory]);

    // Pass WebSocket to prompt store when connected
    useEffect(() => {
        if (ws) {
            setPromptWebSocket(ws);
        }
    }, [ws, setPromptWebSocket]);

    // Fetch backend presets on connection
    useEffect(() => {
        if (!connected) return;

        // Only fetch if not already cached
        if (!hasPresetsLoaded()) {
            console.log('[Chat] Fetching backend presets...');
            fetchBackendPresets()
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
    }, [connected, setBackendAgents]);

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
                    // Add step to current assistant message
                    if (currentAssistantIndex.current !== null) {
                        setConversationHistory((prev) => {
                            const updated = [...prev];
                            const currentMsg = updated[currentAssistantIndex.current!];
                            updated[currentAssistantIndex.current!] = {
                                ...currentMsg,
                                executionSteps: [...(currentMsg.executionSteps || []), data.step],
                            };
                            return updated;
                        });
                    }
                } else if (data.type === 'chat_response') {
                    console.log('[Chat] Received response:', data);

                    // Update current assistant message with final response
                    if (currentAssistantIndex.current !== null) {
                        setConversationHistory((prev) => {
                            const updated = [...prev];
                            const currentMsg = updated[currentAssistantIndex.current!];
                            updated[currentAssistantIndex.current!] = {
                                ...currentMsg,
                                content: data.content,
                                isStreaming: false,
                            };
                            return updated;
                        });
                    }

                    setIsWaitingForResponse(false);
                    currentAssistantIndex.current = null;
                }
            } catch (error) {
                console.error('[Chat] Error parsing message:', error);
            }
        };

        ws.addEventListener('message', handleMessage);

        return () => {
            ws.removeEventListener('message', handleMessage);
        };
    }, [ws, currentAgent, message]);

    const handleSend = async () => {
        if (!message.trim() || !currentAgent || isWaitingForResponse || !connected) {
            return;
        }

        const userMessage = message.trim();
        setMessage('');

        // Add user message and placeholder assistant message
        setConversationHistory((prev) => [
            ...prev,
            { role: 'user', content: userMessage },
            { role: 'assistant', content: '', executionSteps: [], isStreaming: true },
        ]);

        // Track index of current assistant message
        currentAssistantIndex.current = conversationHistory.length + 1;

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
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
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
                            {msg.role === 'assistant' && msg.executionSteps && (
                                <ExecutionSteps steps={msg.executionSteps} />
                            )}
                            {msg.content && (
                                <p className={`whitespace-pre-wrap ${msg.executionSteps ? 'mt-3' : ''}`}>
                                    {msg.content}
                                </p>
                            )}
                            {msg.isStreaming && !msg.content && (
                                <div className="flex items-center gap-2">
                                    <div className="animate-pulse text-sm">Thinking...</div>
                                </div>
                            )}
                        </div>
                    </div>
                ))}

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
