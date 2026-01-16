/**
 * Main Worker Entry Point - Simplified Switchboard Architecture
 *
 * The Worker now acts as a simple relay and LLM API proxy.
 * All ReAct logic has moved to the Rust app.
 */

import { getAllDefaultAgents, getAllDefaultPrompts } from './presets/default-presets';

export { UserSwitchboard } from './durable-objects/UserSwitchboard';

interface Env {
	AI: Ai;
	SWITCHBOARD: DurableObjectNamespace;
	ENVIRONMENT: string;
}

export default {
	async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
		const url = new URL(request.url);

		// CORS headers
		const corsHeaders = {
			'Access-Control-Allow-Origin': '*',
			'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
			'Access-Control-Allow-Headers': 'Content-Type',
		};

		if (request.method === 'OPTIONS') {
			return new Response(null, { headers: corsHeaders });
		}

		// Route: WebSocket connection (relay to Durable Object)
		if (url.pathname === '/connect') {
			const id = env.SWITCHBOARD.idFromName('default'); // TODO: use user ID for multi-user
			const stub = env.SWITCHBOARD.get(id);
			return stub.fetch(request);
		}

		// Route: LLM API with native tool calling support
		if (url.pathname === '/api/llm' && request.method === 'POST') {
			try {
				const { messages, model, tools } = await request.json() as {
					messages: Array<{ role: string; content: string }>;
					model: string;
					tools?: Array<any>;
				};

				if (!model) {
					return new Response(JSON.stringify({ error: 'Model ID required' }), {
						status: 400,
						headers: { ...corsHeaders, 'Content-Type': 'application/json' }
					});
				}

				// Build AI request with optional tools
				const aiRequest: any = {
					messages,
					stream: false,
				};

				// Add tools if provided - Cloudflare Workers AI native tool calling
				if (tools && tools.length > 0) {
					aiRequest.tools = tools;
					console.log('[LLM API] Calling with', tools.length, 'tools');
				}

				// Call Cloudflare Workers AI
				const response = await env.AI.run(model as any, aiRequest) as any;
				console.log('[LLM API] Response has tool_calls:', !!response.tool_calls);

				// Return both response text and tool_calls if present
				return new Response(JSON.stringify({
					response: response.response || response.text || '',
					tool_calls: response.tool_calls || null,
					usage: response.usage,
				}), {
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			} catch (error: any) {
				return new Response(JSON.stringify({ error: error.message }), {
					status: 500,
					headers: { ...corsHeaders, 'Content-Type': 'application/json' }
				});
			}
		}

		// Route: Get default presets (read-only templates)
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
			const id = env.SWITCHBOARD.idFromName('default');
			const stub = env.SWITCHBOARD.get(id);

			const response = await stub.fetch(new Request('http://internal/status'));
			const status = await response.json();

			return new Response(JSON.stringify(status), {
				headers: { ...corsHeaders, 'Content-Type': 'application/json' }
			});
		}

		// Default response
		return new Response(JSON.stringify({
			message: 'CF AI Local Tools Worker - Switchboard Architecture',
			version: '3.0.0',
			architecture: 'Simple relay + LLM API proxy',
			endpoints: {
				'/connect': 'WebSocket relay (desktop & web-viewer)',
				'/api/llm': 'POST - LLM inference API',
				'/api/status': 'GET - Connection status',
				'/api/presets': 'GET - Default agent/prompt templates',
			}
		}), {
			headers: { ...corsHeaders, 'Content-Type': 'application/json' }
		});
	},
};
