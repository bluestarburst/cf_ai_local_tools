/**
 * Execution Step Component
 * Displays a single ReAct iteration as an expandable chat message
 */

import React, { useState } from 'react';
import { ChevronDown, ChevronRight } from 'lucide-react';

interface ExecutionStepProps {
    step: {
        stepNumber: number;
        thought: string;
        action?: {
            tool: string;
            parameters: Record<string, any>;
        };
        observation?: {
            result: any;
            error?: string;
        };
        // Agent info stored on the step itself (for delegation scenarios)
        agentId?: string;
        agentName?: string;
    };
    // Fallback agent name if not stored on step
    agentName: string;
}

export const ExecutionStep: React.FC<ExecutionStepProps> = ({ step, agentName: fallbackAgentName }) => {
    // Use step's agent name if available (from delegation), otherwise use fallback
    const displayAgentName = step.agentName || fallbackAgentName;
    const [isExpanded, setIsExpanded] = useState(true);

    const hasError = !!step.observation?.error;
    const hasAction = !!step.action;

    // Determine summary text
    const getSummary = () => {
        if (!hasAction) {
            // Pure thought/reasoning step
            return step.thought.length > 60
                ? step.thought.substring(0, 60) + '...'
                : step.thought;
        }

        // Action step
        const actionName = step.action?.tool.replace(/_/g, ' ');
        const status = hasError ? '✗' : '✓';
        return `${status} ${actionName}`;
    };

    return (
        <div className="flex justify-start">
            <div className="max-w-[80%] rounded-lg text-gray-900">
                {/* Header - Always visible */}
                <button
                    onClick={() => setIsExpanded(!isExpanded)}
                    className="w-full py-2 flex items-start gap-2 rounded-t-lg transition-opacity opacity-50 hover:opacity-100"
                >
                    <div className="mt-0.5">
                        {isExpanded ? (
                            <ChevronDown className="w-4 h-4 text-gray-600" />
                        ) : (
                            <ChevronRight className="w-4 h-4 text-gray-600" />
                        )}
                    </div>
                    <div className="flex-1 text-left">
                        <div className="text-xs font-medium text-gray-500 mb-1">
                            {displayAgentName} • Step {step.stepNumber}
                        </div>
                        <div className={`text-sm font-medium ${hasError ? 'text-red-700' : 'text-gray-800'}`}>
                            {getSummary()}
                        </div>
                    </div>
                </button>

                {/* Expanded content */}
                {isExpanded && (
                    <div className="px-4 pb-3 space-y-3 border-t border-gray-200">
                        {/* Thought */}
                        <div>
                            <div className="text-xs font-medium text-gray-600 mb-1 mt-2">Thought</div>
                            <div className="text-sm text-gray-700 bg-blue-50 p-2 rounded">
                                {step.thought}
                            </div>
                        </div>

                        {/* Action */}
                        {step.action && (
                            <div>
                                <div className="text-xs font-medium text-gray-600 mb-1">Action</div>
                                <div className="bg-purple-50 border border-purple-200 rounded p-2">
                                    <div className="font-mono text-sm text-purple-900 font-medium">
                                        {step.action.tool}
                                    </div>
                                    {Object.keys(step.action.parameters).length > 0 && (
                                        <pre className="text-xs text-gray-600 mt-1 overflow-x-auto">
                                            {JSON.stringify(step.action.parameters, null, 2)}
                                        </pre>
                                    )}
                                </div>
                            </div>
                        )}

                        {/* Observation */}
                        {step.observation && (
                            <div>
                                <div className="text-xs font-medium text-gray-600 mb-1">
                                    {step.observation.error ? 'Error' : 'Result'}
                                </div>
                                <div
                                    className={`text-sm p-2 rounded ${
                                        step.observation.error
                                            ? 'bg-red-50 border border-red-200 text-red-800'
                                            : 'bg-green-50 border border-green-200 text-gray-700'
                                    }`}
                                >
                                    {step.observation.error ? (
                                        <div>
                                            <div className="font-semibold">Error:</div>
                                            <div className="mt-1">{step.observation.error}</div>
                                        </div>
                                    ) : (
                                        <div className="whitespace-pre-wrap">
                                            {typeof step.observation.result === 'string'
                                                ? step.observation.result
                                                : JSON.stringify(step.observation.result, null, 2)}
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
};
