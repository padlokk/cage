# Next Tasks Reference

## CLI-01: RSB Flag Alignment [3 pts] 🔴
- **Location:** Phase 3 Optional Enhancements
- **Key Files:**
  - `src/bin/cli_age.rs`
  - `src/cage/requests.rs`
- **Objectives:**
  - Add backwards-compatible CLI flag aliases
  - Update request structs for lock/unlock
  - Align with RSB options macro
- **Impact:** Enables QA-02 regression tests

## CAGE-03: JSON BackupRegistry [5 pts] 🟡
- **Reference:** `docs/ref/cage/BACKUP_RETENTION_DESIGN.md`
- **Key Files:**
  - `BackupManager` implementation
  - `AgeConfig` configuration
- **Remaining Tasks:**
  - Implement JSON-backed BackupRegistry
  - Add backup generation tracking
  - Create backup discovery helpers
  - Write integration tests
- **Priority:** Support Ignite manifest audits

## OBS-01: Structured Audit & Telemetry [3 pts] 🔴
- **Key Files:**
  - `AuditLogger` implementation
  - Configuration system
- **Objectives:**
  - Capture structured metadata for events
  - Redact sensitive fields
  - Add JSON/structured output toggle
- **Implementation Notes:**
  - Extend to emit JSON lines
  - Add `telemetry.format` config
  - Reference `docs/ref/ignite/AUTHORITY_PROTOCOL.md`
- **Priority:** High (Padlock/Ignite need machine-readable audit trails)

## CAGE-12: Identity Streaming Gaps [5 pts] 🟡
- **Reference:**
  - `src/cage/adapter_v2.rs`
  - `docs/ref/cage/LIBRARY_USAGE.md`
- **Remaining Tasks:**
  - Implement identity-based streaming encrypt
  - Mark current limitation explicitly
  - Ensure `health_check` exposes accurate streaming support
- **Notes:**
  - Critical for Ignite key rotation flows
  - Requires clear documentation of streaming capabilities

## Additional Context
- **Current Version:** 0.5.0
- **Roadmap:** `docs/procs/ROADMAP.md`
- **Story Point Scale:**
  - 1 pt = 1-2 hours
  - 3 pts = 4-6 hours
  - 5 pts = 1-2 days
  - 8 pts = 3-5 days

**DISCLAIMER:** This summary reflects the project state as of 2025-09-29 and may require updates during implementation.