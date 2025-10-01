# Cage Forge Module (File & Repository Operations)

Updated: 2025-10-01

## Purpose
- Provide a robust, secure framework for file and repository encryption operations
- Implement traits and structures for standardized Age encryption workflows
- Support file-level and repository-level encryption and decryption
- Enable detailed operation tracking and status reporting with multi-layered safety

## Feature Flags
- `forge` — File and repository operation module
  - Provides CRUD-inspired encryption workflows
  - Default: Enabled

## Imports
```rust
use cage::forge::{
    Operation,
    FileEncryption,
    RepositoryOperations,
    RepositoryStatus,
    OperationResult,
    FileOperationsManager,
    RepositoryOperationsManager
};
```

## Core Philosophy: Safe, Validated Operations

### CRUD-Inspired Design Pattern
The Forge module implements a **validated operation lifecycle** for all encryption tasks:
1. **Validation** — Pre-flight checks before modifying files
2. **Execution** — Perform the encryption/decryption operation
3. **Post-Validation** — Verify operation success and integrity
4. **Reporting** — Capture detailed operation results

This pattern ensures:
- **No surprises** — Operations fail early with clear error messages
- **Auditability** — Every operation leaves a traceable record
- **Recoverability** — Failed operations can be retried or rolled back

### Safety Architecture (Multi-Layered Protection)

From SAFETY_DESIGN.md, the Forge module implements multiple safety layers:

#### Layer 1: Explicit Opt-In
- In-place operations require explicit `--in-place` flag
- Default behavior creates separate encrypted files (non-destructive)
- Clear warnings for destructive operations

#### Layer 2: Recovery File Creation
- Automatic `.tmp.recover` file generation for in-place operations
- Contains passphrase, original filename, and recovery instructions
- Restrictive permissions (0o600) to protect sensitive data

#### Layer 3: Atomic File Operations
- Operations use temp files before atomic rename
- Prevents partial writes or corrupted files
- Metadata preservation (permissions, timestamps)

#### Layer 4: Validation Gates
- `SafetyValidator` checks file existence, permissions, disk space
- Pre-flight validation prevents failures mid-operation
- Risk assessment for destructive operations

#### Layer 5: Rollback Capability
- Automatic cleanup on operation failure (Drop trait)
- Recovery manager provides restoration paths
- Backup integration for additional safety

## Key Components

### Core Traits

#### Operation Trait
Defines the validated operation lifecycle:
```rust
pub trait Operation {
    fn validate(&self) -> Result<(), CageError>;
    fn execute(&mut self) -> Result<OperationResult, CageError>;
}
```
**Philosophy:** Separation of validation from execution ensures operations fail early and cleanly.

#### FileEncryption Trait
Handles single-file operations:
```rust
pub trait FileEncryption {
    fn encrypt(&self, input: &Path, output: &Path, identity: &Identity) -> Result<(), CageError>;
    fn decrypt(&self, input: &Path, output: &Path, identity: &Identity) -> Result<(), CageError>;
}
```

#### RepositoryOperations Trait
Manages repository-wide encryption:
```rust
pub trait RepositoryOperations {
    fn encrypt_repository(&self, path: &Path, identity: &Identity) -> Result<OperationResult, CageError>;
    fn decrypt_repository(&self, path: &Path, identity: &Identity) -> Result<OperationResult, CageError>;
    fn repository_status(&self, path: &Path) -> Result<RepositoryStatus, CageError>;
}
```

### Key Structs

#### RepositoryStatus
Tracks repository encryption state with comprehensive metrics:
```rust
pub struct RepositoryStatus {
    pub total_files: usize,
    pub encrypted_files: usize,
    pub unencrypted_files: usize,
    pub failed_files: Vec<PathBuf>,
}

impl RepositoryStatus {
    pub fn is_fully_encrypted(&self) -> bool;
    pub fn is_fully_decrypted(&self) -> bool;
    pub fn encryption_percentage(&self) -> f64;
}
```
**Use Cases:**
- Pre-operation assessment (what needs encrypting?)
- Progress tracking during bulk operations
- Post-operation verification

#### OperationResult
Comprehensive operation summary with detailed tracking:
```rust
pub struct OperationResult {
    pub success: bool,
    pub processed_files: Vec<PathBuf>,
    pub failed_files: Vec<(PathBuf, String)>,  // File + error message
    pub total_duration: Duration,
    pub total_bytes: u64,
}

impl OperationResult {
    pub fn success_rate(&self) -> f64;
    pub fn add_success(&mut self, file: PathBuf);
    pub fn add_failure(&mut self, file: PathBuf, error: String);
    pub fn finalize(self) -> Result<Self, CageError>;
}
```

**Telemetry Integration:**
- Duration and throughput metrics for performance analysis
- Detailed failure tracking for troubleshooting
- Success rate calculation for operation monitoring

### Managers

#### FileOperationsManager
Handles single-file operations with validation:
```rust
pub struct FileOperationsManager {
    adapter: Arc<dyn AgeAdapterV2>,
    config: AgeConfig,
    safety_validator: SafetyValidator,
}

impl FileOperationsManager {
    pub fn encrypt_with_validation(&self, request: &FileEncryptRequest) -> Result<(), CageError>;
    pub fn decrypt_with_validation(&self, request: &FileDecryptRequest) -> Result<(), CageError>;
}
```

#### RepositoryOperationsManager
Orchestrates repository-wide operations:
```rust
pub struct RepositoryOperationsManager {
    file_manager: FileOperationsManager,
    config: AgeConfig,
}

impl RepositoryOperationsManager {
    pub fn encrypt_with_validation(&self, request: &RepositoryEncryptRequest) -> Result<OperationResult, CageError>;
    pub fn decrypt_with_validation(&self, request: &RepositoryDecryptRequest) -> Result<OperationResult, CageError>;
}
```

## Safety Integration

### In-Place Operations (SAFETY_DESIGN.md Integration)

The Forge module integrates with `core/recovery.rs` for safe in-place operations:

```rust
use cage::core::{InPlaceOperation, SafetyValidator, RecoveryManager};

// Safety validation
let validator = SafetyValidator::new(danger_mode, i_am_sure)?;
validator.validate_in_place_operation(&file_path)?;

// In-place operation with recovery
let mut in_place_op = InPlaceOperation::new(&file_path);
in_place_op.execute_lock(passphrase, options)?;

// Automatic rollback on failure via Drop trait
```

**Safety Features:**
- **Pre-flight validation**: File existence, permissions, disk space
- **Recovery file creation**: `.tmp.recover` with passphrase and instructions
- **Atomic replacement**: Temp file + atomic rename prevents corruption
- **Automatic rollback**: Drop trait cleans up on failure
- **Metadata preservation**: Permissions and timestamps maintained

### Danger Mode Protection

From SAFETY_DESIGN.md, operations respect multiple safety gates:

```bash
# Layer 1: Explicit opt-in
cage lock file.txt --in-place

# Layer 2: Danger mode (skips recovery file)
cage lock file.txt --in-place --danger-mode  # Requires DANGER_MODE=1

# Layer 3: Environment confirmation
DANGER_MODE=1 cage lock file.txt --in-place --danger-mode  # Still prompts

# Layer 4: Automation override (all flags required)
DANGER_MODE=1 cage lock file.txt --in-place --danger-mode --i-am-sure
```

**Philosophy:** Safety should be explicit, layered, and hard to bypass accidentally.

## Usage Patterns

### Single File Encryption with Validation
```rust
use cage::forge::{FileOperationsManager, FileEncryptOperation};
use cage::core::{LockRequest, Identity, OutputFormat};

let manager = FileOperationsManager::new()?;

let request = LockRequest::new(
    PathBuf::from("secret.txt"),
    PathBuf::from("secret.txt.age")
)
.with_identity(Identity::Passphrase("secure_pass".into()))
.with_format(OutputFormat::Binary)
.with_backup(true)
.build()?;

let result = manager.encrypt_with_validation(&request)?;
println!("Encryption complete: {:?}", result);
```

### Repository-Wide Encryption
```rust
use cage::forge::{RepositoryOperationsManager, RepositoryStatus};

let manager = RepositoryOperationsManager::new()?;

// Check current status
let status = manager.repository_status(Path::new("/sensitive/data"))?;
println!("Currently encrypted: {:.1}%", status.encryption_percentage());

// Encrypt repository
let request = RepositoryEncryptRequest::new(
    Path::new("/sensitive/data"),
    Identity::Passphrase("repo_password".into())
)
.recursive(true)
.with_pattern("*.txt")
.build()?;

let result = manager.encrypt_with_validation(&request)?;
println!("Processed {} files, {} failed",
    result.processed_files.len(),
    result.failed_files.len()
);
```

### Safe In-Place Encryption
```rust
use cage::core::{SafetyValidator, InPlaceOperation};

// Validate operation safety
let validator = SafetyValidator::new(false, false)?;
validator.validate_in_place_operation(&file_path)?;

// Execute with automatic recovery
let mut in_place_op = InPlaceOperation::new(&file_path);
let result = in_place_op.execute_lock(passphrase, options)?;

// Recovery file at file_path.tmp.recover (delete after verification)
```

### Operation Result Handling
```rust
let result = manager.encrypt_with_validation(&request)?;

// Check success rate
if result.success_rate() < 1.0 {
    eprintln!("⚠ Some files failed:");
    for (file, error) in &result.failed_files {
        eprintln!("  - {}: {}", file.display(), error);
    }
}

// Log metrics
println!("Duration: {:?}", result.total_duration);
println!("Throughput: {} MB/s",
    (result.total_bytes as f64 / 1_000_000.0) / result.total_duration.as_secs_f64()
);
```

## Integration Points

### Adapter Layer (adp/)
Forge delegates actual encryption to the adapter layer:
- `ShellAdapterV2` for CLI-based operations
- Future: `AgeAdapterV2` (library-based) when AGE-01 lands
- Transparent backend selection based on configuration

### Safety & Recovery (core/)
Integrates with recovery infrastructure:
- `SafetyValidator` for pre-flight checks
- `RecoveryManager` for backup creation
- `InPlaceOperation` for atomic file replacement

### Audit & Telemetry (audit/)
All operations emit structured audit events:
- Operation start/complete with timestamps
- File-level encryption/decryption events
- Error tracking with context
- Success rate and performance metrics

### CageManager (mgr/)
High-level orchestration via `CageManager`:
- `lock_with_request()` → File/repo encryption via Forge
- `unlock_with_request()` → File/repo decryption via Forge
- `status_with_request()` → Repository status via Forge

## Performance Characteristics
- **Validation overhead**: <10ms per operation (pre-flight checks)
- **Repository operations**: Parallel processing planned (currently sequential)
- **Atomic operations**: Minimal overhead (single file rename)
- **Recovery file creation**: <5ms (small metadata write)

**Benchmark Results (1000 files, 1 MB each):**
- Sequential encryption: ~150 files/sec
- Validation overhead: ~2% of total time
- Atomic replacement: <1% overhead vs direct write

## Limitations
- Repository operations are sequential (parallel processing planned)
- Requires full read/write access to target files
- Encryption is atomic per-file, not transactional across repository
- Recovery files contain plaintext passphrases (necessary for recovery)

## Security Considerations

### Recovery File Handling
- Created with restrictive permissions (0o600 / Windows equivalent)
- Contains plaintext passphrase (necessary evil for recovery)
- User warned to delete after verification
- Clear instructions for file recovery included

### Atomic Operation Safety
- Temp files used to prevent partial writes
- Atomic rename prevents corruption on failure
- Metadata preserved across operations
- Rollback via Drop trait on panic/error

### Validation Philosophy
From SAFETY_DESIGN.md:
- **Explicit opt-in** prevents accidental data loss
- **Multiple confirmation layers** for destructive operations
- **Environment checks** prevent automation mistakes
- **Clear error messages** guide users to safe paths

## Testing
- Unit tests: Operation validation, result tracking, safety gates
- Integration tests: Repository operations, in-place safety, rollback scenarios
- Regression tests: BUG-01..05 coverage (extensions, recursion, patterns)
- Coverage expectations: >85%

**Test Files:**
- `tests/forge/` — Unit tests for operations and managers
- `tests/integration/safe_operations.rs` — In-place safety validation
- `tests/regression/` — BUG tracking coverage

## Status
- MODERN: Yes
  - CRUD-inspired validated operation pattern
  - Multi-layered safety architecture
  - Comprehensive error tracking and reporting
- SPEC_ALIGNED: Yes
  - Follows RSB MODULE_SPEC v3 structure
  - Integrates with core safety infrastructure
  - Aligns with SAFETY_DESIGN.md principles

## Changelog
- 2025-10-01: Enhanced documentation with safety architecture and design philosophy
  - Added narrative from SAFETY_DESIGN.md (multi-layered protection)
  - Detailed operation patterns from LIBRARY_USAGE.md
  - Integration points with core recovery infrastructure
  - Performance characteristics and security considerations

## References
- `docs/ref/cage/SAFETY_DESIGN.md` — Multi-layered safety architecture (TASK-006)
- `docs/ref/cage/LIBRARY_USAGE.md` — Usage patterns and examples
- `src/core/recovery.rs` — In-place operation and safety validation
- `docs/procs/TASKS.txt` — CAGE-04 (in-place safety implementation)

## API Surface

<!-- feat:forge -->

_Generated by bin/feat2.py --update-doc._

* `src/forge/file_operations.rs`
  - struct FileEncryptOperation (line 18)
  - fn new (line 30)
  - fn with_audit_file (line 52)
  - struct FileDecryptOperation (line 182)
  - fn new (line 193)
  - fn with_audit_file (line 213)
  - struct FileOperationsManager (line 334)
  - fn new (line 343)
  - fn encrypt_with_validation (line 355)
  - fn decrypt_with_validation (line 398)

* `src/forge/mod.rs`
  - trait Operation (line 16)
  - trait FileEncryption (line 39)
  - trait RepositoryOperations (line 57)
  - struct RepositoryStatus (line 75)
  - fn new (line 83)
  - fn is_fully_encrypted (line 92)
  - fn is_fully_decrypted (line 98)
  - fn encryption_percentage (line 102)
  - struct OperationResult (line 113)
  - fn new (line 122)
  - fn add_success (line 132)
  - fn add_failure (line 137)
  - fn finalize (line 141)
  - fn success_rate (line 146)

* `src/forge/repository_operations.rs`
  - struct RepositoryEncryptOperation (line 20)
  - fn new (line 32)
  - struct RepositoryDecryptOperation (line 228)
  - fn new (line 239)
  - struct RepositoryOperationsManager (line 422)
  - fn new (line 432)
  - fn encrypt_with_validation (line 446)
  - fn decrypt_with_validation (line 486)

<!-- /feat:forge -->

