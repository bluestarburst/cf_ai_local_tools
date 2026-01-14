# CF AI Local Tools

Control your local computer using Cloudflare Workers AI through a beautiful web interface.

## Architecture

This project consists of three main components:

```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────────┐
│  Web Viewer     │◄────────┤  CF Worker       │◄────────┤  Rust Local App │
│  (Vite/React)   │  HTTPS  │  (AI + WebSocket)│  WSS    │  (rustautogui)  │
└─────────────────┘         └──────────────────┘         └─────────────────┘
     Browser                  Cloudflare Edge                Your Computer
```

### 1. **Rust Local App** ([/src](./src))
- WebSocket client connecting to Cloudflare Worker
- GUI automation using `rustautogui` library
- Executes commands: mouse control, keyboard input, screenshots
- Reconnects automatically if connection drops

### 2. **Cloudflare Worker** ([/cf-worker](./cf-worker))
- Hosts WebSocket endpoint for Rust app connection
- Integrates with Cloudflare Workers AI (LLM)
- Uses Durable Objects for persistent connections
- Provides REST API for web interface
- Handles tool calling and command routing

### 3. **Web Viewer** ([/web-viewer](./web-viewer))
- React + TypeScript + Tailwind CSS
- Real-time chat interface
- Displays LLM thinking and tool executions
- Shows connection status
- Deployable to Cloudflare Pages

## Features

✨ **AI-Powered Control**: Natural language commands to control your computer  
🔄 **Real-time Updates**: See LLM reasoning and tool calls as they happen  
🔌 **Auto-Reconnect**: Rust app maintains connection automatically  
🎨 **Beautiful UI**: Modern, responsive interface with dark mode  
🛠️ **Tool Calling**: LLM autonomously executes multi-step tasks  
📊 **Status Monitoring**: Live connection status and session info  

## Quick Start

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Node.js**: v18+ (for Worker and Web Viewer)
- **Cloudflare Account**: Free tier works!
- **Wrangler CLI**: `npm install -g wrangler`

### Setup Steps

#### 1. Rust Local App

```bash
# Install dependencies and run
cargo build --release
cargo run
```

The app will attempt to connect to `ws://localhost:8787/connect` by default. Set `WORKER_WS_URL` environment variable to change:

```bash
WORKER_WS_URL=wss://your-worker.workers.dev/connect cargo run
```

#### 2. Cloudflare Worker

```bash
cd cf-worker

# Install dependencies
npm install

# Login to Cloudflare
wrangler login

# Deploy to production
wrangler deploy

# Or run locally for development
wrangler dev
```

**Important**: Update the worker URL in your Rust app after deployment!

#### 3. Web Viewer

```bash
cd web-viewer

# Install dependencies
npm install

# Update .env.local with your worker URL
echo "VITE_WORKER_URL=https://your-worker.workers.dev" > .env.local

# Run development server
npm run dev

# Build and deploy to Cloudflare Pages
npm run build
wrangler pages deploy dist
```

## Usage

### Starting the System

1. **Start Cloudflare Worker** (development):
   ```bash
   cd cf-worker && wrangler dev
   ```

2. **Start Rust App**:
   ```bash
   cargo run
   ```
   You should see: `Connected successfully!`

3. **Start Web Viewer**:
   ```bash
   cd web-viewer && npm run dev
   ```
   Open http://localhost:3000

### Example Commands

Try these natural language commands in the web interface:

- "Move the mouse to the center of the screen"
- "Move mouse to position 500, 300"
- "Click the left mouse button"
- "Type hello world"
- "Press the return key"
- "Scroll down 5 times"
- "What is the current mouse position?"

### Direct API Usage

You can also send commands directly to the worker:

```bash
# Send a command
curl -X POST https://your-worker.workers.dev/api/command \
  -H "Content-Type: application/json" \
  -d '{"type":"mouse_move","x":500,"y":500,"duration":1.0}'

# Check connection status
curl https://your-worker.workers.dev/api/status

# Chat with AI
curl -X POST https://your-worker.workers.dev/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"Move the mouse to 500, 500 and click"}'
```

## Available Commands

The Rust app supports these command types:

### Mouse Commands

- `mouse_move`: Move cursor to position
  ```json
  {"type": "mouse_move", "x": 500, "y": 500, "duration": 1.0}
  ```

- `mouse_click`: Click a button
  ```json
  {"type": "mouse_click", "button": "left"}  // left, right, middle
  ```

- `mouse_scroll`: Scroll wheel
  ```json
  {"type": "mouse_scroll", "direction": "down", "intensity": 3}  // up, down, left, right
  ```

- `get_mouse_position`: Get current position
  ```json
  {"type": "get_mouse_position"}
  ```

### Keyboard Commands

- `keyboard_input`: Type text
  ```json
  {"type": "keyboard_input", "text": "Hello World!"}
  ```

- `keyboard_command`: Press special keys
  ```json
  {"type": "keyboard_command", "command": "return"}  // return, tab, backspace, etc.
  ```

See [rustautogui keyboard commands](https://github.com/DavorMar/rustautogui#keyboard-functions) for all available keys.

## Configuration

### Environment Variables

**Rust App** (`.env` or system env):
```bash
WORKER_WS_URL=wss://your-worker.workers.dev/connect
```

**Web Viewer** (`.env.local`):
```bash
VITE_WORKER_URL=https://your-worker.workers.dev
```

### Worker Configuration

Edit [cf-worker/wrangler.toml](./cf-worker/wrangler.toml):

```toml
name = "cf-ai-local-tools-worker"
main = "src/index.ts"
compatibility_date = "2024-12-18"

[ai]
binding = "AI"  # Cloudflare Workers AI

[[durable_objects.bindings]]
name = "CONNECTIONS"
class_name = "ConnectionManager"
```

## Deployment

### Deploy Worker to Production

```bash
cd cf-worker
wrangler deploy
```

Your worker will be available at: `https://cf-ai-local-tools-worker.YOUR-SUBDOMAIN.workers.dev`

### Deploy Web Viewer to Cloudflare Pages

```bash
cd web-viewer
npm run build
wrangler pages deploy dist --project-name ai-local-tools-viewer
```

Your site will be available at: `https://ai-local-tools-viewer.pages.dev`

### Run Rust App on Your Machine

The Rust app must run locally on the machine you want to control. Update the `WORKER_WS_URL` to your deployed worker:

```bash
WORKER_WS_URL=wss://cf-ai-local-tools-worker.YOUR-SUBDOMAIN.workers.dev/connect cargo run --release
```

Consider running it as a background service:

**macOS (launchd)**:
```bash
# Create a .plist file in ~/Library/LaunchAgents/
```

**Linux (systemd)**:
```bash
# Create a service file in /etc/systemd/system/
```

## Security Considerations

⚠️ **Important Security Notes**:

1. **This gives remote control of your computer** - Only use in trusted environments
2. **No authentication by default** - Add auth before exposing publicly
3. **WebSocket is unencrypted in dev** - Use WSS (TLS) in production
4. **Cloudflare Workers AI** - All prompts go through Cloudflare's AI

### Recommended Security Enhancements

1. **Add authentication** to the Worker endpoints
2. **Use API keys** or OAuth for access control
3. **Rate limiting** on the Worker
4. **Whitelist specific actions** in the Rust app
5. **Log all commands** for audit trail
6. **Use Cloudflare Access** for the web viewer

## Troubleshooting

### Rust App Can't Connect

- Check `WORKER_WS_URL` is correct (starts with `ws://` or `wss://`)
- Verify worker is running: `curl https://your-worker.workers.dev/api/status`
- Check firewall/network settings

### Worker Errors

- Run `wrangler tail` to see live logs
- Verify Durable Objects are enabled on your account
- Check AI binding is configured in wrangler.toml

### Web Viewer Shows Disconnected

- Check browser console for errors
- Verify `VITE_WORKER_URL` in `.env.local`
- Test worker endpoint directly: `curl https://your-worker.workers.dev/api/status`

### rustautogui Not Working on macOS

- Grant Accessibility permissions: System Preferences → Security & Privacy → Accessibility
- Add Terminal/iTerm to allowed apps

## Development

### Project Structure

```
cf_ai_local_tools/
├── src/                    # Rust app source
│   └── main.rs            # WebSocket client + automation
├── cf-worker/             # Cloudflare Worker
│   ├── src/
│   │   ├── index.ts       # Main worker logic
│   │   └── durable-objects/
│   │       └── ConnectionManager.ts  # WebSocket manager
│   ├── wrangler.toml      # Worker configuration
│   └── package.json
├── web-viewer/            # React web interface
│   ├── src/
│   │   ├── App.tsx        # Main UI component
│   │   └── main.tsx
│   ├── vite.config.ts
│   └── package.json
├── Cargo.toml             # Rust dependencies
└── README.md
```

### Adding New Commands

1. **Add command type** to Rust `Command` enum in [src/main.rs](./src/main.rs)
2. **Implement handler** in `AutomationHandler::handle_command()`
3. **Add tool definition** in Worker [cf-worker/src/index.ts](./cf-worker/src/index.ts) `/api/chat` endpoint
4. **Test** with web interface or direct API call

## Credits

- [rustautogui](https://github.com/DavorMar/rustautogui) - Rust GUI automation
- [Cloudflare Workers AI](https://developers.cloudflare.com/workers-ai/) - LLM inference
- [Cloudflare Durable Objects](https://developers.cloudflare.com/durable-objects/) - Persistent WebSocket connections

## License

MIT - Use at your own risk. This is a powerful tool that gives remote control of your computer.

## Contributing

Contributions welcome! Areas for improvement:

- [ ] Screenshot capture support
- [ ] Image recognition for click targets
- [ ] Multi-monitor support
- [ ] Windows/Linux testing
- [ ] Authentication system
- [ ] Command history/replay
- [ ] Macros/scripting
- [ ] Mobile app viewer

---

**⚠️ Use Responsibly**: This tool provides remote control of your computer. Always ensure proper security measures before exposing to the internet.
