#!/bin/bash
# Test DuckDuckGo web search with the custom implementation

set -e

cd /Users/bryanthargreaves/Documents/personal/cf_ai_local_tools

echo "=========================================="
echo "Testing DuckDuckGo Web Search Fix"
echo "=========================================="
echo ""

# Build the project
echo "[1/3] Building project..."
cargo build --quiet 2>/dev/null
echo "✓ Build successful"
echo ""

# Run the test
echo "[2/3] Running web search test..."
echo "Query: 'rust programming'"
echo ""

TEST_OUTPUT=$(cargo test test_real_web_search -- --nocapture 2>&1 || true)

echo "[3/3] Checking results..."
echo "$TEST_OUTPUT" | grep -A 20 "Query: rust programming" | head -25

# Check if we got results
if echo "$TEST_OUTPUT" | grep -q '"result_count": [1-9]'; then
    echo ""
    echo "✅ SUCCESS: DuckDuckGo search is working!"
    echo "$TEST_OUTPUT" | grep '"result_count"'
    echo ""
    echo "The fix resolved the issue of 0 results being returned."
    echo "The problem was that DuckDuckGo proxy URLs (containing 'duckduckgo.com')"
    echo "were being filtered out incorrectly. The custom implementation now:"
    echo ""
    echo "1. Detects DuckDuckGo proxy URLs (//duckduckgo.com/l/?uddg=...)"
    echo "2. Extracts the actual destination URL from the uddg parameter"
    echo "3. Returns the real external URLs instead of filtering them out"
    exit 0
elif echo "$TEST_OUTPUT" | grep -q "test_real_web_search.*ok"; then
    echo ""
    echo "✓ Test passed"
    exit 0
else
    echo ""
    echo "❌ Could not verify results - check output above"
    exit 1
fi
