# Configuration Example

Copy this file to `.env` in the project root and customize for your setup.

```bash
# Rust App Configuration
# WebSocket URL to connect to Cloudflare Worker
WORKER_WS_URL=ws://localhost:8787/connect

# For production, use wss:// (secure WebSocket)
# WORKER_WS_URL=wss://your-worker.workers.dev/connect

# Logging level (error, warn, info, debug, trace)
RUST_LOG=info

# Reconnection settings (optional, defaults shown)
# RECONNECT_DELAY_SECONDS=5
# MAX_RECONNECT_ATTEMPTS=0  # 0 = infinite
```

## Web Viewer Configuration

Create `web-viewer/.env.local`:

```bash
# Worker API URL
VITE_WORKER_URL=http://localhost:8787

# For production
# VITE_WORKER_URL=https://your-worker.workers.dev
```

## Cloudflare Worker Configuration

Edit `cf-worker/wrangler.toml`:

```toml
name = "cf-ai-local-tools-worker"
main = "src/index.ts"
compatibility_date = "2024-12-18"

# AI Model Selection
[ai]
binding = "AI"

# Available models:
# - @cf/meta/llama-3.3-70b-instruct-fp8-fast (default, recommended)
# - @cf/meta/llama-3.1-8b-instruct-fast (faster, less capable)
# - @cf/mistral/mistral-7b-instruct-v0.1 (alternative)

# Durable Objects (required for WebSocket persistence)
[[durable_objects.bindings]]
name = "CONNECTIONS"
class_name = "ConnectionManager"

[[migrations]]
tag = "v1"
new_classes = ["ConnectionManager"]

# Environment-specific settings
[env.production]
vars = { ENVIRONMENT = "production" }

[env.development]
vars = { ENVIRONMENT = "development" }
```

## Advanced Configuration

### Custom Port for Worker (Development)

```bash
cd cf-worker
wrangler dev --port 8080
```

Then update `.env`:
```bash
WORKER_WS_URL=ws://localhost:8080/connect
```

And `web-viewer/.env.local`:
```bash
VITE_WORKER_URL=http://localhost:8080
```

### Custom Port for Web Viewer

Edit `web-viewer/vite.config.ts`:

```typescript
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,  // Change to your preferred port
  },
})
```

### Rust App as Background Service

#### macOS (launchd)

Create `~/Library/LaunchAgents/com.cf-ai-local-tools.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.cf-ai-local-tools</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/cf_ai_local_tools/target/release/cf_ai_local_tools</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>WORKER_WS_URL</key>
        <string>wss://your-worker.workers.dev/connect</string>
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/cf-ai-local-tools.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/cf-ai-local-tools.error.log</string>
</dict>
</plist>
```

Load it:
```bash
launchctl load ~/Library/LaunchAgents/com.cf-ai-local-tools.plist
```

#### Linux (systemd)

Create `/etc/systemd/system/cf-ai-local-tools.service`:

```ini
[Unit]
Description=CF AI Local Tools
After=network.target

[Service]
Type=simple
User=your-username
WorkingDirectory=/path/to/cf_ai_local_tools
Environment="WORKER_WS_URL=wss://your-worker.workers.dev/connect"
Environment="RUST_LOG=info"
ExecStart=/path/to/cf_ai_local_tools/target/release/cf_ai_local_tools
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable cf-ai-local-tools
sudo systemctl start cf-ai-local-tools
sudo systemctl status cf-ai-local-tools
```

### Security Configuration

#### Add Basic Authentication

Modify `cf-worker/src/index.ts`:

```typescript
// Add this helper function
function authenticate(request: Request): boolean {
  const authHeader = request.headers.get('Authorization');
  const expectedAuth = 'Bearer YOUR_SECRET_TOKEN';
  return authHeader === expectedAuth;
}

// Use in your routes
if (url.pathname === '/api/command' && request.method === 'POST') {
  if (!authenticate(request)) {
    return new Response('Unauthorized', { status: 401 });
  }
  // ... rest of handler
}
```

Update web viewer to send token:

```typescript
const response = await fetch(`${WORKER_URL}/api/chat`, {
  method: 'POST',
  headers: { 
    'Content-Type': 'application/json',
    'Authorization': 'Bearer YOUR_SECRET_TOKEN'
  },
  body: JSON.stringify({ message: input })
});
```

#### Use Cloudflare Access

Protect your worker with Cloudflare Access:

1. Go to Cloudflare Dashboard → Zero Trust → Access
2. Create an application for your worker domain
3. Configure authentication (Google, GitHub, etc.)
4. Users must authenticate before accessing

### Performance Tuning

#### Rust App

```toml
# Cargo.toml - optimize for speed
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

#### Worker

```typescript
// Cache AI responses for common queries
const AI_CACHE = new Map<string, any>();

async function getCachedAIResponse(message: string) {
  if (AI_CACHE.has(message)) {
    return AI_CACHE.get(message);
  }
  const response = await env.AI.run(...);
  AI_CACHE.set(message, response);
  return response;
}
```

### Monitoring

#### Add Request Logging

In `cf-worker/src/index.ts`:

```typescript
export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const start = Date.now();
    const response = await handleRequest(request, env);
    const duration = Date.now() - start;
    
    console.log({
      method: request.method,
      path: new URL(request.url).pathname,
      status: response.status,
      duration: `${duration}ms`,
      timestamp: new Date().toISOString()
    });
    
    return response;
  }
}
```

View logs:
```bash
wrangler tail --format pretty
```

## Environment Variables Reference

### Rust App

| Variable | Default | Description |
|----------|---------|-------------|
| `WORKER_WS_URL` | `ws://localhost:8787/connect` | WebSocket URL |
| `RUST_LOG` | `info` | Log level |

### Web Viewer

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_WORKER_URL` | `http://localhost:8787` | Worker API URL |

### Cloudflare Worker

Configured in `wrangler.toml`:

| Setting | Description |
|---------|-------------|
| `name` | Worker name |
| `[ai]` | AI model binding |
| `[durable_objects.bindings]` | WebSocket manager |
| `vars` | Environment variables |

## Troubleshooting Configuration

### Check Current Configuration

**Rust App:**
```bash
echo $WORKER_WS_URL
echo $RUST_LOG
```

**Web Viewer:**
```bash
cat web-viewer/.env.local
```

**Worker:**
```bash
cat cf-worker/wrangler.toml
```

### Reset to Defaults

```bash
# Rust
unset WORKER_WS_URL
unset RUST_LOG

# Web Viewer
cd web-viewer
echo "VITE_WORKER_URL=http://localhost:8787" > .env.local

# Worker - edit wrangler.toml manually
```
