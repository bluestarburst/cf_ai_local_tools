# Development Guide

## Local Development Setup

### 1. Terminal 1: Cloudflare Worker

```bash
cd cf-worker
npm install
wrangler dev --port 8787
```

The worker will be available at `http://localhost:8787`

### 2. Terminal 2: Rust Local App

```bash
# Make sure worker is running first
cargo run
```

You should see:
```
INFO cf_ai_local_tools: Starting automation client...
INFO cf_ai_local_tools: Will connect to: ws://localhost:8787/connect
INFO cf_ai_local_tools: Connecting to WebSocket...
INFO cf_ai_local_tools: Connected successfully!
```

### 3. Terminal 3: Web Viewer

```bash
cd web-viewer
npm install
npm run dev
```

Visit http://localhost:3000

## Testing the Flow

### Test 1: Connection Status

```bash
curl http://localhost:8787/api/status
```

Expected response:
```json
{
  "connected": true,
  "sessions": [
    {
      "clientId": "uuid-here",
      "connectedAt": "2026-01-14T...",
      "uptime": 12345
    }
  ]
}
```

### Test 2: Direct Command

```bash
curl -X POST http://localhost:8787/api/command \
  -H "Content-Type: application/json" \
  -d '{
    "type": "mouse_move",
    "x": 500,
    "y": 500,
    "duration": 1.0
  }'
```

Your mouse should move to position (500, 500)!

### Test 3: LLM Command

```bash
curl -X POST http://localhost:8787/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Move the mouse to the center of the screen"
  }'
```

### Test 4: Web Interface

1. Open http://localhost:3000
2. Check that status shows "Connected" (green)
3. Type: "Move mouse to 500, 500"
4. Click Send
5. Watch your mouse move!

## Debugging

### Enable Rust Logging

```bash
RUST_LOG=debug cargo run
```

### Watch Worker Logs

In a separate terminal:
```bash
cd cf-worker
wrangler tail
```

### Check WebSocket Connection

```bash
# Install wscat if needed: npm install -g wscat
wscat -c ws://localhost:8787/connect
```

Then type:
```json
{"type":"handshake","client":"test","version":"1.0"}
```

## Common Issues

### Issue: Rust app won't connect

**Solution**: Make sure worker is running on port 8787
```bash
cd cf-worker && wrangler dev --port 8787
```

### Issue: Worker says "No client connected"

**Solution**: Start Rust app before sending commands
```bash
cargo run
# Wait for "Connected successfully!"
```

### Issue: Mouse movements don't work on macOS

**Solution**: Grant Accessibility permissions
1. Open System Preferences → Security & Privacy → Privacy → Accessibility
2. Add Terminal (or your terminal app)
3. Restart terminal

### Issue: Web viewer can't connect

**Solution**: Check `.env.local` has correct URL
```bash
cd web-viewer
echo "VITE_WORKER_URL=http://localhost:8787" > .env.local
npm run dev
```

## Code Hot Reload

- **Worker**: Changes auto-reload with `wrangler dev`
- **Web Viewer**: Vite hot-reloads automatically
- **Rust App**: Must restart manually (`cargo run`)

## Production Testing

### Deploy Worker

```bash
cd cf-worker
wrangler deploy
# Note the URL: https://cf-ai-local-tools-worker.YOUR-SUBDOMAIN.workers.dev
```

### Update Rust App

```bash
export WORKER_WS_URL=wss://cf-ai-local-tools-worker.YOUR-SUBDOMAIN.workers.dev/connect
cargo run --release
```

### Deploy Web Viewer

```bash
cd web-viewer
echo "VITE_WORKER_URL=https://cf-ai-local-tools-worker.YOUR-SUBDOMAIN.workers.dev" > .env.local
npm run build
wrangler pages deploy dist --project-name ai-local-tools
# Note the URL: https://ai-local-tools.pages.dev
```

## Adding New Tools

### Example: Add Screenshot Command

**1. Update Rust Command enum** ([src/main.rs](../src/main.rs)):

```rust
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Command {
    // ... existing commands
    Screenshot { path: Option<String> },
}
```

**2. Add handler**:

```rust
impl AutomationHandler {
    fn handle_command(&self, cmd: Command) -> Response {
        match cmd {
            // ... existing handlers
            Command::Screenshot { path } => {
                // Use rustautogui to capture screenshot
                // Save to path or return base64
                Response::Success { 
                    message: "Screenshot captured".to_string() 
                }
            }
        }
    }
}
```

**3. Add tool to Worker** ([cf-worker/src/index.ts](../cf-worker/src/index.ts)):

```typescript
const tools = [
    // ... existing tools
    {
        name: 'screenshot',
        description: 'Capture a screenshot of the screen',
        parameters: {
            type: 'object',
            properties: {
                path: { 
                    type: 'string', 
                    description: 'Optional file path to save' 
                }
            }
        }
    }
];
```

**4. Test**:

```bash
curl -X POST http://localhost:8787/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Take a screenshot"}'
```

## Performance Tips

### Reduce Latency

1. **Use Cloudflare Workers on nearest edge**
   - Worker runs close to users
   - But Rust app location matters more

2. **WebSocket vs HTTP**
   - WebSocket: Lower latency, persistent connection
   - HTTP: Simpler, works through more proxies

3. **Batch Commands**
   - Send multiple commands in one request
   - Reduce round-trips

### Optimize for Speed

**Worker**:
- Cache AI responses for common commands
- Use Workers Analytics to monitor performance

**Rust App**:
- Compile with `--release` for production
- Reduce logging in production builds

**Web Viewer**:
- Use production build: `npm run build`
- Enable caching on Cloudflare Pages

## Monitoring

### Worker Analytics

View in Cloudflare dashboard:
- Request count
- Error rate
- CPU time
- AI inference time

### Custom Logging

Add to Worker:
```typescript
console.log(`Command executed: ${command.type} in ${Date.now() - start}ms`);
```

View with:
```bash
wrangler tail
```

### Rust App Metrics

Add to [src/main.rs](../src/main.rs):
```rust
info!("Command processed in {:?}", start.elapsed());
```

## Security Checklist

Before exposing to internet:

- [ ] Add authentication to Worker endpoints
- [ ] Use HTTPS/WSS only (no HTTP/WS)
- [ ] Implement rate limiting
- [ ] Add IP allowlist
- [ ] Log all commands
- [ ] Add command confirmation UI
- [ ] Restrict dangerous operations
- [ ] Use Cloudflare Access for web viewer
- [ ] Rotate API keys regularly
- [ ] Monitor for suspicious activity

## Next Steps

1. **Add Authentication**: See [SECURITY.md](./SECURITY.md) (to be created)
2. **Add More Tools**: Screenshots, window management, etc.
3. **Improve UI**: Add more visualizations
4. **Mobile App**: React Native or Flutter viewer
5. **Multi-Computer**: Control multiple machines
6. **Recording**: Record and replay command sequences
