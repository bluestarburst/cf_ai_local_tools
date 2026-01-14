# Protocol Specification

## Message Format

All messages between components are JSON objects with specific formats.

### Command Message (Worker → Rust App)

```json
{
  "type": "command_type",
  "param1": "value1",
  "param2": 123,
  "commandId": "cmd_0_1768427191716"
}
```

**Fields:**
- `type` (string, required): Command type (e.g., "mouse_move", "keyboard_input")
- `commandId` (string, optional): Unique ID for tracking this command
- Additional fields depend on command type

### Response Message (Rust App → Worker)

Success:
```json
{
  "type": "success",
  "message": "Operation successful",
  "commandId": "cmd_0_1768427191716"
}
```

Error:
```json
{
  "type": "error",
  "error": "Error message describing what went wrong",
  "commandId": "cmd_0_1768427191716"
}
```

Data response:
```json
{
  "type": "mouse_position",
  "x": 500,
  "y": 300,
  "commandId": "cmd_0_1768427191716"
}
```

### Protocol Messages

**Handshake (Rust → Worker):**
```json
{
  "type": "handshake",
  "client": "rust-automation",
  "version": "0.1.0"
}
```

**Handshake Acknowledgment (Worker → Rust):**
```json
{
  "type": "handshake_ack",
  "server": "cloudflare-worker",
  "timestamp": 1768427156951
}
```

These messages are handled at the protocol level and NOT parsed as commands.

## Command Types & Parameters

### Mouse Commands

#### mouse_move
Move cursor to absolute position.

```json
{
  "type": "mouse_move",
  "x": 500,
  "y": 500,
  "duration": 1.0
}
```

**Parameters:**
- `x` (u32): X coordinate (0-based from left)
- `y` (u32): Y coordinate (0-based from top)
- `duration` (f32, optional): Seconds to move (default: 1.0)

#### mouse_click
Click a mouse button.

```json
{
  "type": "mouse_click",
  "button": "left"
}
```

**Parameters:**
- `button` (string): "left", "right", or "middle"

#### mouse_scroll
Scroll the mouse wheel.

```json
{
  "type": "mouse_scroll",
  "direction": "down",
  "intensity": 3
}
```

**Parameters:**
- `direction` (string): "up", "down", "left", or "right"
- `intensity` (u32, optional): Number of scroll steps (default: 3)

#### get_mouse_position
Get current mouse position (no parameters).

```json
{
  "type": "get_mouse_position"
}
```

**Response:**
```json
{
  "type": "mouse_position",
  "x": 500,
  "y": 300
}
```

### Keyboard Commands

#### keyboard_input
Type text.

```json
{
  "type": "keyboard_input",
  "text": "hello world"
}
```

**Parameters:**
- `text` (string): Text to type

**Supported characters:**
- Letters: a-z, A-Z
- Numbers: 0-9
- Special: ! @ # $ % ^ & * ( ) - _ = + [ ] { } ; : ' " , . < > / ? | \
- Whitespace: space, newline (represented as \n)

#### keyboard_command
Press a special key.

```json
{
  "type": "keyboard_command",
  "command": "return"
}
```

**Parameters:**
- `command` (string): Key name

**Common keys:**
- `return` - Enter
- `tab` - Tab
- `backspace` - Backspace
- `delete` - Forward delete
- `escape` - Escape
- `space` - Spacebar
- `up`, `down`, `left`, `right` - Arrow keys
- `home`, `end` - Home/End
- `page_up`, `page_down` - Page Up/Down
- `F1` - `F20` - Function keys
- `shift`, `control`, `alt`, `command` - Modifier keys

## Type Coercion Rules

The Worker normalizes LLM tool arguments to ensure type safety:

| Field | Expected Type | Coercion |
|-------|---------------|----------|
| `x`, `y` | u32 (unsigned 32-bit int) | Parse string to int, truncate negatives |
| `intensity` | u32 | Parse string to int |
| `duration` | f32 (32-bit float) | Parse string to float |
| `button`, `direction`, `command`, `text` | string | Use as-is |

**Example coercions:**
```json
// Input from LLM
{"type": "mouse_move", "x": "500", "y": "300", "duration": "1.5"}

// After normalization
{"type": "mouse_move", "x": 500, "y": 300, "duration": 1.5}
```

## Error Handling

### Type Errors
If the Rust app receives invalid types:
```json
{
  "type": "error",
  "error": "invalid type: string \"abc\", expected u32"
}
```

### Unknown Command
```json
{
  "type": "error",
  "error": "unknown variant `unknown_cmd`, expected one of ..."
}
```

### Execution Error
```json
{
  "type": "error",
  "error": "Failed to move mouse: Permission denied"
}
```

## Message Flow Diagram

```
Worker                              Rust App
  │                                    │
  ├─────────────── CONNECT ────────────▶
  │
  ├──── handshake_ack ────────────────▶  (ignored)
  │
  ├─── mouse_move(x:500, y:300) ──────▶
  │◀────── {type:"success",...} ──────┤
  │
  ├─── keyboard_input("hello") ───────▶
  │◀────── {type:"success",...} ──────┤
  │
  ├─── get_mouse_position() ──────────▶
  │◀────── {type:"mouse_position",...} ┤
```

## Best Practices

1. **Always send numeric values as numbers**, not strings
   - ✅ `"x": 500`
   - ❌ `"x": "500"`

2. **Include commandId for traceability**
   - Helps track responses to requests
   - Format: `cmd_<sequence>_<timestamp>`

3. **Handle protocol messages gracefully**
   - Don't try to parse handshake_ack as a command
   - Silently ignore pongs

4. **Validate arguments before sending**
   - Worker does this, but it's good practice

5. **Use proper key names**
   - `x`, `y` not `X`, `Y`
   - `button` not `btn`
   - `direction` not `dir`

## Example: Complete Mouse Movement

**LLM generates:**
```json
{
  "type": "mouse_move",
  "x": "500",
  "y": "300",
  "duration": 1.0
}
```

**Worker normalizes:**
```json
{
  "type": "mouse_move",
  "x": 500,
  "y": 300,
  "duration": 1.0,
  "commandId": "cmd_42_1768427191716"
}
```

**Rust app receives and parses:**
- Type: mouse_move
- X: 500 (u32)
- Y: 300 (u32)
- Duration: 1.0 (f32)
- CommandId: cmd_42_1768427191716

**Rust app executes:**
```rust
gui.move_mouse_to_pos(500, 300, 1.0)?
```

**Rust app responds:**
```json
{
  "type": "success",
  "message": "Moved mouse to (500, 300)",
  "commandId": "cmd_42_1768427191716"
}
```

**Worker receives and sends to Web Viewer:**
```json
{
  "tool": "mouse_move",
  "arguments": {"x": 500, "y": 300, "duration": 1.0},
  "result": {"type": "success", "message": "Moved mouse to (500, 300)"}
}
```
