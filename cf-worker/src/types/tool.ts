/**
 * Tool Definition Types
 */

export interface ToolParameter {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'array' | 'object';
  description: string;
  required?: boolean;
  enum?: string[] | number[];
  default?: any;
}

export interface ToolDefinition {
  id: string;
  name: string;
  description: string;
  category: 'mouse' | 'keyboard' | 'system' | 'search' | 'utility' | 'orchestration';
  parameters: ToolParameter[];
  returnsObservation: boolean; // Whether tool result is formatted as observation
}

export interface ToolConfig extends ToolDefinition {
  enabled: boolean;
  metadata?: {
    usageCount?: number;
    lastUsed?: string;
  };
}

export interface ToolCallRequest {
  toolId: string;
  parameters: Record<string, any>;
}

export interface ToolCallResult {
  toolId: string;
  success: boolean;
  result?: any;
  error?: string;
  executionTime: number; // milliseconds
}
