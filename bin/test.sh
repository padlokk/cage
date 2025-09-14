#!/bin/bash
# Cage Test Entry Point
# Unified interface for running all Cage tests

set -e

# Configuration
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TEST_DIR="$ROOT_DIR/tests"

# Try to find boxy for pretty output (optional)
BOXY=""
if command -v boxy >/dev/null 2>&1; then
    BOXY="boxy"
elif [[ -f "./target/release/boxy" ]]; then
    BOXY="./target/release/boxy"
elif [[ -f "../boxy/target/release/boxy" ]]; then
    BOXY="../boxy/target/release/boxy"
fi


# Parse optional flags (can be anywhere in arguments)
VERBOSE_MODE="false"
QUICK_MODE="true"  # Default to quick mode
COMPREHENSIVE_MODE="false"
ARGS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --verbose|-v)
            VERBOSE_MODE="true"
            shift 1
            ;;
        --quick)
            QUICK_MODE="true"
            COMPREHENSIVE_MODE="false"
            shift 1
            ;;
        --comprehensive|--full)
            QUICK_MODE="false"
            COMPREHENSIVE_MODE="true"
            shift 1
            ;;
        *)
            ARGS+=("$1")
            shift 1
            ;;
    esac
done

# Restore non-flag arguments
set -- "${ARGS[@]}"

# Available tests
declare -A TESTS=(
    # Core Cage functionality tests
    ["unit"]="unit_tests"                        # Core unit tests (Rust)
    ["pty"]="pty_automation"                     # PTY automation tests
    ["cli"]="cli_interface"                      # CLI interface tests
    ["integration"]="integration_tests"          # End-to-end integration tests
    ["driver"]="driver_demo"                     # PTY driver demonstration

    # Feature-specific tests
    ["lock"]="lock_operations"                   # File locking/encryption tests
    ["unlock"]="unlock_operations"               # File unlocking/decryption tests
    ["status"]="status_operations"               # Status checking tests
    ["batch"]="batch_operations"                 # Batch processing tests

    # Security and validation tests
    ["security"]="security_validation"           # Security validation tests
    ["error"]="error_handling"                   # Error handling tests
    ["audit"]="audit_logging"                    # Audit logging tests

    # Configuration and adapter tests
    ["config"]="configuration"                   # Configuration management tests
    ["adapters"]="adapter_tests"                 # Age adapter tests

    # Comprehensive suites
    ["all"]="all_cage_tests"                    # Run all cage test categories
    ["smoke"]="smoke_tests"                      # Quick smoke test suite
    ["full"]="comprehensive_suite"               # Full cage validation suite

    # Performance and compatibility
    ["perf"]="performance_tests"                 # Performance benchmarking
    ["cross"]="cross_platform"                  # Cross-platform compatibility
)

show_help() {
    if [[ -n "$BOXY" ]]; then
        cat <<-EOF | $BOXY --theme info --title "🔒 Cage Test Runner" --width max
Available Commands:
  test.sh [--comprehensive|--verbose] run <test>    Run specific test
  test.sh list                                      List available tests
  test.sh help                                      Show this help

Options:
  --comprehensive        Run full validation test suite
  --quick                Force quick mode (default)
  --verbose              Show detailed test output

Available Tests:
  unit                   Core cage unit tests (Rust)
  pty                    PTY automation tests
  cli                    CLI interface tests
  integration            End-to-end cage workflow tests
  driver                 PTY driver demonstration
  lock                   File locking/encryption tests
  unlock                 File unlocking/decryption tests
  status                 Status checking tests
  batch                  Batch processing tests
  security               Security validation tests
  all                    Run all cage test categories
  smoke                  Quick smoke test suite
  full                   Full cage validation suite
EOF
    else
        echo "🔒 CAGE TEST RUNNER"
        echo "=================="
        echo
        echo "Available Commands:"
        echo "  test.sh [--comprehensive|--verbose] run <test>    Run specific test"
        echo "  test.sh list                                      List available tests"
        echo "  test.sh help                                      Show this help"
        echo
        echo "Options:"
        echo "  --comprehensive        Run full validation test suite"
        echo "  --quick                Force quick mode (default)"
        echo "  --verbose              Show detailed test output"
        echo
        echo "Available Tests:"
        echo "  unit                   Core cage unit tests (Rust)"
        echo "  pty                    PTY automation tests"
        echo "  cli                    CLI interface tests"
        echo "  integration            End-to-end cage workflow tests"
        echo "  driver                 PTY driver demonstration"
        echo "  lock                   File locking/encryption tests"
        echo "  unlock                 File unlocking/decryption tests"
        echo "  status                 Status checking tests"
        echo "  batch                  Batch processing tests"
        echo "  security               Security validation tests"
        echo "  all                    Run all cage test categories"
        echo "  smoke                  Quick smoke test suite"
        echo "  full                   Full cage validation suite"
    fi
}

list_tests() {
    if [[ -n "$BOXY" ]]; then
        {
            echo "Available Tests:"
            echo
            for test_name in $(printf "%s\n" "${!TESTS[@]}" | sort); do
                test_file="${TESTS[$test_name]}"
                
                # Special handling for sanity package
                if [[ "$test_name" == "sanity" ]]; then
                    if [[ -f "$TEST_DIR/sanity_main.rs" ]]; then
                        echo "✅ $test_name → sanity_main.rs (core + baseline)"
                    else
                        echo "❌ $test_name → sanity_main.rs (missing)"
                    fi
                elif [[ -f "$TEST_DIR/$test_file.sh" ]]; then
                    echo "✅ $test_name → $test_file.sh"
                elif [[ -f "$TEST_DIR/$test_file" ]]; then
                    echo "✅ $test_name → $test_file"
                elif [[ -f "$TEST_DIR/$test_file.rs" ]]; then
                    echo "✅ $test_name → $test_file.rs"
                else
                    echo "❌ $test_name → $test_file (missing)"
                fi
            done
            echo
            echo "Auto‑discovered wrappers:"
            for wrap in $(find "$TEST_DIR" -maxdepth 1 -type f -name "*.rs" -printf "%f\n" | sort); do
                base="${wrap%.rs}"
                printf "  • %s\n" "$base"
            done
        } | $BOXY --theme info --title "🗂️ Available Cage Tests" --width max
    else
        echo "🗂️ AVAILABLE CAGE TESTS"
        echo "======================"
        for test_name in $(printf "%s\n" "${!TESTS[@]}" | sort); do
            test_file="${TESTS[$test_name]}"
            
            # Special handling for sanity package
            if [[ "$test_name" == "sanity" ]]; then
                if [[ -f "$TEST_DIR/sanity_main.rs" ]]; then
                    echo "✅ $test_name → sanity_main.rs (core + baseline)"
                else
                    echo "❌ $test_name → sanity_main.rs (missing)"
                fi
            elif [[ -f "$TEST_DIR/$test_file.sh" ]]; then
                echo "✅ $test_name → $test_file.sh"
            elif [[ -f "$TEST_DIR/$test_file" ]]; then
                echo "✅ $test_name → $test_file"
            elif [[ -f "$TEST_DIR/$test_file.rs" ]]; then
                echo "✅ $test_name → $test_file.rs"
            else
                echo "❌ $test_name → $test_file (missing)"
            fi
        done
        echo
        echo "Auto‑discovered wrappers:"
        for wrap in $(find "$TEST_DIR" -maxdepth 1 -type f -name "*.rs" -printf "%f\n" | sort); do
            base="${wrap%.rs}"
            echo "  • $base"
        done
    fi
}

run_test() {
    local test_name="$1"
    
    if [[ -z "$test_name" ]]; then
        echo "❌ Error: Test name required"
        echo "Use: test.sh run <test>"
        echo "Available tests: ${!TESTS[*]}"
        exit 1
    fi
    
    if [[ ! "${TESTS[$test_name]+exists}" ]]; then
        # Fallback: run by wrapper filename or shell script name
        if [[ -f "$TEST_DIR/$test_name.rs" ]]; then
            echo "ℹ️  Running auto‑discovered wrapper: $test_name.rs"
            cargo test --test "$test_name" -- --nocapture
            exit 0
        elif [[ -f "$TEST_DIR/sh/$test_name.sh" ]]; then
            echo "ℹ️  Running shell test: tests/sh/$test_name.sh"
            exec bash "$TEST_DIR/sh/$test_name.sh"
        else
            echo "❌ Error: Unknown test '$test_name'"
            echo "Available tests: ${!TESTS[*]}"
            echo "Auto wrappers available:"
            find "$TEST_DIR" -maxdepth 1 -type f -name "*.rs" -printf "  • %f\n" | sed 's/\.rs$//'
            exit 1
        fi
    fi
    
    local test_file="${TESTS[$test_name]}"
    
    # If mapping points to a Rust wrapper (tests/<name>.rs), run as Cargo test
    if [[ "$test_file" == *.rs && -f "$TEST_DIR/$test_file" ]]; then
        local wrapper_name="${test_file%.rs}"
        if [[ -n "$BOXY" ]]; then
            echo "🦀 Running Rust wrapper: $test_file" | $BOXY --theme success --title "🧪 RSB Test Runner" --width max
        else
            echo "🦀 Running Rust wrapper: $test_file"
        fi
        cargo test --test "$wrapper_name" -- --nocapture
        exit 0
    fi
    
    # Header
    if [[ -n "$BOXY" ]]; then
        echo "🚀 Running RSB test: $test_name" | $BOXY --theme success --title "🧪 RSB Test Runner" --width max
    else
        echo "🚀 Running RSB test: $test_name"
        echo "=========================="
    fi
    echo
    
    # Change to project root 
    cd "$ROOT_DIR"
    
    # Export test configuration
    export RSB_TEST_MODE="true"
    export RSB_VERBOSE="${VERBOSE_MODE}"
    export RSB_QUICK_MODE="${QUICK_MODE}"
    export RSB_COMPREHENSIVE="${COMPREHENSIVE_MODE}"
    
    # Handle different test types
    case "$test_name" in
        "all")
            # Run a broad set of tests across categories
            "$0" run unit
            "$0" run pty
            "$0" run cli
            "$0" run driver
            "$0" run lock
            "$0" run unlock
            "$0" run status
            "$0" run security
            ;;
        "smoke")
            # Quick validation: core unit tests + driver demo
            "$0" run unit
            "$0" run driver
            ;;
        "unit")
            # Core cage unit tests
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                echo "🦀 Running cage unit tests with verbose output..."
                cargo test --lib -- --nocapture
            else
                echo "🦀 Running cage unit tests..."
                cargo test --lib
            fi
            ;;
        "driver")
            # PTY driver demonstration
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                echo "🚀 Running PTY driver demo with verbose output..."
                cargo run --bin driver
            else
                echo "🚀 Running PTY driver demo..."
                cargo run --bin driver
            fi
            ;;
        "pty")
            # PTY automation tests
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                echo "🔧 Running PTY automation tests with verbose output..."
                cargo test pty -- --nocapture
            else
                echo "🔧 Running PTY automation tests..."
                cargo test pty
            fi
            ;;
        "cli")
            # CLI interface tests
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                echo "⌨️ Running CLI interface tests with verbose output..."
                cargo test cli -- --nocapture
            else
                echo "⌨️ Running CLI interface tests..."
                cargo test cli
            fi
            ;;
        "integration")
            # End-to-end integration tests (when we have proper ones)
            echo "🔗 Integration tests not yet implemented"
            echo "   Run 'driver' test for manual PTY validation"
            ;;
        "lock")
            # File locking/encryption tests
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                echo "🔐 Running file locking tests with verbose output..."
                cargo test lock -- --nocapture
            else
                echo "🔐 Running file locking tests..."
                cargo test lock
            fi
            ;;
        "unlock")
            # File unlocking/decryption tests
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                echo "🔓 Running file unlocking tests with verbose output..."
                cargo test unlock -- --nocapture
            else
                echo "🔓 Running file unlocking tests..."
                cargo test unlock
            fi
            ;;
        "status")
            # Status checking tests
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                echo "📊 Running status checking tests with verbose output..."
                cargo test status -- --nocapture
            else
                echo "📊 Running status checking tests..."
                cargo test status
            fi
            ;;
        "batch")
            # Batch processing tests
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                echo "📦 Running batch processing tests with verbose output..."
                cargo test batch -- --nocapture
            else
                echo "📦 Running batch processing tests..."
                cargo test batch
            fi
            ;;
        "security")
            # Security validation tests
            if [[ "$VERBOSE_MODE" == "true" ]]; then
                echo "🛡️ Running security validation tests with verbose output..."
                cargo test security -- --nocapture
            else
                echo "🛡️ Running security validation tests..."
                cargo test security
            fi
            ;;
        *)
            # Shell-based tests
            local test_path=""
            
            # Try different file extensions and paths
            if [[ -f "$TEST_DIR/$test_file.sh" ]]; then
                test_path="$TEST_DIR/$test_file.sh"
            elif [[ -f "$TEST_DIR/$test_file" ]]; then
                # If this is a Rust wrapper but we didn't catch it above, run via cargo
                if [[ "$test_file" == *.rs ]]; then
                    local wrapper_name="${test_file%.rs}"
                    cargo test --test "$wrapper_name" -- --nocapture
                    exit 0
                fi
                test_path="$TEST_DIR/$test_file"
            elif [[ -f "$TEST_DIR/sh/$test_file.sh" ]]; then
                test_path="$TEST_DIR/sh/$test_file.sh"
            else
                echo "❌ Error: Test file not found for '$test_name'"
                echo "    Checked: $TEST_DIR/$test_file.sh"
                echo "    Checked: $TEST_DIR/$test_file"
                echo "    Checked: $TEST_DIR/sh/$test_file.sh"
                exit 1
            fi
            
            echo "📜 Executing shell test: $test_path"
            exec bash "$test_path"
            ;;
    esac
}

# Main command dispatch
case "${1:-help}" in
    "run")
        run_test "$2"
        ;;
    "list")
        list_tests
        ;;
    "help"|"--help"|"-h")
        show_help
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo "Use: test.sh help"
        exit 1
        ;;
esac
