# AGENTS.md

This guide is for agentic coding agents working in this repository.

## Build/Test Commands

### Rust Backend (Root)
```bash
cargo run                        # Run application
cargo test                       # Run all tests
cargo test <test_name>           # Run single test
cargo test -- --ignored          # Run integration tests (Worker required)
cargo test -- --verbose          # Verbose output
cargo fmt                        # Format code
cargo clippy                     # Lint code
cargo build --release            # Build release binary
```

### Cloudflare Worker (`cf-worker/`)
```bash
npm install && npm run dev        # Start local dev (port 8787)
npm run deploy                   # Deploy to Cloudflare
npm run tail                     # Tail Worker logs
```

### Web Viewer (`web-viewer/`)
```bash
npm install && npm run dev        # Start Vite dev server
npm run build                    # Build for production (tsc + vite build)
npm run deploy                   # Build and deploy to Pages
```

### Integration Tests
```bash
./run_agent_tests.sh all         # Run all agent tests (Worker required)
./run_agent_tests.sh specific <test_name>  # Run specific test
```

**Note:** Integration tests require Worker running (`cd cf-worker && npm run dev`)

## Code Style Guidelines

### Rust Code

**Imports:** Group imports (std, external, local), use `crate::path`, re-export with `pub use`
```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::agents::AgentConfig;
```

**Types:** Structs use `PascalCase`, fields use `snake_case`. Use serde rename for JSON camelCase. Always derive Debug, Clone, Serialize, Deserialize.
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub system_prompt: String,
    #[serde(rename = "modelId")]
    pub model_id: String,
}
```

**Error Handling:** Use `anyhow::Result<T>`, `context()` for error chains, `bail!()` for early returns
```rust
let result = risky_operation().context("Failed operation")?;
```

**Async:** Use `#[tokio::test]` for async tests, `#[ignore]` for external deps, `tokio::sync::mpsc` for channels

**Logging:** Use `tracing::{info, debug, warn, error}` at appropriate levels

### TypeScript/React Code

**Files:** Components: `PascalCase.tsx`, Hooks/Utils: `camelCase.ts`

**Types:** Define interfaces, avoid `as Type` assertions
```typescript
interface Agent { id: string; name: string; systemPrompt: string; }
```

**Styling:** Use Tailwind CSS, prefer semantic class names, avoid inline styles

## Project Architecture

**Ownership:** Rust Backend owns all agent storage/tools/execution. Web Viewer is frontend-only (no localStorage). Cloudflare Worker relays WebSocket messages and proxies LLM API calls.

**Key Modules:**
- `src/agents/storage.rs`: Agent CRUD operations, persisted to `~/.config/cf_ai_local_tools/agents.json`
- `src/agents/react_loop.rs`: ReAct loop execution with tool calling and delegation support
- `src/agents/presets.rs`: Default agent templates and tool definitions
- `src/llm/client.rs`: HTTP client for Worker's /api/llm endpoint
- `src/tools/`: Tool execution modules (web_search, computer_automation, etc.)

**Agent Storage:** 
- Rust backend only, stored at `~/.config/cf_ai_local_tools/agents.json`
- Web viewer sends WebSocket messages (get_agents, create_agent, update_agent, delete_agent)
- Backend persists changes and returns confirmation
- Default agents defined in code, cannot be deleted (only reset)

## Testing Patterns

**Rust Tests:** Unit tests in `mod.rs` with `#[cfg(test)]`, integration tests in `tests.rs`
```rust
#[tokio::test]
async fn test_feature() {
    let test_agent = create_test_agent();
    assert_eq!(test_agent.name, "Test Agent");
}
```

**Marking Tests:** Use `#[ignore]` for tests requiring external services. Run integration tests with `cargo test -- --ignored`.

**Test Helper Pattern:** Create helper functions to generate test data consistently across tests.

## Important Notes

- Run tests before committing: `cargo test` (and `npm run build` for TypeScript)
- Format with `cargo fmt` before committing
- No agent localStorage in web viewer - use backend WebSocket messages
- Tool definitions are Rust-side only
- Default agents (`isDefault: true`) cannot be deleted, only reset
- Use `anyhow::Context` for descriptive error messages
