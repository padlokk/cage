#!/bin/bash
#
# ⚡ Padlock Quick Test - Ultra-Fast Development Feedback
#
# Optimized for rapid development cycles with minimal overhead.
# Runs only the fastest, most critical tests to catch obvious issues.
#

set -e

# Script metadata
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Timing
START_TIME=$(date +%s)

print_status() {
    echo -e "${BLUE}⚡${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅${NC} $1"
}

print_error() {
    echo -e "${RED}❌${NC} $1"
}

print_timing() {
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))
    echo -e "${YELLOW}⏱️${NC} Completed in ${duration}s"
}

# Quick smoke tests
main() {
    echo -e "${BLUE}⚡ QUICK TEST - Ultra-Fast Development Feedback${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    cd "$PROJECT_ROOT"
    
    # 1. Quick compile check (fastest way to catch syntax errors)
    print_status "Compile check..."
    if cargo check --quiet > /dev/null 2>&1; then
        print_success "Compile check passed"
    else
        print_error "Compile check failed"
        cargo check
        exit 1
    fi
    
    # 2. Quick clippy check (catch common issues)
    print_status "Clippy check..."
    if cargo clippy --quiet -- -D warnings > /dev/null 2>&1; then
        print_success "Clippy check passed"
    else
        print_error "Clippy warnings found"
        cargo clippy -- -D warnings
        exit 1
    fi
    
    # 3. Fast unit tests only (skip integration)
    print_status "Fast unit tests..."
    if timeout 15s cargo test --lib --quiet -- --test-threads=1 > /dev/null 2>&1; then
        print_success "Unit tests passed"
    else
        print_error "Unit tests failed or timed out"
        cargo test --lib
        exit 1
    fi
    
    # 4. Binary build check
    print_status "Binary build check..."
    if cargo build --bin cli_age --bin cli_auth --quiet > /dev/null 2>&1; then
        print_success "Binaries built successfully"
    else
        print_error "Binary build failed"
        cargo build --bin cli_age --bin cli_auth
        exit 1
    fi
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_success "🎉 All quick tests passed!"
    print_timing
    echo
    echo -e "${YELLOW}💡 For comprehensive testing, run: ./bin/test.sh${NC}"
}

main "$@"