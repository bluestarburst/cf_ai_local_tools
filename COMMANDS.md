# Commands Reference

## Command Types

All commands are sent as JSON with a `type` field specifying the command.

## Mouse Commands

### Move Mouse

Move the cursor to a specific position on screen.

```json
{
  "type": "mouse_move",
  "x": 500,
  "y": 500,
  "duration": 1.0
}
```

**Parameters:**
- `x` (number, required): X coordinate in pixels
- `y` (number, required): Y coordinate in pixels  
- `duration` (number, optional): Time to move in seconds (default: 1.0)

**Example LLM prompts:**
- "Move the mouse to 500, 500"
- "Move cursor to the top left corner"
- "Move mouse to the center of the screen"

### Click Mouse

Click a mouse button at the current position.

```json
{
  "type": "mouse_click",
  "button": "left"
}
```

**Parameters:**
- `button` (string, required): `"left"`, `"right"`, or `"middle"`

**Example LLM prompts:**
- "Click the left mouse button"
- "Right click"
- "Middle click"

### Scroll Mouse

Scroll the mouse wheel.

```json
{
  "type": "mouse_scroll",
  "direction": "down",
  "intensity": 3
}
```

**Parameters:**
- `direction` (string, required): `"up"`, `"down"`, `"left"`, or `"right"`
- `intensity` (number, optional): Number of scroll steps (default: 3)

**Example LLM prompts:**
- "Scroll down 5 times"
- "Scroll up"
- "Scroll to the right"

### Get Mouse Position

Get the current cursor position.

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
  "y": 500
}
```

**Example LLM prompts:**
- "What is the current mouse position?"
- "Where is the cursor?"
- "Get mouse coordinates"

## Keyboard Commands

### Type Text

Type a string of text.

```json
{
  "type": "keyboard_input",
  "text": "Hello, World!"
}
```

**Parameters:**
- `text` (string, required): Text to type

**Example LLM prompts:**
- "Type hello world"
- "Type my email address"
- "Write 'Testing 123'"

**Supported characters:**
- Alphanumeric: `a-z`, `A-Z`, `0-9`
- Special: `! @ # $ % ^ & * ( ) - _ = + [ ] { } ; : ' " , . < > / ? | \`
- Spaces and line breaks

### Press Key

Press a special keyboard key.

```json
{
  "type": "keyboard_command",
  "command": "return"
}
```

**Parameters:**
- `command` (string, required): Key name (see table below)

**Example LLM prompts:**
- "Press enter"
- "Press the tab key"
- "Hit backspace"

### Common Keys

| Key Name | Description |
|----------|-------------|
| `return` | Enter/Return key |
| `tab` | Tab key |
| `backspace` | Backspace/Delete |
| `delete` | Forward delete |
| `escape` | Escape key |
| `space` | Spacebar |
| `up` | Up arrow |
| `down` | Down arrow |
| `left` | Left arrow |
| `right` | Right arrow |
| `home` | Home key |
| `end` | End key |
| `page_up` / `pgup` | Page Up |
| `page_down` / `pgdown` | Page Down |

### Function Keys

| Key Name | Description |
|----------|-------------|
| `F1` - `F12` | Function keys |
| `F13` - `F20` | Extended function keys (macOS) |

### Modifier Keys

| Key Name | Description |
|----------|-------------|
| `shift` | Shift key |
| `control` / `ctrl` | Control key |
| `alt` / `option` | Alt/Option key |
| `command` / `super` | Command/Windows key |

**Note:** For key combinations, you need to use the LLM to coordinate multiple key presses.

## Complete Key List

See [rustautogui Keyboard Commands](https://github.com/DavorMar/rustautogui/blob/main/Keyboard_commands.md) for the full list of available keys.

## Direct API Usage

### Send Command

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

**Response:**
```json
{
  "type": "success",
  "message": "Moved mouse to (500, 500)"
}
```

or

```json
{
  "type": "error",
  "error": "Mouse move failed: ..."
}
```

### Chat with AI

```bash
curl -X POST http://localhost:8787/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Move the mouse to 500, 500 and click",
    "conversationHistory": []
  }'
```

**Response:**
```json
{
  "response": "I moved the mouse and clicked.",
  "toolCalls": [
    {
      "name": "mouse_move",
      "arguments": { "x": 500, "y": 500, "duration": 1.0 }
    },
    {
      "name": "mouse_click",
      "arguments": { "button": "left" }
    }
  ],
  "executedTools": [
    {
      "tool": "mouse_move",
      "arguments": { "x": 500, "y": 500, "duration": 1.0 },
      "result": { "type": "success", "message": "Moved mouse to (500, 500)" }
    },
    {
      "tool": "mouse_click",
      "arguments": { "button": "left" },
      "result": { "type": "success", "message": "Clicked left button" }
    }
  ]
}
```

## Example Workflows

### Open Browser and Search

```json
[
  {"type": "keyboard_command", "command": "command"},
  {"type": "keyboard_input", "text": "space"},
  {"type": "keyboard_input", "text": "chrome"},
  {"type": "keyboard_command", "command": "return"}
]
```

Or just ask the LLM: **"Open Chrome and search for cats"**

### Copy Text

```json
[
  {"type": "mouse_move", "x": 100, "y": 100, "duration": 0.5},
  {"type": "mouse_click", "button": "left"},
  {"type": "keyboard_command", "command": "command"},
  {"type": "keyboard_input", "text": "a"},
  {"type": "keyboard_command", "command": "command"},
  {"type": "keyboard_input", "text": "c"}
]
```

Or just ask: **"Select all and copy"**

### Take Screenshot (Future)

```json
{
  "type": "screenshot",
  "path": "/tmp/screenshot.png"
}
```

*Not yet implemented - coming soon!*

## Error Handling

All commands return either a success or error response:

**Success:**
```json
{
  "type": "success",
  "message": "Operation completed successfully"
}
```

**Error:**
```json
{
  "type": "error",
  "error": "Description of what went wrong"
}
```

## Tips

1. **Coordinate Ranges**: Most displays range from (0,0) to (1920, 1080) or similar
2. **Duration**: Use longer durations (1-2 seconds) for visible movements
3. **Error Recovery**: If a command fails, the LLM can try alternative approaches
4. **Chaining**: The LLM can execute multiple commands in sequence
5. **Natural Language**: Just describe what you want in plain English!

## Testing Commands

Use the web interface at http://localhost:3000 or send direct API calls:

```bash
# Test mouse movement
curl -X POST http://localhost:8787/api/command \
  -H "Content-Type: application/json" \
  -d '{"type":"get_mouse_position"}'

# Test keyboard
curl -X POST http://localhost:8787/api/command \
  -H "Content-Type: application/json" \
  -d '{"type":"keyboard_input","text":"test"}'
```
