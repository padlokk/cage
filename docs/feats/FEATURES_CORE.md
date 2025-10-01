# Cage Core Primitives Module

Updated: 2025-10-01

## Purpose
- Provide foundational configuration and request types for Cage encryption operations
- Centralize core primitives including configuration management, request structures, age engine interface, and recovery operations
- Serve as the type foundation for all Cage modules
- Enable consistent configuration and request handling across the system

## Feature Flags
- `core` — Core primitives module (config, requests, engine, recovery)
  - Provides fundamental types for Cage operations
  - Default: Enabled (always included)

## Imports
```rust
use cage::core::{
    AgeConfig,
    OutputFormat,
    TtyMethod,
    SecurityLevel,
    TelemetryFormat,
    RetentionPolicyConfig,
    LockRequest,
    UnlockRequest,
    RotateRequest,
    VerifyRequest,
    StatusRequest,
    BatchRequest,
    Identity,
    Recipient,
    AgeAutomator,
    RecoveryManager,
    InPlaceOperation,
    SafetyValidator
};
```

## Core API

### Configuration Types (`config.rs`)
- `AgeConfig` — Main configuration structure for Age automation
  - Path configuration (age_binary_path, default_output_dir, backup_directory)
  - Behavior settings (armor_output, force_overwrite, backup_cleanup)
  - Security settings (security_level, audit_log_path, telemetry_format)
  - Performance tuning (parallel_batch_size, operation_timeout)
  - Integration (recipient_groups, streaming_strategy)

- `OutputFormat` — Age output format specification
  - `Binary` — Default binary .age format (efficient)
  - `AsciiArmor` — Text-safe ASCII armor format (-a flag)

- `TtyMethod` — TTY automation method selection
  - `Auto` — Automatic method selection
  - `Script` — Use `script` command for automation
  - `Expect` — Use `expect` tool for automation
  - `Pty` — Use PTY wrapper (portable-pty library)

- `SecurityLevel` — Security validation level
  - `Strict` — Maximum security validation
  - `Standard` — Balanced security and performance
  - `Permissive` — Minimal validation (development only)

- `TelemetryFormat` — Audit log output format
  - `Text` — Human-readable text format
  - `Json` — Structured JSON format (machine-parseable)

- `RetentionPolicyConfig` — Backup retention policy
  - `KeepAll` — Keep all backups indefinitely
  - `KeepLast(n)` — Keep only last N backups
  - `TimeBasedDays(days)` — Keep backups for N days
  - `Disabled` — No backup retention

### Request Types (`requests.rs`)
- `LockRequest` — File encryption request
  - Input/output paths
  - Passphrase or recipients
  - Output format (binary/armor)
  - Common options (force, backup, audit)

- `UnlockRequest` — File decryption request
  - Input/output paths
  - Passphrase or identity
  - Common options

- `RotateRequest` — Key rotation request
  - Old and new passphrases/identities
  - Repository or file scope
  - Backup and audit settings

- `VerifyRequest` — Integrity verification request
  - Files to verify
  - Expected recipients/identities
  - Detailed reporting options

- `StatusRequest` — Repository status request
  - Scan scope (directory/files)
  - Report format (text/json)
  - Include/exclude patterns

- `BatchRequest` — Batch operation request
  - Multiple operations (lock/unlock/rotate)
  - Parallel processing settings
  - Progress reporting

- `Identity` — Age identity types
  - `Passphrase(String)` — Passphrase-based identity
  - `X25519(PathBuf)` — X25519 key file
  - `SshEd25519(PathBuf)` — SSH Ed25519 key
  - `SshRsa(PathBuf)` — SSH RSA key

- `Recipient` — Age recipient types
  - `X25519(String)` — X25519 public key
  - `SshEd25519(String)` — SSH Ed25519 public key
  - `SshRsa(String)` — SSH RSA public key
  - `Group(String)` — Named recipient group

- `AuthorityTier` — Multi-recipient authority tiers
  - `Standard` — Standard recipient
  - `Elevated` — Elevated authority recipient
  - `Emergency` — Emergency recovery recipient

- `RecipientGroup` — Named collection of recipients
  - Group name and tier
  - List of recipients
  - Metadata (description, created_at)

- `MultiRecipientConfig` — Multi-recipient configuration
  - Multiple recipient groups
  - Authority tier management
  - Group composition

### Traits
- `FromCliArgs` — Convert CLI arguments to request types
  - Enables typed request creation from command-line input

- `ToOperationParams` — Convert requests to operation parameters
  - Enables consistent operation invocation

### Age Engine (`engine.rs`)
- `AgeAutomator` — Main Age automation coordinator
  - High-level interface for Age operations
  - Integrates with adapters, audit logging, and PTY automation
  - Coordinates encryption/decryption workflows

### Recovery & Safety (`recovery.rs`)
- `RecoveryManager` — In-place operation recovery
  - Manages backup creation and restoration
  - Handles atomic file replacement
  - Provides rollback capability

- `SafetyValidator` — Operation safety validation
  - Pre-flight checks (file existence, permissions, disk space)
  - Risk assessment for destructive operations
  - Input validation and sanitization

- `InPlaceOperation` — Atomic in-place file operation
  - Safe file replacement with backup
  - Automatic cleanup on success
  - Rollback on failure

- `InPlaceOptions` — Configuration for in-place operations
  - Backup behavior (always/never/on_failure)
  - Cleanup preferences
  - Safety validation level

## Patterns
- Configuration management with file, environment, and default sources
- Typed request structures for clean API boundaries
- Multi-recipient support with authority tiers
- Retention policies for backup management
- In-place operations with atomic guarantees
- Safety validation with configurable strictness

## Examples

### Configuration Management
```rust
use cage::core::{AgeConfig, SecurityLevel, TelemetryFormat};

// Load configuration from default locations
let config = AgeConfig::load_default()?;

// Create custom configuration
let mut config = AgeConfig::default();
config.security_level = SecurityLevel::Strict;
config.telemetry_format = TelemetryFormat::Json;
config.parallel_batch_size = 8;

// Validate configuration
config.validate()?;
```

### Request API
```rust
use cage::core::{LockRequest, Identity, OutputFormat};
use std::path::PathBuf;

// Build a lock (encryption) request
let request = LockRequest::new(
    PathBuf::from("secrets.txt"),
    PathBuf::from("secrets.txt.age")
)
.with_passphrase("secure_passphrase")
.with_output_format(OutputFormat::AsciiArmor)
.with_backup(true)
.build()?;

// Execute request through CageManager
let manager = CageManager::new(config)?;
manager.execute_lock(request)?;
```

### Multi-Recipient Configuration
```rust
use cage::core::{
    RecipientGroup,
    Recipient,
    AuthorityTier,
    MultiRecipientConfig
};

// Create recipient groups with authority tiers
let admin_group = RecipientGroup::new("admins")
    .with_tier(AuthorityTier::Elevated)
    .add_recipient(Recipient::X25519("age1...".to_string()))
    .add_recipient(Recipient::X25519("age1...".to_string()));

let emergency_group = RecipientGroup::new("emergency")
    .with_tier(AuthorityTier::Emergency)
    .add_recipient(Recipient::X25519("age1...".to_string()));

// Build multi-recipient config
let multi_config = MultiRecipientConfig::new()
    .add_group(admin_group)
    .add_group(emergency_group);
```

### In-Place Operations
```rust
use cage::core::{InPlaceOperation, InPlaceOptions, RecoveryManager};

// Configure in-place operation with recovery
let options = InPlaceOptions::new()
    .with_backup_always()
    .with_cleanup_on_success(true);

let mut op = InPlaceOperation::new(&file_path)
    .with_options(options);

// Perform operation with automatic backup/recovery
op.execute(|temp_path| {
    // Perform modifications on temp_path
    std::fs::write(temp_path, "new content")?;
    Ok(())
})?;

// Automatic cleanup and finalization
```

## Integration
- **Adapters (adp/)**: Provides configuration and request types for adapter implementations
- **PTY (pty/)**: Configures TTY automation methods and timeouts
- **Audit (audit/)**: Supplies telemetry format and audit settings
- **Manager (manager/)**: Consumes request types for operation execution
- **Operations (operations/)**: Uses configuration for file/repository operations
- **Keygen (keygen/)**: Integrates recipient and identity types

## Testing
- Unit tests located in `tests/` directory
- Request builder pattern tests
- Configuration validation tests
- Multi-recipient group management tests
- In-place operation safety tests
- Recovery and rollback scenario tests
- Coverage expectations: >85%

## Performance Characteristics
- Minimal overhead for configuration loading (one-time operation)
- Request structures use move semantics (zero-copy where possible)
- Configuration validation is fast (milliseconds)
- In-place operations use atomic file replacement
- Recovery manager uses efficient backup strategies

## Limitations
- Configuration file must be valid TOML
- Request validation happens at build time (no runtime schema evolution)
- In-place operations require sufficient disk space for backups
- Multi-recipient operations may have increased overhead

## Status
- MODERN: Yes
  - Clean separation of concerns (config, requests, engine, recovery)
  - Builder pattern for request construction
  - Type-safe configuration management
  - Atomic in-place operations
- SPEC_ALIGNED: Yes
  - Follows RSB MODULE_SPEC v3 structure
  - Proper module organization with mod.rs
  - Re-exports core types for convenience

## Changelog
- 2025-10-01: MOD4-04 - Consolidated core primitives into core/ module
  - Moved config.rs → core/config.rs
  - Moved requests.rs → core/requests.rs
  - Moved age_engine.rs → core/engine.rs
  - Moved in_place.rs → core/recovery.rs
  - Created core/mod.rs with proper re-exports
  - Updated all import paths across codebase (11 source files)
  - All 68 tests passing, 2 ignored

## References
- `.analysis/mod_spec_reorg_plan.md` - MOD4 refactor plan
- `docs/feats/FEATURES_ADP.md` - Adapter module documentation
- `docs/feats/FEATURES_PTY.md` - PTY automation documentation
- Age Encryption Specification: https://age-encryption.org/

---

_Generated for MOD4-04: Core Primitives Consolidation_
