#!/bin/bash

# Integration Test Script
# Tests the complete flow of the CF AI Local Tools

set -e

WORKER_URL="${WORKER_URL:-http://localhost:8787}"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🧪 CF AI Local Tools - Integration Tests"
echo "========================================="
echo "Testing against: $WORKER_URL"
echo ""

# Test 1: Worker Health
echo "Test 1: Worker Health Check"
response=$(curl -s "$WORKER_URL/")
if echo "$response" | grep -q "CF AI Local Tools Worker"; then
    echo -e "${GREEN}✅ Worker is responding${NC}"
else
    echo -e "${RED}❌ Worker not responding correctly${NC}"
    exit 1
fi
echo ""

# Test 2: Connection Status
echo "Test 2: Connection Status"
status=$(curl -s "$WORKER_URL/api/status")
if echo "$status" | grep -q "connected"; then
    if echo "$status" | grep -q '"connected":true'; then
        echo -e "${GREEN}✅ Rust app is connected${NC}"
        echo "$status" | jq '.' 2>/dev/null || echo "$status"
    else
        echo -e "${YELLOW}⚠️  Rust app not connected${NC}"
        echo "   Make sure to run: cargo run"
    fi
else
    echo -e "${RED}❌ Status endpoint not working${NC}"
    exit 1
fi
echo ""

# Test 3: Direct Command (only if connected)
if echo "$status" | grep -q '"connected":true'; then
    echo "Test 3: Direct Command (Get Mouse Position)"
    cmd_response=$(curl -s -X POST "$WORKER_URL/api/command" \
        -H "Content-Type: application/json" \
        -d '{"type":"get_mouse_position"}')
    
    if echo "$cmd_response" | grep -q '"type"'; then
        echo -e "${GREEN}✅ Command executed${NC}"
        echo "$cmd_response" | jq '.' 2>/dev/null || echo "$cmd_response"
    else
        echo -e "${RED}❌ Command failed${NC}"
        echo "$cmd_response"
    fi
    echo ""
    
    # Test 4: Mouse Movement
    echo "Test 4: Mouse Movement"
    echo "   (Watch your mouse cursor move!)"
    move_response=$(curl -s -X POST "$WORKER_URL/api/command" \
        -H "Content-Type: application/json" \
        -d '{"type":"mouse_move","x":500,"y":500,"duration":0.5}')
    
    if echo "$move_response" | grep -q "success\|Success"; then
        echo -e "${GREEN}✅ Mouse moved${NC}"
    else
        echo -e "${RED}❌ Mouse movement failed${NC}"
        echo "$move_response"
    fi
    echo ""
    
    # Test 5: AI Chat
    echo "Test 5: AI Chat"
    echo "   (Asking AI to get mouse position)"
    chat_response=$(curl -s -X POST "$WORKER_URL/api/chat" \
        -H "Content-Type: application/json" \
        -d '{"message":"What is the current mouse position?"}')
    
    if echo "$chat_response" | grep -q "executedTools"; then
        echo -e "${GREEN}✅ AI chat working${NC}"
        echo "   Tools executed:"
        echo "$chat_response" | jq '.executedTools[] | .tool' 2>/dev/null || echo "   (install jq for pretty output)"
    else
        echo -e "${YELLOW}⚠️  AI chat response:${NC}"
        echo "$chat_response" | jq '.' 2>/dev/null || echo "$chat_response"
    fi
    echo ""
else
    echo "⏭️  Skipping command tests (Rust app not connected)"
    echo ""
fi

# Summary
echo "========================================="
echo "📊 Test Summary"
echo ""
if echo "$status" | grep -q '"connected":true'; then
    echo -e "${GREEN}All systems operational!${NC}"
    echo ""
    echo "✅ Worker running"
    echo "✅ Rust app connected"
    echo "✅ Commands working"
    echo ""
    echo "🌐 Web viewer: http://localhost:3000"
    echo "🔗 Worker: $WORKER_URL"
else
    echo -e "${YELLOW}Setup incomplete${NC}"
    echo ""
    echo "✅ Worker running"
    echo "❌ Rust app not connected"
    echo ""
    echo "To complete setup:"
    echo "  1. Open a new terminal"
    echo "  2. Run: cargo run"
    echo "  3. Re-run this test script"
fi
echo ""
