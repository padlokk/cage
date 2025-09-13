# Padlock Test Suite

This directory contains the comprehensive test suite for the Padlock Age automation and authority chain systems.

## 🚀 Quick Start

**Recommended**: Use the user-friendly test entry points in `bin/`:

```bash
# Run all tests with user-friendly interface
./bin/test.sh

# Quick development feedback (< 30 seconds)
./bin/quick-test.sh

# Test specific modules using CLI tools
./bin/test.sh --cli-age      # Age automation module direct testing
./bin/test.sh --cli-auth     # Authority chain module direct testing

# Comprehensive CLI testing
./bin/test.sh --cli --verbose
```

## Test Structure

```
tests/
├── cli/                    # CLI interface tests (expanded)
│   ├── test_cli_age_direct.sh        # Age automation module direct testing
│   ├── test_cli_auth_direct.sh       # Authority chain module direct testing  
│   ├── test_cli_comprehensive.sh     # Complete CLI functionality testing
│   └── test_workflow_integration.sh  # Generate→Status workflow validation
├── integration/            # Integration and end-to-end tests  
│   ├── test_age_basic.rs              # Basic Age integration validation
│   ├── test_authority_operations_direct.rs  # Direct authority operations
│   ├── test_authority_workflow_simple.rs    # Authority workflow testing
│   └── test_end_to_end_workflow.rs          # Complete E2E workflow
├── comprehensive_api_tests.rs         # Complete API test coverage
└── run_all_tests.sh                   # Comprehensive backend (used by bin/test.sh)
```

## User-Friendly Test Entry Points

### Main Interface (`bin/test.sh`)
```bash
# Selective testing
./bin/test.sh --unit              # Unit tests only
./bin/test.sh --cli               # All CLI tests  
./bin/test.sh --cli-age           # cli_age direct module tests
./bin/test.sh --cli-auth          # cli_auth direct module tests
./bin/test.sh --integration       # Integration tests only

# Output control
./bin/test.sh --verbose           # Detailed output
./bin/test.sh --quiet             # Minimal output
./bin/test.sh --ci                # CI/CD mode

# Development workflows
./bin/test.sh --quick --verbose   # Fast feedback with details
./bin/test.sh --all               # Everything (default)
```

### Quick Development (`bin/quick-test.sh`)
Ultra-fast feedback for development cycles:
- Compile check
- Clippy validation  
- Fast unit tests (< 15s timeout)
- Binary build validation

### CI/CD Integration (`bin/ci-test.sh`)  
Optimized for automated pipelines:
- Machine-readable output
- Comprehensive reporting
- Artifact generation
- Timeout management

## Running Tests

### Unit and API Tests
```bash
# Run all Rust unit tests
cargo test

# Run specific test file
cargo test --test comprehensive_api_tests
```

### CLI Tests
```bash
# Run complete CLI functionality tests
./tests/cli/test_cli_comprehensive.sh

# Run workflow integration tests
./tests/cli/test_workflow_integration.sh
```

### Integration Tests
```bash
# Build and run specific integration test
rustc tests/integration/test_age_basic.rs && ./test_age_basic

# Run authority operations test
rustc tests/integration/test_authority_operations_direct.rs --extern padlock=target/debug/libpadlock.rlib -L target/debug/deps && ./test_authority_operations_direct
```

### All Tests
```bash
# Run everything (requires Age binaries installed)
cargo test && ./tests/cli/test_cli_comprehensive.sh
```

## ⚠️ Known Issues

### TTY Automation Timeout Issue
**Status**: 🚨 CRITICAL - Identified and Mitigated  
**Symptom**: CLI tools (`cli_age`, `cli_auth`) hang indefinitely on encrypt/decrypt operations  
**Root Cause**: Age TTY subversion pattern lacks timeout configuration  
**Mitigation**: 5-second timeouts implemented in test suite  
**Impact**: Test suite protected, but TTY automation needs timeout fixes  

```bash
# Commands that currently timeout (expected behavior):
./bin/test.sh --cli-age   # Times out on lock/unlock interface tests
./bin/test.sh --cli-auth  # Times out on encrypt/decrypt interface tests

# Safe commands (work properly):
./bin/test.sh --unit      # Unit tests work fine
./bin/quick-test.sh       # Quick tests avoid TTY operations
```

**Next Steps**: TTY automation timeout configuration needed in `src/encryption/age_automation/tty_automation.rs`

## Test Categories

### 1. Unit Tests (`comprehensive_api_tests.rs`)
- Complete API coverage for all modules
- Authority chain functionality
- Age automation components  
- Ignition key workflows
- Security and validation tests

### 2. CLI Tests (`cli/`) - Direct Module Testing
The CLI tests validate the `cli_age` and `cli_auth` tools, which serve as **direct interfaces to the two key modules**:

#### CLI Age (`cli_age`) - Age Automation Module Interface
- **Purpose**: Direct access to Age automation system for testing/debugging
- **Testing**: Interface validation, status operations, format options
- **Commands**: `lock`, `unlock`, `status`, `verify`, `demo`, etc.
- **Issue**: Currently times out on encrypt/decrypt due to TTY automation hanging

#### CLI Auth (`cli_auth`) - Authority Chain Module Interface  
- **Purpose**: Direct access to X->M->R->I->D authority chain for testing/debugging
- **Testing**: Key generation, authority operations, ignition workflows
- **Commands**: `generate`, `encrypt`, `decrypt`, `status`, `ignition`, etc.
- **Issue**: Currently times out on authority operations due to TTY automation hanging

#### Comprehensive CLI Testing
- **Direct Module Tests**: `test_cli_age_direct.sh`, `test_cli_auth_direct.sh`
- **Integration Tests**: `test_cli_comprehensive.sh`, `test_workflow_integration.sh`
- **Regression Testing**: Flag changes and functionality preservation
- **Consistency Testing**: Flag patterns and user experience

### 3. Integration Tests (`integration/`)
- **Age Basic**: Core Age encryption/decryption functionality
- **Authority Operations**: Direct authority chain operations testing
- **Workflow Simple**: Authority workflow validation
- **End-to-End**: Complete system integration testing

## Prerequisites

### Required
- Rust/Cargo toolchain
- `age` and `age-keygen` binaries installed

### Installation
```bash
# On Ubuntu/Debian
sudo apt install age

# On macOS
brew install age

# Or install from source: https://github.com/FiloSottile/age
```

## Test Results

All tests should pass for a production-ready system:

```
✅ Unit Tests: PASSED  
✅ CLI Tests: PASSED
✅ Integration Tests: PASSED
✅ Regression Tests: PASSED
```

## Troubleshooting

### Common Issues
1. **Age binary not found**: Install `age` and `age-keygen`
2. **Permission denied**: Ensure test scripts are executable (`chmod +x`)  
3. **Compilation errors**: Ensure `cargo build` succeeds first
4. **TTY automation tests**: Some tests may require actual Age binary interaction

### Debug Mode
Add `--verbose` flag to CLI tests for detailed output:
```bash
./tests/cli/test_cli_comprehensive.sh --verbose
```