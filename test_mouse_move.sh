#!/bin/bash
# Quick test script for mouse_move tool

echo "Testing mouse_move tool..."
echo ""

# Test 1: Basic mouse move
echo "Test 1: Moving mouse to (500, 600)"
cargo run --quiet -- <<EOF
{
  "action": "execute_tool",
  "tool": "mouse_move",
  "args": {
    "x": 500,
    "y": 600
  }
}
EOF

echo ""
echo "Test 2: Get current mouse position"
cargo run --quiet -- <<EOF
{
  "action": "execute_tool",
  "tool": "get_mouse_position",
  "args": {}
}
EOF

echo ""
echo "Test 3: Move mouse to (100, 100)"
cargo run --quiet -- <<EOF
{
  "action": "execute_tool",
  "tool": "mouse_move",
  "args": {
    "x": 100,
    "y": 100
  }
}
EOF

echo ""
echo "Test 4: Verify position changed"
cargo run --quiet -- <<EOF
{
  "action": "execute_tool",
  "tool": "get_mouse_position",
  "args": {}
}
EOF

echo ""
echo "All tests completed!"
