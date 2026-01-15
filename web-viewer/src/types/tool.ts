/**
 * Tool Types (Web Viewer)
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
  category: 'mouse' | 'keyboard' | 'system' | 'search' | 'utility';
  parameters: ToolParameter[];
  returnsObservation: boolean;
}

export interface ToolConfig extends ToolDefinition {
  enabled: boolean;
}
