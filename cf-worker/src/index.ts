/**
 * Main Worker Entry Point
 */

import { executeReActLoop, ReActContext } from './agents/react-loop';
import { createAgent, getAgent, getAllAgents, updateAgent, deleteAgent, duplicateAgent, registerDefaultAgent } from './agents/agent-manager';
import { getAllToolsAsync, refreshToolCache } from './tools/tool-registry';
import { getAllDefaultAgents, getAllDefaultPrompts } from './presets/default-presets';

export { ConnectionManager } from './durable-objects/ConnectionManager';

interface Env {
	AI: Ai;
	CONNECTIONS: DurableObjectNamespace;
	ENVIRONMENT: string;
}

// Initialize default agents on worker startup
let initialized = false;
function initializeDefaults() {
	if (initialized) return;
	const defaultAgents = getAllDefaultAgents();
	for (const preset of defaultAgents) {
		registerDefaultAgent(preset.content);
	}
	initialized = true;
}

export default {
	async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
		const url = new URL(request.url);

		// Initialize defaults on first request
		initializeDefaults();

		// CORS headers
		const corsHeaders = {
			'Access-Control-Allow-Origin': '*',
			'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
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

		// Route: Chat with LLM (with ReAct loop) - Streaming SSE
		if (url.pathname === '/api/chat' && request.method === 'POST') {
			try {
				// Refresh tool cache at start of chat request
				await refreshToolCache(env);

				const { message, agentId, conversationHistory = [], stream = true } = await request.json() as {
					message: string;
					agentId?: string;
					conversationHistory?: any[];
					stream?: boolean;
				};

				// Get or use default agent
				let agent = agentId ? getAgent(agentId) : getAllAgents().find(a => a.isDefault);
				if (!agent) {
					// Fallback to first default agent
					agent = getAllAgents()[0];
				}

				if (!agent) {
					return new Response(JSON.stringify({
						error: 'No agent available. Please create an agent first.'
					}), {
						status: 400,
						headers: { ...corsHeaders, 'Content-Type': 'application/json' }
					});
				}

				// Execute ReAct loop
				const context: ReActContext = {
					agent,
					userMessage: message,
					conversationHistory,
					env,
				};

				// If streaming is requested, use Server-Sent Events
				if (stream) {
					const { readable, writable } = new TransformStream();
					const writer = writable.getWriter();
					const encoder = new TextEncoder();

					// Send SSE events
					const sendEvent = (event: string, data: any) => {
						const message = `event: ${event}\ndata: ${JSON.stringify(data)}\n\n`;
						writer.write(encoder.encode(message));
					};

					// Execute ReAct loop in background with streaming
					(async () => {
						try {
							const executionLog = await executeReActLoop(
								context,
								undefined,
								(streamEvent) => {
									// Send each stream event to client
									sendEvent(streamEvent.type, streamEvent);
								}
							);

							// Send final execution log
							sendEvent('complete', {
								agentName: agent.name,
								...executionLog,
							});
						} catch (error: any) {
							sendEvent('error', { error: error.message, details: error.stack });
						} finally {
							await writer.close();
						}
					})();

					return new Response(readable, {
						headers: {
							...corsHeaders,
							'Content-Type': 'text/event-stream',
							'Cache-Control': 'no-cache',
							'Connection': 'keep-alive',
						},
					});
				}

				// Non-streaming fallback
				const executionLog = await executeReActLoop(context);

				return new Response(JSON.stringify({
					agentName: agent.name,
					...executionLog,
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

		// Route: Get all tools (dynamically from connected Rust client + Worker tools)
		if (url.pathname === '/api/tools' && request.method === 'GET') {
			const tools = await getAllToolsAsync(env);
			return new Response(JSON.stringify({ tools }), {
				headers: { ...corsHeaders, 'Content-Type': 'application/json' }
			});
		}

		// Route: Get all agents
		if (url.pathname === '/api/agents' && request.method === 'GET') {
			const agents = getAllAgents();
			return new Response(JSON.stringify({ agents }), {
				headers: { ...corsHeaders, 'Content-Type': 'application/json' }
			});
		}

		// Route: Get specific agent
		if (url.pathname.startsWith('/api/agents/') && request.method === 'GET') {
			const agentId = url.pathname.split('/')[3];
			const agent = getAgent(agentId);
			
			if (!agent) {
				return new Response(JSON.stringify({ error: 'Agent not found' }), {
					status: 404,
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			}

			return new Response(JSON.stringify({ agent }), {
				headers: { ...corsHeaders, 'Content-Type': 'application/json' }
			});
		}

		// Route: Create agent
		if (url.pathname === '/api/agents' && request.method === 'POST') {
			try {
				const config = await request.json() as any;
				const agent = createAgent(config);
				
				return new Response(JSON.stringify({ agent }), {
					status: 201,
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			} catch (error: any) {
				return new Response(JSON.stringify({ error: error.message }), {
					status: 400,
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			}
		}

		// Route: Update agent
		if (url.pathname.startsWith('/api/agents/') && request.method === 'PUT') {
			try {
				const agentId = url.pathname.split('/')[3];
				const updates = await request.json() as any;
				const agent = updateAgent(agentId, updates);
				
				if (!agent) {
					return new Response(JSON.stringify({ error: 'Agent not found' }), {
						status: 404,
						headers: { ...corsHeaders, 'Content-Type': 'application/json' }
					});
				}

				return new Response(JSON.stringify({ agent }), {
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			} catch (error: any) {
				return new Response(JSON.stringify({ error: error.message }), {
					status: 400,
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			}
		}

		// Route: Delete agent
		if (url.pathname.startsWith('/api/agents/') && request.method === 'DELETE') {
			try {
				const agentId = url.pathname.split('/')[3];
				const deleted = deleteAgent(agentId);
				
				if (!deleted) {
					return new Response(JSON.stringify({ error: 'Agent not found or cannot be deleted' }), {
						status: 404,
						headers: { ...corsHeaders, 'Content-Type': 'application/json' }
					});
				}

				return new Response(JSON.stringify({ success: true }), {
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			} catch (error: any) {
				return new Response(JSON.stringify({ error: error.message }), {
					status: 400,
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			}
		}

		// Route: Duplicate agent
		if (url.pathname.startsWith('/api/agents/') && url.pathname.endsWith('/duplicate') && request.method === 'POST') {
			try {
				const agentId = url.pathname.split('/')[3];
				const { name } = await request.json() as { name?: string };
				const agent = duplicateAgent(agentId, name);
				
				if (!agent) {
					return new Response(JSON.stringify({ error: 'Agent not found' }), {
						status: 404,
						headers: { ...corsHeaders, 'Content-Type': 'application/json' }
					});
				}

				return new Response(JSON.stringify({ agent }), {
					status: 201,
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			} catch (error: any) {
				return new Response(JSON.stringify({ error: error.message }), {
					status: 400,
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			}
		}

		// Route: Get default presets
		if (url.pathname === '/api/presets' && request.method === 'GET') {
			const agents = getAllDefaultAgents();
			const prompts = getAllDefaultPrompts();
			
			return new Response(JSON.stringify({ 
				agents,
				prompts,
			}), {
				headers: { ...corsHeaders, 'Content-Type': 'application/json' }
			});
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
			message: 'CF AI Local Tools Worker - Agentic AI System',
			version: '2.0.0',
			endpoints: {
				'/connect': 'WebSocket endpoint for Rust app',
				'/api/command': 'POST - Send command to local app',
				'/api/chat': 'POST - Chat with AI agent (ReAct loop)',
				'/api/status': 'GET - Connection status',
				'/api/tools': 'GET - List all available tools',
				'/api/agents': 'GET - List all agents | POST - Create agent',
				'/api/agents/:id': 'GET - Get agent | PUT - Update agent | DELETE - Delete agent',
				'/api/agents/:id/duplicate': 'POST - Duplicate agent',
				'/api/presets': 'GET - Get default presets',
			}
		}), {
			headers: { ...corsHeaders, 'Content-Type': 'application/json' }
		});
	},
};
