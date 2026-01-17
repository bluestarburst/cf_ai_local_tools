#!/bin/bash
cd /Users/bryanthargreaves/Documents/personal/cf_ai_local_tools

echo "Testing custom DuckDuckGo implementation..."
echo ""

# Run the test and capture output
cargo test test_real_web_search -- --nocapture 2>&1 | tee /tmp/ddg_custom_test.out

echo ""
echo "Test output saved to /tmp/ddg_custom_test.out"
echo "Checking results..."

# Check if we got results
if grep -q "result_count.*:" /tmp/ddg_custom_test.out; then
    echo "✓ Test completed"
    grep "result_count" /tmp/ddg_custom_test.out
else
    echo "✗ No result_count found in output"
fi
