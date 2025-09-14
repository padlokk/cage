#!/bin/bash
#
# 🧪 Padlock Test Suite - User-Friendly Test Entry Point
# 
# A comprehensive test runner with selective execution and user-friendly options.
# This script provides a clean interface to the Padlock testing system while
# delegating to the robust test infrastructure in tests/run_all_tests.sh.
#

set -e

# Script metadata
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TESTS_DIR="$PROJECT_ROOT/tests"
COMPREHENSIVE_RUNNER="$TESTS_DIR/run_all_tests.sh"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Default options
RUN_UNIT=false
RUN_CLI=false
RUN_CLI_AGE=false
RUN_CLI_AUTH=false
RUN_INTEGRATION=false
RUN_ALL=true
QUICK_MODE=false
VERBOSE_MODE=false
QUIET_MODE=false
PARALLEL_MODE=false
CI_MODE=false
SKIP_PREREQ=false
HELP_MODE=false

# Test timing
START_TIME=""
END_TIME=""

# Utility functions
print_header() {
    if [[ "$QUIET_MODE" != "true" ]]; then
        echo -e "${BLUE}${BOLD}"
        echo "╔════════════════════════════════════════════════════════════╗"
        echo "║                    🧪 PADLOCK TEST SUITE                   ║"
        echo "║                  User-Friendly Test Runner                 ║"
        echo "╚════════════════════════════════════════════════════════════╝"
        echo -e "${NC}"
    fi
}

print_info() {
    if [[ "$QUIET_MODE" != "true" ]]; then
        echo -e "${CYAN}[INFO]${NC} $1"
    fi
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_verbose() {
    if [[ "$VERBOSE_MODE" == "true" ]]; then
        echo -e "${BLUE}[VERBOSE]${NC} $1"
    fi
}

# Show help
show_help() {
    cat << EOF
🧪 Padlock Test Suite - User-Friendly Test Runner

USAGE:
    ./bin/test.sh [OPTIONS]

TEST CATEGORIES:
    --unit              Run unit tests only (cargo test)
    --cli               Run CLI tests only (cli_age, cli_auth direct + comprehensive)
    --cli-age           Run cli_age direct module tests only
    --cli-auth          Run cli_auth direct module tests only
    --integration       Run integration tests only (requires Age binary)
    --all               Run all test categories (default)
    --quick             Run quick smoke tests for rapid development

OUTPUT OPTIONS:
    --verbose           Show detailed test output and timing
    --quiet             Minimal output (errors and final results only)
    --ci                CI/CD mode (machine-readable output, timing)

EXECUTION OPTIONS:
    --parallel          Run tests in parallel where possible
    --no-prereq         Skip prerequisite checks (faster startup)
    --help              Show this help message

EXAMPLES:
    # Run all tests (default behavior)
    ./bin/test.sh

    # Quick development feedback
    ./bin/test.sh --quick --verbose

    # Test specific CLI tools (direct module testing)
    ./bin/test.sh --cli-age          # Test Age automation module via cli_age
    ./bin/test.sh --cli-auth         # Test authority chain module via cli_auth

    # Comprehensive CLI testing
    ./bin/test.sh --cli              # All CLI tests including direct module tests

    # Unit and CLI tests with detailed output
    ./bin/test.sh --unit --cli --verbose

    # Test combinations
    ./bin/test.sh --cli-age --cli-auth --unit

    # CI/CD pipeline
    ./bin/test.sh --ci --parallel

    # Silent mode for automation
    ./bin/test.sh --quiet

PREREQUISITES:
    Required: Rust/Cargo toolchain
    Optional: age, age-keygen (for integration tests)

EXIT CODES:
    0 - All selected tests passed
    1 - One or more tests failed
    2 - Invalid arguments or missing dependencies

For more details, see: tests/README.md
EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --unit)
                RUN_UNIT=true
                RUN_ALL=false
                shift
                ;;
            --cli)
                RUN_CLI=true
                RUN_ALL=false
                shift
                ;;
            --cli-age)
                RUN_CLI_AGE=true
                RUN_ALL=false
                shift
                ;;
            --cli-auth)
                RUN_CLI_AUTH=true
                RUN_ALL=false
                shift
                ;;
            --integration)
                RUN_INTEGRATION=true
                RUN_ALL=false
                shift
                ;;
            --all)
                RUN_ALL=true
                RUN_UNIT=false
                RUN_CLI=false
                RUN_CLI_AGE=false
                RUN_CLI_AUTH=false
                RUN_INTEGRATION=false
                shift
                ;;
            --quick)
                QUICK_MODE=true
                RUN_ALL=false
                shift
                ;;
            --verbose)
                VERBOSE_MODE=true
                QUIET_MODE=false
                shift
                ;;
            --quiet)
                QUIET_MODE=true
                VERBOSE_MODE=false
                shift
                ;;
            --parallel)
                PARALLEL_MODE=true
                shift
                ;;
            --ci)
                CI_MODE=true
                QUIET_MODE=true
                shift
                ;;
            --no-prereq)
                SKIP_PREREQ=true
                shift
                ;;
            --help|-h)
                HELP_MODE=true
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                echo
                show_help
                exit 2
                ;;
        esac
    done
}

# Validate environment and dependencies
validate_environment() {
    print_verbose "Validating test environment..."

    # Check if we're in the right directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        print_error "Not in a Rust project directory. Please run from project root or bin/ directory."
        exit 2
    fi

    # Check for comprehensive test runner
    if [[ ! -f "$COMPREHENSIVE_RUNNER" ]]; then
        print_error "Comprehensive test runner not found: $COMPREHENSIVE_RUNNER"
        print_info "Run this script from the project root directory"
        exit 2
    fi

    # Make test scripts executable
    chmod +x "$COMPREHENSIVE_RUNNER" 2>/dev/null || true
    find "$TESTS_DIR" -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true

    print_verbose "Environment validation complete"
}

# Run quick smoke tests
run_quick_tests() {
    print_info "Running quick smoke tests..."
    local start_time=$(date +%s)

    # Quick cargo check
    print_verbose "Quick compile check..."
    if ! cargo check --quiet > /dev/null 2>&1; then
        print_error "Project fails to compile"
        return 1
    fi

    # Quick unit test subset (fast tests only)
    print_verbose "Quick unit tests..."
    if ! timeout 30s cargo test --lib --quiet -- --test-threads=1 > /dev/null 2>&1; then
        print_error "Quick unit tests failed"
        return 1
    fi

    # Quick CLI binary check
    print_verbose "CLI binary validation..."
    if ! cargo build --bin cli_age --bin cli_auth --quiet > /dev/null 2>&1; then
        print_error "CLI binaries failed to build"
        return 1
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    print_success "Quick tests passed in ${duration}s"
    return 0
}

# Run comprehensive tests using existing infrastructure
run_comprehensive_tests() {
    print_verbose "Delegating to comprehensive test runner: $COMPREHENSIVE_RUNNER"
    
    # Set environment variables for the comprehensive runner
    export PADLOCK_TEST_VERBOSE="$VERBOSE_MODE"
    export PADLOCK_TEST_QUIET="$QUIET_MODE"
    export PADLOCK_TEST_CI="$CI_MODE"
    export PADLOCK_TEST_PARALLEL="$PARALLEL_MODE"
    
    # Run the comprehensive test suite
    local exit_code=0
    
    if [[ "$VERBOSE_MODE" == "true" ]]; then
        "$COMPREHENSIVE_RUNNER" || exit_code=$?
    elif [[ "$QUIET_MODE" == "true" ]]; then
        "$COMPREHENSIVE_RUNNER" > /dev/null 2>&1 || exit_code=$?
        if [[ $exit_code -eq 0 ]]; then
            print_success "All comprehensive tests passed"
        else
            print_error "Comprehensive tests failed"
        fi
    else
        "$COMPREHENSIVE_RUNNER" || exit_code=$?
    fi
    
    return $exit_code
}

# Run selective tests based on user options
run_selective_tests() {
    print_info "Running selective tests..."
    local total_failures=0
    local start_time=$(date +%s)

    # Unit tests
    if [[ "$RUN_UNIT" == "true" ]]; then
        print_info "Running unit tests..."
        if [[ "$VERBOSE_MODE" == "true" ]]; then
            cargo test --lib || ((total_failures++))
        else
            cargo test --lib --quiet > /dev/null 2>&1 || ((total_failures++))
        fi
        
        if [[ $total_failures -eq 0 ]]; then
            print_success "Unit tests passed"
        else
            print_error "Unit tests failed"
        fi
    fi

    # CLI Age tests (direct module testing)
    if [[ "$RUN_CLI_AGE" == "true" ]]; then
        print_info "Running CLI Age direct module tests..."
        
        if ! cargo build --bin cli_age --quiet > /dev/null 2>&1; then
            print_error "cli_age binary failed to build"
            ((total_failures++))
        else
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                "$TESTS_DIR/cli/test_cli_age_direct.sh" || ((total_failures++))
            else
                "$TESTS_DIR/cli/test_cli_age_direct.sh" > /dev/null 2>&1 || ((total_failures++))
            fi
            
            if [[ $total_failures -eq 0 ]]; then
                print_success "CLI Age direct module tests passed"
            else
                print_error "CLI Age direct module tests failed"
            fi
        fi
    fi

    # CLI Auth tests (direct module testing)
    if [[ "$RUN_CLI_AUTH" == "true" ]]; then
        print_info "Running CLI Auth direct module tests..."
        
        if ! cargo build --bin cli_auth --quiet > /dev/null 2>&1; then
            print_error "cli_auth binary failed to build"
            ((total_failures++))
        else
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                "$TESTS_DIR/cli/test_cli_auth_direct.sh" || ((total_failures++))
            else
                "$TESTS_DIR/cli/test_cli_auth_direct.sh" > /dev/null 2>&1 || ((total_failures++))
            fi
            
            if [[ $total_failures -eq 0 ]]; then
                print_success "CLI Auth direct module tests passed"
            else
                print_error "CLI Auth direct module tests failed"
            fi
        fi
    fi

    # All CLI tests (comprehensive)
    if [[ "$RUN_CLI" == "true" ]]; then
        print_info "Running comprehensive CLI tests..."
        local cli_failures=0
        
        # Build CLI binaries first
        if ! cargo build --bin cli_age --bin cli_auth --quiet > /dev/null 2>&1; then
            print_error "CLI binaries failed to build"
            ((total_failures++))
        else
            # Run all CLI tests including direct module tests
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                "$TESTS_DIR/cli/test_cli_age_direct.sh" || ((cli_failures++))
                "$TESTS_DIR/cli/test_cli_auth_direct.sh" || ((cli_failures++))
                "$TESTS_DIR/cli/test_cli_comprehensive.sh" || ((cli_failures++))
                "$TESTS_DIR/cli/test_workflow_integration.sh" || ((cli_failures++))
            else
                "$TESTS_DIR/cli/test_cli_age_direct.sh" > /dev/null 2>&1 || ((cli_failures++))
                "$TESTS_DIR/cli/test_cli_auth_direct.sh" > /dev/null 2>&1 || ((cli_failures++))
                "$TESTS_DIR/cli/test_cli_comprehensive.sh" > /dev/null 2>&1 || ((cli_failures++))
                "$TESTS_DIR/cli/test_workflow_integration.sh" > /dev/null 2>&1 || ((cli_failures++))
            fi
            
            if [[ $cli_failures -eq 0 ]]; then
                print_success "Comprehensive CLI tests passed"
            else
                print_error "Comprehensive CLI tests failed"
                ((total_failures++))
            fi
        fi
    fi

    # Integration tests
    if [[ "$RUN_INTEGRATION" == "true" ]]; then
        print_info "Running integration tests..."
        
        # Check for Age binary
        if ! command -v age &> /dev/null || ! command -v age-keygen &> /dev/null; then
            print_warning "Age binaries not found - skipping integration tests"
            print_info "Install age and age-keygen to enable integration tests"
        else
            local int_failures=0
            
            # Run integration tests
            cd "$TESTS_DIR/integration"
            
            # Age basic test
            if rustc test_age_basic.rs > /dev/null 2>&1 && ./test_age_basic > /dev/null 2>&1; then
                print_verbose "Age basic integration test passed"
                rm -f test_age_basic
            else
                print_error "Age basic integration test failed"
                ((int_failures++))
            fi
            
            # Authority operations test
            if rustc test_authority_operations_direct.rs --extern padlock="$PROJECT_ROOT/target/debug/libpadlock.rlib" -L "$PROJECT_ROOT/target/debug/deps" > /dev/null 2>&1 && ./test_authority_operations_direct > /dev/null 2>&1; then
                print_verbose "Authority operations test passed"
                rm -f test_authority_operations_direct
            else
                print_error "Authority operations test failed"
                ((int_failures++))
            fi
            
            cd "$PROJECT_ROOT"
            
            if [[ $int_failures -eq 0 ]]; then
                print_success "Integration tests passed"
            else
                print_error "Integration tests failed"
                ((total_failures++))
            fi
        fi
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    if [[ $total_failures -eq 0 ]]; then
        print_success "All selective tests passed in ${duration}s"
        return 0
    else
        print_error "$total_failures test categories failed"
        return 1
    fi
}

# Generate CI-friendly output
print_ci_summary() {
    if [[ "$CI_MODE" == "true" ]]; then
        local end_time=$(date +%s)
        local total_duration=$((end_time - START_TIME))
        
        echo "PADLOCK_TEST_RESULT=$1"
        echo "PADLOCK_TEST_DURATION=${total_duration}s"
        echo "PADLOCK_TEST_TIMESTAMP=$(date -Iseconds)"
    fi
}

# Main execution function
main() {
    START_TIME=$(date +%s)
    
    parse_args "$@"
    
    if [[ "$HELP_MODE" == "true" ]]; then
        show_help
        exit 0
    fi
    
    print_header
    
    if [[ "$SKIP_PREREQ" != "true" ]]; then
        validate_environment
    fi
    
    local test_result=0
    
    # Determine which tests to run
    if [[ "$QUICK_MODE" == "true" ]]; then
        print_info "Quick mode: Running fast smoke tests"
        run_quick_tests || test_result=1
    elif [[ "$RUN_ALL" == "true" ]]; then
        print_info "Running comprehensive test suite"
        run_comprehensive_tests || test_result=1
    else
        print_info "Running selective tests"
        run_selective_tests || test_result=1
    fi
    
    END_TIME=$(date +%s)
    
    # Final summary
    if [[ "$test_result" -eq 0 ]]; then
        if [[ "$QUIET_MODE" != "true" ]]; then
            echo
            print_success "🎉 All tests completed successfully!"
            print_info "Total execution time: $((END_TIME - START_TIME))s"
        fi
        print_ci_summary "SUCCESS"
    else
        if [[ "$QUIET_MODE" != "true" ]]; then
            echo
            print_error "❌ Some tests failed"
            print_info "Total execution time: $((END_TIME - START_TIME))s"
            print_info "Run with --verbose for detailed output"
        fi
        print_ci_summary "FAILURE"
    fi
    
    exit $test_result
}

# Run main function with all arguments
main "$@"