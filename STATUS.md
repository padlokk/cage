# Cage Readiness Verification – 2025-09-30

## Overview
- Scope: end-to-end review of Cage CLI + library parity claims on branch `main` (commit a567176 per CONTINUE.md).
- Result: core encryption workflows work, but several parity and hardening gaps remain before declaring production readiness for both surfaces.

## Evidence Collected
- Process documents hydrating current status (`docs/procs/PROCESS.txt`, `CONTINUE.md`, `TASKS.txt`).
- Source review of CLI entry point (`src/bin/cli_age.rs`) and library modules under `src/cage/`.
- Full test run: `cargo test --all` (passes with PTY/age gating warnings, one retention test ignored pending bug).
- Targeted CLI regression checks: recursive lock now succeeds under `--ignored`; glob filtering still fails without `--recursive`.

## CLI Findings
- ✅ Primary commands (`lock`, `unlock`, `status`, `rotate`, `verify`, `batch`, `proxy`, `config`, `adapter`) dispatch via shared request/CrudManager paths and honour recipient + SSH options.
- ⚠️ `cage init` / `cage install` remain placeholders with TODOs and do not provision config directories or dependency checks.
- ⚠️ Streaming strategy flags only set `CAGE_STREAMING_STRATEGY`; actual encrypt/decrypt still default to file-based flows unless callers drop into adapter streaming APIs.
- ⚠️ Glob filtering regression test still fails because directory operations require `--recursive`; revisit CLI UX for include/exclude patterns.

## Library Findings
- ✅ `CrudManager` supports multi-recipient/SSH workflows and is covered by integration tests.
- ✅ `RotateRequest` now flows through `CrudManager::rotate_with_request`; the CLI consumes the same builder, keeping parity with the legacy path.
- ✅ Streaming reader/writer support is exposed via `CrudManager::stream_with_request`, and the CLI now provides `cage stream` for parity with the streaming adapters.
- ✅ `StatusRequest` and `BatchRequest` share the same CRUD pathways as the CLI, and verify now honours deep verification with passphrase/identity inputs.
- ⚠️ Backup retention registry stores stale paths when conflict resolution renames backups; `test_backup_manager_with_retention` stays ignored.
- ✅ Deterministic(derived) key workflows remain backlogged (CAGE-15) by design due to upstream `age` limitations—no change required.

## Documentation Gaps
- `docs/ref/cage/LIBRARY_USAGE.md` Feature Status still lists multi-recipient lifecycle as "not yet implemented" despite CAGE-16 completion.

## Testing Notes
- `cargo test --all` passes; warnings highlight unused imports in older modules and the single ignored retention test.
- Manual invocation of ignored CLI tests confirms recursion flows now pass (`test_recursive_directory_lock`) while glob filtering still needs work.

## Recommended Follow-Up
1. Finish backup registry conflict handling and re-enable `test_backup_manager_with_retention`.
2. Implement the real logic behind `cage init` (CAGE-20) for XDG scaffolding.
3. Refresh `docs/ref/cage/LIBRARY_USAGE.md` feature table to reflect current state.

Derived-key support remains deferred until the upstream age crate exposes a stable API (tracked in CAGE-15); no immediate action required beyond monitoring.
