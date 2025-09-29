# Age Library Adoption Strategy

_Last updated: 2025-09-29_

## 1. Overview

We currently shell out to the `age` CLI for all cryptographic work. The long-term goal is to consume the Rust `age` crate directly while retaining the CLI path as an optional fallback. This document captures the migration strategy, outlines gaps in the crate, and enumerates the Cage changes required to support both backends in parallel.

## 2. Capability Assessment

### 2.1 `age` crate (0.11.1)
- ✅ Stream & file encrypt/decrypt (Reader/Writer APIs)
- ✅ ASCII armor support
- ✅ X25519 recipients & identities
- ✅ SSH recipient/identity helpers (`age::ssh` feature)
- ✅ Plugin infrastructure (`plugin` feature)
- ✅ Passphrase mode (expects passphrase reader)
- ⚠️ Deterministic key derivation (`age::Encryptor::with_derive`) lives behind `age-core/unstable`
- ❌ No built-in UX (TTY prompts, config layering, logging)

### 2.2 Gaps vs. Cage
- Passphrase UX: We must continue supplying passphrases via our PassphraseManager. No crate-provided PTY shim.
- Config/request ergonomics: remain Cage-specific; crate only exposes primitives.
- Capability reporting: CLI probing must be replaced with crate-based checks.
- Deterministic derive: API exists but is marked unstable; we must guard or isolate usage.

## 3. Parallel Backend Plan

### 3.1 Adapter abstraction
We already route operations through `AgeAdapter`. The strategy is to add a new `LibraryAdapter` that wraps `age::Encryptor` / `age::Decryptor`. Adapter selection becomes configurable:
- Default: CLI adapter (until parity proven)
- Optional: library adapter (flag or config key)

### 3.2 Configuration hook
Add a switch (e.g., `backend = "cli" | "library"`) inside `AgeConfig` and command-line flag `--backend`. CrudManager will instantiate the appropriate adapter via `AdapterFactory`.

### 3.3 Passphrase delivery
Reuse existing PassphraseManager/PTY automation to gather passphrases. For the library path we bypass PTY entirely, passing the collected string into the crate APIs. PTY automation remains required when the CLI backend is chosen.

### 3.4 Plugin discovery
The CLI path introspects plugins via `age --version` output. For the library path we must:
- Enumerate plugin directories (shared logic with CLI path)
- Expose capability booleans based on discovered binaries
- Ensure parity between health/capability reporting for both backends

## 4. Migration Tasks

These tasks align with Phase 4 entries (AGE-01..04) in `docs/procs/TASKS.txt`:

1. **AGE-01 – Library Adapter Implementation**
   - Implement `LibraryAdapter` bridging CrudManager requests to `age::Encryptor`/`Decryptor`
   - Support file + streaming workflows
   - Keep CLI adapter available until parity validated

2. **AGE-02 – Passphrase & Identity Bridging**
   - Integrate PassphraseManager with the crate-based adapter
   - Map SSH identities/recipients to `age` crate types
   - Add config/flag to choose backend per environment

3. **AGE-03 – Capability & Plugin Surface**
   - Mirror capability reporting using crate introspection and explicit configuration
   - Detect optional plugins (e.g., `age-plugin-yubikey`) without running the CLI
   - Document fallback matrix and environment requirements

4. **AGE-04 – Parity Validation & Rollout**
   - Regression suite comparing CLI vs library outputs for representative workflows
   - Documentation updates (README, Library Usage) describing backend selection & limitations
   - Promote library backend to default once parity is proven; retain CLI path as opt-in escape hatch

## 5. Deterministic Key Derivation Strategy (CAGE-15)

- Short term: implement derive support using CLI backend (`age --decrypt --derive`).
- Library migration: once AGE-01 lands, add a thin wrapper that enables the `age-core/unstable` feature conditionally.
- Guard usage behind feature flag/config toggle to allow downgrading if upstream changes.
- Regression tests: ensure CLI & library derive results match for deterministic vectors.

## 6. Testing & Validation

- Dual-backend test suite: run core integrations twice (CLI + library) in CI.
- Benchmark parity: ensure streaming throughput remains comparable (document deviations).
- Plugin coverage: test scenarios with and without plugin binaries present.
- UAT checklist: confirm Ignition/Padlock scenarios (multi-recipient, derive, SSH) pass against both backends.

## 7. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Crate `unstable` derive API changes | Encapsulate derive logic behind trait + feature flag; monitor upstream release notes |
| Plugin behaviour differences | Keep CLI fallback until plugin handling validated; add health diagnostics |
| Regression complexity | Expand automation test matrix; start with allow-listed regression groups |
| Configuration drift | Centralize backend selection in `AgeConfig` + CLI flag; document defaults |

## 8. Rollout Checklist

1. Deliver AGE-01..AGE-04 tasks with tests.
2. Update docs and CHANGELOG.
3. Run full regression matrix (CLI + library) on staging hardware.
4. Enable library backend by default; mark CLI backend as fallback.
5. Monitor downstream (Ignition, Padlock). If issues arise, switch back via config.

## 9. References

- `docs/procs/TASKS.txt` (Phase 4 – Age Library Migration)
- `docs/procs/ROADMAP.md` (Phase 4 milestones)
- `src/cage/adapter_v2.rs` (current adapter implementation)
- `age` crate documentation: https://docs.rs/age
- `rage` repository: https://github.com/str4d/rage

