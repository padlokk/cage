# Cage Chunker Module Strategy

_Last updated: 2025-09-30_

## Goal

Design and stage a reusable “chunker” subsystem for Cage that enables bounded-memory processing of
large files during encryption, backup, and audit operations. The chunker should draw on the
StreamSweep/JSONM research artifacts (`docs/ref/chunker/`) while aligning with Cage’s architecture,
RSB patterns, and Age integration.

This document provides zero-context agents with the background, requirements, and rollout plan to
implement the module without rediscovering prior research.

---

## Background

### Why We Need It
- Cage currently operates on whole files in memory or via simple streaming pipes. Large payloads
  (>GB) stress temp-file staging and hamper telemetry.
- Backup retention and future AGE-01 adapters could benefit from chunk-aware processing (e.g., resume,
  partial verification, forensic metadata).
- Padlock/Ignite integration may need chunk-level manifests or recovery workflows.

### Available Research
- `docs/ref/chunker/generic_stream_chunker.rs`: Generic byte-range chunker with checkpoints, resumable
  handlers, and state accumulation.
- `docs/ref/chunker/jsonm_*`: Streaming JSON parser proving the viability of bounded-memory structural
  analysis plus forensic metadata extraction.
- `streamsweep_whitepaper.md`: Theoretical framework for O(1) streaming, multi-pass analysis, and
  impossibility proof for naive partitioning.
- `stream_chunker_examples.rs`: Ready-made use-cases (resume after interruption, random access, statistical
  accumulation) demonstrating how to wire the chunker into real operations.

**Takeaway:** We already have a tested, resumable chunking core. We need to adapt it for Cage’s file
operations and integrate it with our CLI, backups, and forthcoming AGE adapters.

---

## High-Level Strategy

1. **Abstract Chunker Core (`cage::chunker` module)**
   - Embed/port `StreamChunker` (including checkpointing) into Cage, namespaced under `src/cage/chunker/`.
   - Provide a generic trait `ChunkProcessor` so different consumers (backup sync, streaming encryptor,
     manifest generator) can supply their own handlers.
   - Harmonize serialization with Cage’s existing serde usage (e.g., `.cage_backups.json`).

2. **Integration Points**
   - **Backup Retention:** generate chunk manifests for large backups; allow resuming partial cleanup.
   - **Streaming Encrypt/Decrypt:** chunk files when using temp-file staging to cap memory, record progress,
     and support resume-on-failure (tying into Age’s CLI or future AGE-01 adapter).
   - **Telemetry:** feed chunk-level metrics (bytes processed, rate, anomalies) into audit logs.
   - **CLI Enhancements:** new subcommands (`cage chunk plan`, `cage chunk resume`, `cage chunk inspect`)
     to inspect and operate on chunk metadata.

3. **Documentation & Safety**
   - Leverage StreamSweep theory to guide safe partitioning (validate chunk boundaries when splitting JSON
     or structured data).
   - Provide guardrails (e.g., disable resuming if the source file changed since checkpoint).
   - Update README/docs to explain when chunking is used automatically vs. explicit commands.

4. **Roadmap Alignment**
   - Stage chunker work under a new task line (e.g., `CAGE-21: Chunker Integration`) with subtasks for core
     module, CLI, backup integration, and AGE backend support.
   - Coordinate with AGE-01 library adapter work: chunker should serve both CLI (current `age` binary) and
     future Rust-based implementations.

---

## Detailed Plan

### Phase 1 – Core Module Extraction ✅
- Ported `generic_stream_chunker.rs` into `src/cage/chunker/mod.rs` with Cage naming and serde checkpoints.
- Added unit tests for planning/processing and exposed the API via `cage::chunker::*`.
- Wired RSB’s terminal progress reporters (colors + Unicode) directly into chunk processing and removed Cage’s legacy progress module.
- Documented the module here and in the streaming research notes.

### Phase 2 – Backup Integration
- Extend `BackupManager` to optionally record chunk manifests for large backups:
  - On backup creation, run chunk planning (configurable threshold, e.g., >500 MB).
  - Persist chunk metadata alongside the registry (e.g., `.cage_backups.chunks.json`).
  - Expose methods to resume backup creation or verify chunk integrity (e.g., hash per chunk).
- CLI additions:
  - `cage backup chunks <file>` – show chunk plan, sizes, and status.
  - `cage backup resume <file>` – resume backup creation using checkpoint.

### Phase 3 – Streaming Pipeline Enhancement
- Wrap temp-file streaming (`encrypt_stream_temp`, `decrypt_stream_temp`) with chunker support:
  - Process large temp files in chunks to prevent full-file memory reads.
  - If an error occurs mid-stream, checkpoint allows resuming from the last completed chunk.
  - Emit chunk-level telemetry (bytes processed, throughput).
- Add knobs to configure chunk size (`cage.toml`, CLI flag) and disable chunking for small files.

### Phase 4 – Chunk-Aware Telemetry & Manifests
- Augment `AuditLogger` to record chunk progress events (start/end timestamps, bytes, anomalies).
- Extend Padlock/Ignite integration to capture chunk manifests for forensic audits (alignment with
  `streamsweep_whitepaper` goals).

### Phase 5 – AGE-01 Alignment
- Ensure chunker works with forthcoming Rust `age` adapter:
  - Provide `ChunkInput`/`ChunkOutput` traits that adapters can consume for zero-copy streaming.
  - Support chunk-level encryption with the library backend, including streaming across network sockets.

---

## Implementation Notes

- **Checkpoints:** reuse serde JSON for portability; consider optional encryption of checkpoint data.
- **File Integrity:** record a hash of the source file when checkpoints are created; refuse to resume if the
  hash changes.
- **Concurrency:** chunker currently operates sequentially. Future optimization: support multi-threaded chunk
  processing with a channel-based dispatcher (ensure determinism for telemetry).
- **Security:** ensure checkpoints do not leak sensitive data (only store structural metadata, not payload
  content).
- **Testing:** integrate with existing benchmark harness to validate chunked streaming throughput.
- **Progress UX:** all chunker-driven workflows should rely on `rsb::progress::{ProgressManager, TerminalReporter}`
  so CLI consumers get consistent, color-aware dashboards (no bespoke progress module in Cage).

---

## Documentation & Onboarding
- Add references to this strategy in PROCESS/QUICK_REF once the module is underway.
- Provide developer onboarding steps in `docs/ref/cage/CHUNKER_STRATEGY.md`:
  - Review StreamSweep whitepaper for theoretical context.
  - Study `generic_stream_chunker.rs` tests before modifying Cage’s core.
  - Use `cage chunk plan` CLI commands (to be implemented) for manual inspection.

---

## Open Questions
- Which operations should enable chunking by default (backups, streaming encrypt, verify)?
- How granular should chunk telemetry be to avoid overwhelming logs?
- Do we expose chunk manifests to end users or keep them internal to Cage/Padlock integration?

Feedback and design iterations should be captured in this document before implementation begins.
