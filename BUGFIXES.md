# Bug Fixes - January 14, 2026

## Issues Found & Fixed

### Issue 1: Handshake Message Parsing Error

**Error:**
```
ERROR cf_ai_local_tools: Failed to parse command after removing commandId: 
unknown variant `handshake_ack`, expected one of `mouse_move`, `mouse_click`, ...
```

**Root Cause:**
The Worker sends a `handshake_ack` message after the initial connection, but the Rust app was trying to parse ALL incoming messages as automation commands, including protocol messages.

**Fix:**
Added message type filtering in [src/main.rs](src/main.rs):
```rust
// Handle protocol messages (don't try to parse as commands)
if let Some(msg_type) = value.get("type").and_then(|v| v.as_str()) {
    match msg_type {
        "handshake_ack" => {
            info!("Server handshake acknowledged");
            continue;  // Skip command parsing
        }
        "pong" => {
            continue;  // Skip command parsing
        }
        _ => {} // Continue to parse as command
    }
}
```

**Result:** ✅ Protocol messages are now silently acknowledged

---

### Issue 2: Type Mismatch - String Coordinates Instead of Numbers

**Error:**
```
ERROR cf_ai_local_tools: Failed to parse command after removing commandId: 
invalid type: string "0", expected u32
```

Example problematic message:
```json
{"type":"mouse_move","x":"0","y":"0","commandId":"cmd_1_..."}
```

**Root Cause:**
The Cloudflare AI LLM was returning tool arguments as strings (e.g., `"x": "0"`) but the Rust app expected numbers (u32/f32). This happens because LLM tool calling sometimes serializes numeric values as strings.

**Fix:**
Added argument normalization in [cf-worker/src/index.ts](cf-worker/src/index.ts):
```typescript
// Normalize arguments to ensure correct types
const args = toolCall.arguments || {};
const normalizedArgs: any = {};

// Ensure numeric fields are numbers
for (const [key, value] of Object.entries(args)) {
    if ((key === 'x' || key === 'y' || key === 'intensity') && typeof value === 'string') {
        normalizedArgs[key] = parseInt(value, 10);
    } else if (key === 'duration' && typeof value === 'string') {
        normalizedArgs[key] = parseFloat(value);
    } else {
        normalizedArgs[key] = value;
    }
}
```

**Result:** ✅ All numeric fields are now properly converted before sending to Rust app

---

### Issue 3: Missing Optional Field - Duration

**Error:**
```
ERROR cf_ai_local_tools: Failed to parse command after removing commandId: missing field `duration`
```

Example problematic message:
```json
{"type":"mouse_move","x":500,"y":500,"commandId":"cmd_0_..."}
```

**Root Cause:**
The Worker LLM tool definitions had `duration` and `intensity` as optional parameters, but the Rust `Command` enum required them. When the LLM didn't explicitly set these values, the Rust parser failed.

**Fix:**
Made optional fields have defaults in [src/main.rs](src/main.rs):
```rust
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Command {
    MouseMove { x: u32, y: u32, #[serde(default = "default_duration")] duration: f32 },
    MouseClick { button: String },
    MouseScroll { direction: String, #[serde(default = "default_intensity")] intensity: u32 },
    // ...
}

fn default_duration() -> f32 {
    1.0  // Default movement duration
}

fn default_intensity() -> u32 {
    3    // Default scroll intensity
}
```

**Result:** ✅ Optional fields now use sensible defaults when omitted

---

## How to Test the Fixes

### Option 1: Run Integration Test
```bash
./test-fixes.sh
```

### Option 2: Manual Testing

**Terminal 1 - Start Worker:**
```bash
cd cf-worker && npx wrangler dev --port 8787
```

**Terminal 2 - Start Rust App:**
```bash
cargo run
```

You should see:
```
INFO cf_ai_local_tools: Connected successfully!
INFO cf_ai_local_tools: Server handshake acknowledged
```
(No errors about `handshake_ack`)

**Terminal 3 - Test Commands:**

```bash
# Test 1: Get mouse position (basic command)
curl -X POST http://localhost:8787/api/command \
  -H "Content-Type: application/json" \
  -d '{"type":"get_mouse_position"}'

# Test 2: Move mouse (tests numeric coordinates)
curl -X POST http://localhost:8787/api/command \
  -H "Content-Type: application/json" \
  -d '{"type":"mouse_move","x":500,"y":300,"duration":1.0}'

# Test 2b: Move mouse WITHOUT duration (tests defaults)
curl -X POST http://localhost:8787/api/command \
  -H "Content-Type: application/json" \
  -d '{"type":"mouse_move","x":500,"y":300}'

# Test 3: Scroll without intensity (tests defaults)
curl -X POST http://localhost:8787/api/command \
  -H "Content-Type: application/json" \
  -d '{"type":"mouse_scroll","direction":"down"}'

# Test 3: AI command with complex task
curl -X POST http://localhost:8787/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"Move the mouse to 100, 200 and click"}'
```

---

## Files Modified

1. **[src/main.rs](src/main.rs)**
   - Added protocol message handling (handshake_ack, pong)
   - Messages are now silently acknowledged instead of parsed as commands

2. **[cf-worker/src/index.ts](cf-worker/src/index.ts)**
   - Added argument type normalization
   - String coordinates are converted to numbers before sending
   - String intensities/durations are converted to appropriate numeric types

3. **[src/main.rs](src/main.rs)**
   - Made optional fields have defaults (duration, intensity)
   - Uses sensible defaults: 1.0s for duration, 3 for scroll intensity
   - Deserialization now succeeds even if optional fields are omitted

---

## Verification

Run this to verify the fixes work:

```bash
# 1. Rebuild Rust app
cargo build

# 2. Test with integration script
./test-fixes.sh

# 3. Or manually test via web interface
# Visit http://localhost:3000 and type: "Move mouse to 500, 500"
```

---

## Summary of Changes

| Component | Issue | Fix | Status |
|-----------|-------|-----|--------|
| Rust App | Handshake parsing | Message filtering | ✅ Fixed |
| Worker | Type conversion | Argument normalization | ✅ Fixed |
| Rust App | Missing optional fields | Default values | ✅ Fixed |
| Rust App | Type validation | Added in Worker | ✅ Fixed |
| Worker | Type conversion | Argument normalization | ✅ Fixed |

The system should now work smoothly without type errors!
