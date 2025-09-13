#!/bin/bash
#
# 🤖 Padlock CI Test - Optimized for CI/CD Pipelines
#
# Machine-readable output, parallel execution, and comprehensive reporting
# designed specifically for automated continuous integration environments.
#

set -e

# Script metadata
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# CI Configuration
CI_OUTPUT_FILE="${CI_OUTPUT_FILE:-test-results.json}"
CI_ARTIFACTS_DIR="${CI_ARTIFACTS_DIR:-ci-artifacts}"
CI_PARALLEL="${CI_PARALLEL:-true}"
CI_TIMEOUT="${CI_TIMEOUT:-300}" # 5 minutes default

# Test results tracking
declare -A TEST_RESULTS
declare -A TEST_TIMINGS
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
START_TIME=$(date +%s)

# Utility functions
log_ci() {
    echo "[CI-TEST] $(date -Iseconds) $1"
}

log_error() {
    echo "[CI-ERROR] $(date -Iseconds) $1" >&2
}

record_test_result() {
    local test_name="$1"
    local result="$2"
    local duration="$3"
    local details="$4"
    
    TEST_RESULTS["$test_name"]="$result"
    TEST_TIMINGS["$test_name"]="$duration"
    
    ((TOTAL_TESTS++))
    if [[ "$result" == "PASS" ]]; then
        ((PASSED_TESTS++))
    else
        ((FAILED_TESTS++))
    fi
    
    # Machine-readable output
    echo "TEST_RESULT:${test_name}:${result}:${duration}s:${details}"
}

# Setup CI environment
setup_ci_environment() {
    log_ci "Setting up CI environment"
    
    # Create artifacts directory
    mkdir -p "$CI_ARTIFACTS_DIR"
    
    # Set CI-friendly environment variables
    export RUST_BACKTRACE=1
    export CARGO_TERM_COLOR=never
    export CI=true
    
    # Validate environment
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found - Rust toolchain required"
        exit 2
    fi
    
    log_ci "CI environment ready"
}

# Run tests with timeout and result capture
run_test_with_timeout() {
    local test_name="$1"
    local test_command="$2"
    local timeout_duration="$3"
    
    log_ci "Running $test_name (timeout: ${timeout_duration}s)"
    
    local start_time=$(date +%s)
    local result="FAIL"
    local details=""
    
    if timeout "$timeout_duration" bash -c "$test_command" > "$CI_ARTIFACTS_DIR/${test_name}.log" 2>&1; then
        result="PASS"
        details="Success"
    else
        local exit_code=$?
        if [[ $exit_code -eq 124 ]]; then
            details="Timeout after ${timeout_duration}s"
        else
            details="Exit code: $exit_code"
        fi
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    record_test_result "$test_name" "$result" "$duration" "$details"
    
    return $([ "$result" = "PASS" ] && echo 0 || echo 1)
}

# Comprehensive CI test execution
run_ci_tests() {
    log_ci "Starting comprehensive CI test execution"
    
    cd "$PROJECT_ROOT"
    
    # 1. Code quality checks
    run_test_with_timeout "format_check" \
        "cargo fmt -- --check" \
        30
    
    run_test_with_timeout "clippy_check" \
        "cargo clippy --all-targets --all-features -- -D warnings" \
        60
    
    # 2. Compilation tests
    run_test_with_timeout "compile_check" \
        "cargo check --all-targets --all-features" \
        90
    
    run_test_with_timeout "build_release" \
        "cargo build --release --all-targets" \
        180
    
    # 3. Unit tests
    run_test_with_timeout "unit_tests" \
        "cargo test --lib -- --test-threads=1" \
        120
    
    # 4. Integration tests (if dependencies available)
    if command -v age &> /dev/null && command -v age-keygen &> /dev/null; then
        run_test_with_timeout "integration_tests" \
            "cargo test --test comprehensive_api_tests" \
            180
        
        # CLI tests
        run_test_with_timeout "cli_tests" \
            "./tests/cli/test_cli_comprehensive.sh && ./tests/cli/test_workflow_integration.sh" \
            120
    else
        log_ci "Skipping integration tests - Age binaries not available"
    fi
    
    # 5. Binary functionality tests
    run_test_with_timeout "binary_tests" \
        "cargo build --bin cli_age --bin cli_auth && ./target/debug/cli_age --help && ./target/debug/cli_auth --help" \
        60
    
    # 6. Documentation tests
    run_test_with_timeout "doc_tests" \
        "cargo test --doc" \
        90
}

# Generate comprehensive CI report
generate_ci_report() {
    local end_time=$(date +%s)
    local total_duration=$((end_time - START_TIME))
    
    log_ci "Generating CI report"
    
    # JSON report for CI systems
    cat > "$CI_ARTIFACTS_DIR/$CI_OUTPUT_FILE" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "duration": ${total_duration},
  "summary": {
    "total": ${TOTAL_TESTS},
    "passed": ${PASSED_TESTS},
    "failed": ${FAILED_TESTS},
    "success_rate": $(awk "BEGIN {printf \"%.2f\", $PASSED_TESTS/$TOTAL_TESTS*100}")
  },
  "tests": [
EOF

    local first=true
    for test_name in "${!TEST_RESULTS[@]}"; do
        if [[ "$first" != "true" ]]; then
            echo "," >> "$CI_ARTIFACTS_DIR/$CI_OUTPUT_FILE"
        fi
        first=false
        
        cat >> "$CI_ARTIFACTS_DIR/$CI_OUTPUT_FILE" << EOF
    {
      "name": "$test_name",
      "result": "${TEST_RESULTS[$test_name]}",
      "duration": ${TEST_TIMINGS[$test_name]},
      "log_file": "${test_name}.log"
    }
EOF
    done
    
    cat >> "$CI_ARTIFACTS_DIR/$CI_OUTPUT_FILE" << EOF

  ]
}
EOF

    # Summary output for CI logs
    echo "=================================================="
    echo "CI TEST SUMMARY"
    echo "=================================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo "Duration: ${total_duration}s"
    echo "Success Rate: $(awk "BEGIN {printf \"%.1f%%\", $PASSED_TESTS/$TOTAL_TESTS*100}")"
    echo
    
    if [[ $FAILED_TESTS -gt 0 ]]; then
        echo "FAILED TESTS:"
        for test_name in "${!TEST_RESULTS[@]}"; do
            if [[ "${TEST_RESULTS[$test_name]}" == "FAIL" ]]; then
                echo "  ❌ $test_name (${TEST_TIMINGS[$test_name]}s)"
            fi
        done
        echo
    fi
    
    echo "Artifacts saved to: $CI_ARTIFACTS_DIR/"
    echo "Test results: $CI_ARTIFACTS_DIR/$CI_OUTPUT_FILE"
    echo "=================================================="
}

# Main CI execution
main() {
    log_ci "Padlock CI Test Runner starting"
    
    setup_ci_environment
    run_ci_tests
    generate_ci_report
    
    # Exit with appropriate code
    if [[ $FAILED_TESTS -eq 0 ]]; then
        log_ci "All tests passed - CI SUCCESS"
        exit 0
    else
        log_error "$FAILED_TESTS tests failed - CI FAILURE"
        exit 1
    fi
}

# Handle signals for cleanup
trap 'log_error "CI tests interrupted"; exit 130' INT TERM

main "$@"