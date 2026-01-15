/**
 * Observation Formatter
 * Formats tool execution results as observations for the ReAct loop
 */

import { ToolCallResult } from '../types/tool';

export interface FormattedObservation {
  text: string;
  success: boolean;
  data?: any;
}

/**
 * Format tool result as an observation string
 */
export function formatObservation(result: ToolCallResult): FormattedObservation {
  if (!result.success) {
    return {
      text: `Observation: Tool ${result.toolId} failed. Error: ${result.error}`,
      success: false,
    };
  }

  // Format based on tool type
  let observationText = `Observation: Tool ${result.toolId} executed successfully.`;

  if (result.result) {
    if (typeof result.result === 'object') {
      observationText += ` Result: ${JSON.stringify(result.result)}`;
    } else {
      observationText += ` Result: ${result.result}`;
    }
  }

  observationText += ` (${result.executionTime}ms)`;

  return {
    text: observationText,
    success: true,
    data: result.result,
  };
}

/**
 * Format error as observation
 */
export function formatErrorObservation(toolId: string, error: string): FormattedObservation {
  return {
    text: `Observation: Tool ${toolId} encountered an error: ${error}. Please adjust your approach or try a different tool.`,
    success: false,
  };
}

/**
 * Format validation error as observation
 */
export function formatValidationError(
  toolId: string,
  errors: string[]
): FormattedObservation {
  return {
    text: `Observation: Tool ${toolId} has invalid parameters:\n${errors.map((e) => `- ${e}`).join('\n')}\nPlease correct the parameters and try again.`,
    success: false,
  };
}

/**
 * Create observation for max iterations reached
 */
export function formatMaxIterationsObservation(
  currentIteration: number,
  maxIterations: number
): FormattedObservation {
  return {
    text: `Observation: Maximum iterations (${maxIterations}) reached. Task may be incomplete. Current iteration: ${currentIteration}.`,
    success: false,
  };
}

/**
 * Create observation for successful task completion
 */
export function formatCompletionObservation(message?: string): FormattedObservation {
  return {
    text: `Observation: Task completed successfully.${message ? ` ${message}` : ''}`,
    success: true,
  };
}

/**
 * Parse observation from text to extract key info
 */
export function parseObservation(observationText: string): {
  toolId?: string;
  success: boolean;
  result?: any;
  error?: string;
} {
  const toolMatch = observationText.match(/Tool\s+(\w+)/i);
  const toolId = toolMatch ? toolMatch[1] : undefined;

  const isSuccess =
    observationText.includes('successfully') ||
    observationText.includes('completed');
  const isError =
    observationText.includes('failed') ||
    observationText.includes('error') ||
    observationText.includes('invalid');

  const errorMatch = observationText.match(/Error:\s*(.+?)(?:\.|$)/i);
  const error = errorMatch ? errorMatch[1].trim() : undefined;

  const resultMatch = observationText.match(/Result:\s*(.+?)(?:\s*\(|$)/i);
  let result: any = resultMatch ? resultMatch[1].trim() : undefined;

  // Try to parse JSON result
  if (result) {
    try {
      result = JSON.parse(result);
    } catch {
      // Keep as string if not valid JSON
    }
  }

  return {
    toolId,
    success: isSuccess && !isError,
    result,
    error,
  };
}

/**
 * Create context window for observations (limit size)
 */
export function createObservationContext(
  observations: FormattedObservation[],
  maxTokens = 2000
): string {
  // Rough estimate: 4 chars per token
  const maxChars = maxTokens * 4;

  let context = '';
  let currentLength = 0;

  // Add observations from most recent backwards
  for (let i = observations.length - 1; i >= 0; i--) {
    const obs = observations[i];
    if (currentLength + obs.text.length > maxChars) {
      // Truncate if needed
      const remaining = maxChars - currentLength;
      if (remaining > 100) {
        context = obs.text.substring(0, remaining) + '...\n' + context;
      }
      break;
    }
    context = obs.text + '\n' + context;
    currentLength += obs.text.length;
  }

  return context.trim();
}
