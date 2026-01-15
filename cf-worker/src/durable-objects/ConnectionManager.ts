/**
 * Durable Object for managing persistent WebSocket connections
 * Maintains connection between Cloudflare Worker and local Rust app
 * Stores dynamic tool registry received from connected clients
 */

interface ToolParameter {
	name: string;
	type: string;
	description: string;
	required: boolean;
	enum?: string[];
	default?: any;
}

interface ToolDefinition {
	id: string;
	name: string;
	description: string;
	category: string;
	parameters: ToolParameter[];
	returnsObservation: boolean;
}

interface Session {
	webSocket: WebSocket;
	clientId: string;
	connectedAt: number;
	clientName?: string;
	clientVersion?: string;
}

export class ConnectionManager {
	private state: DurableObjectState;
	private sessions: Map<string, Session>;
	private pendingCommands: Map<string, { resolve: Function; reject: Function; timeout: any }>;
	private commandIdCounter: number;
	private toolRegistry: Map<string, ToolDefinition>;

	constructor(state: DurableObjectState, env: any) {
		this.state = state;
		this.sessions = new Map();
		this.pendingCommands = new Map();
		this.commandIdCounter = 0;
		this.toolRegistry = new Map();
	}

	async fetch(request: Request): Promise<Response> {
		const url = new URL(request.url);

		// WebSocket upgrade for Rust app connection
		if (request.headers.get('Upgrade') === 'websocket') {
			const pair = new WebSocketPair();
			const [client, server] = Object.values(pair);

			await this.handleWebSocketSession(server);

			return new Response(null, {
				status: 101,
				webSocket: client,
			});
		}

		// Internal API: Send command to connected client
		if (url.pathname === '/send-command' && request.method === 'POST') {
			try {
				const command = await request.json();
				const result = await this.sendCommandToClient(command);
				return new Response(JSON.stringify(result), {
					headers: { 'Content-Type': 'application/json' },
				});
			} catch (error: any) {
				return new Response(JSON.stringify({ error: error.message }), {
					status: 500,
					headers: { 'Content-Type': 'application/json' },
				});
			}
		}

		// Internal API: Get connection status
		if (url.pathname === '/status') {
			const status = {
				connected: this.sessions.size > 0,
				sessions: Array.from(this.sessions.values()).map(s => ({
					clientId: s.clientId,
					clientName: s.clientName,
					clientVersion: s.clientVersion,
					connectedAt: new Date(s.connectedAt).toISOString(),
					uptime: Date.now() - s.connectedAt,
				})),
				toolCount: this.toolRegistry.size,
			};
			return new Response(JSON.stringify(status), {
				headers: { 'Content-Type': 'application/json' },
			});
		}

		// Internal API: Get available tools from connected client
		if (url.pathname === '/tools') {
			const tools = Array.from(this.toolRegistry.values());
			return new Response(JSON.stringify({
				connected: this.sessions.size > 0,
				tools,
			}), {
				headers: { 'Content-Type': 'application/json' },
			});
		}

		// Internal API: Get single tool definition
		if (url.pathname.startsWith('/tools/')) {
			const toolId = url.pathname.replace('/tools/', '');
			const tool = this.toolRegistry.get(toolId);
			if (tool) {
				return new Response(JSON.stringify(tool), {
					headers: { 'Content-Type': 'application/json' },
				});
			}
			return new Response(JSON.stringify({ error: 'Tool not found' }), {
				status: 404,
				headers: { 'Content-Type': 'application/json' },
			});
		}

		return new Response('Not Found', { status: 404 });
	}

	private async handleWebSocketSession(webSocket: WebSocket) {
		webSocket.accept();

		const clientId = crypto.randomUUID();
		const session: Session = {
			webSocket,
			clientId,
			connectedAt: Date.now(),
		};

		this.sessions.set(clientId, session);
		console.log(`Client connected: ${clientId}`);

		webSocket.addEventListener('message', (event: MessageEvent) => {
			try {
				const data = JSON.parse(event.data as string);

				// Handle handshake with tool registration
				if (data.type === 'handshake') {
					console.log(`Handshake from ${data.client} v${data.version}`);

					// Update session with client info
					session.clientName = data.client;
					session.clientVersion = data.version;

					// Register tools from client
					if (data.tools && Array.isArray(data.tools)) {
						// Clear previous tools and register new ones
						this.toolRegistry.clear();
						for (const tool of data.tools) {
							this.toolRegistry.set(tool.id, tool);
						}
						console.log(`Registered ${data.tools.length} tools from client`);
					}

					webSocket.send(JSON.stringify({
						type: 'handshake_ack',
						server: 'cloudflare-worker',
						timestamp: Date.now(),
						toolsRegistered: this.toolRegistry.size
					}));
					return;
				}

				// Handle command responses
				if (data.commandId) {
					const pending = this.pendingCommands.get(data.commandId);
					if (pending) {
						clearTimeout(pending.timeout);
						this.pendingCommands.delete(data.commandId);
						pending.resolve(data);
					}
				}
			} catch (error) {
				console.error('Error handling message:', error);
			}
		});

		webSocket.addEventListener('close', () => {
			console.log(`Client disconnected: ${clientId}`);
			this.sessions.delete(clientId);

			// Clear tool registry when client disconnects
			this.toolRegistry.clear();
			console.log('Tool registry cleared');

			// Reject all pending commands for this session
			for (const [commandId, pending] of this.pendingCommands.entries()) {
				clearTimeout(pending.timeout);
				pending.reject(new Error('Client disconnected'));
				this.pendingCommands.delete(commandId);
			}
		});

		webSocket.addEventListener('error', (event: ErrorEvent) => {
			console.error(`WebSocket error for ${clientId}:`, event);
			this.sessions.delete(clientId);
		});
	}

	private async sendCommandToClient(command: any): Promise<any> {
		// Get first available session
		const session = Array.from(this.sessions.values())[0];

		if (!session) {
			throw new Error('No client connected');
		}

		// Generate command ID
		const commandId = `cmd_${this.commandIdCounter++}_${Date.now()}`;
		const commandWithId = { ...command, commandId };

		// Send command
		session.webSocket.send(JSON.stringify(commandWithId));

		// Wait for response with timeout
		return new Promise((resolve, reject) => {
			const timeout = setTimeout(() => {
				this.pendingCommands.delete(commandId);
				reject(new Error('Command timeout - no response from client'));
			}, 30000); // 30 second timeout

			this.pendingCommands.set(commandId, { resolve, reject, timeout });
		});
	}
}
