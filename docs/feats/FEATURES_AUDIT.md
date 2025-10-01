# Cage Audit Module (Security Logging and Validation)

Updated: 2025-10-01

## Purpose
- Provide comprehensive audit logging for all Age encryption operations
- Enable security validation and injection prevention
- Support structured telemetry with JSON and text formats
- Facilitate compliance and security monitoring through detailed event tracking

## Feature Flags
- `audit` — Audit logging and security validation module
  - Enables comprehensive security event tracking
  - Default: Enabled

## Imports
```rust
use cage::audit::{
    AuditLogger,
    SecurityValidator,
};
```

## Core API
### Types
- `AuditLogger` — Comprehensive audit logging for security events and operations
- `SecurityValidator` — Security validation for paths, passphrases, and inputs

### AuditLogger Methods
- `new(log_path_opt)` — Create audit logger with optional file output
- `with_file(component, log_path)` — Create logger with specific file path
- `with_format(log_path_opt, format)` — Create logger with telemetry format (Text or JSON)
- `log_operation_start(operation, input, output)` — Log operation initiation
- `log_operation_success(operation, input, output)` — Log successful operation
- `log_operation_failure(operation, input, output, error)` — Log operation failure
- `log_operation_start_single(operation, path)` — Log single-path operation start
- `log_operation_complete(operation, path, result)` — Log operation completion with metrics
- `log_status_check(path, status)` — Log repository status check
- `log_encryption_event_extended(path, recipients, identity_type, success, strategy, tier)` — Log encryption with metadata
- `log_decryption_event_extended(path, identity_type, success, strategy)` — Log decryption with metadata
- `log_authority_operation(operation, recipient)` — Log authority/recipient operations
- `log_emergency_operation(operation, path)` — Log emergency operations
- `log_health_check(status)` — Log health check results
- `log_info(message)` — Log informational message
- `log_warning(message)` — Log warning message
- `log_error(message)` — Log error message

### SecurityValidator Methods
- `new(strict_mode)` — Create validator with optional strict mode
- `validate_file_path(path)` — Validate file path for traversal and sensitive access
- `validate_passphrase_security(passphrase)` — Validate passphrase for injection patterns

## Patterns
- Dual output: stderr for immediate visibility + optional file logging
- Structured JSON telemetry for machine parsing
- Text format for human-readable logs
- Security-conscious recipient hashing (MD5) to avoid logging sensitive keys
- Injection pattern detection and blocking

## Examples
```rust
// Basic audit logging
fn example_audit_logging() -> AgeResult<()> {
    // Create logger with file output
    let logger = AuditLogger::with_file(
        "cage_automation",
        Path::new("/var/log/cage/audit.log")
    )?;

    // Log operation lifecycle
    logger.log_operation_start("encrypt",
        Path::new("input.txt"),
        Path::new("output.age"))?;

    // ... perform encryption ...

    logger.log_operation_success("encrypt",
        Path::new("input.txt"),
        Path::new("output.age"))?;

    Ok(())
}

// JSON telemetry with extended metadata
fn example_json_telemetry() -> AgeResult<()> {
    let logger = AuditLogger::with_format(
        Some(PathBuf::from("/var/log/cage/events.jsonl")),
        TelemetryFormat::Json
    )?;

    // Log encryption event with full metadata
    logger.log_encryption_event_extended(
        Path::new("secret.txt"),
        Some(vec!["age1abc123".to_string()]),
        "age_identity",
        true,
        Some("pipe"),
        Some("M")  // Ignition authority tier
    )?;

    Ok(())
}

// Security validation
fn example_validation() -> AgeResult<()> {
    let validator = SecurityValidator::new(true);  // strict mode

    // Validate file path
    validator.validate_file_path(Path::new("./data/file.txt"))?;

    // Validate passphrase security
    validator.validate_passphrase_security("secure_passphrase")?;

    Ok(())
}

// Operation result tracking
fn example_operation_tracking() -> AgeResult<()> {
    let logger = AuditLogger::with_format(
        Some(PathBuf::from("/tmp/audit.jsonl")),
        TelemetryFormat::Json
    )?;

    let mut result = OperationResult::new();
    result.processed_files.push("file1.txt".to_string());
    result.processed_files.push("file2.txt".to_string());
    result.execution_time_ms = 150;

    logger.log_operation_complete(
        "lock",
        Path::new("/repo"),
        &result
    )?;

    Ok(())
}
```

## Integration
- Integrated with all Cage operations (encryption, decryption, status checks)
- Used by CageManager for lifecycle coordination tracking
- Used by adapters for backend operation logging
- Supports external monitoring systems via JSON telemetry format

## Testing
- Unit tests located in `src/cage/audit/mod.rs` (tests module)
- Comprehensive tests for audit logger creation and formatting
- Security validator tests for injection prevention
- JSON and text telemetry format validation
- Coverage expectations: >90%

## Performance Characteristics
- Dual output: stderr + optional file logging
- Minimal overhead with buffered file writes
- Structured JSON format for efficient parsing
- MD5 hashing for recipient audit trails (privacy-preserving)
- No blocking I/O in critical paths

## Limitations
- File logging requires write permissions to log directory
- MD5 recipient hashing is for audit purposes only (not cryptographic security)
- Structured JSON events require external parsing tools for analysis
- Stderr output may interfere with some shell scripting patterns

## Security Features
- Path traversal detection (.. patterns)
- Sensitive path access prevention in strict mode (/etc, /proc, /sys, /dev)
- Injection pattern detection (command injection, null bytes)
- Recipient key redaction (only logs MD5 hash for audit trail)
- Comprehensive operation lifecycle tracking

## Telemetry Formats

### Text Format
```
[2025-10-01 12:34:56 UTC] [INFO] [cage_automation] OPERATION_START encrypt input.txt -> output.age
```

### JSON Format
```json
{
  "timestamp": "2025-10-01T12:34:56.789Z",
  "level": "INFO",
  "component": "cage_automation",
  "event_type": "encryption",
  "path": "/path/to/file.txt",
  "identity_type": "age_identity",
  "recipient_count": 2,
  "recipient_group_hash": "abc123def456",
  "streaming_strategy": "pipe",
  "authority_tier": "M",
  "success": true
}
```

## Status
- MODERN: Yes
  - Structured logging with JSON telemetry support
  - Security-first validation approach
  - Comprehensive event metadata
- SPEC_ALIGNED: Yes
  - MODULE_SPEC v3 compliant single-file module
  - Follows security guardian patterns (Edgar)
  - Privacy-preserving audit logging

## Changelog
- 2025-10-01: MOD4-03 - Consolidated security.rs into audit module
  - Moved from scattered security.rs to organized audit/ directory
  - Updated all import paths from cage::security to cage::audit
  - Maintained all existing functionality and tests
  - Documentation created following MODULE_SPEC patterns

## References
- Cage PTY Automation Documentation (FEATURES_PTY.md)
- Cage Adapter Documentation (FEATURES_ADP.md)
- RSB MODULE_SPEC v3
- Age Encryption Security Best Practices

---

_Generated for Cage MOD4-03 Module Consolidation_

<!-- feat:audit -->

_Generated by bin/feat2.py --update-doc._

* `src/audit/mod.rs`
  - struct AuditLogger (line 19)
  - fn new (line 27)
  - fn with_file (line 48)
  - fn with_format (line 63)
  - fn log_operation_start (line 70)
  - fn log_operation_success (line 86)
  - fn log_operation_failure (line 102)
  - fn log_health_check (line 120)
  - fn log_info (line 126)
  - fn log_warning (line 131)
  - fn log_error (line 136)
  - fn log_operation_start_single (line 141)
  - fn log_operation_complete (line 156)
  - fn log_status_check (line 187)
  - fn log_authority_operation (line 210)
  - fn log_encryption_event_extended (line 235)
  - fn log_encryption_event (line 297)
  - fn log_decryption_event_extended (line 314)
  - fn log_decryption_event (line 359)
  - fn log_emergency_operation (line 406)
  - struct SecurityValidator (line 464)
  - fn new (line 470)
  - fn validate_file_path (line 475)
  - fn validate_passphrase_security (line 503)

<!-- /feat:audit -->

