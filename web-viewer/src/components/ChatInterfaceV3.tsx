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

const CollapsibleStepItem: React.FC<{
    title: React.ReactNode;
    content: React.ReactNode;
    defaultExpanded?: boolean;
    icon?: string;
    type?: 'thought' | 'action' | 'observation' | 'error';
}> = ({ title, content, defaultExpanded = false, icon, type = 'thought' }) => {
    const [isExpanded, setIsExpanded] = React.useState(defaultExpanded);

    const getHeaderColor = () => {
        switch (type) {
            case 'error': return 'text-red-600';
            case 'action': return 'text-purple-600';
            case 'observation': return 'text-green-600';
            default: return 'text-gray-500';
        }
    };

    return (
        <div className="text-sm">
            <button
                onClick={() => setIsExpanded(!isExpanded)}
                className={`w-full text-left py-1 flex items-center gap-2 hover:bg-gray-50 rounded select-none ${getHeaderColor()}`}
            >
                <span className="text-[10px] w-4">{isExpanded ? '▼' : '▶'}</span>
                {icon && <span>{icon}</span>}
                <span className="font-medium">{title}</span>
            </button>
            {isExpanded && (
                <div className="pl-6 py-2 overflow-x-auto">
                    {content}
                </div>
            )}
        </div>
    );
};

// Helper to normalize steps - some come with JSON-encoded content that needs parsing
const normalizeStep = (step: ExecutionStep): ExecutionStep => {
    // Check if content looks like a JSON-encoded step
    if (step.content && step.content.startsWith('{') && step.content.includes('"step_type"')) {
        try {
            const parsed = JSON.parse(step.content);
            // Return the parsed step data
            return {
                step_number: parsed.step_number ?? step.step_number,
                step_type: parsed.step_type ?? step.step_type,
                content: parsed.content ?? '',
                tool_call: parsed.tool_call ?? step.tool_call,
                tool_observation: parsed.tool_observation ?? step.tool_observation,
                timestamp: parsed.timestamp ?? step.timestamp,
            };
        } catch {
            // Not valid JSON, return as-is
            return step;
        }
    }
    return step;
};

const ExecutionSteps: React.FC<{ steps: ExecutionStep[] }> = ({ steps }) => {
    if (!steps || steps.length === 0) return null;

    // Normalize steps and dedupe by step_number + step_type
    const normalizedSteps = steps.map(normalizeStep);
    const uniqueSteps = normalizedSteps.filter((step, idx, arr) => {
        // Keep if this is the first occurrence of this step_number + step_type combo
        return arr.findIndex(s => s.step_number === step.step_number && s.step_type === step.step_type) === idx;
    });

    return (
        <div className="mt-2 space-y-1">
            {uniqueSteps.map((step, idx) => {
                const stepType = step.step_type;
                const stepNum = step.step_number;

                // Thinking / Planning / Reflection
                if (['Thinking', 'Planning', 'Reflection'].includes(stepType)) {
                    return (
                        <CollapsibleStepItem
                            key={`${stepType}-${stepNum}`}
                            type="thought"
                            icon={stepType === 'Planning' ? '📋' : stepType === 'Reflection' ? '🔍' : '💭'}
                            title={`Step ${stepNum}: ${stepType}`}
                            content={<div className="text-gray-700 whitespace-pre-wrap">{step.content}</div>}
                        />
                    );
                }

                // Action
                if (stepType === 'Action') {
                    const toolName = step.tool_call?.tool_name || 'Unknown Tool';
                    return (
                        <CollapsibleStepItem
                            key={`action-${stepNum}`}
                            type="action"
                            icon="⚡"
                            title={`Step ${stepNum}: Action - ${toolName}`}
                            content={
                                <div className="space-y-2">
                                    <div className="text-sm text-gray-700">{step.content}</div>
                                    {step.tool_call && (
                                        <>
                                            <div className="font-mono text-xs text-purple-800 bg-purple-50 p-2 rounded">
                                                {step.tool_call.tool_name}
                                            </div>
                                            {step.tool_call.arguments && (
                                                <pre className="text-xs text-gray-600 bg-gray-50 p-2 rounded font-mono">
                                                    {JSON.stringify(step.tool_call.arguments, null, 2)}
                                                </pre>
                                            )}
                                        </>
                                    )}
                                </div>
                            }
                        />
                    );
                }

                // Observation
                if (stepType === 'Observation') {
                    const hasError = step.tool_observation ? !step.tool_observation.success : false;
                    return (
                        <CollapsibleStepItem
                            key={`obs-${stepNum}`}
                            type={hasError ? 'error' : 'observation'}
                            icon={hasError ? '❌' : '✅'}
                            title={`Step ${stepNum}: ${hasError ? 'Error' : 'Observation'}`}
                            content={
                                <div className="space-y-2">
                                    <div className="text-sm text-gray-700">{step.content}</div>
                                    {step.tool_observation && (
                                        <div className={`text-xs font-mono p-2 rounded ${hasError
                                            ? 'bg-red-50 text-red-800 border border-red-100'
                                            : 'bg-green-50 text-green-900 border border-green-100'
                                            }`}>
                                            {hasError ? (
                                                <>
                                                    <div className="font-bold mb-1">Error:</div>
                                                    <div>{step.tool_observation.error}</div>
                                                </>
                                            ) : (
                                                <>
                                                    <div>{step.tool_observation.message}</div>
                                                    {step.tool_observation.data && (
                                                        <div className="mt-2 pt-1 border-t border-green-200">
                                                            <pre className="overflow-x-auto">
                                                                {JSON.stringify(step.tool_observation.data, null, 2)}
                                                            </pre>
                                                        </div>
                                                    )}
                                                </>
                                            )}
                                        </div>
                                    )}
                                </div>
                            }
                        />
                    );
                }

                // Fallback for other types
                return (
                    <CollapsibleStepItem
                        key={`${stepType}-${stepNum}`}
                        title={`Step ${stepNum}: ${stepType}`}
                        content={<div className="text-gray-700 whitespace-pre-wrap">{step.content}</div>}
                    />
                );
            })}
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
                    const currentIndex = currentAssistantIndex.current;
                    // Add step to current assistant message
                    if (currentIndex !== null) {
                        setConversationHistory((prev) => {
                            const updated = [...prev];
                            // Check if message exists at index
                            if (updated[currentIndex]) {
                                const currentMsg = updated[currentIndex];
                                updated[currentIndex] = {
                                    ...currentMsg,
                                    executionSteps: [...(currentMsg.executionSteps || []), data.step],
                                };
                            }
                            return updated;
                        });
                    }
                } else if (data.type === 'chat_response') {
                    console.log('[Chat] Received response:', data);
                    const currentIndex = currentAssistantIndex.current;
                    console.log('[Chat] Current assistant index:', currentIndex);

                    // Update current assistant message with final response
                    if (currentIndex !== null) {
                        setConversationHistory((prev) => {
                            const updated = [...prev];
                            console.log('[Chat] Updating message at index:', currentIndex);
                            if (updated[currentIndex]) {
                                const currentMsg = updated[currentIndex];
                                updated[currentIndex] = {
                                    ...currentMsg,
                                    content: data.content,
                                    isStreaming: false,
                                };
                            } else {
                                console.error('[Chat] Message not found at index:', currentIndex);
                            }
                            return updated;
                        });
                    } else {
                        console.warn('[Chat] No active assistant message to update');
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
                        className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'
                            }`}
                    >
                        <div
                            className={`max-w-2xl rounded-lg px-4 py-3 ${msg.role === 'user'
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
