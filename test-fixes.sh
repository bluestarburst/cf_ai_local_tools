#!/bin/bash

# Quick test after fixes
# Run from project root

set -e

echo "🧪 Testing fixes..."
echo ""

WORKER_URL="http://localhost:8787"

# Test 1: Check worker health
echo "1️⃣  Checking worker health..."
if curl -s "$WORKER_URL" > /dev/null; then
    echo "✅ Worker responding"
else
    echo "❌ Worker not responding. Start it with: cd cf-worker && wrangler dev"
    exit 1
fi
echo ""

# Test 2: Check connection status
echo "2️⃣  Checking connection status..."
status=$(curl -s "$WORKER_URL/api/status")
echo "Status: $(echo $status | jq '.connected' 2>/dev/null || echo '(install jq for pretty output)')"

if echo "$status" | grep -q '"connected":true'; then
    echo "✅ Rust app connected"
    echo ""
    
    # Test 3: Test mouse position (numeric)
    echo "3️⃣  Testing mouse position command..."
    response=$(curl -s -X POST "$WORKER_URL/api/command" \
        -H "Content-Type: application/json" \
        -d '{"type":"get_mouse_position"}')
    
    if echo "$response" | grep -q '"type":"mouse_position"'; then
        echo "✅ Mouse position command works"
        echo "Response: $response"
    else
        echo "❌ Mouse position failed"
        echo "Response: $response"
    fi
    echo ""
    
    # Test 4: Test AI with mouse move
    echo "4️⃣  Testing AI with mouse move..."
    chat=$(curl -s -X POST "$WORKER_URL/api/chat" \
        -H "Content-Type: application/json" \
        -d '{"message":"Move mouse to 100, 200"}')
    
    if echo "$chat" | grep -q 'executedTools'; then
        echo "✅ AI command execution works"
        echo "Checking tool arguments are numeric..."
        x_val=$(echo "$chat" | jq '.executedTools[0].arguments.x' 2>/dev/null || echo "error")
        echo "  x value type: $(echo "$x_val" | head -c 20)"
        if echo "$x_val" | grep -qE '^[0-9]+$'; then
            echo "✅ Numeric values confirmed"
        fi
    else
        echo "❌ AI execution failed"
        echo "Response: $chat"
    fi
else
    echo "⏭️  Rust app not connected. Start with: cargo run"
fi

echo ""
echo "🎉 Tests complete!"
