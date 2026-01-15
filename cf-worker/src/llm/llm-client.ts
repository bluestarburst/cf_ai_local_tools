/**
 * LLM Client
 * Unified interface for calling CloudFlare AI models
 */

export interface LLMMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export interface LLMToolDefinition {
  name: string;
  description: string;
  parameters: {
    type: 'object';
    properties: Record<string, any>;
    required?: string[];
  };
}

export interface LLMRequest {
  modelId: string;
  messages: LLMMessage[];
  tools?: LLMToolDefinition[];
  stream?: boolean;
  temperature?: number;
  maxTokens?: number;
}

export interface LLMResponse {
  content?: string;
  response?: string;
  tool_calls?: Array<{
    name: string;
    id?: string;
    arguments: Record<string, any>;
  }>;
  usage?: {
    promptTokens: number;
    completionTokens: number;
  };
}

/**
 * Call CloudFlare AI with standardized interface
 */
export async function callLLM(
  env: any,
  request: LLMRequest
): Promise<LLMResponse> {
  try {
    const response = await env.AI.run(request.modelId, {
      messages: request.messages,
      tools: request.tools,
      stream: request.stream ?? false,
      temperature: request.temperature,
      max_tokens: request.maxTokens,
    });

    return response as LLMResponse;
  } catch (error: any) {
    throw new Error(`LLM call failed: ${error.message || String(error)}`);
  }
}

/**
 * Parse tool calls from LLM response
 */
export function parseToolCalls(response: LLMResponse): Array<{
  toolId: string;
  parameters: Record<string, any>;
}> {
  const toolCalls: Array<{ toolId: string; parameters: Record<string, any> }> =
    [];

  if (response.tool_calls && Array.isArray(response.tool_calls)) {
    for (const call of response.tool_calls) {
      const toolId = call.name || call.id || 'unknown';
      toolCalls.push({
        toolId,
        parameters: call.arguments || {},
      });
    }
  }

  return toolCalls;
}

/**
 * Extract text content from LLM response
 */
export function extractContent(response: LLMResponse): string {
  return response.content || response.response || '';
}

/**
 * Build messages with conversation history
 */
export function buildMessageHistory(
  systemPrompt: string,
  conversationHistory: Array<{ role: string; content: string }>,
  userMessage: string
): LLMMessage[] {
  return [
    { role: 'system', content: systemPrompt },
    ...conversationHistory.map((msg) => ({
      role: msg.role as 'user' | 'assistant',
      content: msg.content,
    })),
    { role: 'user', content: userMessage },
  ] as LLMMessage[];
}
