# 🔒 Cage

**Encryption Automation CLI**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Multi-License](https://img.shields.io/badge/multi--license-model-blue.svg)](./docs/lics/LICENSE_OVERVIEW.txt)
[![Version](https://img.shields.io/badge/version-0.3.1-green.svg)](Cargo.toml)

Cage provides bulletproof encryption automation tools while maintaining cryptographic security standards. Features production-grade PTY automation with comprehensive error handling and security validation.

> Formerly in the Oxidex Framework repos (padlock), spun off into its own repo for better organization

## ✨ Features

- **🛡️ PTY Automation** - Native PTY wrapper for seamless Age encryption without manual interaction
- **🔐 Complete CRUD Operations** - Lock, unlock, status, rotate, verify, and batch operations
- **📄 ASCII Armor Support** - Optional text-safe encryption format for email/text transmission
- **🚀 Batch Processing** - High-performance parallel operations on multiple files
- **🔍 Security Validation** - Comprehensive injection prevention and audit logging
- **⚙️ RSB Framework** - Modern CLI architecture with built-in help, inspection, and global context
- **🏷️ Smart Extensions** - Uses `.cage` extension by default, configurable for integration
- **🔗 Age Proxy** - Direct Age binary access with PTY automation for any Age command
- **💡 Interactive Passphrases** - Secure terminal input with multiple input modes
- **🛡️ In-Place Safety** - Multi-layered protection for in-place file operations with recovery mechanisms
- **📊 Progress Indicators** - Professional progress bars, spinners, and telemetry for long operations
- **🖥️ Cross-Platform** - Linux, macOS support with Windows compatibility planned

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/padlokk/cage.git
cd cage

# Build and install
./bin/build.sh
./bin/deploy.sh

# Verify installation
cage --version
```

### Basic Usage

```bash
# Encrypt a file (RSB framework - streamlined syntax)
cage lock secret.txt mysecretpassword

# Decrypt a file (note: cage uses .cage extension by default)
cage unlock secret.txt.cage mysecretpassword

# In-place encryption with safety checks
cage lock secret.txt --in-place --passphrase mysecretpassword

# Operations with progress indicators
cage lock /large-directory --recursive --progress --passphrase secret

# Check encryption status
cage status /path/to/files

# Rotate encryption keys
cage rotate /documents --old-passphrase "old" --new-passphrase "new"

# Batch encrypt directory
cage batch /documents --operation lock --passphrase secret

# Built-in RSB commands
cage help           # Show help with enhanced formatting
cage inspect        # List all available functions
cage test --progress-demo  # Demo progress indicators
```

## 🔧 RSB Framework Integration

Cage now uses the **RSB (Rust Shell Bridge) framework** for enhanced CLI architecture:

### Enhanced Features
- **Built-in Commands**: `help`, `inspect`, `stack` for debugging and exploration
- **Global Context**: Unified state management across all operations
- **Streamlined Syntax**: Reduced boilerplate with intuitive argument parsing
- **Advanced Debugging**: Function registry and call stack inspection

### Architecture Benefits
- **90% Code Reduction**: From 500+ lines of CLI boilerplate to ~50 lines
- **Better Performance**: Faster compilation without clap dependency
- **Enhanced UX**: Colored output, improved help formatting, auto-completion ready

### Framework Commands
```bash
cage help           # Enhanced help with colored formatting
cage inspect        # Show all registered functions
cage stack          # Display current call stack for debugging
```

## 📦 Installation

### Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Age binary** - Automatically installed by build script or install manually:
  - Ubuntu/Debian: `sudo apt-get install age`
  - RHEL/CentOS: `sudo yum install age`
  - Arch Linux: `sudo pacman -S age`
  - macOS: `brew install age`

### Build from Source

```bash
# Clone repository
git clone https://github.com/padlokk/cage.git
cd cage

# Build with automatic age installation
./bin/build.sh

# Deploy to ~/.local/bin/cage
./bin/deploy.sh

# Run tests
cargo test
```

### Manual Installation

```bash
# Build release binary
cargo build --release

# Install binary
cp target/release/cage ~/.local/bin/

# Or system-wide
sudo cp target/release/cage /usr/local/bin/
```

## 📚 Library Usage

Cage can be used as a Rust library for integrating Age encryption automation into your own applications.

### Cargo.toml

```toml
[dependencies]
cage = { path = "path/to/cage" }
```

### Basic Library Usage

```rust
use cage::cage::{
    CrudManager, LockOptions, UnlockOptions,
    OutputFormat, PassphraseManager, PassphraseMode
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize CRUD manager
    let mut crud_manager = CrudManager::with_defaults()?;

    // Create lock options
    let options = LockOptions {
        recursive: false,
        pattern: None,
        backup: false,
        format: OutputFormat::Binary,
        preserve_encrypted: false,
        audit_log: None,
    };

    // Encrypt a file
    let result = crud_manager.lock(
        &std::path::Path::new("secret.txt"),
        "mypassword",
        options
    )?;

    println!("Encrypted {} files", result.processed_files.len());
    Ok(())
}
```

### Progress Integration

```rust
use cage::cage::progress::{ProgressManager, ProgressStyle, TerminalReporter};
use std::sync::Arc;

let manager = ProgressManager::new();
manager.add_reporter(Arc::new(TerminalReporter::new()));

let task = manager.start_task("Encrypting files", ProgressStyle::Bar { total: 10 });
for i in 0..10 {
    task.update(i + 1, &format!("Processing file {}", i + 1));
    // Your encryption work here
}
task.complete("All files encrypted");
```

### In-Place Operations

```rust
use cage::cage::{SafetyValidator, InPlaceOperation};

// Safety validation
let safety_validator = SafetyValidator::new(false, false)?;
safety_validator.validate_in_place_operation(&path)?;

// Create in-place operation
let mut in_place_op = InPlaceOperation::new(&path);

// Execute with recovery
in_place_op.execute(|| {
    crud_manager.lock(&path, passphrase, options)
})?;
```

### PTY Automation

```rust
use cage::cage::pty_wrap::PtyAgeAutomator;

let automator = PtyAgeAutomator::new()?;
let result = automator.execute_age_command(
    &["--encrypt", "--passphrase", "input.txt"],
    Some("mypassword"),
    30000  // 30 second timeout
)?;
```

### Available Modules

- **`cage::cage::CrudManager`** - Core file encryption/decryption operations
- **`cage::cage::progress`** - Progress reporting framework
- **`cage::cage::pty_wrap`** - PTY automation for Age binary
- **`cage::cage::SafetyValidator`** - In-place operation safety checks
- **`cage::cage::InPlaceOperation`** - Atomic in-place file operations
- **`cage::cage::PassphraseManager`** - Secure passphrase handling

📖 **[Complete Library Documentation](docs/LIBRARY_USAGE.md)** - Comprehensive API guide with examples

## 🔧 Usage

### Command Overview

| Command | Description | Status |
|---------|-------------|--------|
| `lock` | Encrypt files/directories | ✅ Fully Implemented |
| `unlock` | Decrypt files/directories | ✅ Fully Implemented |
| `status` | Check encryption status | ✅ Fully Implemented |
| `rotate` | Rotate encryption keys | ✅ Fully Implemented |
| `verify` | Verify file integrity | ✅ Fully Implemented |
| `batch` | Bulk operations | ✅ Fully Implemented |
| `proxy` | Direct Age commands with PTY | ✅ Fully Implemented |
| `test` | Run test suite & demos | ✅ Fully Implemented |
| `demo` | Show demonstrations | ✅ Fully Implemented |

### Detailed Examples

#### File Encryption

```bash
# Basic encryption
cage lock document.pdf --passphrase "strongpassword"

# ASCII armor format (text-safe)
cage --format ascii lock document.pdf --passphrase "strongpassword"

# Recursive directory encryption
cage lock /sensitive-docs --recursive --passphrase "strongpassword"

# Pattern-based encryption
cage lock /logs --recursive --pattern "*.log" --passphrase "strongpassword"

# With backup creation
cage lock important.txt --backup --passphrase "strongpassword"

# In-place encryption (overwrites original with recovery file)
cage lock document.pdf --in-place --passphrase "strongpassword"

# In-place with danger mode (no recovery file)
DANGER_MODE=1 cage lock document.pdf --in-place --danger-mode --passphrase "strongpassword"

# Show progress for long operations
cage lock /large-directory --recursive --progress --passphrase "strongpassword"
```

#### File Decryption

```bash
# Basic decryption (cage uses .cage extension by default)
cage unlock document.pdf.cage --passphrase "strongpassword"

# Interactive passphrase input (secure terminal input)
cage unlock document.pdf.cage

# Stdin passphrase input (for automation)
echo "mypassword" | cage unlock document.pdf.cage --stdin-passphrase

# Preserve encrypted files after decryption
cage unlock document.pdf.cage --preserve --passphrase "strongpassword"

# Selective decryption with patterns
cage unlock /encrypted-docs --pattern "*.txt.cage" --passphrase "strongpassword"
```

#### Status and Management

```bash
# Check current directory status
cage status

# Check specific path
cage status /encrypted-files

# Verbose status with details
cage --verbose status /encrypted-files

# Verify integrity (coming soon)
cage verify /encrypted-files
```

#### Batch Operations

```bash
# Bulk encrypt directory
cage batch /documents --operation lock --passphrase "secret"

# Bulk decrypt with pattern
cage batch /encrypted --operation unlock --pattern "*.age" --passphrase "secret"

# With audit logging
cage --audit-log /var/log/cage.log batch /docs --operation lock --passphrase "secret"
```

#### Age Proxy Commands

```bash
# Direct Age encryption with PTY automation
cage proxy --age-p --age-o=/tmp/output.cage input.txt

# Direct Age decryption
cage proxy --age-d --age-o=/tmp/decrypted.txt encrypted.cage

# Age with custom identity files
cage proxy --age-d --age-i=private.key encrypted.cage

# ASCII armor encryption
cage proxy --age-p --age-a --age-o=output.asc input.txt

# Pass passphrase via stdin for automation
echo "secret" | cage proxy --age-p --age-o=output.cage --stdin-passphrase input.txt
```

### Configuration Options

#### Global Flags

- `--verbose, -v` - Show detailed operation progress
- `--progress` - Display professional progress indicators for long operations
- `--audit-log <PATH>` - Write audit log for security compliance
- `--format <FORMAT>` - Encryption format: `binary` (default) or `ascii`

#### In-Place Operation Flags

- `--in-place` - Encrypt/decrypt files in-place (overwrites original)
- `--danger-mode` - Skip recovery file creation (requires DANGER_MODE=1 env var)
- `--i-am-sure` - Automation override for scripted operations

#### Output Formats

- **Binary** (default) - Compact binary format
- **ASCII Armor** - Text-safe format for email/messaging

## 🏗️ Architecture

### Core Components

```
cage/
├── src/
│   ├── lib.rs              # Library exports and public API
│   ├── bin/cli_age.rs      # CLI application entry point
│   └── cage/               # Core library modules
│       ├── mod.rs          # Module exports
│       ├── adapter.rs      # Age backend adapters
│       ├── pty_wrap.rs     # PTY automation core
│       ├── lifecycle/      # CRUD operations
│       ├── operations/     # File/repository operations
│       ├── security.rs     # Security validation
│       ├── error.rs        # Comprehensive error handling
│       └── config.rs       # Configuration management
```

### PTY Automation

Cage uses advanced PTY (pseudo-terminal) automation to interact with the Age binary seamlessly:

```rust
// Example PTY automation (simplified)
let pty_system = native_pty_system();
let pair = pty_system.openpty(pty_size)?;
let child = pair.slave.spawn_command(age_command)?;

// Automated passphrase handling
if output.contains("passphrase") {
    writer.write_all(passphrase.as_bytes())?;
    writer.write_all(b"\n")?;
}
```

### Security Model

- **Input Validation** - All inputs validated against injection attacks
- **Audit Logging** - Complete operation audit trail
- **Secure Defaults** - Security-first configuration
- **Error Handling** - Comprehensive error reporting with guidance

## 🧪 Development

### Project Structure

```bash
cage/
├── Cargo.toml           # Rust dependencies and metadata
├── README.md           # This file
├── ROADMAP.md          # Development roadmap
├── src/                # Source code
├── tests/              # Integration tests
├── docs/               # Documentation
└── bin/                # Build and deployment scripts
    ├── build.sh        # Build with age auto-install
    └── deploy.sh       # Deploy to system
```

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with verbose output
cargo run -- --verbose demo
```

### Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test '*'

# Test specific functionality
cargo test pty_automation

# Manual PTY test
cargo run --bin driver
```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Run the test suite (`cargo test`)
5. Commit your changes (`git commit -am 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

See [ROADMAP.md](ROADMAP.md) for current development priorities.

## 📚 Documentation

### Library & API Documentation
- **[📖 Library Usage Guide](docs/LIBRARY_USAGE.md)** - Comprehensive Rust library documentation with examples
- **[🏗️ ROADMAP.md](docs/procs/ROADMAP.md)** - Development roadmap and feature status
- **[📋 TASKS.txt](docs/procs/TASKS.txt)** - Detailed task list and development plan

### Feature Documentation
- **[🛡️ Safety Design](docs/ref/SAFETY_DESIGN.md)** - In-place operation safety architecture
- **[📊 Progress Framework](docs/features/FEATURES_PROGRESS.md)** - Progress reporting system details
- **[⚙️ RSB Framework](docs/rsb/RSB_ARCH.md)** - RSB framework architecture and usage

### API Documentation

```bash
# Generate and view complete API docs
cargo doc --open

# Quick library examples
cargo run --example basic_encryption
cargo run --example progress_demo
```

## 🔐 Security

### Security Features

- **Command Injection Prevention** - All inputs sanitized
- **Path Traversal Protection** - File paths validated
- **Audit Logging** - Complete operation trail
- **Secure Defaults** - Security-first configuration

### Reporting Security Issues

Please report security vulnerabilities via private channels:
- Email: security@cage-project.org (if available)
- GitHub Security Advisories (preferred)

## 🤝 Dependencies

### Runtime Dependencies

- **[age](https://github.com/FiloSottile/age)** - Age encryption tool (auto-installed)
- **[rsb](https://github.com/oodx/rsb-framework)** - RSB framework for enhanced CLI experience

### Rust Dependencies

- `portable-pty = "0.9"` - PTY automation core
- `clap = "4.4"` - CLI parsing with derive macros
- `tempfile = "3.8"` - Temporary file handling
- `chrono = "0.4"` - Timestamp management
- `serde = "1.0"` - Serialization support
- `thiserror = "2"` - Error handling macros

## 🚧 Current Status

**Version:** 0.3.1
**Production Readiness:** 85%
**Development Phase:** Feature Complete (Phase 2)

### ✅ Implemented Features

- Complete CLI interface with comprehensive help
- PTY automation for Age binary interaction
- File/directory encryption and decryption (`.cage` extension by default)
- Interactive passphrase prompting with secure terminal input
- Age proxy commands for direct Age binary access
- Status checking and reporting
- Batch operations with pattern matching
- Security validation and audit logging
- RSB framework integration
- Configurable file extensions for library integration
- **In-place operations** with multi-layered safety architecture
- **Progress indicators** with professional terminal output
- **Recovery mechanisms** for in-place operations
- **Comprehensive telemetry** and progress hooks

### ⚠️ In Development

- Configuration file support
- Windows compatibility
- RageAdapter implementation

See [docs/procs/TASKS.txt](docs/procs/TASKS.txt) for detailed development plan.

## 🧪 Testing

Cage has **comprehensive test coverage** across multiple categories:

### Test Statistics
- **64 Total Tests** across 6 test suites
- **38 Unit Tests** - Core library functionality validation
- **12 RSB Integration Tests** - Complete framework compatibility validation
- **5 PTY Tests** - Pseudo-terminal automation testing
- **7 Integration Tests** - End-to-end functionality verification
- **2 CLI Tests** - Command-line interface validation
- **1 Doc Test** - Documentation code example verification

### Running Tests
```bash
# Run all tests
cargo test --all

# Run specific test categories
cargo test --lib                    # Unit tests only
cargo test --test rsb_integration   # RSB framework tests
cargo test --test pty_test         # PTY automation tests
cargo test --bin cage             # CLI tests

# Run tests with output
cargo test -- --nocapture
```

### Test Coverage Areas
- ✅ **Core Operations** - Lock, unlock, status, rotate, verify, batch
- ✅ **Security Validation** - Injection prevention, audit logging
- ✅ **PTY Automation** - Terminal interaction without manual input
- ✅ **RSB Framework** - Bootstrap, dispatch, global context, Args wrapper
- ✅ **Error Handling** - Comprehensive error scenarios
- ✅ **File Operations** - Encryption format detection, backup systems

## 🙏 Dependencies

- **RSB Framework** - Enhanced CLI utilities and patterns
- **[FiloSottile/age](https://github.com/FiloSottile/age)** 
- **[portable-pty](https://crates.io/crates/portable-pty)** 

---

**Built with ❤️ and 🔒 for secure, automated encryption workflows**

## License

RSB Framework, Oxidex (ODX), and REBEL libraries, services, and software are offered under a **multi-license model**:

| License | Who it’s for | Obligations |
|---------|--------------|-------------|
| [AGPL-3.0](./LICENSE) | Open-source projects that agree to release their own source code under the AGPL. | Must comply with the AGPL for any distribution or network service. |
| [Community Edition License](./docs/LICENSE_COMMUNITY.txt) | Personal, educational, or academic use **only**. Not for companies, organizations, or anyone acting for the benefit of a business. | Must meet all CE eligibility requirements and follow its terms. |
| [Commercial License](./docs/LICENSE_COMMERCIAL.txt) | Companies, contractors, or anyone needing to embed the software in closed-source, SaaS, or other commercial products. | Requires a signed commercial agreement with Dr. Vegajunk Hackware. |

By **downloading, installing, linking to, or otherwise using RSB Framework, Oxidex, or REBEL**, you:

1. **Accept** the terms of one of the licenses above, **and**  
2. **Represent that you meet all eligibility requirements** for the license you have chosen.

> Questions about eligibility or commercial licensing: **licensing@vegajunk.com**
