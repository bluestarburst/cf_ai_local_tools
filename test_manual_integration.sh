#!/bin/bash
# Manual integration test for desktop agent
# Requires: CF Worker and Desktop App running

echo "🧪 Manual Integration Test - Desktop Agent"
echo ""
echo "Prerequisites check:"
echo "==================="

# Check CF Worker
echo -n "1. Checking CF Worker... "
if curl -s http://localhost:8787/health > /dev/null 2>&1; then
    echo "✅ Running"
else
    echo "❌ Not running"
    echo "   Start with: cd cf-worker && wrangler dev"
    exit 1
fi

# Check Desktop connection
echo -n "2. Checking Desktop App... "
STATUS=$(curl -s http://localhost:8787/api/status)
DESKTOP=$(echo $STATUS | grep -o '"desktop":true' || echo "")
if [ -n "$DESKTOP" ]; then
    echo "✅ Connected"
else
    echo "❌ Not connected"
    echo "   Start with: cargo run"
    exit 1
fi

echo ""
echo "Running Integration Test:"
echo "========================"
cargo test test_desktop_agent_mouse_move_improved -- --ignored --nocapture

echo ""
echo "Test complete!"
