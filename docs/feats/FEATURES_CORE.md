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

<!-- feat:core -->

_Generated by bin/feat2.py --update-doc._

* `src/core/config.rs`
  - enum OutputFormat (line 17)
  - fn age_flag (line 26)
  - fn description (line 34)
  - fn detect_from_path (line 42)
  - enum TtyMethod (line 61)
  - fn description (line 72)
  - fn dependencies (line 81)
  - enum TelemetryFormat (line 99)
  - enum SecurityLevel (line 114)
  - fn validation_timeout (line 125)
  - enum RetentionPolicyConfig (line 141)
  - struct AgeConfig (line 156)
  - fn new (line 238)
  - fn production (line 243)
  - fn development (line 256)
  - fn testing (line 269)
  - fn validate (line 283)
  - fn with_output_format (line 344)
  - fn with_tty_method (line 350)
  - fn with_security_level (line 356)
  - fn with_timeout (line 362)
  - fn with_audit_logging (line 368)
  - fn with_audit_log_path (line 374)
  - fn with_age_binary (line 380)
  - fn with_extension (line 386)
  - fn for_padlock (line 392)
  - fn extension_with_dot (line 404)
  - fn load_default (line 412)
  - fn get_config_search_paths (line 423)
  - fn format_layers (line 428)
  - fn add_recipient_group (line 512)
  - fn get_recipient_group (line 517)
  - fn get_recipient_group_mut (line 525)
  - fn remove_recipient_group (line 533)
  - fn list_recipient_groups (line 541)
  - fn is_encrypted_file (line 546)
  - fn get_recipient_group_count (line 573)
  - fn get_total_recipients_count (line 578)
  - fn get_groups_by_tier (line 583)

* `src/core/engine.rs`
  - struct AgeAutomator (line 17)
  - fn new (line 25)
  - fn with_defaults (line 48)
  - fn encrypt (line 55)
  - fn decrypt (line 95)
  - fn health_check (line 128)
  - fn adapter_info (line 140)

* `src/core/mod.rs`
  - pub use engine::AgeAutomator (line 23)
  - pub use recovery::{InPlaceOperation, InPlaceOptions, RecoveryManager, SafetyValidator} (line 24)

* `src/core/recovery.rs`
  - struct RecoveryManager (line 20)
  - fn new (line 26)
  - fn create_recovery_file (line 34)
  - struct SafetyValidator (line 87)
  - fn new (line 94)
  - fn validate_in_place_operation (line 107)
  - struct InPlaceOperation (line 168)
  - fn new (line 176)
  - fn execute_lock (line 186)
  - struct InPlaceOptions (line 279)

* `src/core/requests.rs`
  - struct CommonOptions (line 17)
  - enum Identity (line 36)
  - enum Recipient (line 52)
  - enum AuthorityTier (line 72)
  - fn as_str (line 87)
  - fn from_str (line 98)
  - struct RecipientGroup (line 112)
  - fn new (line 129)
  - fn with_tier (line 139)
  - fn add_recipient (line 146)
  - fn remove_recipient (line 153)
  - fn contains_recipient (line 163)
  - fn len (line 168)
  - fn is_empty (line 173)
  - fn set_tier (line 178)
  - fn group_hash (line 183)
  - fn set_metadata (line 190)
  - fn get_metadata (line 195)
  - struct MultiRecipientConfig (line 202)
  - fn new (line 218)
  - fn with_primary_group (line 228)
  - fn add_group (line 234)
  - fn with_authority_validation (line 240)
  - fn with_hierarchy_enforcement (line 246)
  - fn flatten_recipients (line 252)
  - fn total_recipients (line 271)
  - fn all_groups (line 276)
  - struct LockRequest (line 298)
  - fn new (line 335)
  - fn with_recipients (line 352)
  - fn with_multi_recipient_config (line 358)
  - fn recursive (line 364)
  - fn with_pattern (line 370)
  - fn with_format (line 376)
  - struct UnlockRequest (line 388)
  - fn new (line 419)
  - fn recursive (line 434)
  - fn selective (line 440)
  - fn preserve_encrypted (line 446)
  - fn with_pattern (line 452)
  - struct RotateRequest (line 464)
  - fn new (line 495)
  - fn with_new_recipients (line 510)
  - fn atomic (line 516)
  - struct VerifyRequest (line 528)
  - enum ReportFormat (line 553)
  - fn new (line 566)
  - fn deep_verify (line 579)
  - fn with_report_format (line 586)
  - struct StatusRequest (line 598)
  - fn new (line 620)
  - fn detailed (line 632)
  - struct StreamRequest (line 644)
  - enum StreamOperation (line 666)
  - fn encrypt (line 675)
  - fn decrypt (line 687)
  - enum BatchOperation (line 705)
  - struct BatchRequest (line 714)
  - fn new (line 751)
  - fn with_recipients (line 768)
  - fn with_pattern (line 774)
  - fn recursive (line 780)
  - fn with_format (line 786)
  - fn backup (line 792)
  - fn preserve_encrypted (line 798)
  - fn verify_before_unlock (line 804)
  - trait FromCliArgs (line 816)
  - trait ToOperationParams (line 826)

<!-- /feat:core -->

