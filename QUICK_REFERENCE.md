# Quick Reference Card

## 🚀 Quick Start (3 Terminals)

### Terminal 1: Worker
```bash
cd cf-worker && npx wrangler dev --port 8787
```

### Terminal 2: Rust App
```bash
cargo run
```

### Terminal 3: Web Viewer
```bash
cd web-viewer && npm run dev
```

**Then open:** http://localhost:3000

---

## 📁 Project Structure

```
cf_ai_local_tools/
├── src/main.rs              # Rust app
├── cf-worker/src/           # Cloudflare Worker
└── web-viewer/src/          # React website
```

---

## 🔧 Common Commands

### Setup
```bash
./setup.sh                   # Install all dependencies
```

### Development
```bash
cargo run                    # Run Rust app
cd cf-worker && wrangler dev # Run Worker
cd web-viewer && npm run dev # Run website
```

### Testing
```bash
./test-integration.sh        # Test everything
cargo test                   # Test Rust
cd cf-worker && npm test     # Test Worker
```

### Production
```bash
cargo build --release        # Build Rust
cd cf-worker && wrangler deploy  # Deploy Worker
cd web-viewer && npm run build   # Build website
```

---

## 🎯 Example Prompts

Type these in the web interface:

- "Move the mouse to 500, 500"
- "Click the left mouse button"
- "Type hello world"
- "Scroll down 5 times"
- "Get the mouse position"
- "Move to center and click"

---

## 🔌 API Endpoints

### Check Status
```bash
curl http://localhost:8787/api/status
```

### Send Command
```bash
curl -X POST http://localhost:8787/api/command \
  -H "Content-Type: application/json" \
  -d '{"type":"get_mouse_position"}'
```

### Chat with AI
```bash
curl -X POST http://localhost:8787/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"Move mouse to 500, 500"}'
```

---

## 📝 Available Commands

### Mouse
- `mouse_move` - Move cursor
- `mouse_click` - Click button (left/right/middle)
- `mouse_scroll` - Scroll wheel (up/down/left/right)
- `get_mouse_position` - Get coordinates

### Keyboard
- `keyboard_input` - Type text
- `keyboard_command` - Press key (return, tab, etc.)

See [COMMANDS.md](./COMMANDS.md) for full list.

---

## 🐛 Troubleshooting

### Rust app won't connect
```bash
# Check worker is running
curl http://localhost:8787/api/status

# Check URL is correct
echo $WORKER_WS_URL
```

### Worker errors
```bash
# View logs
cd cf-worker && wrangler tail
```

### Web viewer disconnected
```bash
# Check .env.local
cat web-viewer/.env.local
```

### macOS permissions
System Preferences → Security → Accessibility → Add Terminal

---

## 📚 Documentation

- [README.md](./README.md) - Full documentation
- [DEVELOPMENT.md](./DEVELOPMENT.md) - Development guide
- [COMMANDS.md](./COMMANDS.md) - Command reference
- [CONFIGURATION.md](./CONFIGURATION.md) - Configuration
- [PROJECT_SUMMARY.md](./PROJECT_SUMMARY.md) - Overview

---

## 🔗 URLs

### Development
- Worker: http://localhost:8787
- Web Viewer: http://localhost:3000

### Production
- Worker: https://your-worker.workers.dev
- Web Viewer: https://your-site.pages.dev

---

## 🔑 Environment Variables

### Rust App (.env)
```bash
WORKER_WS_URL=ws://localhost:8787/connect
RUST_LOG=info
```

### Web Viewer (.env.local)
```bash
VITE_WORKER_URL=http://localhost:8787
```

---

## ⚡ Quick Fixes

### "No client connected"
→ Start Rust app: `cargo run`

### "Worker not found"
→ Start Worker: `cd cf-worker && wrangler dev`

### "Connection refused"
→ Check ports: Worker (8787), Viewer (3000)

### Mouse doesn't move
→ macOS: Grant Accessibility permissions

---

## 🎨 Color Codes

- 🟢 Green = Connected
- 🔴 Red = Disconnected
- 🟡 Yellow = Warning
- 🔵 Blue = Info

---

## 📊 Status Indicators

### Web Interface
- **Connected** (green) = Ready to use
- **Disconnected** (red) = Start Rust app

### Terminal
- `INFO` = Normal operation
- `WARN` = Non-critical issue
- `ERROR` = Something failed

---

## 🚨 Emergency Commands

### Stop Everything
```bash
# Ctrl+C in all terminals
```

### Clean Build
```bash
cargo clean
rm -rf cf-worker/node_modules
rm -rf web-viewer/node_modules
./setup.sh
```

### Reset Configuration
```bash
unset WORKER_WS_URL
cd web-viewer && echo "VITE_WORKER_URL=http://localhost:8787" > .env.local
```

---

## 💡 Pro Tips

1. Use `--release` for faster Rust builds
2. Run `wrangler tail` to see live logs
3. Check `/api/status` to verify connection
4. Use longer durations for visible movements
5. Natural language works best with the AI

---

## 🆘 Need Help?

1. Check [README.md](./README.md)
2. Run `./test-integration.sh`
3. View logs: `wrangler tail`
4. Check connection: `curl localhost:8787/api/status`
5. Read error messages carefully

---

**⚠️ Security Warning:**  
This tool provides remote control of your computer.  
Only use in trusted environments!
