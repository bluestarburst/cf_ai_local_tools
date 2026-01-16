#!/bin/bash
# Agent Integration Test Runner
# This script helps run the agent integration tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Agent Integration Test Runner ===${NC}\n"

# Check if WebSocket server is running
check_server() {
    if ! nc -z localhost 8787 2>/dev/null; then
        echo -e "${YELLOW}⚠️  WebSocket server is not running on port 8787${NC}"
        echo -e "${YELLOW}Please start it in another terminal:${NC}"
        echo ""
        echo "  cd cf-worker"
        echo "  npx wrangler dev --port 8787"
        echo ""
        read -p "Press enter when the server is running..."
    else
        echo -e "${GREEN}✓ WebSocket server is running${NC}\n"
    fi
}

# Run all tests
run_all_tests() {
    echo -e "${YELLOW}Running all agent integration tests...${NC}\n"
    cargo test -- --ignored --test-threads=1
}

# Run specific agent tests
run_agent_tests() {
    local agent=$1
    echo -e "${YELLOW}Running ${agent} agent tests...${NC}\n"
    cargo test $agent -- --ignored
}

# Run specific test
run_specific_test() {
    local test=$1
    echo -e "${YELLOW}Running test: ${test}${NC}\n"
    cargo test $test -- --ignored
}

# Show help
show_help() {
    echo "Usage: ./run_tests.sh [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  all                      Run all agent tests"
    echo "  orchestrator             Run orchestrator agent tests"
    echo "  desktop_automation       Run desktop automation tests"
    echo "  web_research             Run web research agent tests"
    echo "  code_assistant           Run code assistant tests"
    echo "  conversational           Run conversational agent tests"
    echo "  test_debug               Run test & debug agent tests"
    echo "  general                  Run general assistant tests"
    echo "  specific <test_name>     Run a specific test by name"
    echo "  help                     Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./run_tests.sh all"
    echo "  ./run_tests.sh orchestrator"
    echo "  ./run_tests.sh specific test_orchestrator_greeting_no_delegation"
    echo ""
    echo "Prerequisites:"
    echo "  1. Start WebSocket server: cd cf-worker && npx wrangler dev --port 8787"
    echo "  2. In another terminal, run: ./run_tests.sh [COMMAND]"
}

# Main logic
if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

check_server

case "$1" in
    all)
        run_all_tests
        ;;
    orchestrator)
        run_agent_tests "orchestrator"
        ;;
    desktop_automation)
        run_agent_tests "desktop_automation"
        ;;
    web_research)
        run_agent_tests "web_research"
        ;;
    code_assistant)
        run_agent_tests "code_assistant"
        ;;
    conversational)
        run_agent_tests "conversational"
        ;;
    test_debug)
        run_agent_tests "test_debug"
        ;;
    general)
        run_agent_tests "general"
        ;;
    specific)
        if [ -z "$2" ]; then
            echo -e "${RED}Error: Test name required${NC}"
            echo "Usage: ./run_tests.sh specific <test_name>"
            exit 1
        fi
        run_specific_test "$2"
        ;;
    help)
        show_help
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        show_help
        exit 1
        ;;
esac

echo -e "\n${GREEN}✓ Tests completed${NC}"
