# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CF AI Local Tools is a desktop automation system that uses Cloudflare Workers AI to enable natural language control of your computer. The architecture consists of three components:

1. **Rust Local App** ([src/](src/)) - WebSocket client + GUI automation using `rustautogui`
2. **Cloudflare Worker** ([cf-worker/](cf-worker/)) - AI inference + WebSocket endpoint + ReAct agent orchestration
3. **Web Viewer** ([web-viewer/](web-viewer/)) - React UI for agent configuration and chat

The system implements an agentic architecture with:
- **ReAct Loop** (Reasoning + Acting): LLM generates thoughts, executes tool calls, receives observations, and iterates until task completion
- **Agent Presets**: Pre-configured agents with specific purposes, tools, and system prompts
- **Tool Registry**: Centralized validation and execution of mouse/keyboard/screenshot commands
- **LocalStorage-first**: Client-side preset management with optional cloud sync

## Common Commands

### Rust App (Local Automation)

```bash
# Development
cargo run

# Production build
cargo build --release

# With custom worker URL
WORKER_WS_URL=wss://your-worker.workers.dev/connect cargo run
```

### Cloudflare Worker (AI + WebSocket)

```bash
cd cf-worker

# Local development (runs on localhost:8787)
npm run dev

# Deploy to production
npm run deploy

# View live logs
npm run tail
```

### Web Viewer (React UI)

```bash
cd web-viewer

# Development (runs on localhost:3000)
npm run dev

# Production build
npm run build

# Deploy to Cloudflare Pages
npm run deploy
```

### Full System Startup (Development)

```bash
# Terminal 1: Start worker
cd cf-worker && npm run dev

# Terminal 2: Start Rust app
cargo run

# Terminal 3: Start web UI
cd web-viewer && npm run dev
```

## Architecture

### Data Flow: User Message → AI Response

1. User sends message via Web Viewer ([web-viewer/src/App.tsx](web-viewer/src/App.tsx))
2. POST to `/api/chat` endpoint ([cf-worker/src/index.ts](cf-worker/src/index.ts))
3. ReAct loop orchestrator executes ([cf-worker/src/agents/react-loop.ts](cf-worker/src/agents/react-loop.ts)):
   - Load agent config (system prompt, tools, max iterations)
   - Send messages to Cloudflare AI via LLM client ([cf-worker/src/llm/llm-client.ts](cf-worker/src/llm/llm-client.ts))
   - Parse tool calls from LLM response
   - Validate tool calls ([cf-worker/src/tools/tool-registry.ts](cf-worker/src/tools/tool-registry.ts))
   - Send commands to Rust app via WebSocket (Durable Object)
   - Rust app executes via `rustautogui` ([src/main.rs](src/main.rs))
   - Return observation to LLM
   - Repeat until task complete or max iterations reached
4. Return execution log to Web Viewer

### Key Abstractions

**Agent** ([cf-worker/src/types/agent.ts](cf-worker/src/types/agent.ts)):
- Configuration for AI behavior (name, purpose, system prompt, tools, model ID, max iterations)
- Managed by agent-manager ([cf-worker/src/agents/agent-manager.ts](cf-worker/src/agents/agent-manager.ts))
- Presets defined in [cf-worker/src/presets/default-presets.ts](cf-worker/src/presets/default-presets.ts)

**Tool** ([cf-worker/src/types/tool.ts](cf-worker/src/types/tool.ts)):
- Available actions: `mouse_move`, `mouse_click`, `mouse_scroll`, `keyboard_input`, `keyboard_command`, `get_mouse_position`, `take_screenshot`
- Registry validates parameters before execution
- Commands sent via WebSocket to Rust app

**ExecutionLog** ([cf-worker/src/types/agent.ts](cf-worker/src/types/agent.ts)):
- Tracks full ReAct iteration history
- Each step: thought (LLM reasoning) → action (tool call) → observation (result)

**Durable Object (ConnectionManager)** ([cf-worker/src/durable-objects/ConnectionManager.ts](cf-worker/src/durable-objects/ConnectionManager.ts)):
- Manages persistent WebSocket connection to Rust app
- Routes commands and responses between Worker and local machine

### Prompt Engineering

System prompts use template interpolation with variables:
- `{purpose}`: Agent's purpose (from agent config)
- `{tools}`: Available tools description

Prompt templates ([cf-worker/src/presets/default-presets.ts](cf-worker/src/presets/default-presets.ts)):
- **Chain-of-Thought (CoT)**: Pure reasoning, no tool calls
- **ReAct Basic**: Thought → Action → Observation loop
- **ReAct Advanced**: Includes self-critique and error recovery
- **Hybrid CoT-ReAct**: Balanced approach
- **Precise Executor**: Minimalist, action-focused

### State Management (Frontend)

The Web Viewer uses Zustand stores:
- **agentStore** ([web-viewer/src/store/agentStore.ts](web-viewer/src/store/agentStore.ts)): CRUD operations for agents
- **executionStore** ([web-viewer/src/store/executionStore.ts](web-viewer/src/store/executionStore.ts)): Chat history and execution logs
- **storageSchema** ([web-viewer/src/store/storageSchema.ts](web-viewer/src/store/storageSchema.ts)): LocalStorage persistence layer

Storage is prefixed with `cf-ai-presets:*` keys and includes:
- Versioned backups (max 5 per preset, 30-day retention)
- Import/export functionality
- Storage quota monitoring (90% warning threshold)

## Configuration

### Environment Variables

**Rust App**:
- `WORKER_WS_URL`: WebSocket endpoint (default: `ws://localhost:8787/connect`)

**Web Viewer**:
- Create `.env.local`: `VITE_WORKER_URL=https://your-worker.workers.dev`

### Worker Configuration

[cf-worker/wrangler.toml](cf-worker/wrangler.toml):
- AI binding: `[ai] binding = "AI"`
- Durable Objects binding: `CONNECTIONS` → `ConnectionManager`
- Compatibility date: Determines available Cloudflare features

## API Endpoints

All endpoints support CORS (`Access-Control-Allow-Origin: *`):

- `GET /` - Worker info and endpoint list
- `GET /connect` - WebSocket upgrade for Rust app
- `POST /api/command` - Send direct command to Rust app
- `POST /api/chat` - Chat with agent (ReAct loop)
- `GET /api/status` - Check Rust app connection status
- `GET /api/tools` - List all available tools
- `GET /api/agents` - List all agents
- `POST /api/agents` - Create new agent
- `GET /api/agents/:id` - Get specific agent
- `PUT /api/agents/:id` - Update agent
- `DELETE /api/agents/:id` - Delete agent (locked agents cannot be deleted)
- `POST /api/agents/:id/duplicate` - Duplicate agent
- `GET /api/presets` - Get default presets (agents + prompts)

## Development Patterns

### Adding a New Tool

1. Define in [cf-worker/src/tools/tool-registry.ts](cf-worker/src/tools/tool-registry.ts):
   ```typescript
   export const TOOL_REGISTRY: Record<string, ToolDefinition> = {
     my_tool: {
       id: 'my_tool',
       name: 'My Tool',
       description: 'What it does',
       category: 'mouse' | 'keyboard' | 'system',
       parameters: [/* ... */],
       returnsObservation: true,
     },
   };
   ```

2. Add command variant to Rust [src/main.rs](src/main.rs):
   ```rust
   #[derive(Debug, Deserialize)]
   #[serde(tag = "type", rename_all = "snake_case")]
   enum Command {
     MyTool { param: String },
     // ...
   }
   ```

3. Implement handler in `AutomationHandler::handle_command()`:
   ```rust
   Command::MyTool { param } => {
     // Implementation
   }
   ```

### Creating a New Agent Preset

1. Define in [cf-worker/src/presets/default-presets.ts](cf-worker/src/presets/default-presets.ts):
   ```typescript
   export const DEFAULT_AGENTS: Record<string, PresetDefinition<AgentConfig>> = {
     'my-agent': {
       id: 'my-agent',
       name: 'My Agent',
       content: {
         name: 'My Agent',
         purpose: 'What it does',
         systemPrompt: DEFAULT_PROMPTS['react-basic'].content,
         tools: [/* tool references */],
         modelId: '@cf/meta/llama-3.3-70b-instruct-fp8-fast',
         maxIterations: 5,
       },
       isLocked: true,
       // ...
     },
   };
   ```

2. Agent will be automatically registered on worker startup

### Modifying the ReAct Loop

The orchestrator is in [cf-worker/src/agents/react-loop.ts](cf-worker/src/agents/react-loop.ts). Key behavior:
- Builds conversation context with system prompt + user message + iteration history
- Calls LLM with tools schema
- Parses tool calls from response
- Validates parameters before execution
- Sends commands via Durable Object WebSocket
- Aggregates observations and continues loop
- Stops when: LLM returns no tool calls, max iterations reached, or error occurs

### Working with Durable Objects

The ConnectionManager ([cf-worker/src/durable-objects/ConnectionManager.ts](cf-worker/src/durable-objects/ConnectionManager.ts)) maintains WebSocket state:
- Accepts upgrade requests from Rust app at `/connect`
- Stores WebSocket reference in Durable Object state
- Handles internal routes:
  - `POST /send-command`: Forward command to Rust app
  - `GET /status`: Check connection status
- Supports request-response pattern with timeout

## Deployment

### Production Deployment

1. **Deploy Worker**:
   ```bash
   cd cf-worker && npm run deploy
   ```
   Note the deployed URL: `https://cf-ai-local-tools-worker.YOUR-SUBDOMAIN.workers.dev`

2. **Deploy Web Viewer**:
   ```bash
   cd web-viewer
   echo "VITE_WORKER_URL=https://your-worker.workers.dev" > .env.local
   npm run build
   npx wrangler pages deploy dist --project-name ai-local-tools-viewer
   ```

3. **Run Rust App**:
   ```bash
   WORKER_WS_URL=wss://your-worker.workers.dev/connect cargo run --release
   ```

### Running as Background Service

The Rust app must run continuously on the controlled machine. Consider using:
- **macOS**: launchd (create `.plist` in `~/Library/LaunchAgents/`)
- **Linux**: systemd (create service file in `/etc/systemd/system/`)
- **Windows**: Task Scheduler or Windows Service

## Security

This system provides remote desktop control. Production deployments should:
- Add authentication to Worker endpoints
- Use API keys or OAuth for access control
- Implement rate limiting
- Whitelist allowed actions in Rust app
- Log all commands for audit trail
- Consider Cloudflare Access for web viewer
- Never expose to untrusted networks without proper authentication

## Troubleshooting

**Worker logs**: `cd cf-worker && npm run tail`

**Rust app won't connect**:
- Verify `WORKER_WS_URL` is correct
- Check firewall/network settings
- Test worker: `curl https://your-worker.workers.dev/api/status`

**Web viewer shows disconnected**:
- Check browser console for errors
- Verify `VITE_WORKER_URL` in `.env.local`

**rustautogui not working on macOS**:
- Grant Accessibility permissions: System Preferences → Security & Privacy → Accessibility

## Technology Stack

- **Rust**: tokio, tokio-tungstenite (WebSocket), rustautogui (automation), serde (JSON)
- **Cloudflare Worker**: TypeScript, Durable Objects (WebSocket state), Workers AI (LLM)
- **Web Viewer**: React 18, TypeScript, Vite, Tailwind CSS, Zustand (state), lucide-react (icons)
- **AI Models**: Cloudflare Workers AI (`@cf/meta/llama-3.3-70b-instruct-fp8-fast`)
