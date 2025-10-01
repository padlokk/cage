# Cage Manager Module (Orchestration & Lifecycle)

Updated: 2025-10-01

## Purpose
- Provide centralized orchestration for all Cage encryption operations
- Implement comprehensive CRUD (Create/Read/Update/Delete) workflows with safety validation
- Coordinate TTY automation, authority management, and multi-recipient configurations
- Handle backup, retention, and lifecycle management for encrypted resources
- Serve as the primary integration point for Padlock/Ignite and other high-level consumers

## Feature Flags
- `mgr` — Manager orchestration module
  - Provides high-level coordination for all Cage operations
  - Default: Enabled

## Imports
```rust
use cage::mgr::{
    CageManager,
    LockOptions,
    UnlockOptions,
    VerificationResult,
    BackupManager,
    BackupRegistry,
    RetentionPolicy
};
use cage::core::{
    LockRequest,
    UnlockRequest,
    RotateRequest,
    StatusRequest,
    VerifyRequest,
    BatchRequest
};
```

## Core Philosophy: Orchestrated Operations with Safety

The Manager module serves as the **command center** for Cage operations, coordinating:
- **Request processing** — Typed request builders (LockRequest, UnlockRequest, etc.)
- **Safety validation** — Pre-flight checks and multi-layered protection
- **Backup lifecycle** — Retention policies, generation tracking, recovery
- **Authority management** — Multi-recipient configurations with tier-based access
- **Audit integration** — Comprehensive operation logging and telemetry

**Design Principles (from LIBRARY_USAGE.md):**
1. **Request API First** — All operations accept typed request structs for consistency
2. **Safety by Default** — Destructive operations require explicit opt-in
3. **Backup Awareness** — Automatic backup creation with retention enforcement
4. **Authority Flexibility** — Support for single passphrases to complex multi-tier groups
5. **Audit Everything** — All operations emit structured telemetry events

## Key Components

### CageManager
Central orchestration point for all Cage operations:
```rust
pub struct CageManager {
    config: AgeConfig,
    adapter: Arc<dyn AgeAdapterV2>,
    backup_manager: BackupManager,
    audit_logger: AuditLogger,
    safety_validator: SafetyValidator,
}

impl CageManager {
    // Primary request-based API
    pub fn lock_with_request(&mut self, request: &LockRequest) -> Result<OperationResult>;
    pub fn unlock_with_request(&mut self, request: &UnlockRequest) -> Result<OperationResult>;
    pub fn rotate_with_request(&mut self, request: &RotateRequest) -> Result<OperationResult>;
    pub fn status_with_request(&self, request: &StatusRequest) -> Result<RepositoryStatus>;
    pub fn verify_with_request(&self, request: &VerifyRequest) -> Result<VerificationResult>;
    pub fn batch_with_request(&mut self, request: &BatchRequest) -> Result<OperationResult>;

    // Authority management
    pub fn allow(&mut self, identity: &Identity, recipients: Vec<Recipient>) -> Result<()>;
    pub fn revoke(&mut self, identity: &Identity, recipients: Vec<Recipient>) -> Result<()>;
    pub fn reset(&mut self, identity: &Identity, recipients: Vec<Recipient>) -> Result<()>;

    // Recipient group management
    pub fn list_recipient_groups(&self) -> Vec<RecipientGroup>;
    pub fn add_recipient_to_group(&mut self, group: &str, recipient: Recipient) -> Result<()>;
    pub fn create_recipient_group(&mut self, name: &str, tier: AuthorityTier) -> Result<RecipientGroup>;
}
```

**Philosophy:** CageManager is the **single source of truth** for operation orchestration, delegating to specialized components while maintaining cohesive workflow.

### BackupManager
Handles backup lifecycle with retention enforcement:
```rust
pub struct BackupManager {
    backup_dir: Option<PathBuf>,
    backup_extension: String,
    cleanup_on_success: bool,
    retention_policy: RetentionPolicy,
    registry: BackupRegistry,
}

impl BackupManager {
    pub fn create_backup_with_retention(&mut self, file: &Path) -> Result<BackupInfo>;
    pub fn restore_backup_generation(&self, file: &Path, generation: u32) -> Result<()>;
    pub fn list_backups(&self, file: &Path) -> Vec<BackupEntry>;
    pub fn enforce_retention(&mut self) -> Result<Vec<PathBuf>>;  // Returns deleted backups
}
```

**From BACKUP_RETENTION_DESIGN.md:**
- **Retention policies** — KeepAll, KeepDays(u32), KeepLast(usize), KeepLastAndDays{last, days}
- **Generation tracking** — Automatic numbering (1 = latest, 2 = previous, etc.)
- **Conflict resolution** — Timestamped `.conflict` files when backups collide
- **Atomic operations** — Registry updates are atomic (write to temp, then rename)

### BackupRegistry
JSON-backed registry for backup metadata:
```rust
pub struct BackupRegistry {
    backups: HashMap<PathBuf, Vec<BackupEntry>>,
}

pub struct BackupEntry {
    pub backup_path: PathBuf,
    pub created_at: SystemTime,
    pub size_bytes: u64,
    pub generation: u32,  // 1 = most recent
}

impl BackupRegistry {
    pub fn load(backup_dir: &Path) -> Result<Self>;  // From .cage_backups.json
    pub fn save(&self, backup_dir: &Path) -> Result<()>;
    pub fn register(&mut self, original: PathBuf, backup: BackupEntry);
    pub fn apply_retention(&mut self, policy: &RetentionPolicy) -> Vec<PathBuf>;
}
```

**Persistence:** Stored as `.cage_backups.json` with atomic write semantics.

### RetentionPolicy
Configurable backup retention strategies:
```rust
pub enum RetentionPolicy {
    KeepAll,                                    // Keep indefinitely
    KeepDays(u32),                              // Delete after N days
    KeepLast(usize),                           // Keep only last N backups
    KeepLastAndDays { last: usize, days: u32 } // Combined strategy
}

impl RetentionPolicy {
    pub fn apply(&self, backups: &[BackupEntry]) -> Vec<usize>;  // Indices to delete
}
```

**Examples from BACKUP_RETENTION_DESIGN.md:**
- `KeepLast(3)` — Keep 3 most recent, delete older
- `KeepDays(7)` — Delete backups older than 7 days
- `KeepLastAndDays{last: 3, days: 30}` — Always keep 3 most recent + anything within 30 days

## Usage Patterns

### Basic Lock/Unlock Workflow
```rust
use cage::mgr::CageManager;
use cage::core::{LockRequest, UnlockRequest, Identity, OutputFormat};

let mut manager = CageManager::with_defaults()?;

// Lock (encrypt) a file
let lock_req = LockRequest::new(
    PathBuf::from("secrets.txt"),
    PathBuf::from("secrets.txt.age")
)
.with_identity(Identity::Passphrase("secure_pass".into()))
.with_format(OutputFormat::Binary)
.with_backup(true)
.build()?;

let result = manager.lock_with_request(&lock_req)?;
println!("Encrypted {} files", result.processed_files.len());

// Unlock (decrypt) the file
let unlock_req = UnlockRequest::new(
    PathBuf::from("secrets.txt.age"),
    PathBuf::from("secrets.txt")
)
.with_identity(Identity::Passphrase("secure_pass".into()))
.preserve_encrypted(true)  // Keep .age file after decryption
.build()?;

manager.unlock_with_request(&unlock_req)?;
```

### Multi-Recipient Configuration
From LIBRARY_USAGE.md - Multi-Recipient Lifecycle:
```rust
use cage::core::{RecipientGroup, Recipient, AuthorityTier, MultiRecipientConfig};

// Create recipient groups with tiers
let admin_group = RecipientGroup::new("admins")
    .with_tier(AuthorityTier::Elevated)
    .add_recipient(Recipient::X25519("age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p".into()))
    .add_recipient(Recipient::X25519("age1t5wn2rua8zacvgm3jjgmv9f7e87qvvvnv2wfqlwqvgxwaq6ql0sqm6u5gm".into()));

let emergency_group = RecipientGroup::new("emergency")
    .with_tier(AuthorityTier::Emergency)
    .add_recipient(Recipient::X25519("age1yubikey1...".into()));

// Build multi-recipient config
let multi_config = MultiRecipientConfig::new()
    .add_group(admin_group)
    .add_group(emergency_group);

// Use in lock request
let lock_req = LockRequest::new(input, output)
    .with_multi_recipient_config(multi_config)
    .build()?;

manager.lock_with_request(&lock_req)?;
```

### Backup Management with Retention
From BACKUP_RETENTION_DESIGN.md:
```rust
use cage::mgr::{BackupManager, RetentionPolicy};

// Configure retention policy
let mut backup_mgr = BackupManager::new()
    .with_backup_dir(PathBuf::from("/backups"))
    .with_retention(RetentionPolicy::KeepLast(5))
    .with_cleanup(true);

// Create backup with automatic retention enforcement
let backup_info = backup_mgr.create_backup_with_retention(&file_path)?;
println!("Backup created: {}", backup_info.backup_path.display());

// List available backups
let backups = backup_mgr.list_backups(&file_path);
for (gen, backup) in backups.iter().enumerate() {
    println!("Generation {}: {} ({} bytes, created {})",
        gen + 1,
        backup.backup_path.display(),
        backup.size_bytes,
        backup.created_at.elapsed()?.as_secs()
    );
}

// Restore from specific generation
backup_mgr.restore_backup_generation(&file_path, 2)?;  // Restore generation 2
```

### Safe In-Place Operations
From SAFETY_DESIGN.md integration:
```rust
use cage::core::{SafetyValidator, InPlaceOperation};

// Validate safety before in-place operation
let validator = SafetyValidator::new(
    false,  // danger_mode
    false   // i_am_sure
)?;

validator.validate_in_place_operation(&file_path)?;

// Execute in-place encryption with recovery file
let mut in_place_op = InPlaceOperation::new(&file_path);
in_place_op.execute_lock(passphrase, options)?;

// Recovery file at file_path.tmp.recover - delete after verification
```

### Key Rotation
```rust
use cage::core::RotateRequest;

let rotate_req = RotateRequest::new(
    PathBuf::from("/repo"),
    Identity::Passphrase("old_pass".into()),
    Identity::Passphrase("new_pass".into())
)
.recursive(true)
.atomic(true)  // All-or-nothing rotation
.build()?;

let result = manager.rotate_with_request(&rotate_req)?;
if result.success {
    println!("Rotated {} files", result.processed_files.len());
} else {
    eprintln!("Rotation failed: {:?}", result.failed_files);
}
```

### Repository Status & Verification
```rust
use cage::core::{StatusRequest, VerifyRequest};

// Check repository encryption status
let status_req = StatusRequest::new(PathBuf::from("/repo"))
    .recursive(true)
    .detailed(true)
    .build()?;

let status = manager.status_with_request(&status_req)?;
println!("Repository: {:.1}% encrypted ({}/{})",
    status.encryption_percentage(),
    status.encrypted_files,
    status.total_files
);

// Verify encrypted files
let verify_req = VerifyRequest::new(vec![PathBuf::from("/repo/file.age")])
    .deep_verify(true)  // Full integrity check
    .build()?;

let verify_result = manager.verify_with_request(&verify_req)?;
if verify_result.is_valid() {
    println!("Verification passed");
}
```

### Batch Operations
```rust
use cage::core::{BatchRequest, BatchOperation};

let batch_req = BatchRequest::new(
    PathBuf::from("/repo"),
    BatchOperation::Lock,
    Identity::Passphrase("batch_pass".into())
)
.with_pattern("*.txt".to_string())
.recursive(true)
.backup(true)
.with_format(OutputFormat::AsciiArmor)
.build()?;

let result = manager.batch_with_request(&batch_req)?;
println!("Batch processed: {} success, {} failed",
    result.processed_files.len(),
    result.failed_files.len()
);
```

## Integration Points

### Adapter Layer (adp/)
Manager delegates encryption to adapters:
- `ShellAdapterV2` for CLI-based Age operations
- Future: Library-based adapter when AGE-01 lands
- Transparent backend selection via configuration

### Safety & Recovery (core/)
Integrates multi-layered safety:
- `SafetyValidator` for pre-flight validation
- `RecoveryManager` for backup creation
- `InPlaceOperation` for atomic file replacement
- See SAFETY_DESIGN.md for 5-layer protection model

### Audit & Telemetry (audit/)
All operations emit structured events:
- Operation start/complete with timing
- File-level encryption/decryption events
- Authority tier tracking (X/M/R/I/D)
- Success rates and error context

### Forge Operations (forge/)
Manager coordinates with forge layer:
- `FileOperationsManager` for single-file operations
- `RepositoryOperationsManager` for bulk operations
- Validation and execution separation

## Backup Retention Philosophy

From BACKUP_RETENTION_DESIGN.md:

### Design Goals
1. **Automatic lifecycle management** — No manual cleanup required
2. **Configurable retention** — Policy-driven backup retention
3. **Generation tracking** — Restore any previous version
4. **Conflict resolution** — Handle backup collisions gracefully
5. **Atomic operations** — Registry updates are safe and consistent

### Retention Policy Examples

**KeepLast(3)** — Keep only 3 most recent backups:
- Keeps: backup.txt.bak (gen 1, 2, 3)
- Deletes: backup.txt.bak (gen 4+)

**KeepDays(7)** — Keep backups for 7 days:
- Keeps: All backups < 7 days old
- Deletes: Backups >= 7 days old

**KeepLastAndDays{last: 3, days: 30}** — Combined strategy:
- Always keeps last 3 backups (even if > 30 days)
- Plus any additional backups within 30 days
- Deletes: Beyond 3 generations AND older than 30 days

### Backup Discovery
```rust
// List all backups for a file
let backups = manager.list_backups(&file_path);
for backup in backups {
    println!("Gen {}: {} ({})",
        backup.generation,
        backup.backup_path.display(),
        backup.age_days()
    );
}

// Restore specific generation
manager.restore_backup_generation(&file_path, 2)?;
```

## Safety Integration

From SAFETY_DESIGN.md, the Manager respects multi-layered safety:

### Layer 1: Explicit Opt-In
- In-place operations require `--in-place` flag
- Default behavior is non-destructive (separate output file)

### Layer 2: Recovery Files
- `.tmp.recover` files created for in-place operations
- Contains passphrase and recovery instructions
- Restrictive permissions (0o600)

### Layer 3: Atomic Operations
- Temp file → atomic rename prevents corruption
- Metadata preservation (permissions, timestamps)

### Layer 4: Validation Gates
- Pre-flight checks (existence, permissions, disk space)
- Risk assessment for destructive operations

### Layer 5: Rollback Capability
- Automatic cleanup on failure (Drop trait)
- Backup integration for additional safety

**CLI Safety Flow:**
```bash
# Safe (creates recovery file)
cage lock file.txt --in-place

# Danger mode (requires env var + confirmation)
DANGER_MODE=1 cage lock file.txt --in-place --danger-mode

# Automation (all flags required)
DANGER_MODE=1 cage lock file.txt --in-place --danger-mode --i-am-sure
```

## Authority Management

### Multi-Tier Recipients
```rust
use cage::core::{AuthorityTier, RecipientGroup};

// Create tiered recipient groups
let standard = RecipientGroup::new("users")
    .with_tier(AuthorityTier::Standard);

let elevated = RecipientGroup::new("admins")
    .with_tier(AuthorityTier::Elevated);

let emergency = RecipientGroup::new("recovery")
    .with_tier(AuthorityTier::Emergency);

// Add to config
let mut config = AgeConfig::default();
config.add_recipient_group(standard);
config.add_recipient_group(elevated);
config.add_recipient_group(emergency);

let manager = CageManager::with_config(config)?;
```

### Authority Operations
```rust
// Grant access (add recipients)
manager.allow(
    &Identity::Passphrase("current_pass".into()),
    vec![Recipient::X25519("age1new...".into())]
)?;

// Revoke access (remove recipients)
manager.revoke(
    &Identity::Passphrase("current_pass".into()),
    vec![Recipient::X25519("age1old...".into())]
)?;

// Reset access (replace all recipients)
manager.reset(
    &Identity::Passphrase("current_pass".into()),
    vec![Recipient::X25519("age1only...".into())]
)?;
```

## Performance Characteristics
- **Request processing**: <1ms overhead (validation + routing)
- **Backup creation**: ~50ms per file (includes registry update)
- **Retention enforcement**: <10ms for 1000 backups
- **Repository operations**: Scales linearly (sequential processing)

**Benchmark Results (CAGE-03 validation):**
- Backup + retention (100 files): 5 seconds
- Registry load/save: <50ms
- Generation tracking: O(n) where n = backup count per file

## Limitations
- Repository operations are sequential (parallel planned)
- Backup registry stored locally (no distributed support)
- Authority changes require re-encryption (no ACL updates)
- Retention enforcement is eager (no lazy cleanup option)

## Testing
- Unit tests: Request processing, backup lifecycle, retention policies
- Integration tests: Multi-recipient workflows, key rotation, backup recovery
- Regression tests: BUG-07/08 (retention logic, registry tracking)
- Coverage expectations: >85%

**Test Files:**
- `tests/mgr/` — Manager orchestration tests
- `tests/integration/backup_retention.rs` — Retention policy validation
- `tests/integration/multi_recipient.rs` — Authority management tests

## Status
- MODERN: Yes
  - Request-based API for consistency
  - Backup lifecycle with retention policies
  - Multi-tier authority management
  - Comprehensive safety integration
- SPEC_ALIGNED: Yes
  - Follows RSB MODULE_SPEC v3 structure
  - Integrates with all Cage modules
  - Aligns with LIBRARY_USAGE.md patterns

## Changelog
- 2025-10-01: Enhanced documentation with comprehensive usage patterns
  - Added narrative from LIBRARY_USAGE.md (request API, multi-recipient)
  - Detailed backup retention philosophy from BACKUP_RETENTION_DESIGN.md
  - Safety integration from SAFETY_DESIGN.md (5-layer protection)
  - Authority management and repository workflows
  - Performance characteristics and testing coverage

## Reference Documentation

This feature specification is informed by the following design documents:

- [`docs/ref/cage/LIBRARY_USAGE.md`](../ref/cage/LIBRARY_USAGE.md) — Comprehensive usage examples and patterns
- [`docs/ref/cage/BACKUP_RETENTION_DESIGN.md`](../ref/cage/BACKUP_RETENTION_DESIGN.md) — Retention policy design and lifecycle management
- [`docs/ref/cage/SAFETY_DESIGN.md`](../ref/cage/SAFETY_DESIGN.md) — Multi-layered safety architecture (5-layer protection model)

### Additional Resources

- `docs/procs/TASKS.txt` — CAGE-03, CAGE-16, OBS-01 implementation notes

## API Surface

<!-- feat:mgr -->

_Generated by bin/feat2.py --update-doc._

* `src/mgr/cage_manager.rs`
  - struct LockOptions (line 34)
  - struct UnlockOptions (line 56)
  - struct AuthorityResult (line 76)
  - struct VerificationResult (line 85)
  - enum RetentionPolicy (line 94)
  - fn apply (line 114)
  - struct BackupManager (line 185)
  - fn new (line 195)
  - fn with_backup_dir (line 206)
  - fn with_extension (line 220)
  - fn with_cleanup (line 230)
  - fn with_retention (line 236)
  - fn create_backup (line 242)
  - fn restore_backup (line 295)
  - fn cleanup_backup (line 312)
  - fn create_backup_with_retention (line 322)
  - fn list_backups (line 362)
  - fn restore_backup_generation (line 367)
  - fn registry_stats (line 394)
  - fn enforce_retention (line 399)
  - struct BackupInfo (line 561)
  - fn age_seconds (line 569)
  - struct BackupEntry (line 582)
  - fn age_seconds (line 591)
  - fn age_days (line 596)
  - struct BackupRegistry (line 603)
  - fn new (line 610)
  - fn load (line 617)
  - fn save (line 638)
  - fn register (line 668)
  - fn update_backup_path (line 677)
  - fn next_generation (line 688)
  - fn apply_retention (line 697)
  - fn list_for_file (line 748)
  - fn file_count (line 756)
  - fn total_backups (line 761)
  - fn remove_file (line 766)
  - struct FileVerificationStatus (line 773)
  - fn is_valid (line 783)
  - struct EmergencyResult (line 794)
  - struct CageManager (line 802)
  - struct OperationRecord (line 812)
  - fn new (line 845)
  - fn with_defaults (line 869)
  - fn lock_with_request (line 880)
  - fn unlock_with_request (line 928)
  - fn rotate_with_request (line 948)
  - fn status_with_request (line 986)
  - fn stream_with_request (line 1025)
  - fn verify_with_request (line 1051)
  - fn lock (line 1106)
  - fn status (line 1153)
  - fn rotate (line 1176)
  - fn unlock (line 1444)
  - fn allow (line 1729)
  - fn revoke (line 1752)
  - fn reset (line 1773)
  - fn list_recipient_groups (line 1811)
  - fn add_recipient_to_group (line 1833)
  - fn remove_recipient_from_group (line 1859)
  - fn create_recipient_group (line 1894)
  - fn audit_recipient_groups (line 1924)
  - fn get_groups_by_tier (line 1956)
  - fn get_adapter_info_with_groups (line 1962)
  - fn verify (line 2001)
  - fn emergency_unlock (line 2051)
  - fn batch_with_request (line 2083)
  - fn batch_process (line 2178)
  - fn get_operation_history (line 3041)
  - fn encrypt_to_path (line 3046)

* `src/mgr/mod.rs`
  - pub use cage_manager::{CageManager, LockOptions, UnlockOptions, VerificationResult} (line 13)

<!-- /feat:mgr -->

