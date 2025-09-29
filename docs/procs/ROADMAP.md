# Cage Roadmap – Stakeholder MVP Alignment

## Overview

Stakeholders expect Cage to deliver a first-class **library + CLI** experience with
feature parity and secure operational guarantees. The current implementation meets
only a subset of those expectations (primarily CLI flows and ASCII armor). This
roadmap defines the milestones, strategies, and tasks required to reach the
minimum viable product expected by stakeholders.

### Guiding Principles
- **API parity**: every CLI capability must be accessible through an ergonomic
  Rust API without invoking the binary.
- **Streaming-first**: large-file automation must avoid temporary files and
  offer streaming encrypt/decrypt (Reader/Writer based).
- **Key agility**: support deterministic keys, SSH identities, and multiple
  recipients with lifecycle management primitives.
- **Security posture**: centralize string literals to reduce binary snooping
  and ensure secrets never leak via compiled artifacts or logs.
- **Testability**: maintain end-to-end coverage for both CLI and library
  surfaces, with optional gating when external binaries (age) are required.

### Stakeholder MVP Requirements
1. Cage must be fully usable as both a CLI and a library with parity across surfaces.
2. A comprehensive configuration surface (`CageConfig`) captures operational knobs.
3. Streaming encrypt/decrypt flows are first-class, avoiding temporary file staging.
4. SSH identities are supported for encrypt/decrypt workflows.
5. ASCII armor output remains available for applicable environments.
6. Deterministic/derived key workflows (`age --derive`) are supported.
7. Multi-recipient lifecycle management (structs, helpers, auditing) is provided.

## Phase 1 – API Foundations & Parity

**Objective:** Close the gap between CLI and library usage by introducing
structured configuration and operation requests.

### Milestone 1.1 – CageConfig + Operation Requests
*Strategy*
- Introduce `CageConfig` (extends `AgeConfig`) capturing passphrase handling,
  recipients, SSH identities, streaming options, and logging preferences.
- Replace ad-hoc CLI flag plumbing with typed structs (e.g. `LockRequest`,
  `UnlockRequest`, `KeyRotateRequest`). These structs become the single source
  for both library calls and CLI dispatch.
- Update `CrudManager` methods to accept request structs instead of primitive
  parameter lists.
- Maintain backwards compatibility via helper constructors in the CLI module.

### Milestone 1.2 – Adapter Abstraction Refresh
*Strategy*
- Extend `AgeAdapter` trait to support both file-based and stream-based
  operations (`encrypt_file`, `encrypt_stream`, `decrypt_to_writer`).
- Provide `ShellAdapter` implementation using age binary while keeping PTY
  automation for passphrase flows.
- Introduce feature-flagged `NativeAgeAdapter` stub for future “rage” or
  library-backed integrations (still optional, but interface ready).

### Milestone 1.3 – Test & Documentation Sync
*Strategy*
- Document the new API in `docs/LIBRARY_USAGE.md` with parity examples.
- Build unit tests for each request struct and adapter method.
- Update CLI integration tests to call into the same request pipeline.

## Phase 2 – Advanced Encryption Capabilities

**Objective:** Deliver the remaining stakeholder requirements: streaming,
SSH identities, deterministic keys, and multi-recipient lifecycle features.

### Milestone 2.1 – Streaming Encryption & Decryption
*Strategy*
- Implement streaming encrypt/decrypt using the age binary’s stdin/stdout
  flows (`age -p -o -` etc.) or via pipes.
- Expose `encrypt_stream` / `decrypt_stream` on `CrudManager` and integrate
  with `LockRequest` / `UnlockRequest` (e.g. set `source` to `Stream` vs
  `Path`).
- Provide fallbacks for environments where streaming is unsupported.
- Add regression tests using in-memory buffers and large test files.
- Document passphrase streaming limitations: age requires PTY for passphrases, so pipe streaming is only available for recipient/identity flows (see `.analysis/CAGE-12b_investigation.md`).

### Milestone 2.2 – SSH Identity Support
*Strategy*
- Extend configuration to accept SSH private keys (paths or raw strings).
- Implement parsing and validation (leveraging `age` CLI for conversion or
  using `age::ssh` modules if adopting a library dependency).
- Ensure both encrypt and decrypt flows honor identity files, with tests that
  feed known SSH key material.
- Update CLI to accept `--identity` flags while library consumers set
  identities via `CageConfig`.

### Milestone 2.3 – Deterministic/Derived Keys
*Strategy*
- Add support for age’s `--derive` functionality (deterministic keys derived
  from passphrases + salt).
- Extend request structs with derivation parameters (salt, recipients, reason).
- Document security implications and add opt-in gating.
*Status:* Deferred until Phase 4 (library adapter). No immediate Padlock/Ignite dependency; revisit when AGE-01 lands.

### Milestone 2.4 – Multi-Recipient Lifecycle
*Strategy*
- Redesign `LockRequest` to accept `Vec<Recipient>` (typed wrapper) and ensure
  CLI/config paths populate it.
- Support recipient groups (e.g. team + audit) with helper enums or builder
  patterns.
- Update unlock flows to reason about recipient metadata (for auditing) and
  provide lifecycle helpers (add/remove recipients, inspect metadata).
- Add tests ensuring all recipients can decrypt (fixtures with generated keys).
*Ignite/Padlock Priority:* Required for repo/ignition/distro rotations. Align recipient group design with `docs/ref/ignite/IGNITE_CONCEPTS.md` and Padlock authority chain expectations.

## Phase 3 – Hardening & Tooling

**Objective:** Strengthen security posture, testing, and developer ergonomics.

### Milestone 3.1 – String Management & Binary Snooping
*Strategy*
- Introduce `src/cage/strings.rs` (or a dedicated module) containing all
  user-visible string literals and sensitive command fragments.
- Audit the codebase for inline `"..."` constants; migrate to the string
  module or localized `const` definitions.
- Provide tooling (`cargo tint` or custom lint script) to detect new inline
  strings in critical modules.
- Update build pipeline notes to highlight the requirement.

### Milestone 3.2 – Observability & Telemetry
*Strategy*
- Expand audit logging to capture streaming, identity, and recipient events.
- Ensure logs never leak passphrases or derived material.
- Add structured logging (JSON option) for downstream systems.

### Milestone 3.3 – QA & Release Readiness
*Strategy*
- Gate age-dependent tests (BUG-04 regression suite, streaming tests) to
  skip gracefully when the binary is unavailable.
- Establish end-to-end smoke tests covering CLI + library parity.
- Update documentation (`README`, `LIBRARY_USAGE`, new “MVP Feature Matrix”).
- Prepare migration guide for downstream consumers (Padlock, Ignite, etc.).
- Document that PTY-dependent tests auto-skip when the runtime sandbox denies PTY access.

### Milestone 3.4 – Backup Retention & Recovery (CAGE-03)
*Strategy*
- Implement the retention architecture captured in `docs/ref/cage/BACKUP_RETENTION_DESIGN.md`.
- Wire `BackupManager` to config-driven retention policies (keep-last / keep-days / hybrid).
- Introduce the optional JSON registry for listing and pruning prior backups.
- Defer CLI helpers (`cage backup list|restore|cleanup`) until Phase 4 unless explicitly prioritized.

## Phase 4 – Native Age Library Backend (Planned)

**Objective:** Replace shelling out to the `age` binary with direct use of the Rust `age` crate while keeping a CLI fallback for environments that require it.

### Milestone 4.1 – Library Adapter Parity
- Implement a new adapter that wraps `age::Encryptor` / `age::Decryptor` for both file and streaming operations.
- Maintain request/response semantics so CrudManager and existing request structs stay unchanged.
- Keep the shell-based adapter available as a configurable fallback until parity is validated.

### Milestone 4.2 – Passphrase & Plugin Integration
- Reuse PassphraseManager/PTY tooling to provide passphrases directly to the library API.
- Detect and surface plugin availability (e.g., YubiKey) without relying on CLI probes.
- Offer configuration flags/env toggles to choose between library and CLI backends per deployment.

### Milestone 4.3 – Validation & Rollout
- Add regression tests that compare CLI vs library output for representative workflows (passphrase, multi-recipient, SSH, armor, streaming).
- Document backend selection, limitations, and fallback guidance in README and Library Usage docs.
- Promote the library backend to default once automated tests and UAT confirm parity; retain the CLI path as an optional escape hatch.

## Milestone Summary & Dependencies

| Phase | Milestone | Depends On | Outcomes |
|-------|-----------|------------|----------|
| 1 | CageConfig & Operation Requests | None | Typed API, CLI routing parity |
| 1 | Adapter Refresh | 1.1 | Trait supports file + stream operations |
| 1 | Docs & Tests Sync | 1.1, 1.2 | Documentation + regression suite |
| 2 | Streaming Support | 1.x | Stream encrypt/decrypt for CLI & API |
| 2 | SSH Identity Support | 1.x | Key flexibility |
| 2 | Deterministic Keys | 1.x | Derivation workflows |
| 2 | Multi-Recipient Lifecycle | 1.x | Group encryption ready |
| 3 | String Management | 1.x | Binary snooping mitigation |
| 3 | Observability | 1.x, 2.x | Enhanced audit + telemetry |
| 3 | QA & Release Prep | All | MVP readiness |
| 4 | Native Age Library Backend | 1.x, 2.x, 3.x | Direct crate usage with CLI fallback |

## Risk & Mitigation Highlights
- **Binary compatibility**: Running the age binary remains a dependency.
  Mitigate with gating tests and documenting prerequisites.
- **API churn**: Introducing request structs may break existing callers.
  Provide transitional constructors and mark deprecated methods clearly.
- **Security regressions**: Streaming and identity support touch sensitive
  code paths. Require code review checklist updates and threat modeling.
- **String centralization delays**: Incremental adoption with linting will
  prevent regressions while work completes.

## Success Criteria
- Library consumers can perform all operations (lock/unlock/rotate/status) with
  passphrases, identities, derived keys, and multi-recipient lists.
- CLI and API share the same request/response structures.
- Streaming encrypt/decrypt benchmarks show no temporary file usage.
- All user-facing strings sourced from dedicated constants module.
- End-to-end tests covering every stakeholder requirement pass on CI with
  age installed; degraded mode skips gracefully when absent.
