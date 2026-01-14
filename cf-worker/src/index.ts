/**
 * Main Worker Entry Point
 */

export { ConnectionManager } from './durable-objects/ConnectionManager';

interface Env {
	AI: Ai;
	CONNECTIONS: DurableObjectNamespace;
	ENVIRONMENT: string;
}

export default {
	async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
		const url = new URL(request.url);

		// CORS headers
		const corsHeaders = {
			'Access-Control-Allow-Origin': '*',
			'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
			'Access-Control-Allow-Headers': 'Content-Type',
		};

		if (request.method === 'OPTIONS') {
			return new Response(null, { headers: corsHeaders });
		}

		// Route: WebSocket connection from Rust app
		if (url.pathname === '/connect' && request.headers.get('Upgrade') === 'websocket') {
			const id = env.CONNECTIONS.idFromName('default');
			const stub = env.CONNECTIONS.get(id);
			return stub.fetch(request);
		}

		// Route: Send command to local app
		if (url.pathname === '/api/command' && request.method === 'POST') {
			try {
				const command = await request.json();
				
				const id = env.CONNECTIONS.idFromName('default');
				const stub = env.CONNECTIONS.get(id);
				
				const response = await stub.fetch(new Request('http://internal/send-command', {
					method: 'POST',
					body: JSON.stringify(command),
					headers: { 'Content-Type': 'application/json' }
				}));
				
				const result = await response.json();
				return new Response(JSON.stringify(result), {
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			} catch (error: any) {
				return new Response(JSON.stringify({ error: error.message }), {
					status: 500,
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			}
		}

		// Route: Chat with LLM (with tool calling)
		if (url.pathname === '/api/chat' && request.method === 'POST') {
			try {
				const { message, conversationHistory = [] } = await request.json() as { message: string; conversationHistory?: any[] };
				
				const tools = [
					{
						name: 'mouse_move',
						description: 'Move the mouse cursor to a specific position on screen',
						parameters: {
							type: 'object',
							properties: {
								x: { type: 'string', description: 'X coordinate' },
								y: { type: 'string', description: 'Y coordinate' },
								duration: { type: 'string', description: 'Time in seconds for movement' }
							},
							required: ['x', 'y']
						}
					},
					{
						name: 'mouse_click',
						description: 'Click a mouse button',
						parameters: {
							type: 'object',
							properties: {
								button: { type: 'string', description: 'Which button to click' }
							},
							required: ['button']
						}
					},
					{
						name: 'mouse_scroll',
						description: 'Scroll the mouse wheel',
						parameters: {
							type: 'object',
							properties: {
								direction: { type: 'string', description: 'Scroll direction' },
								intensity: { type: 'string', description: 'Scroll intensity' }
							},
							required: ['direction']
						}
					},
					{
						name: 'keyboard_input',
						description: 'Type text using the keyboard',
						parameters: {
							type: 'object',
							properties: {
								text: { type: 'string', description: 'Text to type' }
							},
							required: ['text']
						}
					},
					{
						name: 'keyboard_command',
						description: 'Execute a keyboard command (e.g., return, backspace, tab)',
						parameters: {
							type: 'object',
							properties: {
								command: { type: 'string', description: 'Keyboard command to execute' }
							},
							required: ['command']
						}
					},
					{
						name: 'get_mouse_position',
						description: 'Get the current mouse cursor position',
						parameters: {
							type: 'object',
							properties: {}
						}
					}
				] as any;

				const messages = [
					{
						role: 'system',
						content: 'You are a helpful AI assistant that can control a computer through mouse and keyboard automation. When users ask you to perform actions, use the available tools to control the local machine.'
					},
					...conversationHistory,
					{ role: 'user', content: message }
				];

				// Call LLM with tools
				const aiResponse = await env.AI.run('@cf/meta/llama-3.3-70b-instruct-fp8-fast', {
					messages,
					tools,
					stream: false
				}) as any;

				// Execute any tool calls
				const toolCalls: any[] = [];
				const executedTools: any[] = [];

				if (aiResponse.tool_calls && Array.isArray(aiResponse.tool_calls)) {
					for (const toolCall of aiResponse.tool_calls) {
						toolCalls.push(toolCall);

						// Normalize arguments to ensure correct types
						const args = toolCall.arguments || {};
						const normalizedArgs: any = {};
						
						// Ensure numeric fields are numbers
						for (const [key, value] of Object.entries(args)) {
							if ((key === 'x' || key === 'y' || key === 'intensity') && typeof value === 'string') {
								normalizedArgs[key] = parseInt(value, 10);
							} else if (key === 'duration' && typeof value === 'string') {
								normalizedArgs[key] = parseFloat(value);
							} else {
								normalizedArgs[key] = value;
							}
						}

						// Convert tool call to command
						const command = {
							type: toolCall.name,
							...normalizedArgs
						};

						// Send to local app
						const id = env.CONNECTIONS.idFromName('default');
						const stub = env.CONNECTIONS.get(id);
						
						try {
							const response = await stub.fetch(new Request('http://internal/send-command', {
								method: 'POST',
								body: JSON.stringify(command),
								headers: { 'Content-Type': 'application/json' }
							}));
							
							const result = await response.json();
							executedTools.push({
								tool: toolCall.name,
								arguments: normalizedArgs,
								result
							});
						} catch (error: any) {
							executedTools.push({
								tool: toolCall.name,
								arguments: toolCall.arguments,
								error: error.message
							});
						}
					}
				}

				return new Response(JSON.stringify({
					response: (aiResponse as any).response || (aiResponse as any).content || 'Command executed',
					toolCalls,
					executedTools,
					thinking: aiResponse
				}), {
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});

			} catch (error: any) {
				return new Response(JSON.stringify({ error: error.message, details: error.stack }), {
					status: 500,
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			}
		}

		// Route: Get connection status
		if (url.pathname === '/api/status' && request.method === 'GET') {
			const id = env.CONNECTIONS.idFromName('default');
			const stub = env.CONNECTIONS.get(id);
			
			const response = await stub.fetch(new Request('http://internal/status'));
			const status = await response.json();
			
			return new Response(JSON.stringify(status), {
				headers: { ...corsHeaders, 'Content-Type': 'application/json' }
			});
		}

		// Default response
		return new Response(JSON.stringify({
			message: 'CF AI Local Tools Worker',
			endpoints: {
				'/connect': 'WebSocket endpoint for Rust app',
				'/api/command': 'POST - Send command to local app',
				'/api/chat': 'POST - Chat with LLM',
				'/api/status': 'GET - Connection status'
			}
		}), {
			headers: { ...corsHeaders, 'Content-Type': 'application/json' }
		});
	},
};
