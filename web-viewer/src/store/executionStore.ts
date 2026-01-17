/**
 * Execution Store - Manage ReAct execution state
 */

import { create } from 'zustand';

export interface ExecutionStep {
  step_number: number;
  step_type: 'Thinking' | 'Planning' | 'Action' | 'Observation' | 'Reflection' | 'Completion';
  content: string;
  tool_call?: {
    tool_name: string;
    arguments: any;
    execution_time: {
      secs: number;
      nanos: number;
    } | number;
  };
  tool_observation?: {
    success: boolean;
    message: string;
    data?: any;
    error?: string;
  };
  timestamp: string;
}

interface ExecutionLog {
  agentId: string;
  userMessage: string;
  iterations: ExecutionStep[];
  finalResponse: string;
  toolCallsCount: number;
  executionTime: number;
  completedAt: string;
  status: 'success' | 'error' | 'interrupted';
  error?: string;
}

interface ExecutionState {
  // Current execution
  isExecuting: boolean;
  currentExecution: ExecutionLog | null;

  // Execution history
  executionHistory: ExecutionLog[];

  // Actions
  startExecution: (userMessage?: string, agentId?: string) => void;
  updateCurrentStep: (stepUpdate: Partial<ExecutionStep>) => void;
  addIteration: (iteration: ExecutionStep) => void;
  updateFinalResponse: (finalResponse: string) => void;
  finishExecution: (log: ExecutionLog) => void;
  stopExecution: () => void;
  clearHistory: () => void;
}

export const useExecutionStore = create<ExecutionState>((set, get) => ({
  isExecuting: false,
  currentExecution: null,
  executionHistory: [],

  startExecution: (userMessage = '', agentId = '') => {
    // Set a placeholder execution log to show "in progress" state
    set({
      isExecuting: true,
      currentExecution: {
        agentId,
        userMessage,
        iterations: [],
        finalResponse: '',
        toolCallsCount: 0,
        executionTime: 0,
        completedAt: '',
        status: 'success',
      }
    });
  },

  updateCurrentStep: (stepUpdate: Partial<ExecutionStep>) => {
    const { currentExecution } = get();
    if (!currentExecution) return;

    const iterations = [...currentExecution.iterations];

    // Always work on creating a new iteration for the current step
    // The last iteration might be incomplete
    const lastIndex = iterations.length - 1;

    if (lastIndex >= 0) {
      // Update the last iteration
      iterations[lastIndex] = { ...iterations[lastIndex], ...stepUpdate };
    } else {
      // Create a new iteration if none exists
      iterations.push({
        step_number: 1,
        step_type: 'Thinking',
        content: '',
        timestamp: new Date().toISOString(),
        ...stepUpdate,
      } as ExecutionStep);
    }

    set({
      currentExecution: {
        ...currentExecution,
        iterations,
        toolCallsCount: iterations.filter(i => !!i.tool_call).length,
      },
    });
  },

  addIteration: (iteration: ExecutionStep) => {
    const { currentExecution } = get();
    if (!currentExecution) return;

    set({
      currentExecution: {
        ...currentExecution,
        iterations: [...currentExecution.iterations, iteration],
      },
    });
  },

  updateFinalResponse: (finalResponse: string) => {
    const { currentExecution } = get();
    if (!currentExecution) return;

    set({
      currentExecution: {
        ...currentExecution,
        finalResponse,
      },
    });
  },

  finishExecution: (log: ExecutionLog) => {
    const { executionHistory } = get();
    set({
      isExecuting: false,
      currentExecution: log,
      executionHistory: [log, ...executionHistory].slice(0, 50), // Keep last 50
    });
  },

  stopExecution: () => {
    set({ isExecuting: false });
  },

  clearHistory: () => {
    set({ executionHistory: [], currentExecution: null });
  },
}));
