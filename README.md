# 🔒 Cage

**Encryption Automation CLI**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Multi-License](https://img.shields.io/badge/multi--license-model-blue.svg)](./docs/lics/LICENSE_OVERVIEW.txt)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](Cargo.toml)

Cage provides bulletproof encryption automation tools while maintaining cryptographic security standards. Features production-grade PTY automation with comprehensive error handling and security validation.

> Formerly in the Oxidex Framework repos (padlock), spun off into its own repo for better organization

## ✨ Features

- **🛡️ PTY Automation** - Native PTY wrapper for seamless Age encryption without manual interaction
- **🔐 Complete CRUD Operations** - Lock, unlock, status, rotate, verify, and batch operations
- **📄 ASCII Armor Support** - Optional text-safe encryption format for email/text transmission
- **🚀 Batch Processing** - High-performance parallel operations on multiple files
- **🔍 Security Validation** - Comprehensive injection prevention and audit logging
- **⚙️ RSB Integration** - Enhanced with RSB framework utilities for better CLI experience
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
# Encrypt a file
cage lock secret.txt --passphrase "mysecretpassword"

# Decrypt a file
cage unlock secret.txt.age --passphrase "mysecretpassword"

# Check encryption status
cage status /path/to/files

# Batch encrypt directory
cage batch /documents --operation lock --passphrase "secret" --recursive

# Show all capabilities
cage demo
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

## 🔧 Usage

### Command Overview

| Command | Description | Status |
|---------|-------------|--------|
| `lock` | Encrypt files/directories | ✅ Fully Implemented |
| `unlock` | Decrypt files/directories | ✅ Fully Implemented |
| `status` | Check encryption status | ✅ Fully Implemented |
| `rotate` | Rotate encryption keys | ⚠️ In Development |
| `verify` | Verify file integrity | ⚠️ In Development |
| `batch` | Bulk operations | ✅ Fully Implemented |
| `test` | Run test suite | ⚠️ In Development |
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
```

#### File Decryption

```bash
# Basic decryption
cage unlock document.pdf.age --passphrase "strongpassword"

# Preserve encrypted files after decryption
cage unlock document.pdf.age --preserve --passphrase "strongpassword"

# Selective decryption with patterns
cage unlock /encrypted-docs --pattern "*.txt.age" --passphrase "strongpassword"
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

### Configuration Options

#### Global Flags

- `--verbose, -v` - Show detailed operation progress
- `--audit-log <PATH>` - Write audit log for security compliance
- `--format <FORMAT>` - Encryption format: `binary` (default) or `ascii`

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

- **[ROADMAP.md](ROADMAP.md)** - Development roadmap and feature status
- **[Cargo.toml](Cargo.toml)** - Dependencies and project metadata
- **Inline Documentation** - Comprehensive code documentation

### API Documentation

```bash
# Generate and view API docs
cargo doc --open
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

**Version:** 0.1.0
**Production Readiness:** 60%
**Development Phase:** MVP (Phase 1)

### ✅ Implemented Features

- Complete CLI interface with comprehensive help
- PTY automation for Age binary interaction
- File/directory encryption and decryption
- Status checking and reporting
- Batch operations with pattern matching
- Security validation and audit logging
- RSB framework integration

### ⚠️ In Development

- Key rotation functionality
- File integrity verification
- Backup system implementation
- Enhanced test coverage
- Windows compatibility

See [ROADMAP.md](ROADMAP.md) for detailed development plan.


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
