# Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         CF AI Local Tools System                         │
└─────────────────────────────────────────────────────────────────────────┘

┌──────────────────────┐         ┌──────────────────────┐         ┌──────────────────────┐
│   Web Viewer         │         │  Cloudflare Worker   │         │   Rust Local App     │
│   (Vite + React)     │         │   (Edge Computing)   │         │   (rustautogui)      │
├──────────────────────┤         ├──────────────────────┤         ├──────────────────────┤
│                      │         │                      │         │                      │
│  ┌────────────────┐ │         │  ┌────────────────┐ │         │  ┌────────────────┐ │
│  │  Chat UI       │ │         │  │  REST API      │ │         │  │  WebSocket     │ │
│  │                │ │         │  │  /api/chat     │ │         │  │  Client        │ │
│  │  • Input       │ │         │  │  /api/command  │ │         │  │                │ │
│  │  • History     │ │         │  │  /api/status   │ │         │  │  • Reconnect   │ │
│  │  • Status      │ │         │  │                │ │         │  │  • Heartbeat   │ │
│  └────────────────┘ │         │  └────────────────┘ │         │  └────────────────┘ │
│                      │         │                      │         │                      │
│  ┌────────────────┐ │         │  ┌────────────────┐ │         │  ┌────────────────┐ │
│  │  Tool Display  │ │         │  │  Cloudflare AI │ │         │  │  Automation    │ │
│  │                │ │         │  │  (Llama 3.3)   │ │         │  │  Handler       │ │
│  │  • Tool Calls  │ │         │  │                │ │         │  │                │ │
│  │  • Results     │ │         │  │  • Tool Call   │ │         │  │  • Mouse       │ │
│  │  • Thinking    │ │         │  │  • Planning    │ │         │  │  • Keyboard    │ │
│  └────────────────┘ │         │  └────────────────┘ │         │  │  • Screenshot  │ │
│                      │         │                      │         │  └────────────────┘ │
└──────────────────────┘         └──────────────────────┘         └──────────────────────┘
         │                                │                                │
         │  HTTPS                         │  WebSocket (WSS)               │
         │  /api/*                        │  /connect                      │
         └────────────────────────────────┴────────────────────────────────┘
                                          │
                                          ▼
                            ┌──────────────────────┐
                            │  Durable Object      │
                            │  (ConnectionManager) │
                            ├──────────────────────┤
                            │  • Persistent WS     │
                            │  • Command Queue     │
                            │  • Response Routing  │
                            └──────────────────────┘


┌─────────────────────────────────────────────────────────────────────────┐
│                              Data Flow                                   │
└─────────────────────────────────────────────────────────────────────────┘

User Types: "Move mouse to 500, 500 and click"
                │
                ▼
┌───────────────────────────────────────────────────────────────────────┐
│ 1. Web Viewer → Worker                                                │
│    POST /api/chat                                                     │
│    { message: "Move mouse to 500, 500 and click" }                   │
└───────────────────────────────────────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────────────────────────────┐
│ 2. Worker → Cloudflare AI                                             │
│    AI.run('@cf/meta/llama-3.3-70b-instruct-fp8-fast', {              │
│      messages: [...],                                                 │
│      tools: [mouse_move, mouse_click, ...]                           │
│    })                                                                 │
└───────────────────────────────────────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────────────────────────────┐
│ 3. AI Decides Tool Calls                                              │
│    [                                                                  │
│      { name: "mouse_move", args: {x:500, y:500, duration:1.0} },    │
│      { name: "mouse_click", args: {button:"left"} }                  │
│    ]                                                                  │
└───────────────────────────────────────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────────────────────────────┐
│ 4. Worker → Durable Object → Rust App (via WebSocket)                │
│    For each tool call:                                                │
│      WS.send({"type":"mouse_move","x":500,"y":500,"duration":1.0})  │
│      WS.send({"type":"mouse_click","button":"left"})                 │
└───────────────────────────────────────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────────────────────────────┐
│ 5. Rust App Executes Commands                                         │
│    gui.move_mouse_to_pos(500, 500, 1.0)                              │
│    gui.click(MouseClick::LEFT)                                        │
│    → Returns: {"type":"success","message":"..."}                      │
└───────────────────────────────────────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────────────────────────────┐
│ 6. Results Flow Back                                                  │
│    Rust App → Durable Object → Worker → Web Viewer                   │
│    Display:                                                           │
│      • AI Response                                                    │
│      • Tool Calls Made                                                │
│      • Execution Results                                              │
│      • Any Errors                                                     │
└───────────────────────────────────────────────────────────────────────┘


┌─────────────────────────────────────────────────────────────────────────┐
│                         Component Responsibilities                       │
└─────────────────────────────────────────────────────────────────────────┘

┌──────────────────────┐
│  Web Viewer          │  • User interface
│  (Browser)           │  • Chat display
├──────────────────────┤  • Real-time updates
│  React + Vite        │  • Connection status
│  Tailwind CSS        │  • Tool call visualization
│  TypeScript          │  • Error handling
└──────────────────────┘

┌──────────────────────┐
│  Cloudflare Worker   │  • HTTP API endpoints
│  (Edge)              │  • AI orchestration
├──────────────────────┤  • Tool calling logic
│  TypeScript          │  • Command routing
│  Workers AI          │  • Response formatting
│  Durable Objects     │  • CORS handling
└──────────────────────┘

┌──────────────────────┐
│  Durable Object      │  • WebSocket persistence
│  (Stateful)          │  • Connection management
├──────────────────────┤  • Command queueing
│  ConnectionManager   │  • Response routing
│  WebSocket Server    │  • Session tracking
└──────────────────────┘

┌──────────────────────┐
│  Rust Local App      │  • GUI automation
│  (Your Computer)     │  • WebSocket client
├──────────────────────┤  • Command execution
│  Rust                │  • Error reporting
│  rustautogui         │  • Auto-reconnection
│  tokio               │  • Hardware control
└──────────────────────┘


┌─────────────────────────────────────────────────────────────────────────┐
│                         Security Layers (Future)                         │
└─────────────────────────────────────────────────────────────────────────┘

                          ┌──────────────────────┐
                          │  Cloudflare Access   │
                          │  (Authentication)    │
                          └──────────────────────┘
                                    │
                          ┌──────────────────────┐
                          │  API Keys / JWT      │
                          │  (Authorization)     │
                          └──────────────────────┘
                                    │
                          ┌──────────────────────┐
                          │  Rate Limiting       │
                          │  (Abuse Prevention)  │
                          └──────────────────────┘
                                    │
                          ┌──────────────────────┐
                          │  Command Whitelist   │
                          │  (Restriction)       │
                          └──────────────────────┘
                                    │
                          ┌──────────────────────┐
                          │  Audit Logging       │
                          │  (Monitoring)        │
                          └──────────────────────┘


┌─────────────────────────────────────────────────────────────────────────┐
│                         Deployment Options                               │
└─────────────────────────────────────────────────────────────────────────┘

Development (Local):
  Worker:      wrangler dev (localhost:8787)
  Rust App:    cargo run (connects to localhost)
  Web Viewer:  npm run dev (localhost:3000)

Production:
  Worker:      wrangler deploy → your-worker.workers.dev
  Rust App:    cargo build --release → runs on your machine
  Web Viewer:  wrangler pages deploy → your-site.pages.dev

Alternative:
  Worker:      Same as above
  Rust App:    Docker container / systemd service
  Web Viewer:  Vercel / Netlify / GitHub Pages


┌─────────────────────────────────────────────────────────────────────────┐
│                         Network Topology                                 │
└─────────────────────────────────────────────────────────────────────────┘

                           Internet
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
  ┌──────────┐       ┌──────────────┐       ┌──────────┐
  │  User's  │       │  Cloudflare  │       │  Your    │
  │  Browser │◄─────►│  Edge Network│◄─────►│ Computer │
  └──────────┘       └──────────────┘       └──────────┘
       │                     │                     │
       │ HTTPS               │ WSS                 │
       │ /api/chat           │ /connect            │
       │                     │                     │
       └─── Web Viewer  ─────┴─── Rust App ───────┘

Key Points:
• No inbound connections to your computer (NAT-friendly)
• Rust app connects OUT to Worker (reverse tunnel)
• All traffic encrypted (HTTPS/WSS in production)
• Cloudflare handles scaling and DDoS protection
• Low latency via edge computing


┌─────────────────────────────────────────────────────────────────────────┐
│                         Technology Stack                                 │
└─────────────────────────────────────────────────────────────────────────┘

Frontend:
  ├─ React 18             (UI framework)
  ├─ TypeScript           (Type safety)
  ├─ Vite                 (Build tool)
  ├─ Tailwind CSS         (Styling)
  └─ Lucide React         (Icons)

Backend (Worker):
  ├─ TypeScript           (Language)
  ├─ Cloudflare Workers   (Serverless)
  ├─ Durable Objects      (Stateful WebSocket)
  ├─ Workers AI           (LLM inference)
  └─ Llama 3.3 70B        (AI model)

Local App:
  ├─ Rust 2021            (Language)
  ├─ tokio                (Async runtime)
  ├─ tokio-tungstenite    (WebSocket client)
  ├─ rustautogui          (GUI automation)
  ├─ serde/serde_json     (Serialization)
  └─ tracing              (Logging)

Infrastructure:
  ├─ Cloudflare Pages     (Web hosting)
  ├─ Cloudflare Workers   (API + AI)
  ├─ Durable Objects      (Persistence)
  └─ Wrangler CLI         (Deployment)
