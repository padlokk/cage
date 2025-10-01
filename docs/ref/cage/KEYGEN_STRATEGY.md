# Cage Key Generation Strategy

_Last updated: 2025-10-01_

## 1. Objectives
- Deliver a first-class `cage keygen` workflow for generating and managing Age identities within Cage.
- Provide a single place to document CLI flags, config interactions, and logging expectations so implementation matches stakeholder intent.
- Ensure a safe fallback path exists (proxying to the raw `age-keygen` binary) until the native library adapter exposes in-process key generation.

## 2. MVP Requirements
1. **Primary Surface**: `cage keygen` command wrapping `age-keygen`.
2. **Output Handling**: Default path `${XDG_CONFIG_HOME}/cage/identities/<timestamp>.agekey` with `chmod 0o600` (or Windows equivalent) applied after write.
3. **Structured Result**: Emit JSON summary (`path`, `public_recipient`, `fingerprint_md5`, `fingerprint_sha256`, `created_at`, `registered_groups`) to stdout and mirror in audit logs.
4. **Safety Controls**: Refuse overwrite unless `--force`; human-readable error when destination exists.
5. **Registration Hook**: Optional `--register <group>` flag appends the derived recipient to configured recipient groups via `AgeConfig` helpers.
6. **Recipients Conversion**: Support `--recipients-only`/`-y` mode that accepts an identity path (or stdin) and emits recipients without persisting secrets.
7. **Export Mode**: Support `--export` flag to generate keypair in current directory without registry entry, useful for testing and one-off needs.
8. **Fallback Proxy Switch**: A config/env toggle (e.g. `CAGE_KEYGEN_PROXY=age`) forcing direct passthrough to the upstream binary, ensuring coverage if Cage logic regresses.
9. **Error Reporting**: Detect missing `age-keygen` with actionable guidance; redact secrets from surfaced stderr.
10. **Testing**: CLI integration tests for success, overwrite guard, forced overwrite, custom output, missing binary, export mode, and registration flows.

## 3. Module & Plugin Architecture
- The keygen logic lives under `src/cage/keygen/` following `MODULE_SPEC.md` guidelines:
  - `src/cage/keygen/mod.rs` orchestrates the public surface (`KeygenService`, `KeygenRequest`, `KeygenResult`).
  - `src/cage/keygen/plugin.rs` implements the executable plugin surface that wires CLI parsing to the service, enabling future extraction into a standalone tool.
  - `src/cage/keygen/error.rs`, `helpers.rs`, `audit.rs` (as needed) encapsulate logic separate from orchestration.
- CLI command (`cage keygen`) is a thin wrapper that delegates to `KeygenService` so the same module can be reused by other binaries or future daemons.
- Expose the plugin through the standard plugin registry (see `docs/ref/rsb/MODULE_SPEC.md`); ensure the module can be compiled independently for future spin-off binaries.
- Testing requirements: module-level unit tests inside `src/cage/keygen/tests.rs` plus CLI integration tests under `tests/cli/test_keygen.rs`.

## 4. CLI Flag Matrix
| Flag | Short | Description | Required Backend Behaviour |
|------|-------|-------------|----------------------------|
| `--output <path>` | `-o` | Persist identity/recipients to explicit path; refuse overwrite unless `--force`. | Create directories if needed, enforce permissions, log location. |
| `--force` | `-f` | Overwrite target file. | Only honoured with explicit `--output`; default path still guarded unless flag provided. |
| `--register <group>` | *(repeatable)* | Register public recipient with recipient group(s). | Validate group existence; append recipient; audit change. |
| `--recipients-only` | `-y` | Convert existing identity to recipients. | Accept `--input`/stdin, skip `--register`, skip JSON secret output. |
| `--input <path>` | *(none upstream)* | Optional helper for specifying identity path in `--recipients-only` mode. | Falls back to stdin if omitted. |
| `--stdout-only` | *(new)* | Print identity/recipients without writing files. | Mutually exclusive with `--register` and `--output`; still emits JSON summary. |
| `--export` | *(new)* | Generate keypair to current directory without registry entry. | Write `<timestamp>.agekey` to PWD, skip config store, skip `--register`, useful for testing/one-off needs. |
| `--json` | *(default)* | Emit structured JSON (default on). | `--no-json` disables JSON output for scripting compatibility. |
| `--proxy` | *(new)* | Force passthrough to raw `age-keygen`. | Equivalent to setting `CAGE_KEYGEN_PROXY=age`; bypass Cage wrapping logic. |

## 5. Library Adapter Expectations (Post AGE-01)
- Implement `AgeAdapterV2::generate_identity()` returning `GeneratedIdentity { private, public }` using `age::x25519::Identity::generate()` with zeroization on drop.
- `cage keygen` detects adapter support, skipping the subprocess while retaining identical CLI semantics/output.
- Provide `CageManager::generate_identity_with_request(request: &KeygenRequest)` for library consumers who need in-memory identities.

## 6. Safety & Security Notes
- Secrets written to disk must use restrictive permissions; on Unix call `std::fs::set_permissions` with `0o600` and on Windows set `FILE_ATTRIBUTE_HIDDEN`/`FILE_ATTRIBUTE_ARCHIVE` as appropriate.
- Audit log entries must never include private key material; store only metadata and file paths.
- JSON output should omit the private key unless `--stdout-only` is requested, in which case the user explicitly receives the secret on stdout.

## 7. Task Mapping
- **CAGE-21 (CLI Keypair Generation Workflow)** implements §2 items 1–6, 8–9. See `docs/procs/TASKS.txt` for acceptance criteria.
- **CAGE-22 (Adapter Identity Generation Hook)** implements §4 and removes dependency on the `age-keygen` binary when the library backend is active.
- Any regression or environment without full support can set `CAGE_KEYGEN_PROXY=age` or run `cage keygen --proxy` to leverage raw upstream behaviour (covers §2.7).

## 8. Open Questions / Future Enhancements
- Optional `--comment <text>` field mirroring ssh-keygen for annotating identities.
- Integration with backup registry to snapshot pre-existing keys before overwrite.
- Potential GUI/TTY prompt for passphrase-protected identities if upstream adds support.
