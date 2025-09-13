#!/bin/bash
#
# Padlock Complete Test Runner
# Runs all tests in the proper order with comprehensive validation
#

set -e

echo "🧪 PADLOCK COMPLETE TEST SUITE"
echo "=============================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=0

# Utility functions
log_test_start() {
    echo -e "\n${BLUE}🔵 Starting: $1${NC}"
    ((TOTAL_TESTS++))
}

log_test_pass() {
    echo -e "${GREEN}✅ PASSED: $1${NC}"
    ((TESTS_PASSED++))
}

log_test_fail() {
    echo -e "${RED}❌ FAILED: $1${NC}"
    ((TESTS_FAILED++))
}

# Prerequisite checks
check_prerequisites() {
    echo "🔧 Checking Prerequisites..."
    
    # Check Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo not found. Install Rust toolchain first.${NC}"
        exit 1
    fi
    
    # Check Age binaries
    if ! command -v age &> /dev/null || ! command -v age-keygen &> /dev/null; then
        echo -e "${YELLOW}⚠️ Age binaries not found. Some integration tests will be skipped.${NC}"
        AGE_AVAILABLE=false
    else
        echo -e "${GREEN}✅ Age binaries available${NC}"
        AGE_AVAILABLE=true
    fi
    
    # Build project first
    echo "🔨 Building Padlock project..."
    if cargo build --bin cli_age --bin cli_auth > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Build successful${NC}"
    else
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
}

# Run Rust unit and API tests
run_rust_tests() {
    log_test_start "Rust Unit and API Tests"
    
    if cargo test > /dev/null 2>&1; then
        log_test_pass "Rust Unit and API Tests"
    else
        log_test_fail "Rust Unit and API Tests"
    fi
}

# Run CLI functionality tests
run_cli_tests() {
    log_test_start "CLI Age Direct Module Tests"
    
    if ./tests/cli/test_cli_age_direct.sh > /dev/null 2>&1; then
        log_test_pass "CLI Age Direct Module Tests"
    else
        log_test_fail "CLI Age Direct Module Tests"
    fi
    
    log_test_start "CLI Auth Direct Module Tests"
    
    if ./tests/cli/test_cli_auth_direct.sh > /dev/null 2>&1; then
        log_test_pass "CLI Auth Direct Module Tests"
    else
        log_test_fail "CLI Auth Direct Module Tests"
    fi
    
    log_test_start "CLI Comprehensive Tests"
    
    if ./tests/cli/test_cli_comprehensive.sh > /dev/null 2>&1; then
        log_test_pass "CLI Comprehensive Tests"
    else
        log_test_fail "CLI Comprehensive Tests"
    fi
    
    log_test_start "CLI Workflow Integration Tests"
    
    if ./tests/cli/test_workflow_integration.sh > /dev/null 2>&1; then
        log_test_pass "CLI Workflow Integration Tests"
    else
        log_test_fail "CLI Workflow Integration Tests"
    fi
}

# Run integration tests (requires Age)
run_integration_tests() {
    if [ "$AGE_AVAILABLE" = true ]; then
        log_test_start "Age Basic Integration Test"
        
        cd tests/integration
        if rustc test_age_basic.rs > /dev/null 2>&1 && ./test_age_basic > /dev/null 2>&1; then
            log_test_pass "Age Basic Integration Test"
            rm -f test_age_basic  # cleanup
        else
            log_test_fail "Age Basic Integration Test"
        fi
        cd ../..
        
        log_test_start "Authority Operations Direct Test"
        
        cd tests/integration
        if rustc test_authority_operations_direct.rs --extern padlock=../../target/debug/libpadlock.rlib -L ../../target/debug/deps > /dev/null 2>&1 && ./test_authority_operations_direct > /dev/null 2>&1; then
            log_test_pass "Authority Operations Direct Test"
            rm -f test_authority_operations_direct  # cleanup
        else
            log_test_fail "Authority Operations Direct Test" 
        fi
        cd ../..
    else
        echo -e "${YELLOW}⚠️ Skipping integration tests (Age binaries not available)${NC}"
    fi
}

# Generate test report
generate_report() {
    echo
    echo "📊 TEST EXECUTION REPORT"
    echo "========================"
    echo "Total Tests Run: $TOTAL_TESTS"
    echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "\n${GREEN}🎉 ALL TESTS PASSED!${NC}"
        echo -e "${GREEN}🏆 Padlock system is ready for production use!${NC}"
        exit 0
    else
        echo -e "\n${RED}❌ Some tests failed.${NC}"
        echo -e "${RED}Please review the failures above.${NC}"
        exit 1
    fi
}

# Main execution
main() {
    echo "🚀 Starting complete test suite at $(date)"
    echo
    
    check_prerequisites
    
    echo -e "\n${BLUE}═══════════════════════════════════════${NC}"
    echo -e "${BLUE}           RUNNING TEST SUITE           ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    
    run_rust_tests
    run_cli_tests
    run_integration_tests
    
    echo -e "\n${BLUE}═══════════════════════════════════════${NC}"
    
    generate_report
}

# Make scripts executable
chmod +x tests/cli/*.sh 2>/dev/null || true

# Run main function
main "$@"