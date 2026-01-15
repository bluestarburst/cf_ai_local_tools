/**
 * Chat Interface Component
 */

import React, { useState, useRef, useEffect } from 'react';
import { useAgentStore } from '../store/agentStore';
import { useExecutionStore } from '../store/executionStore';
import { ExecutionStep } from './ExecutionStep';

const API_BASE_URL = import.meta.env.VITE_WORKER_URL || 'http://localhost:8787';

interface ChatMessage {
    role: 'user' | 'assistant' | 'execution';
    content: string;
    agentName?: string;
    executionData?: {
        iterations: any[];
        isComplete: boolean;
    };
}

export const ChatInterface: React.FC = () => {
    const { currentAgent } = useAgentStore();
    const { currentExecution, isExecuting, startExecution, finishExecution, updateCurrentStep, addIteration, updateFinalResponse } = useExecutionStore();
    const [message, setMessage] = useState('');
    const [conversationHistory, setConversationHistory] = useState<ChatMessage[]>([]);
    const messagesEndRef = useRef<HTMLDivElement>(null);

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    };

    useEffect(() => {
        scrollToBottom();
    }, [conversationHistory, currentExecution]);

    const handleSend = async () => {
        if (!message.trim() || !currentAgent || isExecuting) return;

        const userMessage = message.trim();
        setMessage('');

        // Add user message to conversation
        const newHistory: ChatMessage[] = [
            ...conversationHistory,
            { role: 'user', content: userMessage }
        ];
        setConversationHistory(newHistory);

        try {
            startExecution(userMessage, currentAgent.id);

            // Add execution placeholder that will be updated in real-time
            const executionMessageIndex = newHistory.length;
            setConversationHistory([
                ...newHistory,
                { role: 'execution', content: '', executionData: { iterations: [], isComplete: false } }
            ]);

            // Use fetch with streaming
            const response = await fetch(`${API_BASE_URL}/api/chat`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    agentId: currentAgent.id,
                    message: userMessage,
                    conversationHistory: newHistory.filter(m => m.role !== 'execution').map(m => ({ role: m.role, content: m.content })),
                    stream: true,
                }),
            });

            if (!response.ok) throw new Error(`HTTP ${response.status}`);

            const reader = response.body?.getReader();
            const decoder = new TextDecoder();

            if (!reader) throw new Error('No response body');

            let currentStepNumber = 0;
            let currentIteration: any = null;
            let buffer = '';
            let currentEventType = '';

            while (true) {
                const { done, value } = await reader.read();
                if (done) break;

                buffer += decoder.decode(value, { stream: true });
                const lines = buffer.split('\n');

                // Keep the last incomplete line in buffer
                buffer = lines.pop() || '';

                for (const line of lines) {
                    if (!line.trim() || line.startsWith(':')) continue;

                    if (line.startsWith('event:')) {
                        currentEventType = line.substring(6).trim();
                        continue;
                    }

                    if (line.startsWith('data:')) {
                        try {
                            const data = JSON.parse(line.substring(5).trim());
                            const eventType = currentEventType || data.type;

                            // Handle different event types
                            if (eventType === 'step_start') {
                                currentStepNumber = data.stepNumber || 0;
                                currentIteration = {
                                    stepNumber: currentStepNumber,
                                    thought: '',
                                    // Include agent info from stream event
                                    agentId: data.agentId,
                                    agentName: data.agentName,
                                };
                                console.log(`[Stream] Step ${currentStepNumber} started by ${data.agentName}`);
                                addIteration(currentIteration);
                            } else if (eventType === 'thought') {
                                if (currentIteration) {
                                    currentIteration.thought = data.thought || '';
                                    updateCurrentStep({
                                        stepNumber: currentStepNumber,
                                        thought: data.thought,
                                        agentId: data.agentId,
                                        agentName: data.agentName,
                                    });
                                }
                            } else if (eventType === 'action') {
                                if (currentIteration) {
                                    currentIteration.action = data.action;
                                    updateCurrentStep({
                                        action: data.action,
                                        agentId: data.agentId,
                                        agentName: data.agentName,
                                    });
                                }
                            } else if (eventType === 'observation') {
                                if (currentIteration) {
                                    currentIteration.observation = data.observation;
                                    updateCurrentStep({
                                        observation: data.observation,
                                        agentId: data.agentId,
                                        agentName: data.agentName,
                                    });
                                }
                            } else if (eventType === 'step_complete') {
                                currentIteration = null;
                            } else if (eventType === 'delegation_start') {
                                // Log delegation start for debugging
                                console.log(`[Delegation] Starting delegation to ${data.delegatedAgentName}`);
                            } else if (eventType === 'delegation_end') {
                                // Log delegation end for debugging
                                console.log(`[Delegation] Completed delegation to ${data.delegatedAgentName}`);
                            } else if (eventType === 'final_response') {
                                updateFinalResponse(data.finalResponse || '');
                            } else if (eventType === 'complete') {
                                // Mark execution as complete and add final message
                                const completedLog = {
                                    agentId: data.agentId || currentAgent.id,
                                    agentName: currentAgent.name,
                                    agentPurpose: currentAgent.purpose,
                                    userMessage: data.userMessage || userMessage,
                                    iterations: data.iterations || [],
                                    finalResponse: data.finalResponse || '',
                                    toolCallsCount: data.toolCallsCount || 0,
                                    executionTime: data.executionTime || 0,
                                    completedAt: data.completedAt || new Date().toISOString(),
                                    status: data.status || 'success',
                                    error: data.error,
                                };

                                finishExecution(completedLog);

                                // Update conversation to show completed execution + final response
                                setConversationHistory(prev => {
                                    const updated = [...prev];
                                    // Update execution message to be complete
                                    updated[executionMessageIndex] = {
                                        role: 'execution',
                                        content: '',
                                        executionData: {
                                            iterations: completedLog.iterations,
                                            isComplete: true
                                        }
                                    };
                                    // Add final response as assistant message
                                    updated.push({
                                        role: 'assistant',
                                        content: completedLog.finalResponse,
                                        agentName: completedLog.agentName
                                    });
                                    return updated;
                                });
                            } else if (eventType === 'error') {
                                console.error('[ChatInterface] Error from stream:', data.error);
                                throw new Error(data.error);
                            }

                            currentEventType = '';
                        } catch (parseError) {
                            console.error('[ChatInterface] Failed to parse SSE data:', parseError);
                        }
                    }
                }
            }
        } catch (error: any) {
            console.error('Chat error:', error);
            alert(`Error: ${error.message}`);
            finishExecution({
                agentId: currentAgent.id,
                userMessage,
                iterations: [],
                finalResponse: 'Error occurred',
                status: 'error',
                executionTime: 0,
                toolCallsCount: 0,
                completedAt: new Date().toISOString(),
            });
        }
    };

    if (!currentAgent) {
        return (
            <div className="flex items-center justify-center h-full text-gray-500">
                Select an agent to start chatting
            </div>
        );
    }

    return (
        <div className="chat-interface h-full flex flex-col bg-white">
            <div className="p-4 border-b border-gray-200">
                <h2 className="text-lg font-semibold text-gray-800">{currentAgent.name}</h2>
                <p className="text-sm text-gray-600 mt-1">{currentAgent.purpose}</p>
            </div>

            <div className="flex-1 overflow-y-auto p-4 space-y-3">
                {conversationHistory.length === 0 ? (
                    <div className="text-center text-gray-500 mt-8">
                        Start a conversation with {currentAgent.name}
                    </div>
                ) : (
                    conversationHistory.map((msg, i) => {
                        if (msg.role === 'user') {
                            return (
                                <div key={i} className="flex justify-end">
                                    <div className="max-w-[70%] rounded-lg px-4 py-2 bg-blue-600 text-white">
                                        <div className="text-xs font-medium mb-1 opacity-75">You</div>
                                        <div className="whitespace-pre-wrap">{msg.content}</div>
                                    </div>
                                </div>
                            );
                        } else if (msg.role === 'assistant') {
                            return (
                                <div key={i} className="flex justify-start">
                                    <div className="max-w-[70%] rounded-lg px-4 py-2 bg-gray-100 text-gray-900">
                                        <div className="text-xs font-medium mb-1 opacity-75">{msg.agentName || currentAgent.name}</div>
                                        <div className="whitespace-pre-wrap">{msg.content}</div>
                                    </div>
                                </div>
                            );
                        } else if (msg.role === 'execution') {
                            // Show execution steps (either live or completed)
                            const iterations = msg.executionData?.isComplete
                                ? msg.executionData.iterations
                                : currentExecution?.iterations || [];

                            return (
                                <div key={i} className="space-y-2">
                                    {iterations.map((step: any, stepIdx: number) => (
                                        <ExecutionStep
                                            key={stepIdx}
                                            step={step}
                                            agentName={currentAgent.name}
                                        />
                                    ))}
                                    {!msg.executionData?.isComplete && isExecuting && (
                                        <div className="flex justify-start">
                                            <div className="max-w-[70%] rounded-lg px-4 py-2 bg-blue-50 border border-blue-200">
                                                <div className="flex items-center gap-2">
                                                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
                                                    <span className="text-sm text-gray-700">Thinking...</span>
                                                </div>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            );
                        }
                        return null;
                    })
                )}
                <div ref={messagesEndRef} />
            </div>

            <div className="p-4 border-t border-gray-200">
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={message}
                        onChange={(e) => setMessage(e.target.value)}
                        onKeyPress={(e) => e.key === 'Enter' && !e.shiftKey && handleSend()}
                        disabled={isExecuting}
                        placeholder="Type your message..."
                        className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
                    />
                    <button
                        onClick={handleSend}
                        disabled={!message.trim() || isExecuting}
                        className={`px-6 py-2 rounded-md font-medium ${message.trim() && !isExecuting ? 'bg-blue-600 text-white hover:bg-blue-700' : 'bg-gray-200 text-gray-500 cursor-not-allowed'}`}
                    >
                        {isExecuting ? 'Thinking...' : 'Send'}
                    </button>
                </div>
            </div>
        </div>
    );
};
