# Cage Keygen Module (Identity & Key Management)

Updated: 2025-10-01

## Purpose
- Implement secure key generation for Age encryption identities
- Provide flexible identity and recipient key management workflows
- Support programmatic key generation with safety defaults
- Integrate with recipient group management and audit logging
- Enable both CLI-based and library-based key generation strategies

## Feature Flags
- `keygen` — Key generation module
  - Provides identity and recipient key generation
  - Default: Enabled
  - Status: **STUB** (Implementation pending CAGE-21/CAGE-22)

## Imports
```rust
use cage::keygen::{
    KeygenService,
    KeygenRequest,
    KeygenSummary,
    KeygenError
};
```

## Core Philosophy: Secure Identity Generation

From KEYGEN_STRATEGY.md, the keygen module implements:
1. **Safe defaults** — Secure paths, restrictive permissions, overwrite protection
2. **Structured output** — JSON summaries for audit and integration
3. **Flexible backends** — CLI wrapper (MVP) → library-native (AGE-01)
4. **Group integration** — Automatic registration with recipient groups
5. **Proxy fallback** — Direct `age-keygen` passthrough when needed

### Design Goals (§1 from KEYGEN_STRATEGY.md)
- **Single workflow** — `cage keygen` as primary interface
- **Opinionated defaults** — Secure paths and permissions out-of-the-box
- **Structured results** — JSON output for downstream tooling
- **Safety controls** — Refuse overwrite unless explicit `--force`
- **Fallback path** — Proxy mode ensures coverage if Cage logic regresses

## Key Components

### KeygenRequest
Configures key generation parameters:
```rust
pub struct KeygenRequest {
    pub output_path: Option<PathBuf>,        // Default: ${XDG_CONFIG_HOME}/cage/identities/<timestamp>.agekey
    pub register_groups: Vec<String>,        // Recipient groups to register with
    pub recipients_only: bool,               // -y mode: emit public key only
    pub input_identity: Option<PathBuf>,     // For recipients_only mode
    pub stdout_only: bool,                   // Print without writing file
    pub force_overwrite: bool,               // -f: Allow overwrite
    pub json_output: bool,                   // Emit JSON summary (default: true)
    pub proxy_mode: bool,                    // Direct age-keygen passthrough
}
```

**From KEYGEN_STRATEGY.md §2 (MVP Requirements):**
- Default output: `${XDG_CONFIG_HOME}/cage/identities/<timestamp>.agekey`
- Permissions: `chmod 0o600` (Unix) / Windows equivalent after write
- Overwrite protection: Refuse unless `--force` provided
- JSON summary: `{path, public_recipient, fingerprint_md5, fingerprint_sha256, created_at, registered_groups}`

### KeygenSummary
Captures key generation results:
```rust
pub struct KeygenSummary {
    pub output_path: Option<PathBuf>,
    pub public_recipient: Option<String>,    // age1...
    pub fingerprint_md5: Option<String>,
    pub fingerprint_sha256: Option<String>,
    pub created_at: SystemTime,
    pub registered_groups: Vec<String>,      // Groups where recipient was registered
}
```

**Audit Integration:**
- All fields (except private key) logged to audit trail
- Public recipient hashed for telemetry (not plaintext)
- Registration events tracked separately

### KeygenService
Primary key generation interface:
```rust
pub struct KeygenService {
    config: Option<AgeConfig>,
    audit_logger: Option<AuditLogger>,
}

impl KeygenService {
    pub fn new(config: Option<AgeConfig>) -> Self;
    pub fn generate(&self, request: &KeygenRequest) -> Result<KeygenSummary, KeygenError>;
    pub fn config(&self) -> Option<&AgeConfig>;
}
```

**Implementation Status:**
- **Current:** Stub implementation (returns unimplemented error)
- **Phase 1 (CAGE-21):** CLI wrapper delegating to `age-keygen` binary
- **Phase 2 (CAGE-22):** Library-native via `age::x25519::Identity::generate()`

## Generation Capabilities

### MVP (CAGE-21) - CLI Wrapper
From KEYGEN_STRATEGY.md §2:
1. **Identity Generation** — Wrap `age-keygen`, capture stdout
2. **Public Key Extraction** — Run `age-keygen -y` for recipient derivation
3. **File Management** — Write to default/custom paths with secure permissions
4. **JSON Output** — Emit structured summary to stdout + audit logs
5. **Error Handling** — Detect missing binary, provide actionable guidance

### Recipients-Only Mode
From KEYGEN_STRATEGY.md §2.6:
```rust
let request = KeygenRequest {
    recipients_only: true,
    input_identity: Some(PathBuf::from("/keys/identity.age")),
    ..Default::default()
};

let summary = keygen.generate(&request)?;
// Converts identity to public recipient without persisting secrets
```

### Proxy Mode
From KEYGEN_STRATEGY.md §2.7:
```bash
# Direct passthrough to age-keygen
cage keygen --proxy [args...]

# Or via environment
CAGE_KEYGEN_PROXY=age cage keygen
```
**Use Case:** Fallback when Cage wrapper has issues or for advanced `age-keygen` flags.

## CLI Flag Matrix

From KEYGEN_STRATEGY.md §4:

| Flag | Short | Description | Backend Behavior |
|------|-------|-------------|------------------|
| `--output <path>` | `-o` | Explicit output path | Creates dirs, enforces permissions |
| `--force` | `-f` | Allow overwrite | Only with `--output`, guarded otherwise |
| `--register <group>` | (repeatable) | Register with recipient groups | Validates group, appends recipient |
| `--recipients-only` | `-y` | Convert identity to recipient | Accepts `--input`/stdin, skips secrets |
| `--input <path>` | - | Identity path (for `-y` mode) | Falls back to stdin if omitted |
| `--stdout-only` | - | Print without file write | Mutually exclusive with `--register` |
| `--json` | - | Emit JSON (default on) | `--no-json` disables for scripts |
| `--proxy` | - | Force `age-keygen` passthrough | Bypasses Cage logic |

## Usage Patterns

### Basic Key Generation (MVP)
```rust
use cage::keygen::{KeygenService, KeygenRequest};
use std::path::PathBuf;

let keygen = KeygenService::new(Some(config));

// Default output path with timestamp
let request = KeygenRequest::default();
let summary = keygen.generate(&request)?;

println!("Identity created: {}", summary.output_path.unwrap().display());
println!("Public recipient: {}", summary.public_recipient.unwrap());
```

### Custom Output with Group Registration
```rust
let request = KeygenRequest {
    output_path: Some(PathBuf::from("/keys/admin.age")),
    register_groups: vec!["admins".to_string(), "emergency".to_string()],
    force_overwrite: true,
    ..Default::default()
};

let summary = keygen.generate(&request)?;
println!("Registered with groups: {:?}", summary.registered_groups);
```

### Recipients-Only Conversion
```rust
// Convert existing identity to public recipient
let request = KeygenRequest {
    recipients_only: true,
    input_identity: Some(PathBuf::from("/keys/identity.age")),
    stdout_only: true,  // Print to stdout, no file
    ..Default::default()
};

let summary = keygen.generate(&request)?;
println!("Recipient: {}", summary.public_recipient.unwrap());
```

### Programmatic Identity (Future - CAGE-22)
From KEYGEN_STRATEGY.md §5:
```rust
// Library-native generation (post AGE-01)
use cage::keygen::KeygenService;

let service = KeygenService::new(Some(config));
let request = KeygenRequest {
    stdout_only: true,  // In-memory only, no disk write
    ..Default::default()
};

let summary = service.generate(&request)?;
// Returns GeneratedIdentity { private: String, public: String }
// Private key zeroized on drop
```

## Safety & Security

### Secure Defaults (§6 from KEYGEN_STRATEGY.md)
1. **File Permissions:**
   - Unix: `chmod 0o600` immediately after write
   - Windows: `FILE_ATTRIBUTE_HIDDEN` + `FILE_ATTRIBUTE_ARCHIVE`
2. **Overwrite Protection:**
   - Default path refuses overwrite without `--force`
   - Custom `--output` requires explicit `-f` flag
3. **Audit Logging:**
   - Never log private key material
   - Store only metadata (path, fingerprints, timestamp)
   - Public recipients hashed for telemetry
4. **Secret Handling:**
   - JSON output omits private key unless `--stdout-only`
   - User explicitly receives secret when requested

### Error Handling (§2.8)
- **Missing Binary:** "age-keygen not found. Install from: https://age-encryption.org"
- **Overwrite Refused:** "File exists: /path/to/key.age. Use --force to overwrite."
- **Invalid Group:** "Recipient group 'admins' not found in config."
- **Permission Denied:** "Cannot write to /keys/. Check permissions."

## Integration Points

### AgeConfig Integration (§2.5)
```rust
// Register generated identity with recipient group
let mut config = AgeConfig::default();
let group = config.get_recipient_group_mut("admins")?;
group.add_recipient(Recipient::X25519(summary.public_recipient.unwrap()));
config.save()?;
```

### Audit Logging (§6)
All keygen operations emit structured events:
- `keygen_start` — Request parameters (path, groups)
- `keygen_complete` — Summary (public recipient, fingerprints, timestamp)
- `group_registration` — Group name, recipient added
- Sensitive fields (private key) never logged

### Recipient Group Workflow
From KEYGEN_STRATEGY.md §2.5:
```bash
# Generate key and register with group
cage keygen --register admins --register emergency

# Verify registration
cage config recipient-groups list
# Shows "admins" and "emergency" with new recipient
```

## Future Enhancements

### CAGE-22: Library-Native Generation (§5)
From KEYGEN_STRATEGY.md:
- **Adapter Hook:** `AgeAdapterV2::generate_identity()` using `age::x25519::Identity::generate()`
- **Zeroization:** Private keys zeroized on drop
- **In-Memory:** Library consumers get `GeneratedIdentity` without disk writes
- **CLI Parity:** `cage keygen` detects library support, skips subprocess
- **Migration Guide:** Documented for Padlock/Ignite consumers

### Advanced Features (§8)
- **Comment Field:** `--comment <text>` for key annotation (like ssh-keygen)
- **Backup Integration:** Snapshot pre-existing keys before overwrite
- **GUI/TTY Prompts:** Passphrase-protected identities (if upstream adds support)

## Performance Characteristics
- **CLI Wrapper (CAGE-21):** ~100-200ms (subprocess overhead)
- **Library-Native (CAGE-22):** <10ms (in-process generation)
- **File I/O:** ~5-10ms (write + chmod)
- **JSON Serialization:** <1ms

## Limitations
- **Current:** Stub implementation (not functional)
- **MVP:** Requires `age-keygen` binary on PATH
- **No Rotation:** Advanced key rotation deferred to future work
- **Single Format:** Only X25519 keys supported (Age standard)

## Testing

### Planned Test Coverage (CAGE-21 §2.9)
1. **Success Path:**
   - Default output with timestamp
   - Custom `--output` path
   - `--force` overwrite
   - Group registration via `--register`
2. **Error Cases:**
   - Missing `age-keygen` binary (actionable error)
   - Overwrite refused (without `--force`)
   - Invalid group name
   - Insufficient permissions
3. **Recipients-Only Mode:**
   - Stdin input, stdout output
   - `--input` file path
   - Skip secret output
4. **Audit Logging:**
   - Verify log entries for keygen events
   - Confirm no secrets in logs

**Test Files:**
- `tests/cli/test_keygen.rs` — CLI integration tests
- `src/keygen/tests.rs` — Module-level unit tests
- `tests/integration/keygen_workflow.rs` — End-to-end scenarios

## Module Architecture

From KEYGEN_STRATEGY.md §3:
```
src/keygen/
├── mod.rs          // Public surface (KeygenService, KeygenRequest)
├── api.rs          // Core generation logic
├── error.rs        // KeygenError type
├── helpers.rs      // Fingerprint computation, permission helpers
├── audit.rs        // Audit event emission
└── tests.rs        // Unit tests
```

**Plugin Surface (§3):**
- CLI command (`cage keygen`) delegates to `KeygenService`
- Enables future extraction into standalone tool
- Follows `MODULE_SPEC.md` guidelines

## Status
- STUB: Yes (Implementation pending)
- TASK_REFS:
  - **CAGE-21** — CLI Keypair Generation Workflow (MVP)
  - **CAGE-22** — Adapter Identity Generation Hook (Library-Native)
- SPEC_ALIGNED: Yes (design follows RSB MODULE_SPEC v3)

## Changelog
- 2025-10-01: Enhanced documentation with comprehensive design from KEYGEN_STRATEGY.md
  - Added MVP requirements (§2) and CLI flag matrix (§4)
  - Detailed library-native path (§5) and safety considerations (§6)
  - Integration points with AgeConfig, audit logging, recipient groups
  - Test coverage plan and module architecture (§3)
  - Task mapping to CAGE-21/CAGE-22

## Reference Documentation

This feature specification is informed by the following design documents:

- [`docs/ref/cage/KEYGEN_STRATEGY.md`](../ref/cage/KEYGEN_STRATEGY.md) — Complete key generation strategy (CAGE-21/22)

### Additional Resources

- `docs/procs/TASKS.txt` — CAGE-21 (CLI workflow), CAGE-22 (adapter hook)
- Age Encryption Spec: https://age-encryption.org/
- `src/core/requests.rs` — Request API patterns

## API Surface

<!-- feat:keygen -->

_Generated by bin/feat2.py --update-doc._

* `src/keygen/api.rs`
  - struct KeygenRequest (line 14)
  - struct KeygenSummary (line 35)
  - struct KeygenService (line 50)
  - fn new (line 56)
  - fn generate (line 61)
  - fn config (line 67)

* `src/keygen/error.rs`
  - enum KeygenError (line 7)

* `src/keygen/mod.rs`
  - pub use api::{KeygenRequest, KeygenService, KeygenSummary} (line 12)
  - pub use error::KeygenError (line 13)

<!-- /feat:keygen -->

