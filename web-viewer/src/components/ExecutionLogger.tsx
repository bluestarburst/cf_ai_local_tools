/**
 * Execution Logger Component
 * Displays ReAct execution steps
 */

import React from 'react';
import { useExecutionStore } from '../store/executionStore';

export const ExecutionLogger: React.FC = () => {
  const { currentExecution, isExecuting } = useExecutionStore();

  if (!currentExecution) {
    return (
      <div className="p-4 text-gray-500 text-center">
        No execution in progress
      </div>
    );
  }

  // Show loading state if executing but no iterations yet
  if (isExecuting && currentExecution.iterations.length === 0) {
    return (
      <div className="p-4 space-y-4">
        {/* User Message */}
        {currentExecution.userMessage && (
          <div className="bg-white border border-gray-200 rounded-lg p-3">
            <div className="text-xs font-medium text-gray-500 mb-2">Your Request</div>
            <div className="text-sm text-gray-800">{currentExecution.userMessage}</div>
          </div>
        )}
        
        {/* Loading State */}
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 space-y-3">
          <div className="flex items-center space-x-3">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
            <div>
              <div className="text-sm font-medium text-gray-900">Agent is processing...</div>
              <div className="text-xs text-gray-600 mt-0.5">Analyzing request and planning actions</div>
            </div>
          </div>
          
          <div className="space-y-2 text-xs text-gray-600">
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-blue-400 rounded-full animate-pulse"></div>
              <span>Reading system prompt and agent configuration</span>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-blue-400 rounded-full animate-pulse" style={{animationDelay: '0.2s'}}></div>
              <span>Evaluating available tools and capabilities</span>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-blue-400 rounded-full animate-pulse" style={{animationDelay: '0.4s'}}></div>
              <span>Generating reasoning and action plan</span>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="execution-logger h-full overflow-y-auto bg-gray-50 p-4">
      <h3 className="text-sm font-semibold text-gray-700 mb-2">
        Execution Log ({currentExecution.iterations.length} iterations)
      </h3>
      
      {/* Quick Summary */}
      <div className="mb-4 p-3 bg-white border border-gray-200 rounded-md text-xs space-y-1">
        <div className="flex items-center justify-between">
          <span className="font-medium">Quick Summary</span>
          <span className={`px-2 py-0.5 rounded font-semibold ${
            currentExecution.status === 'success' ? 'bg-green-100 text-green-800' :
            currentExecution.status === 'error' ? 'bg-red-100 text-red-800' :
            'bg-yellow-100 text-yellow-800'
          }`}>
            {currentExecution.status.toUpperCase()}
          </span>
        </div>
        {(() => {
          const failed = currentExecution.iterations.filter(i => i.observation?.error).length;
          const successful = currentExecution.iterations.filter(i => i.action && !i.observation?.error).length;
          const thinkingOnly = currentExecution.iterations.filter(i => !i.action).length;
          
          return (
            <div className="grid grid-cols-3 gap-2 mt-2 text-center">
              <div className="p-2 bg-green-50 rounded">
                <div className="font-bold text-green-700">{successful}</div>
                <div className="text-gray-600">Successful</div>
              </div>
              <div className="p-2 bg-red-50 rounded">
                <div className="font-bold text-red-700">{failed}</div>
                <div className="text-gray-600">Failed</div>
              </div>
              <div className="p-2 bg-blue-50 rounded">
                <div className="font-bold text-blue-700">{thinkingOnly}</div>
                <div className="text-gray-600">Reasoning</div>
              </div>
            </div>
          );
        })()}
      </div>

      <div className="space-y-4">
        {currentExecution.iterations.map((iteration, idx) => {
          const hasError = !!iteration.observation?.error;
          const hasAction = !!iteration.action;
          const isThinkingOnly = !hasAction;
          
          return (
            <div 
              key={idx} 
              className={`border rounded-lg p-3 ${
                hasError ? 'bg-red-50 border-red-300' :
                isThinkingOnly ? 'bg-blue-50 border-blue-200' :
                'bg-white border-gray-200'
              }`}
            >
              <div className="text-xs font-medium mb-2 flex items-center justify-between">
                <span className="text-gray-500">Step {idx + 1}</span>
                {hasError && (
                  <span className="text-red-600 font-semibold">FAILED</span>
                )}
                {isThinkingOnly && (
                  <span className="text-blue-600 font-semibold">REASONING</span>
                )}
                {!hasError && hasAction && (
                  <span className="text-green-600 font-semibold">SUCCESS</span>
                )}
              </div>

            {/* Thought */}
            <div className="mb-3">
              <div className="text-xs font-medium text-gray-700 mb-1">💭 Thought</div>
              <div className="text-sm text-gray-800 bg-blue-50 p-2 rounded">
                {iteration.thought}
              </div>
            </div>

            {/* Action */}
            {iteration.action && (
              <div className="mb-3">
                <div className="text-xs font-medium text-gray-700 mb-1">
                  {iteration.observation?.error ? '⚠️ Attempted Action' : '⚡ Action'}
                </div>
                <div 
                  className={`border rounded p-2 text-sm ${
                    iteration.observation?.error 
                      ? 'bg-orange-50 border-orange-200'
                      : 'bg-purple-50 border-purple-200'
                  }`}
                >
                  <div className={`font-mono ${iteration.observation?.error ? 'text-orange-900' : 'text-purple-900'}`}>
                    {iteration.action.tool}
                  </div>
                  {iteration.action.parameters && (
                    <pre className="text-xs text-gray-600 mt-1 overflow-x-auto">
                      {JSON.stringify(iteration.action.parameters, null, 2)}
                    </pre>
                  )}
                </div>
              </div>
            )}

            {/* Observation */}
            {iteration.observation && (
              <div className="mb-3">
                <div className="text-xs font-medium text-gray-700 mb-1">
                  {iteration.observation.error ? '❌ Error' : '✅ Observation'}
                </div>
                <div 
                  className={`text-sm whitespace-pre-wrap p-2 rounded ${
                    iteration.observation.error 
                      ? 'bg-red-50 border border-red-200 text-red-800' 
                      : 'bg-green-50 border border-green-200 text-gray-700'
                  }`}
                >
                  {iteration.observation.error ? (
                    <div>
                      <div className="font-semibold mb-1">Error:</div>
                      <div className="text-sm">{iteration.observation.error}</div>
                      {iteration.observation.result && (
                        <div className="mt-2 pt-2 border-t border-red-300">
                          <div className="font-semibold mb-1">Partial Result:</div>
                          <pre className="text-xs overflow-x-auto">
                            {typeof iteration.observation.result === 'string' 
                              ? iteration.observation.result 
                              : JSON.stringify(iteration.observation.result, null, 2)}
                          </pre>
                        </div>
                      )}
                    </div>
                  ) : (
                    <div>
                      {typeof iteration.observation.result === 'string' 
                        ? iteration.observation.result 
                        : JSON.stringify(iteration.observation.result, null, 2)}
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
          );
        })}

        {/* Final Response */}
        {currentExecution.finalResponse && (
          <div className="mt-4 p-4 bg-green-50 border border-green-200 rounded-md">
            <h4 className="font-medium text-green-900 mb-2">Final Response</h4>
            <p className="text-sm text-gray-700 whitespace-pre-wrap">
              {currentExecution.finalResponse}
            </p>
          </div>
        )}

        {/* Metadata */}
        <div className="mt-4 pt-4 border-t border-gray-200 text-xs text-gray-600 space-y-1">
          <div className="flex items-center gap-2">
            <span>Status:</span>
            <span className={`font-medium px-2 py-0.5 rounded ${
              currentExecution.status === 'success' ? 'bg-green-100 text-green-800' :
              currentExecution.status === 'error' ? 'bg-red-100 text-red-800' :
              'bg-yellow-100 text-yellow-800'
            }`}>
              {currentExecution.status}
            </span>
          </div>
          {currentExecution.error && (
            <div className="text-red-600">
              <span className="font-medium">Error:</span> {currentExecution.error}
            </div>
          )}
          <div>
            <span className="font-medium">Tool Calls:</span> {currentExecution.toolCallsCount} total
            {(() => {
              const failed = currentExecution.iterations.filter(
                i => i.observation?.error
              ).length;
              const successful = currentExecution.toolCallsCount - failed;
              return (
                <span className="ml-2">
                  ({successful} successful, {failed} failed)
                </span>
              );
            })()}
          </div>
          <div>
            <span className="font-medium">Iterations:</span> {currentExecution.iterations.length}
          </div>
          <div>
            <span className="font-medium">Duration:</span> {currentExecution.executionTime}ms
          </div>
          <div className="text-xs text-gray-500 mt-2">
            Completed at: {new Date(currentExecution.completedAt).toLocaleString()}
          </div>
        </div>
      </div>
    </div>
  );
};
