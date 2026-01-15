/**
 * ReAct Loop Orchestrator
 * Implements Reasoning + Acting loop with observation feedback
 *
 * CRITICAL: Observation feedback is provided to the LLM as user messages
 * after each action. This prevents the loop from getting stuck repeating
 * the same action by ensuring the LLM sees the results of previous actions.
 *
 * The conversation flow per iteration is:
 * 1. Assistant: Thought + Action description
 * 2. User: Observation result (success/error)
 * 3. User: Prompt asking "what should you do next?"
 *
 * This forces the LLM to acknowledge results and move forward.
 */

import { Agent, ExecutionLog, ExecutionStep } from '../types/agent';
import { ToolCallRequest, ToolCallResult } from '../types/tool';
import { validateToolCall, getTool } from '../tools/tool-registry';
import { interpolatePrompt, DEFAULT_AGENTS } from '../presets/default-presets';

export interface ReActConfig {
  maxIterations: number;
  stopOnError?: boolean;
  verbose?: boolean;
}

export interface ReActContext {
  agent: Agent;
  userMessage: string;
  conversationHistory: any[];
  env: any; // Worker environment (for AI and CONNECTIONS)
}

// Streaming callback for real-time updates
export type StreamCallback = (event: StreamEvent) => void;

export interface StreamEvent {
  type: 'step_start' | 'step_complete' | 'thought' | 'action' | 'observation' | 'final_response' | 'error' | 'delegation_start' | 'delegation_end';
  stepNumber?: number;
  thought?: string;
  action?: {
    tool: string;
    parameters: Record<string, any>;
  };
  observation?: {
    result: any;
    error?: string;
  };
  finalResponse?: string;
  error?: string;
  // Agent info for tracking which agent created this event
  agentId?: string;
  agentName?: string;
  // For delegation events
  delegatedAgentId?: string;
  delegatedAgentName?: string;
  delegatedTask?: string;
}

/**
 * Execute ReAct loop for an agent
 */
export async function executeReActLoop(
  context: ReActContext,
  config?: Partial<ReActConfig>,
  streamCallback?: StreamCallback
): Promise<ExecutionLog> {
  const startTime = Date.now();
  const {
    maxIterations = context.agent.maxIterations || 5,
    stopOnError = false,
    verbose = true, // Enable verbose logging for debugging
  } = config || {};

  const iterations: ExecutionStep[] = [];
  let isDone = false;
  let currentStep = 0;
  let finalResponse = '';
  let error: string | undefined;

  // Build tools description for prompt
  const toolsDescription = context.agent.tools
    .filter((t) => t.enabled)
    .map((t) => {
      const tool = getTool(t.toolId);
      if (!tool) return '';
      return `- ${tool.name} (${tool.id}): ${tool.description}`;
    })
    .join('\n');

  // Interpolate system prompt
  const systemPrompt = interpolatePrompt(context.agent.systemPrompt, {
    purpose: context.agent.purpose,
    tools: toolsDescription,
  });

  try {
    // Main ReAct loop
    while (!isDone && currentStep < maxIterations) {
      currentStep++;

      if (verbose) {
        console.log(`[ReAct] Step ${currentStep}/${maxIterations}`);
      }

      // Stream: Step started
      if (streamCallback) {
        streamCallback({
          type: 'step_start',
          stepNumber: currentStep,
          agentId: context.agent.id,
          agentName: context.agent.name,
        });
      }

      const step: ExecutionStep = {
        stepNumber: currentStep,
        thought: '',
        agentId: context.agent.id,
        agentName: context.agent.name,
      };

      // Build messages for LLM call
      const messages = [
        { role: 'system', content: systemPrompt },
        ...context.conversationHistory,
        { role: 'user', content: context.userMessage },
      ];

      // Add previous iterations as conversation history
      // CRITICAL: Each iteration needs proper assistant->user flow for tool calling
      for (const iter of iterations) {
        // Add assistant's response (thought + tool call intent)
        if (iter.thought && iter.action) {
          messages.push({
            role: 'assistant',
            content: `Thought: ${iter.thought}\nAction: Using ${iter.action.tool} with ${JSON.stringify(iter.action.parameters)}`,
          });
        } else if (iter.thought) {
          messages.push({
            role: 'assistant',
            content: `Thought: ${iter.thought}`,
          });
        }

        // Add observation as user feedback
        if (iter.observation) {
          const observationText = iter.observation.error
            ? `Error occurred: ${iter.observation.error}`
            : `Result: ${typeof iter.observation.result === 'object' ? JSON.stringify(iter.observation.result) : iter.observation.result}`;

          messages.push({
            role: 'user',
            content: `Observation from previous action:\n${observationText}\n\nAnalyze: Was the action successful? Is the user's request now complete? If yes, respond with your conclusion WITHOUT calling any more tools. If the action succeeded and you're just repeating it, STOP - the task is done.`,
          });
        }
      }

      // Call LLM
      const llmResponse = await callLLM(
        context.env,
        context.agent.modelId,
        messages,
        buildToolDefinitions(context.agent.tools)
      );

      if (verbose) {
        console.log('[ReAct] LLM Response:', JSON.stringify(llmResponse, null, 2));
      }

      // Parse thought from response
      step.thought = extractThought(llmResponse);

      if (verbose) {
        console.log('[ReAct] Extracted thought:', step.thought);
      }

      // Stream: Thought extracted
      if (streamCallback) {
        streamCallback({
          type: 'thought',
          stepNumber: currentStep,
          thought: step.thought,
          agentId: context.agent.id,
          agentName: context.agent.name,
        });
      }

      // Check if LLM wants to use a tool
      const toolCalls = parseToolCalls(llmResponse);
      
      if (verbose) {
        console.log('[ReAct] Parsed tool calls:', JSON.stringify(toolCalls, null, 2));
      }

      if (toolCalls.length === 0) {
        // No tool call - agent is done
        isDone = true;
        finalResponse = llmResponse.response || llmResponse.content || step.thought;
      } else {
        // Execute first tool call
        const toolCall = toolCalls[0];
        
        // Validate tool call
        const validation = validateToolCall(toolCall.toolId, toolCall.parameters);
        if (!validation.valid) {
          step.observation = {
            result: null,
            error: `Tool validation failed: ${validation.errors.join(', ')}`,
          };
          
          if (stopOnError) {
            isDone = true;
            error = validation.errors.join(', ');
          }
        } else {
          // Execute tool
          step.action = {
            tool: toolCall.toolId,
            parameters: toolCall.parameters,
          };

          // Stream: Action about to execute
          if (streamCallback) {
            streamCallback({
              type: 'action',
              stepNumber: currentStep,
              action: step.action,
              agentId: context.agent.id,
              agentName: context.agent.name,
            });
          }

          const toolResult = await executeToolCall(context.env, toolCall, context, streamCallback);

          step.observation = {
            result: toolResult.result,
            error: toolResult.error,
          };

          // Stream: Observation received
          if (streamCallback) {
            streamCallback({
              type: 'observation',
              stepNumber: currentStep,
              observation: step.observation,
              agentId: context.agent.id,
              agentName: context.agent.name,
            });
          }

          if (toolResult.error && stopOnError) {
            isDone = true;
            error = toolResult.error;
          }

          // Check if this was the final action
          if (toolResult.success && isFinalAction(step.thought)) {
            isDone = true;
            finalResponse = `Completed action: ${toolCall.toolId}`;
          }
        }
      }

      if (verbose) {
        console.log('[ReAct] Step complete:', JSON.stringify(step, null, 2));
      }

      iterations.push(step);

      // Stream: Step complete
      if (streamCallback) {
        streamCallback({
          type: 'step_complete',
          stepNumber: currentStep,
          agentId: context.agent.id,
          agentName: context.agent.name,
        });
      }
    }

    // If we hit max iterations, use last thought as final response
    if (!finalResponse && iterations.length > 0) {
      finalResponse = iterations[iterations.length - 1].thought || 'Maximum iterations reached';
    }

    // Stream: Final response
    if (streamCallback && finalResponse) {
      streamCallback({
        type: 'final_response',
        finalResponse,
        agentId: context.agent.id,
        agentName: context.agent.name,
      });
    }

  } catch (e: any) {
    error = e.message || String(e);
    finalResponse = `Error: ${error}`;

    // Stream: Error occurred
    if (streamCallback) {
      streamCallback({
        type: 'error',
        error,
        agentId: context.agent.id,
        agentName: context.agent.name,
      });
    }
  }

  const executionTime = Date.now() - startTime;

  return {
    agentId: context.agent.id,
    userMessage: context.userMessage,
    iterations,
    finalResponse,
    toolCallsCount: iterations.filter((i) => i.action).length,
    executionTime,
    completedAt: new Date().toISOString(),
    status: error ? 'error' : 'success',
    error,
  };
}

/**
 * Call LLM with tool support
 */
async function callLLM(
  env: any,
  modelId: string,
  messages: any[],
  tools: any[]
): Promise<any> {
  const response = await env.AI.run(modelId, {
    messages,
    tools: tools.length > 0 ? tools : undefined,
    stream: false,
  });

  return response;
}

/**
 * Build tool definitions for LLM
 */
function buildToolDefinitions(toolRefs: { toolId: string; enabled: boolean }[]): any[] {
  return toolRefs
    .filter((t) => t.enabled)
    .map((t) => {
      const tool = getTool(t.toolId);
      if (!tool) return null;

      return {
        name: tool.id,
        description: tool.description,
        parameters: {
          type: 'object',
          properties: tool.parameters.reduce((acc: any, param) => {
            acc[param.name] = {
              type: param.type,
              description: param.description,
            };
            return acc;
          }, {}),
          required: tool.parameters.filter((p) => p.required).map((p) => p.name),
        },
      };
    })
    .filter(Boolean) as any[];
}

/**
 * Execute a tool call
 * - Worker-side tools (web_search, fetch_url, delegate_to_agent): Executed on Worker
 * - Local tools (mouse/keyboard/screenshot): Sent to Rust app via WebSocket
 */
async function executeToolCall(
  env: any,
  toolCall: ToolCallRequest,
  parentContext?: ReActContext,
  streamCallback?: StreamCallback
): Promise<ToolCallResult> {
  const startTime = Date.now();

  // Special handling for agent delegation
  if (toolCall.toolId === 'delegate_to_agent') {
    return executeDelegation(env, toolCall.parameters, parentContext, streamCallback);
  }

  // Special handling for worker-side web tools
  if (toolCall.toolId === 'web_search') {
    return executeWebSearch(env, toolCall.parameters, startTime);
  }

  if (toolCall.toolId === 'fetch_url') {
    return executeFetchUrl(env, toolCall.parameters, startTime);
  }

  // All other tools go to Rust app (mouse/keyboard/screenshot)
  try {
    // Get Durable Object stub
    const id = env.CONNECTIONS.idFromName('default');
    const stub = env.CONNECTIONS.get(id);

    // Send command to Rust app
    const response = await stub.fetch(
      new Request('http://internal/send-command', {
        method: 'POST',
        body: JSON.stringify({
          type: toolCall.toolId,
          ...toolCall.parameters,
        }),
        headers: { 'Content-Type': 'application/json' },
      })
    );

    const result = await response.json();

    return {
      toolId: toolCall.toolId,
      success: result.success !== false,
      result: result.result || result,
      executionTime: Date.now() - startTime,
    };
  } catch (e: any) {
    return {
      toolId: toolCall.toolId,
      success: false,
      error: e.message || String(e),
      executionTime: Date.now() - startTime,
    };
  }
}

/**
 * Execute delegation to another agent
 * Passes through the stream callback so delegated agent's steps are streamed to UI
 */
async function executeDelegation(
  env: any,
  parameters: Record<string, any>,
  parentContext?: ReActContext,
  streamCallback?: StreamCallback
): Promise<ToolCallResult> {
  const startTime = Date.now();
  const { agent_id, task } = parameters;

  try {
    // Find the target agent
    const targetAgentPreset = DEFAULT_AGENTS[agent_id];
    if (!targetAgentPreset) {
      return {
        toolId: 'delegate_to_agent',
        success: false,
        error: `Agent not found: ${agent_id}. Available agents: ${Object.keys(DEFAULT_AGENTS).join(', ')}`,
        executionTime: Date.now() - startTime,
      };
    }

    const targetAgent = targetAgentPreset.content;

    // Prevent infinite recursion - don't allow delegating to orchestrator
    if (agent_id === 'orchestrator-agent') {
      return {
        toolId: 'delegate_to_agent',
        success: false,
        error: 'Cannot delegate to orchestrator agent (would cause infinite recursion)',
        executionTime: Date.now() - startTime,
      };
    }

    console.log(`[Delegation] Delegating to ${agent_id}: "${task}"`);

    // Stream: Delegation starting
    if (streamCallback) {
      streamCallback({
        type: 'delegation_start',
        delegatedAgentId: agent_id,
        delegatedAgentName: targetAgent.name,
        delegatedTask: task,
        agentId: parentContext?.agent.id,
        agentName: parentContext?.agent.name,
      });
    }

    // Create context for the delegated agent
    const delegatedContext: ReActContext = {
      agent: targetAgent,
      userMessage: task,
      conversationHistory: [],
      env,
    };

    // Execute the delegated agent with the stream callback
    // This allows delegated agent's steps to stream to the UI in real-time
    const delegatedResult = await executeReActLoop(
      delegatedContext,
      { maxIterations: Math.min(targetAgent.maxIterations, 5) },
      streamCallback // Pass through the stream callback!
    );

    console.log(`[Delegation] ${agent_id} completed with status: ${delegatedResult.status}`);

    // Stream: Delegation ended
    if (streamCallback) {
      streamCallback({
        type: 'delegation_end',
        delegatedAgentId: agent_id,
        delegatedAgentName: targetAgent.name,
        agentId: parentContext?.agent.id,
        agentName: parentContext?.agent.name,
      });
    }

    // Format the result for the orchestrator
    const summary = {
      delegatedAgent: targetAgent.name,
      task,
      status: delegatedResult.status,
      response: delegatedResult.finalResponse,
      stepsExecuted: delegatedResult.iterations.length,
      toolCallsMade: delegatedResult.toolCallsCount,
      // Include the iterations so they can be displayed in the UI
      iterations: delegatedResult.iterations,
    };

    return {
      toolId: 'delegate_to_agent',
      success: delegatedResult.status === 'success',
      result: summary,
      error: delegatedResult.error,
      executionTime: Date.now() - startTime,
    };
  } catch (e: any) {
    return {
      toolId: 'delegate_to_agent',
      success: false,
      error: `Delegation failed: ${e.message || String(e)}`,
      executionTime: Date.now() - startTime,
    };
  }
}

/**
 * Execute web search using external search API
 * This is a Worker-side tool - executed on the Worker, not sent to Rust app
 */
async function executeWebSearch(
  env: any,
  parameters: Record<string, any>,
  startTime: number
): Promise<ToolCallResult> {
  const { query, time_range, language } = parameters;

  try {
    // Use a search API - for now, return a placeholder
    // In production, integrate with SearXNG or another search API
    const searchUrl = `https://api.duckduckgo.com/?q=${encodeURIComponent(query)}&format=json`;

    const response = await fetch(searchUrl);
    const data = await response.json() as any;

    // Parse DuckDuckGo instant answer API response
    const results = [];

    if (data.AbstractText) {
      results.push({
        title: data.Heading || 'Summary',
        url: data.AbstractURL || '',
        snippet: data.AbstractText,
      });
    }

    if (data.RelatedTopics) {
      for (const topic of data.RelatedTopics.slice(0, 5)) {
        if (topic.Text && topic.FirstURL) {
          results.push({
            title: topic.Text.split(' - ')[0] || 'Related',
            url: topic.FirstURL,
            snippet: topic.Text,
          });
        }
      }
    }

    return {
      toolId: 'web_search',
      success: true,
      result: {
        query,
        resultCount: results.length,
        results,
      },
      executionTime: Date.now() - startTime,
    };
  } catch (e: any) {
    return {
      toolId: 'web_search',
      success: false,
      error: `Web search failed: ${e.message || String(e)}`,
      executionTime: Date.now() - startTime,
    };
  }
}

/**
 * Fetch and parse URL content
 * This is a Worker-side tool - executed on the Worker, not sent to Rust app
 */
async function executeFetchUrl(
  env: any,
  parameters: Record<string, any>,
  startTime: number
): Promise<ToolCallResult> {
  const { url, extract_type = 'text' } = parameters;

  try {
    const response = await fetch(url, {
      headers: {
        'User-Agent': 'Mozilla/5.0 (compatible; CFAIBot/1.0)',
      },
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const html = await response.text();

    // Simple HTML to text extraction
    let content = html
      // Remove script and style tags
      .replace(/<script[^>]*>[\s\S]*?<\/script>/gi, '')
      .replace(/<style[^>]*>[\s\S]*?<\/style>/gi, '')
      // Remove HTML tags
      .replace(/<[^>]+>/g, ' ')
      // Decode common entities
      .replace(/&nbsp;/g, ' ')
      .replace(/&amp;/g, '&')
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>')
      .replace(/&quot;/g, '"')
      // Clean up whitespace
      .replace(/\s+/g, ' ')
      .trim();

    // Truncate to reasonable length
    if (content.length > 5000) {
      content = content.substring(0, 5000) + '...';
    }

    return {
      toolId: 'fetch_url',
      success: true,
      result: {
        url,
        contentLength: content.length,
        content,
      },
      executionTime: Date.now() - startTime,
    };
  } catch (e: any) {
    return {
      toolId: 'fetch_url',
      success: false,
      error: `URL fetch failed: ${e.message || String(e)}`,
      executionTime: Date.now() - startTime,
    };
  }
}

/**
 * Extract thought from LLM response
 */
function extractThought(response: any): string {
  if (typeof response === 'string') {
    return response;
  }

  const text = response.response || response.content || '';

  // If there are tool calls but no response text, generate a thought
  if (response.tool_calls && response.tool_calls.length > 0) {
    // Check if there's explicit thought text
    if (text && text.trim()) {
      return text.trim();
    }

    // Generate thought from tool call
    const toolName = response.tool_calls[0].name;
    const params = response.tool_calls[0].arguments || {};
    return `I will use the ${toolName} tool with parameters: ${JSON.stringify(params)}`;
  }

  // No tool calls - this is final response or reasoning
  if (text && text.trim()) {
    // Try to extract "Thought:" pattern
    const thoughtMatch = text.match(/Thought:\s*(.+?)(?:\n|Action:|$)/is);
    if (thoughtMatch) {
      return thoughtMatch[1].trim();
    }
    return text.trim();
  }

  return 'Processing...';
}

/**
 * Parse tool calls from LLM response
 */
function parseToolCalls(response: any): ToolCallRequest[] {
  const toolCalls: ToolCallRequest[] = [];

  // Check for tool_calls array (standard format)
  if (response.tool_calls && Array.isArray(response.tool_calls)) {
    for (const call of response.tool_calls) {
      const parameters = call.arguments || call.parameters || {};
      
      // Coerce string parameters to correct types and clean up empty strings
      const tool = getTool(call.name || call.id);
      if (tool) {
        for (const param of tool.parameters) {
          if (param.name in parameters) {
            const value = parameters[param.name];

            // Remove empty strings for optional parameters (treat as not provided)
            if (typeof value === 'string' && value.trim() === '' && !param.required) {
              delete parameters[param.name];
              continue;
            }

            // Coerce numbers
            if (param.type === 'number' && typeof value === 'string' && !isNaN(Number(value))) {
              parameters[param.name] = Number(value);
            }

            // Coerce booleans
            if (param.type === 'boolean' && typeof value === 'string') {
              const lowerValue = value.toLowerCase();
              if (lowerValue === 'true') {
                parameters[param.name] = true;
              } else if (lowerValue === 'false') {
                parameters[param.name] = false;
              }
            }
          }
        }
      }
      
      toolCalls.push({
        toolId: call.name || call.id,
        parameters,
      });
    }
  }

  // Check for Action: pattern in text
  const text = response.response || response.content || '';
  const actionMatch = text.match(/Action:\s*(\w+)\((.+?)\)/is);
  if (actionMatch && toolCalls.length === 0) {
    try {
      const toolId = actionMatch[1];
      const paramsStr = actionMatch[2];
      const parameters = JSON.parse(`{${paramsStr}}`);
      toolCalls.push({ toolId, parameters });
    } catch {
      // Failed to parse - ignore
    }
  }

  return toolCalls;
}

/**
 * Check if thought indicates this is the final action
 */
function isFinalAction(thought: string): boolean {
  const finalIndicators = [
    'done',
    'complete',
    'finished',
    'final',
    'conclude',
    'task completed',
    'no more actions',
  ];

  const lowerThought = thought.toLowerCase();
  return finalIndicators.some((indicator) => lowerThought.includes(indicator));
}
