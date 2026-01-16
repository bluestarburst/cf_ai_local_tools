# CF AI Local Tools - Complete Reference

**Status**: ✅ Production Ready (v3.0 Switchboard Architecture)  
**Last Updated**: January 15, 2026

---

## Overview

A complete **AI-powered computer automation system** built with Rust, Cloudflare Workers, and React. The system uses an LLM to understand natural language commands and autonomously execute multi-step tasks on your local machine.

### Architecture Diagram

```
Web Viewer ──WebSocket──► Worker (relay) ──WebSocket──► Rust (ReAct + tools)
  (React UI)            (Cloudflare Edge)           (Automation + LLM calls)
                             │
                        LLM API ◄────────────────────── Rust calls for inference
```

---

## Architecture: Backend vs Frontend Ownership

### ✅ What the Rust Backend ("Local App") Owns

The Rust app (`/src`) is the **source of truth** for all agent configuration and tool execution:

1. **Agent Storage & Persistence**
   - All agents stored on disk at `~/.cf_ai_local_tools/agents.json` (via `src/agents/storage.rs`)
   - Create, read, update, delete agents (**CRUD operations**)
   - Validate agent configurations before saving
   - Prevent modification of locked/default agents

2. **Tool Registry**
   - Defines all available tools in `src/agents/presets.rs`
   - Mouse automation, keyboard input, screenshots, etc.
   - LLM uses these tools during ReAct execution
   - Tools are built into the Rust binary, not configurable by users

3. **ReAct Loop Execution**
   - Agent prompt interpolation (`src/agents/prompt.rs`)
   - Tool call parsing and validation
   - Iteration tracking and execution history
   - Integration with LLM for tool calling

4. **Agent Configuration Validation**
   - Ensure selected tools exist
   - Validate system prompts
   - Enforce constraints (max iterations, model selection, etc.)

### ❌ What the Web Viewer Does NOT Do

The web viewer (`/web-viewer`) does **NOT persist or store locally**:

- ❌ Does NOT store agents in localStorage
- ❌ Does NOT execute agents locally (execution is in Rust backend)
- ❌ Does NOT run ReAct loop (runs in Rust backend)
- ❌ Does NOT cache agent data (always gets from backend)

### ✅ What the Web Viewer DOES Do

1. **Display Agent Presets**
   - Load agents from Rust backend on connect
   - Display agent list from backend
   - Show agent details and configuration

2. **Full Agent Editor Interface**
   - ✅ **Create new agents** - Form to create agent in backend
   - ✅ **Edit agents** - Modify agent config and save to backend
   - ✅ **Delete agents** - Remove agents from backend
   - Show available tools from backend
   - Display tool selection with checkboxes
   - Edit system prompts, max iterations, model selection

3. **Chat Interface**
   - Select agent and start chat
   - Display chat messages and responses
   - Show ReAct execution results
   - Connection status monitoring

4. **Send Commands to Rust Backend**
   - `get_agents` - Request current agent list
   - `create_agent` - Send new agent config to backend (backend persists)
   - `update_agent` - Send edited agent config to backend (backend persists)
   - `delete_agent` - Delete agent from backend
   - `chat_request` - Start ReAct execution with selected agent

5. **Receive Updates from Rust Backend**
   - `agents_list` - Current agents from disk
   - `agent_created` - Confirmation + created agent
   - `agent_updated` - Confirmation + updated agent
   - `agent_deleted` - Confirmation of deletion
   - `chat_response` - ReAct results with tool calls
   - `agent_error` - Validation errors (e.g., duplicate ID, locked agent)

### Message Flow

```
User edits agent in Web Viewer
    ↓
Web Viewer sends: { type: "update_agent", agent: {...} }
    ↓
    [via WebSocket relay through Cloudflare Worker]
    ↓
Rust Backend (AgentStorage)
    - Validates agent
    - Checks locked status
    - Saves to ~/.cf_ai_local_tools/agents.json
    - Returns confirmation
    ↓
    [via WebSocket relay through Cloudflare Worker]
    ↓
Web Viewer receives: { type: "agent_updated", agent: {...} }
    ↓
Web Viewer updates UI with confirmed agent
```

### Advantages of Backend Ownership

1. **Single Source of Truth** - Rust app is authoritative
2. **Persistence** - Agents survive web viewer restarts
3. **Security** - No sensitive agent config in browser
4. **Consistency** - Can't accidentally corrupt agent JSON
5. **Multi-Device** - Same agents available from any device connecting to same Rust app
6. **Audit Trail** - Agent changes logged in one place

---

## Implementation Recommendations

Based on the current codebase, here's what's already in place and what should be implemented:

### ✅ Already Implemented

1. **Agent Storage** (`src/agents/storage.rs`)
   - Loads/saves agents from disk
   - CRUD operations
   - Locked agent protection
   - ✅ Ready to use

2. **WebSocket Message Handlers** (in `src/main.rs`)
   - `get_agents` - Returns all agents
   - `create_agent` - Creates new agent with validation
   - `update_agent` - Updates existing agent
   - `delete_agent` - Deletes agent
   - `chat_request` - Starts ReAct execution
   - ✅ All implemented

3. **Tool Registry** (`src/agents/presets.rs`)
   - Default tools defined
   - Tool validation
   - ✅ Ready to use

4. **ReAct Loop** (`src/agents/react_loop_v3.rs`)
   - Executes agents with LLM
   - Handles tool calling
   - ✅ Integrated with WebSocket

### ⏳ Needs Implementation/Updates

1. **Web Viewer Agent Editor Form**
   - **Currently**: Might not have full form for editing
   - **Should**: Complete agent editor with:
     - Agent name input
     - Agent purpose input
     - System prompt textarea
     - Model dropdown (show available models)
     - Max iterations number input
     - Tool selection checkboxes (from backend)
     - Create / Update / Delete buttons
     - Reset to Default button (only for isDefault=true agents)
   - **Action**: Build form component that sends CRUD messages to backend:
     ```typescript
     const handleCreateAgent = (agent: Agent) => {
       websocket.send(JSON.stringify({
         type: "create_agent",
         agent: agent
       }));
       // Wait for "agent_created" response before showing in list
     };

     const handleUpdateAgent = (agent: Agent) => {
       websocket.send(JSON.stringify({
         type: "update_agent",
         id: agent.id,
         agent: agent
       }));
       // Wait for "agent_updated" response before updating UI
     };

     const handleDeleteAgent = (agentId: string) => {
       websocket.send(JSON.stringify({
         type: "delete_agent",
         id: agentId
       }));
       // Wait for "agent_deleted" response before removing from list
     };

     const handleResetAgent = (agentId: string) => {
       websocket.send(JSON.stringify({
         type: "reset_agent",
         id: agentId
       }));
       // Wait for "agent_reset" response before updating UI
     };
     ```

2. **Loading Agent List on Connect**
   - **Currently**: Might be empty or showing defaults
   - **Should**: Request agents from Rust backend on WebSocket connect
   - **Action**: 
     ```typescript
     useEffect(() => {
       if (websocket?.readyState === WebSocket.OPEN) {
         websocket.send(JSON.stringify({ type: "get_agents" }));
       }
     }, [websocket]);
     ```

3. **Response Message Handlers**
   - **Currently**: Might be incomplete
   - **Should**: Handle all CRUD responses from Rust:
     - `agents_list` - populate agent list in UI
     - `agent_created` - add new agent to list
     - `agent_updated` - refresh agent in list
     - `agent_deleted` - remove agent from list
     - `agent_error` - show error toast/modal (e.g., duplicate ID, locked agent)
     - `handshake` - cache tool list for form
   - **Action**: 
     ```typescript
     const handleMessage = (event: MessageEvent) => {
       const data = JSON.parse(event.data);
       switch (data.type) {
         case "agents_list":
           setAgents(data.agents); // from backend
           break;
         case "agent_created":
           setAgents(prev => [...prev, data.agent]);
           showSuccess(`Agent created: ${data.agent.name}`);
           resetForm();
           break;
         case "agent_updated":
           setAgents(prev => prev.map(a => 
             a.id === data.agent.id ? data.agent : a
           ));
           showSuccess(`Agent updated: ${data.agent.name}`);
           setEditingId(null);
           break;
         case "agent_deleted":
           setAgents(prev => prev.filter(a => a.id !== data.id));
           showSuccess(`Agent deleted`);
           break;
         case "agent_reset":
           setAgents(prev => prev.map(a => 
             a.id === data.agent.id ? data.agent : a
           ));
           showSuccess(`Agent reset to default`);
           break;
         case "agent_error":
           showError(`Error: ${data.error}`);
           break;
         case "handshake":
           setAvailableTools(data.tools);
           break;
       }
     };
     ```

4. **Tool Display in Editor**
   - **Currently**: Tools shown on handshake but might not be in form
   - **Should**: Display tools as checkboxes in agent editor
   - **Action**: 
     ```typescript
     {availableTools.map(tool => (
       <label key={tool.id}>
         <input
           type="checkbox"
           checked={selectedTools.includes(tool.id)}
           onChange={() => toggleTool(tool.id)}
         />
         {tool.name}
       </label>
     ))}
     ```

5. **Remove localStorage**
   - **Currently**: Might be storing agents in localStorage
   - **Should**: Delete localStorage usage, only use backend
   - **Action**: Remove lines like:
     ```typescript
     // REMOVE THESE:
     localStorage.setItem('agents', JSON.stringify(agents)); ❌
     const cached = localStorage.getItem('agents'); ❌
     ```

### Development Priority

**Phase 1 - Backend Ready (Already Done)**
- ✅ Rust backend owns all agent storage
- ✅ WebSocket messages for CRUD operations
- ✅ Tool registry and validation

**Phase 2 - Frontend Alignment (Needs Work)**
- [ ] Remove localStorage agent storage from web viewer
- [ ] Wire up agent editor form to WebSocket CRUD messages
- [ ] Load agents on WebSocket connection
- [ ] Handle all response message types
- [ ] Show confirmed agents only (after backend confirmation)

**Phase 3 - Polish (Optional)**
- [ ] Streaming responses during ReAct execution
- [ ] Agent execution history/logs
- [ ] Agent import/export functionality
- [ ] Tool filtering by category

---

## Current Codebase Structure

### Rust Backend (`/src`)

```
src/
├── main.rs (754 lines)
│   ├── WebSocket connection & message handling
│   ├── CRUD message handlers for agents
│   ├── Tool execution dispatch
│   └── ReAct loop invocation
│
├── agents/
│   ├── storage.rs (322 lines) ✅ READY
│   │   ├── Agent struct (disk-persisted format)
│   │   ├── AgentStorage manager
│   │   ├── CRUD operations
│   │   └── Locked agent protection
│   │
│   ├── presets.rs (699 lines) ✅ READY
│   │   ├── Default agents
│   │   ├── Tool definitions
│   │   └── Tool validation
│   │
│   ├── prompt.rs
│   │   └── Prompt interpolation
│   │
│   ├── react_loop_v3.rs ✅ ACTIVE
│   │   └── ReAct execution with native tool calling
│   │
│   ├── react_loop_v2.rs (deprecated)
│   └── react_loop.rs (deprecated)
│
└── llm/
    └── client.rs ✅ READY
        └── HTTP client for Worker's /api/llm endpoint
```

### Key Rust Files for Backend Ownership

1. **`src/agents/storage.rs`**
   - Implements `AgentStorage` struct
   - Methods: `create()`, `update()`, `delete()`, `get()`, `get_all()`
   - Auto-saves to disk after each operation
   - Validates locked status

2. **`src/main.rs` (lines 220-380)**
   - Message handlers: `get_agents`, `create_agent`, `update_agent`, `delete_agent`
   - Each validates agent before persisting
   - Sends responses back via WebSocket

3. **`src/agents/presets.rs`**
   - `get_default_tools()` - List of all available tools
   - Tool validation logic
   - Default agent templates

### Web Viewer (`/web-viewer`)

**Changes Needed**:
- Remove any agent localStorage
- Implement WebSocket CRUD handlers
- Load agents on connect
- Update UI after backend confirmation


├── src/                              # Rust local app
│   ├── main.rs                       # WebSocket client & agent orchestration
│   ├── agents/
│   │   ├── mod.rs                    # Agent module
│   │   ├── prompt.rs                 # Prompt interpolation
│   │   └── react_loop.rs             # ReAct loop execution
│   └── llm/
│       ├── mod.rs                    # LLM module
│       └── client.rs                 # LLM API client
├── Cargo.toml                        # Rust dependencies
│
├── cf-worker/                        # Cloudflare Worker
│   ├── src/
│   │   ├── index.ts                  # Main entry point (relay + LLM proxy)
│   │   ├── agents/                   # Agent definitions
│   │   ├── durable-objects/
│   │   │   └── UserSwitchboard.ts    # WebSocket relay per user
│   │   ├── llm/
│   │   │   └── llm-client.ts         # LLM inference endpoint
│   │   ├── presets/
│   │   │   └── default-presets.ts    # Read-only preset templates
│   │   └── types/
│   │       ├── agent.ts              # Agent types
│   │       ├── preset.ts             # Preset types
│   │       └── tool.ts               # Tool types
│   ├── wrangler.toml                 # Cloudflare config
│   ├── tsconfig.json
│   └── package.json
│
├── web-viewer/                       # React web interface
│   ├── src/
│   │   ├── App.tsx                   # Main app
│   │   ├── main.tsx
│   │   ├── index.css
│   │   ├── components/               # React components
│   │   ├── hooks/
│   │   │   └── useWebSocket.ts       # WebSocket connection hook
│   │   ├── store/                    # State management
│   │   ├── types/                    # TypeScript types
│   │   └── utils/                    # Helper functions
│   ├── index.html
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   ├── postcss.config.js
│   ├── tsconfig.json
│   └── package.json
│
└── README.md (this file)
```

---

## Components

### 1. Rust Local App (`/src`) - **OWNS ALL AGENTS & TOOLS**

**Purpose**: 
- Executes automation tasks locally
- Manages AI interactions
- **OWNS agent storage and configuration** (only source of truth)

**Key Responsibilities**:
- Maintains WebSocket connection to Worker
- Receives agent creation/update/delete requests from web viewer
- Stores agents persistently at `~/.cf_ai_local_tools/agents.json`
- Validates agent configurations
- Executes ReAct loop locally with LLM calls
- Executes tools via `rustautogui`
- Auto-reconnects if connection drops
- **Returns agents to web viewer on demand** (web viewer never stores locally)

**Key Modules**:
- `main.rs` - WebSocket client, CRUD message handlers, ReAct orchestration
- `agents/storage.rs` - Agent CRUD operations, disk persistence, validation
- `agents/react_loop.rs` - ReAct loop execution with tool calling
- `agents/presets.rs` - Tool definitions and default agent templates
- `agents/prompt.rs` - System prompt interpolation
- `llm/client.rs` - HTTP client for LLM API calls to Worker

**Agent Storage Location**: `~/.cf_ai_local_tools/agents.json`

**Environment Variables**:
```bash
WORKER_WS_URL=ws://localhost:8787/connect     # WebSocket URL
WORKER_HTTP_URL=http://localhost:8787         # HTTP endpoint for LLM
```

**Technology**:
- Rust with `tokio` async runtime
- `tokio-tungstenite` for WebSocket
- `reqwest` for HTTP LLM calls
- `rustautogui` for GUI automation
- `serde` for JSON serialization

### 2. Cloudflare Worker (`/cf-worker`)

**Purpose**: Acts as relay + LLM API proxy

**Key Responsibilities**:
- Receives WebSocket connections from Rust app (via `UserSwitchboard` Durable Object)
- Serves LLM inference endpoint at `/api/llm`
- Maintains one Durable Object per connected user
- Relays messages between web viewer and Rust app
- (Optional) Stores default agent presets

**Key Files**:
- `src/index.ts` - Routes & WebSocket relay setup
- `src/durable-objects/UserSwitchboard.ts` - Per-user WebSocket relay
- `src/llm/llm-client.ts` - LLM inference endpoint

**Worker Size**: ~120 lines (simplified from ~350)

**API Endpoints**:
- `GET /connect` → WebSocket upgrade for Rust app
- `POST /api/llm` → LLM inference (called by Rust app)

**Bindings Required** (in wrangler.toml):
- AI binding for `@cf/meta/llama-3.3-70b-instruct-fp8-fast`
- Durable Object binding for `UserSwitchboard`

**Technology**:
- TypeScript/Cloudflare Workers API
- Durable Objects for persistent connections
- Cloudflare AI for LLM inference

### 3. Web Viewer (`/web-viewer`) - **UI FOR AGENT EDITING & CHAT**

**Purpose**: Beautiful React interface for agent editing and ReAct chat execution

**Key Responsibilities**:
- Display agent presets loaded from Rust backend
- Provide agent editor form for CRUD operations
- Send agent changes to Rust backend for persistence
- Display available tools from Rust backend
- Execute chat with selected agent
- Display chat results with tool calls
- Show connection status

**What It DOES (Has Full Agent Editor)**:
- ✅ Display agents from backend
- ✅ Create agents (form → send to backend)
- ✅ Edit agents (form → send changes to backend)
- ✅ Delete agents (send delete to backend)
- ✅ Show available tools (from backend)
- ✅ Chat interface (sends message to backend)
- ✅ Display results (from backend)

**What It Does NOT Do (Never Stores Locally)**:
- ❌ Does NOT store agents in localStorage
- ❌ Does NOT persist agent changes locally
- ❌ Does NOT execute ReAct loop
- ❌ Does NOT manage tools (just displays them)
- ❌ Does NOT run agents
- ❌ Does NOT cache agent data between sessions

**Key Components**:
- `useWebSocket.ts` - Connection hook, sends/receives all messages
- `ChatInterfaceV3.tsx` - Chat UI with message display
- **Agent Editor Form** - Create/edit agents:
  - Agent name, purpose, description
  - System prompt (textarea)
  - Model selection (dropdown)
  - Max iterations (number input)
  - Tool selection (checkboxes)
  - Submit: sends to backend for persistence
- Agent selector - Shows agents loaded from backend
- Tool display - Shows available tools from backend

**WebSocket Message Flow**:
```
LOAD AGENTS (on connect):
Web Viewer → { type: "get_agents" } → Rust Backend
Web Viewer ← { type: "agents_list", agents: [...] } ← Rust Backend
             (agents persisted at ~/.cf_ai_local_tools/agents.json)

CREATE AGENT (form submit):
Web Viewer → { type: "create_agent", agent: {...} } → Rust Backend
Web Viewer ← { type: "agent_created", agent: {...} } ← Rust Backend
             (backend saved to disk, returns confirmation)

UPDATE AGENT (edit form submit):
Web Viewer → { type: "update_agent", id: "...", agent: {...} } → Rust Backend
Web Viewer ← { type: "agent_updated", agent: {...} } ← Rust Backend
             (backend saved to disk, returns confirmation)

DELETE AGENT (delete button):
Web Viewer → { type: "delete_agent", id: "..." } → Rust Backend
Web Viewer ← { type: "agent_deleted", id: "..." } ← Rust Backend
             (backend removed from disk)

START CHAT (send message):
Web Viewer → { type: "chat_request", message: "...", agentId: "..." } → Rust Backend
Web Viewer ← { type: "chat_response", content: "...", ... } ← Rust Backend
             (backend executed ReAct loop with agent from disk)
```

**Environment**:
```bash
VITE_WORKER_URL=http://localhost:8787  # Worker URL (relay only)
```

**Technology**:
- React 18+ with TypeScript
- Vite for bundling
- Tailwind CSS for styling
- WebSocket for real-time communication (all persistence happens in backend)

---

## Message Protocol

### Agent Management Messages

#### Get Agents List
```json
Web Viewer → Rust Backend:
{
  "type": "get_agents"
}

Rust Backend → Web Viewer:
{
  "type": "agents_list",
  "agents": [
    {
      "id": "research-assistant",
      "name": "Research Assistant",
      "systemPrompt": "You are a helpful research assistant...",
      "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
      "maxIterations": 5,
      "tools": ["web_search", "fetch_url"],
      "isLocked": false,
      "createdAt": "2026-01-15T12:00:00Z",
      "updatedAt": "2026-01-15T12:00:00Z"
    }
  ]
}
```

#### Create Agent
```json
Web Viewer → Rust Backend:
{
  "type": "create_agent",
  "agent": {
    "id": "my-agent",
    "name": "My Agent",
    "purpose": "Help with research",
    "systemPrompt": "You are helpful...",
    "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
    "maxIterations": 5,
    "tools": ["web_search"]
  }
}

Rust Backend → Web Viewer:
{
  "type": "agent_created",
  "agent": { ... } // full agent with timestamps
}

OR on error:
{
  "type": "agent_error",
  "error": "Agent with id 'my-agent' already exists"
}
```

#### Update Agent
```json
Web Viewer → Rust Backend:
{
  "type": "update_agent",
  "id": "my-agent",
  "agent": { ... } // modified agent
}

Rust Backend → Web Viewer:
{
  "type": "agent_updated",
  "agent": { ... } // persisted agent
}
```

#### Delete Agent
```json
Web Viewer → Rust Backend:
{
  "type": "delete_agent",
  "id": "my-agent"
}

Rust Backend → Web Viewer:
{
  "type": "agent_deleted",
  "id": "my-agent"
}
```

#### Reset Agent to Default
```json
Web Viewer → Rust Backend:
{
  "type": "reset_agent",
  "id": "research-assistant"  // Only for isDefault=true agents
}

Rust Backend → Web Viewer:
{
  "type": "agent_reset",
  "agent": {
    "id": "research-assistant",
    "name": "Research Assistant",
    "systemPrompt": "You are a helpful research assistant...",
    "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
    "maxIterations": 5,
    "tools": ["web_search", "fetch_url"],
    "isDefault": true,
    "updatedAt": "2026-01-15T16:45:00Z"
  }
}
```

### Chat/ReAct Messages

#### Start Chat
```json
Web Viewer → Rust Backend:
{
  "type": "chat_request",
  "message": "Search for weather in Tokyo",
  "agentId": "research-assistant"  // Agent ID from backend
}

Rust Backend → Web Viewer:
{
  "type": "chat_response",
  "content": "The weather in Tokyo is 72°F and sunny.",
  "iterations": 2,
  "toolsUsed": ["web_search"]
}
```

### Internal: Rust Backend → Worker (LLM API)

```
POST /api/llm
Content-Type: application/json

{
  "messages": [
    {"role": "system", "content": "You are a helpful assistant with tools..."},
    {"role": "user", "content": "Search for weather in Tokyo"}
  ],
  "model": "@cf/meta/llama-3.3-70b-instruct-fp8-fast"
}
```

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ Web Viewer (React - Visual Only)                            │
│ - Renders chat UI                                           │
│ - Agent editor form                                         │
│ - Connection status                                         │
│ NO localStorage agent storage                              │
└─────────────────────────────────────────────────────────────┘
                          ↑ ↓
                    [WebSocket via Worker]
                          ↑ ↓
┌─────────────────────────────────────────────────────────────┐
│ Cloudflare Worker (Relay)                                   │
│ - Relays WebSocket messages                                 │
│ - Proxies LLM API calls                                     │
│ NO agent or tool storage                                    │
└─────────────────────────────────────────────────────────────┘
                          ↑ ↓
                    [WebSocket + HTTP]
                          ↑ ↓
┌─────────────────────────────────────────────────────────────┐
│ Rust Backend (LOCAL - Source of Truth)                      │
│ ✅ Owns all agents (stored at ~/.cf_ai_local_tools/)       │
│ ✅ Manages tool registry                                     │
│ ✅ Executes ReAct loops                                      │
│ ✅ Validates all configurations                             │
│ ✅ Persists all changes to disk                             │
│                                                              │
│ Handles WebSocket messages:                                 │
│ - get_agents → Returns all agents                           │
│ - create_agent → Validate & save to disk                    │
│ - update_agent → Validate & save to disk                    │
│ - delete_agent → Remove from disk (custom agents only)      │
│ - reset_agent → Restore default agent from preset           │
│ - chat_request → Execute ReAct with selected agent          │
└─────────────────────────────────────────────────────────────┘
```

---

## Architecture v3.0 - Switchboard (Migration Complete)

### What Changed from v2

| Aspect | Before | After |
|--------|--------|-------|
| **Worker Role** | Orchestrates ReAct loop | Relay + LLM proxy only |
| **Where Logic Runs** | Worker (edge) | Rust app (local) |
| **Worker Code Size** | ~350 lines | ~120 lines |
| **Tool Call Latency** | High (network per call) | Low (no network per call) |
| **User Isolation** | Shared state | One Durable Object per user |
| **Tool Execution** | In Worker → send to Rust | In Rust → execute locally |

### Benefits

1. **Simpler Worker**: 66% code reduction
2. **Faster Execution**: No network hop per tool call
3. **Clear Separation**: Worker = relay/LLM, Rust = logic/execution
4. **User Isolation**: Ready for multi-user support
5. **Bidirectional Real-time**: WebSocket instead of SSE

### Data Flow

```
1. Web Viewer sends chat_request
   ↓
2. Worker relays to Rust (via UserSwitchboard)
   ↓
3. Rust starts ReAct loop:
   - Interpolates prompt with agent config
   - Calls Worker's /api/llm endpoint
   - Receives LLM response with tool calls
   - Executes tools locally
   - Iterates until completion
   ↓
4. Rust sends chat_response back through Worker
   ↓
5. Web Viewer displays results
```

---

## Quick Start

### Prerequisites

- **Rust**: [rustup.rs](https://rustup.rs/)
- **Node.js**: v18+ (Worker & Web Viewer)
- **Cloudflare Account**: Free tier works
- **Wrangler CLI**: `npm install -g wrangler`

### 1. Start Worker (Terminal 1)

```bash
cd cf-worker
npm install
wrangler dev
```

Expected output:
```
⛅️ wrangler 3.x.x
Your worker has access to:
- AI Bindings: AI (meta/llama-3.3-70b-instruct-fp8-fast)
- Durable Objects: SWITCHBOARD (UserSwitchboard)
```

**Note**: To deploy to production, run `wrangler deploy`

### 2. Start Rust App (Terminal 2)

```bash
cargo run
```

Expected output:
```
[INFO] Starting automation client...
[INFO] Will connect to: ws://localhost:8787/connect
[INFO] Connected successfully!
[INFO] Registering 9 tools with server
[INFO] Server handshake acknowledged
```

### 3. Start Web Viewer (Terminal 3)

```bash
cd web-viewer
npm install
npm run dev
```

Expected output:
```
VITE v5.x.x  ready in 500 ms
➜  Local:   http://localhost:5173/
```

### 4. Test the System

1. Open [http://localhost:5173](http://localhost:5173/)
2. Verify "Connected" status (green badge)
3. Create or select an agent
4. Send a message: "Hello, are you there?"
5. Should receive LLM response

---

## Configuration

### Environment Variables

**Rust App** (`src/main.rs`):
```bash
WORKER_WS_URL=ws://localhost:8787/connect
WORKER_HTTP_URL=http://localhost:8787
```

**Web Viewer** (`.env.local`):
```bash
VITE_WORKER_URL=http://localhost:8787
```

### Cloudflare Setup

**wrangler.toml** must include:

```toml
[[durable_objects.bindings]]
name = "SWITCHBOARD"
class_name = "UserSwitchboard"

[[env.production.durable_objects.bindings]]
name = "SWITCHBOARD"
class_name = "UserSwitchboard"
script_name = "cf-ai-local-tools"
```

### Agent Configuration

Agents are defined in web viewer localStorage or can be sent via messages:

```json
{
  "id": "research-assistant",
  "name": "Research Assistant",
  "systemPrompt": "You are a helpful research assistant...",
  "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
  "maxIterations": 5,
  "tools": ["web_search", "fetch_url", "summarize"]
}
```

---

## Available Tools

Tools are **defined in the Rust backend** (`src/agents/presets.rs`) and **never changed by the web viewer**.

### Tool Registry (Rust Backend Only)

The Rust app registers these tools on startup and sends them to web viewer:

- `mouse_move` - Move mouse pointer to position
- `mouse_click` - Click mouse button (left, right, middle)
- `mouse_scroll` - Scroll in direction (up, down, left, right)
- `keyboard_input` - Type text
- `keyboard_command` - Press special keys (return, tab, backspace, etc.)
- `screenshot` - Capture screen
- `get_mouse_position` - Get current cursor position

### Tool Calling

- Tools are defined in Rust backend only
- Web viewer receives tool list on connect
- LLM decides which tools to use based on user request
- Tool execution happens in Rust app
- Results flow back to web viewer

### Adding New Tools

1. **Add tool definition** in `src/agents/presets.rs`:
   ```rust
   ToolDefinition {
       name: "new_tool",
       description: "Does something...",
       parameters: vec![...],
   }
   ```

2. **Implement execution** in `src/main.rs` command handler

3. **Tools are automatically available** to LLM (no web viewer changes needed)

---

## Agent Configuration

### Agent Structure (Persisted in Rust Backend)

All agents are stored in `~/.cf_ai_local_tools/agents.json` by the Rust backend:

```json
{
  "id": "research-assistant",
  "name": "Research Assistant",
  "purpose": "Help with research tasks",
  "systemPrompt": "You are a helpful research assistant...",
  "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
  "maxIterations": 5,
  "tools": ["web_search", "fetch_url", "summarize"],
  "isLocked": false,
  "createdAt": "2026-01-15T12:00:00Z",
  "updatedAt": "2026-01-15T12:00:00Z"
}
```

### Agent Lifecycle (All Backend Operations)

1. **Web Viewer** requests agent creation via WebSocket
2. **Rust Backend** validates agent configuration
3. **Rust Backend** saves agent to `~/.cf_ai_local_tools/agents.json`
4. **Rust Backend** returns confirmation to web viewer
5. **Web Viewer** displays confirmed agent (doesn't store it)

### Default/Built-in Agents

Rust backend includes **default agents** in `src/agents/presets.rs` that are:
- ✅ Fully editable (users can modify them)
- ✅ Can be saved with modifications
- ❌ Cannot be deleted (protected)
- ✅ Can be reset to original defaults via button
- Available to all sessions
- Marked with metadata: `isDefault: true`

**Default Agents in Agent JSON**:
```json
{
  "id": "research-assistant",
  "name": "Research Assistant",
  "purpose": "Help with research tasks",
  "systemPrompt": "You are a helpful research assistant...",
  "modelId": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
  "maxIterations": 5,
  "tools": ["web_search", "fetch_url"],
  "isLocked": false,
  "isDefault": true,
  "createdAt": "2026-01-15T12:00:00Z",
  "updatedAt": "2026-01-15T16:30:00Z"
}
```

**Reset to Default Button**:
1. User clicks "Reset to Default" button in web viewer (only appears for isDefault=true agents)
2. Web viewer sends: `{ type: "reset_agent", id: "research-assistant" }`
3. Rust backend restores agent from original preset in `src/agents/presets.rs`
4. Rust backend saves reset version to disk
5. Rust backend sends confirmation with restored agent
6. Web viewer displays success message and updates UI

### Custom vs Default Agents

| Property | Custom Agent | Default Agent |
|----------|--------------|---------------|
| **Created by** | User | Built-in/Rust binary |
| **Editable** | ✅ Yes | ✅ Yes (still editable) |
| **Deletable** | ✅ Yes | ❌ No (protected) |
| **Resetable** | ❌ No reset button | ✅ Yes (restore to original) |
| **Stored** | Disk | Disk (with isDefault=true) |
| **Modified** | Stays modified | Can reset to original |

---

## Development

### Architecture Principle: Backend Ownership + Frontend Editing

The Rust backend is the **single source of truth** for:
- ✅ Agent definitions (stored at `~/.cf_ai_local_tools/agents.json`)
- ✅ Agent persistence (saves all changes to disk)
- ✅ Configuration validation (ensures valid agents before saving)
- ✅ Tool registry and execution
- ✅ ReAct loop execution

The web viewer is a **frontend for editing**, not storing:
- ✅ Displays agents from backend
- ✅ Provides agent editor form (create/edit/delete UI)
- ✅ Sends edits to backend (doesn't save locally)
- ✅ Shows confirmation after backend persists
- ❌ Does NOT store edited agents in localStorage
- ❌ Does NOT persist changes locally
- ❌ Does NOT execute agents

### Key Pattern: Send → Confirm → Display

```
User edits agent in form
    ↓
Send update to Rust backend via WebSocket
    ↓
Backend validates and saves to disk
    ↓
Backend sends confirmation with saved agent
    ↓
Web viewer updates UI with confirmed agent
```

**NOT:**
```
User edits agent
    ↓
Web viewer shows immediately
    ↓ (maybe never send to backend)
```

### Adding New Tools (Backend Only)

1. **Define tool** in `src/agents/presets.rs`:
   ```rust
   ToolDefinition {
       id: "my_tool",
       name: "My Tool",
       description: "Does something...",
       parameters: vec![
           ToolParameter {
               name: "param1",
               description: "...",
               required: true,
               param_type: "string",
           },
       ],
   }
   ```

2. **Implement execution** in `src/main.rs` message handler:
   ```rust
   "chat_request" => {
       // ... parse message ...
       // ... execute ReAct loop ...
   }
   ```

3. **Tools are automatically visible** to web viewer (sent on handshake)

4. **LLM automatically uses tools** (no web viewer code changes needed)

### Modifying Agent Prompts (Backend Only)

1. Edit `src/agents/prompt.rs`
2. Modify prompt template
3. Rebuild Rust app: `cargo build --release`
4. All new sessions use updated prompt

### Debugging Agent Storage

1. **View agents file**:
   ```bash
   cat ~/.cf_ai_local_tools/agents.json
   ```

2. **Check Rust logs**:
   ```bash
   RUST_LOG=debug cargo run
   ```

3. **Verify backend receives WebSocket messages**:
   ```bash
   # Look for log output like:
   # [INFO] Received create_agent request
   # [INFO] Created agent: my-agent-id
   # [AgentStorage] Created agent: my-agent-id
   ```

### Frontend Implementation Checklist

When building the web viewer agent editor, ensure:

#### Data Loading & Persistence
- [ ] No agent localStorage: `localStorage.removeItem('agents')`
- [ ] Load agents on WebSocket connect:
   ```typescript
   useEffect(() => {
     if (ws?.readyState === WebSocket.OPEN) {
       ws.send(JSON.stringify({ type: "get_agents" }));
     }
   }, [ws]);
   ```
- [ ] Cache tool list from handshake for editor form:
   ```typescript
   case "handshake":
     setAvailableTools(data.tools);
     break;
   ```

#### Agent Editor Form
- [ ] Create form with fields:
   - Name (text input)
   - Purpose (text input)
   - System Prompt (textarea)
   - Model (dropdown)
   - Max Iterations (number input)
   - Tools (checkboxes from backend tools list)
- [ ] Create Agent button:
   ```typescript
   const handleCreate = (formData) => {
     ws.send(JSON.stringify({
       type: "create_agent",
       agent: formData
     }));
   };
   ```
- [ ] Update Agent button:
   ```typescript
   const handleUpdate = (agentId, formData) => {
     ws.send(JSON.stringify({
       type: "update_agent",
       id: agentId,
       agent: formData
     }));
   };
   ```
- [ ] Delete Agent button (only for custom agents):
   ```typescript
   const handleDelete = (agentId) => {
     ws.send(JSON.stringify({
       type: "delete_agent",
       id: agentId
     }));
   };
   ```
- [ ] Reset to Default button (only shows for isDefault=true agents):
   ```typescript
   const handleReset = (agentId) => {
     if (confirm("Reset this agent to its default configuration?")) {
       ws.send(JSON.stringify({
         type: "reset_agent",
         id: agentId
       }));
     }
   };
   ```

#### Response Handlers
- [ ] Handle agent CRUD responses:
   ```typescript
   case "agents_list":
     setAgents(data.agents); // from backend
     break;
   case "agent_created":
     setAgents(prev => [...prev, data.agent]); // from backend
     resetForm();
     break;
   case "agent_updated":
     setAgents(prev => prev.map(a => 
       a.id === data.agent.id ? data.agent : a
     ));
     setEditingId(null);
     break;
   case "agent_deleted":
     setAgents(prev => prev.filter(a => a.id !== data.id));
     break;
   case "agent_reset":
     setAgents(prev => prev.map(a => 
       a.id === data.agent.id ? data.agent : a
     ));
     showSuccess(`Agent reset to default`);
     break;
   case "agent_error":
     showError(data.error);
     break;
   ```

#### UI Display
- [ ] Show agents loaded from backend
- [ ] Display available tools in editor (from backend handshake)
- [ ] Show confirmation messages after CRUD operations
- [ ] Disable delete button for default agents (isDefault=true)
- [ ] Show reset button only for default agents (isDefault=true)
- [ ] Show loading state while backend processes
- [ ] Show indicator (badge/label) for which agents are built-in defaults

#### Important: Never do this
- ❌ `localStorage.setItem('agents', ...)`
- ❌ Modify agents locally without sending to backend
- ❌ Show agents before backend confirmation
- ❌ Execute ReAct locally in web viewer
- ❌ Cache agent data across sessions

### Debugging Tips

**Web Viewer Not Showing Agents?**
1. Check WebSocket is connected: `console.log(ws.readyState)`
2. Verify message sent: `console.log(JSON.stringify({ type: "get_agents" }))`
3. Check for `agents_list` response: `console.log(event.data)`
4. Verify backend has agents: `cat ~/.cf_ai_local_tools/agents.json`

**Agent Creation Fails?**
1. Check logs: `RUST_LOG=debug cargo run`
2. Look for validation errors: `[AgentStorage]` log lines
3. Verify agent ID is unique
4. Ensure all required fields are present

**Tools Not Showing in Editor?**
1. Tools sent in handshake message
2. Cache them on WebSocket connect:
   ```typescript
   const handleHandshake = (data: any) => {
     setAvailableTools(data.tools);
   };
   ```
3. Display in form: `availableTools.map(t => <Checkbox />)`

---

## Deployment

### Deploy Worker

```bash
cd cf-worker
wrangler deploy
```

### Deploy Web Viewer

```bash
cd web-viewer
npm run build
wrangler pages deploy dist/
```

### Update Rust App

```bash
cargo build --release
./target/release/cf_ai_local_tools
```

Set production Worker URL:
```bash
WORKER_WS_URL=wss://your-worker.workers.dev/connect cargo run
```

---

## Verification Checklist

- [ ] Worker starts without errors
- [ ] Rust app connects successfully (logs show "Connected successfully!")
- [ ] Web viewer shows "Connected" status
- [ ] Can send chat messages
- [ ] Receives LLM responses
- [ ] Agent configuration is respected (model, iterations, tools)
- [ ] Multiple web viewers can connect simultaneously
- [ ] Rust app auto-reconnects if Worker restarts

---

## Known Limitations

1. **Tool Call Parser**: ReAct loop uses basic parser (consider structured output)
2. **Streaming**: Currently sends final response only (no per-iteration streaming)
3. **Tool Execution**: Placeholder implementation (needs full wiring)
4. **Agent Persistence**: Stored in web viewer localStorage (consider Worker KV)
5. **Authentication**: No multi-user auth yet (all connect to same Durable Object per instance)

---

## Future Enhancements

1. **Streaming Responses** - Send iteration updates during ReAct loop
2. **Proper Tool Parser** - Use structured LLM output for tool calls
3. **Multi-User Support** - Route Durable Objects by user ID
4. **QR Code Pairing** - Device authentication
5. **Preset Storage** - Save presets to Worker KV
6. **Tool Library** - More automation tools via rustautogui extensions

---

## Troubleshooting

### Rust App Won't Connect

1. Verify Worker is running: `wrangler dev`
2. Check WebSocket URL: `echo $WORKER_WS_URL`
3. Check logs: `RUST_LOG=debug cargo run`

### Worker Returns 500 Error

1. Check wrangler logs: `wrangler tail`
2. Verify AI binding is configured
3. Ensure Durable Object is registered in wrangler.toml

### Web Viewer Shows "Disconnected"

1. Verify web viewer .env.local points to correct Worker URL
2. Check browser console for WebSocket errors
3. Verify CORS settings in Worker

### Messages Not Flowing

1. Check Worker relay logs: `wrangler tail`
2. Verify Rust app registered tools (logs show "Server handshake acknowledged")
3. Ensure agent config is valid JSON

---

## Technology Stack

| Layer | Technologies |
|-------|-------------|
| **Automation** | Rust, tokio, rustautogui |
| **Edge Computing** | Cloudflare Workers, Durable Objects, AI |
| **Frontend** | React, TypeScript, Vite, Tailwind CSS |
| **Communication** | WebSocket (WSS in production) |
| **LLM** | Cloudflare Workers AI (Llama 3.3 70B) |

---

## Files Overview

### Key Rust Files

- `src/main.rs` - WebSocket connection, message handling, tool registration
- `src/agents/react_loop.rs` - ReAct loop implementation
- `src/agents/prompt.rs` - Prompt template interpolation
- `src/llm/client.rs` - HTTP client for LLM API calls

### Key Worker Files

- `cf-worker/src/index.ts` - Main router
- `cf-worker/src/durable-objects/UserSwitchboard.ts` - WebSocket relay
- `cf-worker/src/llm/llm-client.ts` - LLM inference endpoint
- `cf-worker/wrangler.toml` - Cloudflare configuration

### Key Web Viewer Files

- `web-viewer/src/App.tsx` - Main app component
- `web-viewer/src/hooks/useWebSocket.ts` - WebSocket hook
- `web-viewer/src/components/ChatInterfaceV3.tsx` - Chat UI

---

## Version History

- **v3.0** (Jan 15, 2026) - Switchboard Architecture: Worker as relay, Rust executes ReAct locally
- **v2.0** - Worker executes ReAct loop, returns results
- **v1.0** - Initial architecture with tool-only execution

---

## Support & Contributions

For issues or improvements:
1. Check troubleshooting section above
2. Review workspace logs with `RUST_LOG=debug`
3. Check Cloudflare dashboard for Worker errors

---

**Last Updated**: January 15, 2026  
**Architecture Version**: 3.0 Switchboard  
**Status**: ✅ Production Ready
