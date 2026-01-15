/**
 * Agent Configuration Types
 */

export interface Metadata {
  createdAt: string; // ISO8601
  updatedAt: string; // ISO8601
  version: string;
  author?: string;
  tags?: string[];
}

export interface ToolReference {
  toolId: string;
  enabled: boolean;
}

export interface Agent {
  id: string; // UUID
  name: string;
  purpose: string;
  systemPrompt: string;
  tools: ToolReference[];
  modelId: string; // e.g., "@cf/meta/llama-3.3-70b-instruct-fp8-fast"
  maxIterations: number;
  metadata: Metadata;
  isDefault?: boolean;
  isPinned?: boolean; // Pinned agents appear at top of list
  isDeletable?: boolean; // false = cannot be deleted (default true)
}

export interface AgentConfig {
  name: string;
  purpose: string;
  systemPrompt: string;
  tools: ToolReference[];
  modelId: string;
  maxIterations: number;
}

export interface ExecutionLog {
  agentId: string;
  userMessage: string;
  iterations: ExecutionStep[];
  finalResponse: string;
  toolCallsCount: number;
  executionTime: number; // milliseconds
  completedAt: string; // ISO8601
  status: 'success' | 'error' | 'interrupted';
  error?: string;
}

export interface ExecutionStep {
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
  // Track which agent created this step (for delegation scenarios)
  agentId?: string;
  agentName?: string;
}

export interface ConversationMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
}

export interface ConversationHistory {
  agentId: string;
  messages: ConversationMessage[];
}
