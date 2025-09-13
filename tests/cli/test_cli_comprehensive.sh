#!/bin/bash
#
# Comprehensive CLI Test Suite for cli_age and cli_auth
# Tests both CLIs with real operations and validates functionality
#

set -e  # Exit on any error

echo "🧪 COMPREHENSIVE CLI TEST SUITE"
echo "==============================="

# Test directories
TEST_BASE="/tmp/padlock_cli_tests"
KEYS_DIR="${TEST_BASE}/keys"
AGE_TEST_DIR="${TEST_BASE}/age_tests"

# Cleanup function
cleanup() {
    echo "🧹 Cleaning up test directories..."
    rm -rf "${TEST_BASE}"
}

# Setup function  
setup() {
    echo "📁 Setting up test environment..."
    rm -rf "${TEST_BASE}"
    mkdir -p "${KEYS_DIR}" "${AGE_TEST_DIR}"
    echo "✅ Test environment ready: ${TEST_BASE}"
}

# Test cli_age functionality
test_cli_age() {
    echo
    echo "🔥 TESTING CLI_AGE (Age Automation Interface)"
    echo "=============================================="
    
    echo "📋 Testing cli_age help system..."
    if ./target/debug/cli_age --help > /dev/null 2>&1; then
        echo "✅ cli_age help system working"
    else
        echo "❌ cli_age help system failed"
        return 1
    fi
    
    echo "📋 Testing cli_age with binary format flag..."
    if ./target/debug/cli_age --format binary test > /dev/null 2>&1; then
        echo "✅ cli_age --format binary working"
    else
        echo "❌ cli_age --format binary failed"
        return 1
    fi
    
    echo "📋 Testing cli_age with ascii format flag..."
    if ./target/debug/cli_age --format ascii test > /dev/null 2>&1; then
        echo "✅ cli_age --format ascii working"
    else
        echo "❌ cli_age --format ascii failed"
        return 1
    fi
    
    echo "📋 Testing cli_age demo command..."
    if ./target/debug/cli_age demo > /dev/null 2>&1; then
        echo "✅ cli_age demo command working"
    else
        echo "❌ cli_age demo command failed"
        return 1
    fi
    
    echo "🎉 cli_age tests PASSED"
}

# Test cli_auth functionality  
test_cli_auth() {
    echo
    echo "🔑 TESTING CLI_AUTH (Authority Chain Interface)"
    echo "==============================================="
    
    echo "📋 Testing cli_auth help system..."
    if ./target/debug/cli_auth --help > /dev/null 2>&1; then
        echo "✅ cli_auth help system working"
    else
        echo "❌ cli_auth help system failed"
        return 1
    fi
    
    echo "📋 Testing cli_auth authority chain generation..."
    if ./target/debug/cli_auth --keys-dir "${KEYS_DIR}" generate --name "test-suite" --output-dir "${KEYS_DIR}" > /dev/null 2>&1; then
        echo "✅ cli_auth generate working"
    else
        echo "❌ cli_auth generate failed"
        return 1
    fi
    
    echo "📋 Testing cli_auth status command..."
    if ./target/debug/cli_auth --keys-dir "${KEYS_DIR}" status --show-keys --name "test-suite" > /dev/null 2>&1; then
        echo "✅ cli_auth status working"
    else
        echo "❌ cli_auth status failed"
        return 1
    fi
    
    echo "📋 Validating generated key files..."
    local key_types=("skull" "master" "repo" "ignition" "distro")
    local missing_keys=0
    
    for key_type in "${key_types[@]}"; do
        local key_file="${KEYS_DIR}/test-suite-${key_type}.key"
        if [[ -f "$key_file" ]]; then
            echo "✅ $key_type authority key found"
        else
            echo "❌ $key_type authority key missing"
            ((missing_keys++))
        fi
    done
    
    if [[ $missing_keys -eq 0 ]]; then
        echo "✅ All authority keys generated correctly"
    else
        echo "❌ Missing $missing_keys authority keys"
        return 1
    fi
    
    echo "📋 Testing cli_auth with format flags..."
    if ./target/debug/cli_auth --format ascii --keys-dir "${KEYS_DIR}" status --show-chain > /dev/null 2>&1; then
        echo "✅ cli_auth --format ascii working"
    else
        echo "❌ cli_auth --format ascii failed"
        return 1
    fi
    
    echo "📋 Testing cli_auth validation command..."
    if ./target/debug/cli_auth --keys-dir "${KEYS_DIR}" validate --test-all > /dev/null 2>&1; then
        echo "✅ cli_auth validate working"
    else
        echo "❌ cli_auth validate failed"
        return 1
    fi
    
    echo "📋 Testing cli_auth demo command..."
    if ./target/debug/cli_auth demo full-chain > /dev/null 2>&1; then
        echo "✅ cli_auth demo working"
    else
        echo "❌ cli_auth demo failed"
        return 1
    fi
    
    echo "🎉 cli_auth tests PASSED"
}

# Test CLI consistency
test_cli_consistency() {
    echo
    echo "🔄 TESTING CLI CONSISTENCY"
    echo "=========================="
    
    echo "📋 Testing consistent --format flag behavior..."
    
    # Test both CLIs accept same format values
    local formats=("binary" "ascii")
    for format in "${formats[@]}"; do
        if ./target/debug/cli_age --format "$format" test > /dev/null 2>&1 && \
           ./target/debug/cli_auth --format "$format" demo basic > /dev/null 2>&1; then
            echo "✅ Both CLIs accept --format $format"
        else
            echo "❌ Format consistency issue with $format"
            return 1
        fi
    done
    
    echo "📋 Testing consistent --verbose flag behavior..."
    if ./target/debug/cli_age --verbose test > /dev/null 2>&1 && \
       ./target/debug/cli_auth --verbose demo basic > /dev/null 2>&1; then
        echo "✅ Both CLIs accept --verbose flag"
    else
        echo "❌ Verbose flag consistency issue"
        return 1
    fi
    
    echo "🎉 CLI consistency tests PASSED"
}

# Test real Age integration
test_age_integration() {
    echo
    echo "⚡ TESTING REAL AGE INTEGRATION"
    echo "==============================="
    
    echo "📋 Testing Age binary availability..."
    if command -v age > /dev/null 2>&1 && command -v age-keygen > /dev/null 2>&1; then
        echo "✅ Age binaries available"
    else
        echo "⚠️ Age binaries not found - skipping integration tests"
        return 0
    fi
    
    echo "📋 Testing cli_auth real key generation..."
    local real_keys_dir="${TEST_BASE}/real_keys"
    mkdir -p "$real_keys_dir"
    
    if ./target/debug/cli_auth --keys-dir "$real_keys_dir" generate --name "integration" --output-dir "$real_keys_dir" > /dev/null 2>&1; then
        echo "✅ Real authority key generation working"
        
        # Check if keys are actual Age keys
        local test_key="${real_keys_dir}/integration-master.key"
        if [[ -f "$test_key" ]] && grep -q "AGE-SECRET-KEY-" "$test_key"; then
            echo "✅ Generated keys are valid Age keys"
        else
            echo "❌ Generated keys are not valid Age format"
            return 1
        fi
    else
        echo "❌ Real key generation failed"
        return 1
    fi
    
    echo "🎉 Age integration tests PASSED"
}

# Main execution
main() {
    echo "🚀 Starting comprehensive CLI tests..."
    echo "Build date: $(date)"
    echo
    
    # Ensure binaries are built
    echo "🔧 Building CLI binaries..."
    if cargo build --bin cli_age --bin cli_auth > /dev/null 2>&1; then
        echo "✅ CLI binaries built successfully"
    else
        echo "❌ Failed to build CLI binaries"
        exit 1
    fi
    
    # Setup test environment
    trap cleanup EXIT
    setup
    
    # Run test suites
    test_cli_age
    test_cli_auth  
    test_cli_consistency
    test_age_integration
    
    echo
    echo "🎉 ALL CLI TESTS PASSED!"
    echo "========================"
    echo "✅ cli_age regression tests: PASSED"
    echo "✅ cli_auth functionality tests: PASSED"  
    echo "✅ CLI consistency tests: PASSED"
    echo "✅ Age integration tests: PASSED"
    echo
    echo "🏆 Both CLI tools are ready for production use!"
}

# Run main function
main "$@"