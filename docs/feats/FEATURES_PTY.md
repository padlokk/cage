# Cage PTY Module (Terminal Automation for Age CLI)

Updated: 2025-10-01

## Purpose
- Provide reliable, automated interaction with the Age CLI through pseudo-terminal emulation
- Enable non-interactive encryption/decryption operations that require TTY interaction
- Support multiple automation strategies for maximum compatibility and reliability
- Eliminate manual passphrase entry while maintaining security

## Feature Flags
- `pty` — PTY automation module for Age CLI
  - Enables automated terminal interaction
  - Default: Enabled

## Imports
```rust
use cage::cage::pty::{
    PtyAgeAutomator,
    TtyAutomator,
};
```

## Core API
### Types
- `PtyAgeAutomator` — Primary PTY-based automation using portable-pty library
- `TtyAutomator` — Alternative TTY automation using script/expect methods

### PtyAgeAutomator (Primary Method)
#### Key Methods
- `new()` — Create automator with default configuration
- `with_config(config)` — Create automator with custom config
- `encrypt(input, output, passphrase, format)` — Encrypt file using PTY automation
- `decrypt(input, output, passphrase)` — Decrypt file using PTY automation
- `check_age_binary()` — Verify Age CLI is available
- `perform_health_check()` — Comprehensive system validation
- `execute_age_command(args, passphrase)` — Execute arbitrary Age commands
- `available_methods()` — Get list of available automation methods
- `validate_dependencies()` — Validate all required dependencies

### TtyAutomator (Alternative Method)
#### Key Methods
- `new()` — Create TTY automator
- `encrypt(input, output, passphrase, format)` — Encrypt using script/expect methods
- `decrypt(input, output, passphrase)` — Decrypt using script/expect methods
- `check_age_binary()` — Verify Age CLI is available
- `check_automation_methods()` — Verify script/expect availability
- `perform_health_check()` — Comprehensive system validation
- `available_methods()` — Get list of available methods (script/expect)
- `validate_dependencies()` — Validate all required dependencies

## Patterns
- PTY-based terminal emulation for primary automation
- Fallback to script/expect methods for alternative automation
- Secure passphrase handling with no process list exposure
- Timeout protection against hanging processes
- Comprehensive error handling and reporting

## Examples
```rust
// Basic PTY automation
use cage::cage::pty::PtyAgeAutomator;
use cage::cage::config::OutputFormat;
use std::path::Path;

fn example_pty_automation() -> cage::AgeResult<()> {
    // Create PTY automator
    let automator = PtyAgeAutomator::new()?;

    // Encrypt a file
    automator.encrypt(
        Path::new("input.txt"),
        Path::new("encrypted.age"),
        "secure_passphrase",
        OutputFormat::Binary
    )?;

    // Decrypt the file
    automator.decrypt(
        Path::new("encrypted.age"),
        Path::new("decrypted.txt"),
        "secure_passphrase"
    )?;

    Ok(())
}

// TTY automation with fallback
use cage::cage::pty::TtyAutomator;

fn example_tty_automation() -> cage::AgeResult<()> {
    let automator = TtyAutomator::new()?;

    // Verify dependencies
    automator.validate_dependencies()?;

    // Encrypt using script/expect methods
    automator.encrypt(
        Path::new("input.txt"),
        Path::new("encrypted.age"),
        "secure_passphrase",
        OutputFormat::AsciiArmor
    )?;

    Ok(())
}

// Health check before operations
fn example_health_check() -> cage::AgeResult<()> {
    let automator = PtyAgeAutomator::new()?;

    // Comprehensive validation
    automator.perform_health_check()?;

    println!("PTY automation ready!");
    Ok(())
}
```

## Integration
- Core dependency for Age CLI operations in Cage
- Integrated with adapter layer (v1 and v2 adapters)
- Used by CageManager for lifecycle coordination
- Supports both library and CLI usage patterns
- Dependency: Age CLI binary required for all operations
- Optional: script and expect commands for TTY automation fallback

## Testing
- Unit tests located in module test sections
- Integration tests in `tests/pty_test.rs`
- Comprehensive PTY automation tests
- Health check validation tests
- Coverage expectations: >85%
- Special considerations:
  - Tests may be skipped if Age CLI not installed
  - PTY tests require terminal capabilities
  - Script/expect tests require system commands

## Performance Characteristics
- Low latency for PTY operations (<100ms overhead)
- Minimal memory footprint
- Thread-safe automation with timeouts
- Non-blocking I/O where possible
- Efficient passphrase handling
- Configurable timeout protection (default: 120s)

## Security Features
- No passphrase exposure in process lists
- Secure temporary file handling with automatic cleanup
- Timeout protection against hanging processes
- Process isolation through PTY abstraction
- Comprehensive error reporting without leaking sensitive data
- Audit trail integration for all operations

## Limitations
- Requires Age CLI binary to be installed and in PATH
- PTY automation requires terminal capabilities on the system
- TTY automation requires script/expect commands (optional fallback)
- Platform-specific PTY behavior may vary (handled by portable-pty)
- Cannot automate operations that require human decision-making

## Technical Details
### PTY Automation (wrap.rs)
- Uses `portable-pty` library from Hub's terminal-ext module
- Creates proper pseudo-terminal for Age CLI interaction
- Handles prompt detection and response automation
- Supports both encryption (double prompt) and decryption (single prompt)
- Thread-based automation with timeout protection
- Comprehensive error handling and reporting

### TTY Automation (tty.rs)
- Uses system `script` command for TTY allocation
- Uses `expect` command for interactive automation
- Fallback methods proven in pilot testing
- Eliminated T2.1: TTY Automation Subversion threat
- Automatic method selection based on availability

## Status
- MODERN: Yes
  - Clean module organization
  - Multiple automation strategies
  - Comprehensive API
  - Well-tested and documented
- SPEC_ALIGNED: Yes
  - Follows RSB module patterns
  - Integrated with Hub ecosystem
  - Consistent with project architecture
  - Matches adapter consolidation approach (MOD4-01)

## Changelog
- 2025-10-01: Module consolidation (MOD4-02)
  - Created `src/cage/pty/` directory structure
  - Moved `pty_wrap.rs` → `pty/wrap.rs`
  - Moved `tty_automation.rs` → `pty/tty.rs`
  - Updated all imports across codebase
  - Created comprehensive module documentation
  - All tests passing (68 passed, 2 ignored)

## References
- Age Encryption CLI: https://github.com/FiloSottile/age
- Portable PTY: Hub's terminal-ext module
- RSB Framework: Module organization patterns
- Related: Cage Adapter Module (FEATURES_ADP.md)
- Related: Cage PTY Technical Details (docs/misc/CAGE_PTY_FIX.md)

## Threat Elimination
This module successfully eliminated **T2.1: TTY Automation Subversion** through:
- Proven PTY automation patterns
- Multiple fallback methods
- Comprehensive testing and validation
- Security-focused implementation

---

_Generated by Cage Feature Documentation Generator_

<!-- feat:pty -->

_Generated by bin/feat2.py --update-doc._

* `src/pty/mod.rs`
  - pub use tty::TtyAutomator (line 36)
  - pub use wrap::PtyAgeAutomator (line 37)

* `src/pty/tty.rs`
  - struct TtyAutomator (line 19)
  - fn new (line 29)
  - fn encrypt (line 44)
  - fn decrypt (line 79)
  - fn check_age_binary (line 334)
  - fn check_automation_methods (line 347)
  - fn perform_health_check (line 375)
  - fn available_methods (line 384)
  - fn validate_dependencies (line 409)

* `src/pty/wrap.rs`
  - struct PtyAgeAutomator (line 19)
  - fn new (line 27)
  - fn with_config (line 32)
  - fn encrypt (line 50)
  - fn decrypt (line 311)
  - fn check_age_binary (line 516)
  - fn perform_health_check (line 550)
  - fn execute_age_command (line 611)
  - fn available_methods (line 815)
  - fn validate_dependencies (line 820)

<!-- /feat:pty -->

