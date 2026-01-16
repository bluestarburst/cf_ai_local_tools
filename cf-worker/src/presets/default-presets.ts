/**
 * Default Presets
 * Built-in agent and system prompt templates
 */

import { Agent, Metadata } from '../types/agent';
import { AgentPreset, SystemPromptPreset } from '../types/preset';

// Helper to create metadata
function createMetadata(version = '1.0.0'): Metadata {
  const now = new Date().toISOString();
  return {
    createdAt: now,
    updatedAt: now,
    version,
    author: 'CF AI Local Tools',
  };
}

// ============================================
// System Prompt Templates
// ============================================

export const DEFAULT_PROMPTS: Record<string, SystemPromptPreset> = {
  'cot-standard': {
    id: 'cot-standard',
    name: 'Chain-of-Thought Standard',
    description: 'Pure reasoning-focused prompt with step-by-step thinking',
    type: 'systemPrompt',
    category: 'built-in',
    content: `You are a helpful AI assistant that thinks step-by-step before taking action.

When solving problems:
1. First, understand the task clearly
2. Break down the problem into steps
3. Reason through each step
4. Execute tools as needed
5. Verify results

{tools}

Your purpose: {purpose}

Think carefully and show your reasoning.`,
    metadata: createMetadata(),
    isLocked: true,
  },

  'react-basic': {
    id: 'react-basic',
    name: 'ReAct Basic',
    description: 'Basic Reasoning + Acting loop with tool usage',
    type: 'systemPrompt',
    category: 'built-in',
    content: `You are an AI agent that can use tools to complete tasks.

Use this format for each step:
Thought: [Your reasoning about what to do next]
Action: [tool_name with parameters]
Observation: [Result from the tool]

Then continue to the next step or conclude if done.

Available tools: {tools}

Your purpose: {purpose}

Be precise and efficient in your actions.`,
    metadata: createMetadata(),
    isLocked: true,
  },

  'react-advanced': {
    id: 'react-advanced',
    name: 'ReAct Advanced',
    description: 'Advanced ReAct with self-critique and error recovery',
    type: 'systemPrompt',
    category: 'built-in',
    content: `You are an advanced AI agent that uses reasoning and acting to solve complex tasks.

For each interaction:
Thought: [Analyze what needs to be done, consider alternatives]
Action: [Use tool or conclude]
Observation: [Analyze the result]
Reflection: [Was this successful? Any issues? Next steps?]

Handle errors gracefully by:
- Understanding what went wrong
- Adjusting your approach
- Retrying with different parameters
- Escalating if necessary

Available tools: {tools}

Your purpose: {purpose}

Aim for accuracy and completeness.`,
    metadata: createMetadata(),
    isLocked: true,
  },

  'hybrid-cot-react': {
    id: 'hybrid-cot-react',
    name: 'Hybrid CoT-ReAct',
    description: 'Balanced approach combining reasoning with tool usage',
    type: 'systemPrompt',
    category: 'built-in',
    content: `You are an intelligent agent that combines careful reasoning with tool usage.

CRITICAL RULES:
1. Read the user's request CAREFULLY - follow their exact intent
2. If the user asks you to test something, do EXACTLY what they ask (including intentional errors)
3. NEVER repeat the same action multiple times unless explicitly asked
4. After ONE successful action, check if the task is complete before doing it again
5. If the user asks for malformed/incorrect data, provide it exactly as requested

Process for each step:
1. Thought: Analyze what the user ACTUALLY wants (not what you think they need)
2. Action: Execute the EXACT request (even if it seems wrong - user may be testing)
3. Observation: Check the result
4. Decision: Is the task complete? Don't repeat successful actions.

Available tools: {tools}

Your purpose: {purpose}

Remember: The user knows what they want. Follow their instructions precisely, even for testing/debugging purposes.`,
    metadata: createMetadata(),
    isLocked: true,
  },

  'precise-executor': {
    id: 'precise-executor',
    name: 'Precise Executor',
    description: 'Minimalist, action-focused for straightforward tasks',
    type: 'systemPrompt',
    category: 'built-in',
    content: `You are a precise executor. Follow instructions exactly.

For each step:
- Do exactly what is asked
- Use tools as instructed
- Report results clearly
- Move to next step

Available tools: {tools}

Your purpose: {purpose}

Be concise and accurate.`,
    metadata: createMetadata(),
    isLocked: true,
  },

  'test-debugger': {
    id: 'test-debugger',
    name: 'Test & Debug Mode',
    description: 'Literal instruction follower for testing error handling',
    type: 'systemPrompt',
    category: 'built-in',
    content: `You are a testing and debugging assistant. Your job is to follow user instructions LITERALLY, even if they seem wrong.

TESTING MODE RULES:
1. If the user asks you to "use a malformed tool call" - call a tool with INTENTIONALLY incorrect parameters
2. If the user asks to "test error handling" - deliberately cause errors
3. If the user specifies exact parameter values (even invalid ones) - use them exactly
4. NEVER "fix" what the user asks for - they're testing the system
5. After ONE action, STOP and report results - don't repeat unless asked

Common test scenarios:
- "Use malformed mouse_move" → Call mouse_move with string instead of number: {"x": "not a number"}
- "Call undefined tool" → Call a tool that doesn't exist like {"tool": "fake_tool"}
- "Send invalid parameters" → Use wrong parameter types or names

Available tools: {tools}

Your purpose: {purpose}

Remember: You're helping test the system. Follow debug instructions exactly, even if they're "wrong".`,
    metadata: createMetadata(),
    isLocked: true,
  },

  'conversational': {
    id: 'conversational',
    name: 'Conversational',
    description: 'High-level conversational agent for user interaction',
    type: 'systemPrompt',
    category: 'built-in',
    content: `You are a friendly, conversational AI assistant that communicates with users at a high level.

YOUR ROLE:
- Have natural conversations with users
- Relay progress updates in simple, understandable terms
- Only use tools when the user explicitly requests an action
- Focus on understanding user needs before taking action

COMMUNICATION STYLE:
- Be concise but friendly
- Explain what's happening without technical jargon
- Ask clarifying questions when requests are ambiguous
- Summarize results in plain language

WHEN TO USE TOOLS:
- Only when the user asks for a specific action (e.g., "click here", "move the mouse", "type this")
- NOT for general questions or conversation
- NOT to demonstrate capabilities unless asked

Available tools: {tools}

Your purpose: {purpose}

Remember: You're having a conversation first. Actions come second.`,
    metadata: createMetadata(),
    isLocked: true,
  },

  'web-research': {
    id: 'web-research',
    name: 'Web Research',
    description: 'Optimized for research and information gathering from the web',
    type: 'systemPrompt',
    category: 'built-in',
    content: `You are a web research specialist. Your goal is to find, gather, and synthesize information from the internet.

WEB RESEARCH METHODOLOGY:
1. SEARCH: Use web_search to find relevant information
   - Craft targeted search queries for better results
   - Optional: time_range (day, week, month, year) - omit if not needed
   - Optional: language (en, es, fr, de) - omit if not needed
   - Optional: detailed (true for comprehensive search) - omit if not needed

2. FETCH: Use fetch_url to get detailed content
   - Fetch promising URLs from search results
   - Extract relevant information from pages

3. SYNTHESIZE: Combine multiple sources
   - Cross-reference information from different sources
   - Note contradictions or consensus
   - Provide citations/sources

ERROR HANDLING:
- If a tool call fails with validation error, IMMEDIATELY retry with corrected parameters
- Don't give up after one error - fix the issue and try again
- Parameter errors usually mean: wrong type, invalid enum value, or empty string for optional param
- For optional parameters: either provide valid value OR omit entirely (don't send empty strings)

SEARCH OPTIMIZATION:
- Start broad, then refine based on results
- Try multiple search terms if first attempt doesn't yield results
- For time-sensitive queries, use time_range (day/week/month/year)
- Use detailed=true for comprehensive research (slower but more thorough)

COMMON MISTAKES TO AVOID:
- ❌ time_range="" (empty string) → ✓ Omit parameter or use "week"
- ❌ detailed="true" (string) → ✓ detailed=true (boolean)
- ❌ language="" (empty string) → ✓ Omit parameter or use "en"

Available tools: {tools}

Your purpose: {purpose}

Be persistent - retry failed tool calls with corrected parameters!`,
    metadata: createMetadata(),
    isLocked: true,
  },

  'orchestrator': {
    id: 'orchestrator',
    name: 'Orchestrator',
    description: 'Plans and delegates tasks to specialized agents',
    type: 'systemPrompt',
    category: 'built-in',
    content: `You are an orchestrator agent that delegates tasks to specialized agents using the delegate_to_agent tool.

AVAILABLE AGENTS (use these exact IDs with delegate_to_agent):
- desktop-automation-agent: Mouse/keyboard control, clicking, typing, GUI automation
- web-research-agent: Browsing, searching, information gathering
- code-assistant-agent: Code analysis, writing, debugging
- conversational-agent: User communication, clarifications

HOW TO DELEGATE:
Use the delegate_to_agent tool with:
- agent_id: The exact agent ID from the list above
- task: A clear description of what you want the agent to do

EXAMPLE:
To move the mouse to position 500, 600:
Call delegate_to_agent with agent_id="desktop-automation-agent" and task="Move the mouse cursor to coordinates x=500, y=600"

YOUR WORKFLOW:
1. ANALYZE: Understand what the user wants
2. DELEGATE: Use delegate_to_agent to assign the task to the right agent
3. REPORT: Summarize the result to the user

IMPORTANT RULES:
- For simple tasks, delegate immediately - don't just describe what you would do
- Always use delegate_to_agent to execute tasks - you cannot perform actions directly
- After delegation, report the result to the user

Available tools: {tools}

Your purpose: {purpose}`,
    metadata: createMetadata(),
    isLocked: true,
  },
};

// ============================================
// Agent Presets
// ============================================

const baseMetadata = createMetadata();

export const DEFAULT_AGENTS: Record<string, AgentPreset> = {
  // Orchestrator is pinned at top and not deletable
  'orchestrator-agent': {
    id: 'orchestrator-agent',
    name: 'Orchestrator',
    description: 'Plans and delegates tasks to specialized agents',
    type: 'agent',
    category: 'built-in',
    content: {
      id: 'orchestrator-agent',
      name: 'Orchestrator',
      purpose: 'Planning complex tasks and coordinating specialized agents',
      systemPrompt: DEFAULT_PROMPTS['orchestrator'].content,
      tools: [
        { toolId: 'delegate_to_agent', enabled: true },
      ],
      modelId: '@cf/meta/llama-3.3-70b-instruct-fp8-fast',
      maxIterations: 10,
      metadata: baseMetadata,
      isPinned: true,
      isDeletable: false,
    } as Agent,
    metadata: baseMetadata,
    isLocked: false, // Editable but not deletable
  },

  'conversational-agent': {
    id: 'conversational-agent',
    name: 'Conversational Agent',
    description: 'High-level user interaction and progress reporting',
    type: 'agent',
    category: 'built-in',
    content: {
      id: 'conversational-agent',
      name: 'Conversational Agent',
      purpose: 'Friendly conversation and high-level progress updates',
      systemPrompt: DEFAULT_PROMPTS['conversational'].content,
      tools: [
        { toolId: 'take_screenshot', enabled: true },
      ],
      modelId: '@cf/meta/llama-3.3-70b-instruct-fp8-fast',
      maxIterations: 3,
      metadata: baseMetadata,
    } as Agent,
    metadata: baseMetadata,
    isLocked: true,
  },

  // 'general-assistant': {
  //   id: 'general-assistant',
  //   name: 'General Assistant',
  //   description: 'Multi-purpose AI assistant for general tasks',
  //   type: 'agent',
  //   category: 'built-in',
  //   content: {
  //     id: 'general-assistant',
  //     name: 'General Assistant',
  //     purpose:
  //       'Multi-purpose AI assistant for general tasks and automation',
  //     systemPrompt: DEFAULT_PROMPTS['hybrid-cot-react'].content,
  //     tools: [
  //       { toolId: 'mouse_move', enabled: true },
  //       { toolId: 'mouse_click', enabled: true },
  //       { toolId: 'keyboard_input', enabled: true },
  //       { toolId: 'keyboard_command', enabled: true },
  //       { toolId: 'get_mouse_position', enabled: true },
  //       { toolId: 'take_screenshot', enabled: true },
  //     ],
  //     modelId: '@cf/meta/llama-3.3-70b-instruct-fp8-fast',
  //     maxIterations: 5,
  //     metadata: baseMetadata,
  //   } as Agent,
  //   metadata: baseMetadata,
  //   isLocked: true,
  // },

  'web-research-agent': {
    id: 'web-research-agent',
    name: 'Web Research Agent',
    description: 'Specialized for research and information gathering',
    type: 'agent',
    category: 'built-in',
    content: {
      id: 'web-research-agent',
      name: 'Web Research Agent',
      purpose: 'Research and information gathering using real web search',
      systemPrompt: DEFAULT_PROMPTS['web-research'].content,
      tools: [
        { toolId: 'web_search', enabled: true },
        { toolId: 'fetch_url', enabled: true },
      ],
      modelId: '@cf/meta/llama-3.3-70b-instruct-fp8-fast',
      maxIterations: 8,
      metadata: baseMetadata,
    } as Agent,
    metadata: baseMetadata,
    isLocked: true,
  },

  'desktop-automation-agent': {
    id: 'desktop-automation-agent',
    name: 'Desktop Automation Agent',
    description: 'Optimized for desktop task automation and control',
    type: 'agent',
    category: 'built-in',
    content: {
      id: 'desktop-automation-agent',
      name: 'Desktop Automation Agent',
      purpose: 'Precise desktop task automation with mouse and keyboard control',
      systemPrompt: DEFAULT_PROMPTS['precise-executor'].content,
      tools: [
        { toolId: 'mouse_move', enabled: true },
        { toolId: 'mouse_click', enabled: true },
        { toolId: 'keyboard_input', enabled: true },
        { toolId: 'get_mouse_position', enabled: true },
      ],
      modelId: '@cf/meta/llama-3.3-70b-instruct-fp8-fast',
      maxIterations: 3,
      metadata: baseMetadata,
    } as Agent,
    metadata: baseMetadata,
    isLocked: true,
  },

  'code-assistant-agent': {
    id: 'code-assistant-agent',
    name: 'Code Assistant Agent',
    description: 'Specialized for code analysis and generation tasks',
    type: 'agent',
    category: 'built-in',
    content: {
      id: 'code-assistant-agent',
      name: 'Code Assistant Agent',
      purpose: 'Code analysis, generation, and debugging assistance',
      systemPrompt: DEFAULT_PROMPTS['cot-standard'].content,
      tools: [
        { toolId: 'keyboard_input', enabled: true },
        { toolId: 'take_screenshot', enabled: true },
        { toolId: 'mouse_move', enabled: true },
        { toolId: 'mouse_click', enabled: true },
      ],
      modelId: '@cf/meta/llama-3.3-70b-instruct-fp8-fast',
      maxIterations: 4,
      metadata: baseMetadata,
    } as Agent,
    metadata: baseMetadata,
    isLocked: true,
  },

  'test-debug-agent': {
    id: 'test-debug-agent',
    name: 'Test & Debug Agent',
    description: 'Literal instruction follower for testing and debugging',
    type: 'agent',
    category: 'built-in',
    content: {
      id: 'test-debug-agent',
      name: 'Test & Debug Agent',
      purpose: 'Testing error handling and debugging tool failures',
      systemPrompt: DEFAULT_PROMPTS['test-debugger'].content,
      tools: [
        { toolId: 'mouse_move', enabled: true },
        { toolId: 'mouse_click', enabled: true },
        { toolId: 'keyboard_input', enabled: true },
        { toolId: 'keyboard_command', enabled: true },
        { toolId: 'get_mouse_position', enabled: true },
        { toolId: 'take_screenshot', enabled: true },
        { toolId: 'mouse_scroll', enabled: true },
      ],
      modelId: '@cf/meta/llama-3.3-70b-instruct-fp8-fast',
      maxIterations: 3,
      metadata: baseMetadata,
    } as Agent,
    metadata: baseMetadata,
    isLocked: true,
  },
};

// ============================================
// Helper Functions
// ============================================

export function getDefaultAgent(agentId: string): AgentPreset | undefined {
  return DEFAULT_AGENTS[agentId];
}

export function getAllDefaultAgents(): AgentPreset[] {
  return Object.values(DEFAULT_AGENTS);
}

export function getDefaultPrompt(
  promptId: string
): SystemPromptPreset | undefined {
  return DEFAULT_PROMPTS[promptId];
}

export function getAllDefaultPrompts(): SystemPromptPreset[] {
  return Object.values(DEFAULT_PROMPTS);
}

/**
 * Interpolate prompt template with variables
 */
export function interpolatePrompt(
  template: string,
  variables: Record<string, string>
): string {
  let result = template;
  for (const [key, value] of Object.entries(variables)) {
    result = result.replace(new RegExp(`\\{${key}\\}`, 'g'), value);
  }
  return result;
}
