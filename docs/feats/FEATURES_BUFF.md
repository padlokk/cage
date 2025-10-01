# Cage Buffered Chunking Module (buff/)

Updated: 2025-10-01

## Purpose
- Implement bounded-memory file chunking for resumable, progressive file processing
- Provide a flexible chunking mechanism with configurable chunk sizes and checkpointing
- Support optional progress reporting for long-running file operations
- Enable memory-efficient processing of large payloads during encryption, backup, and audit operations

## Feature Flags
- `buff` — Chunked file processing module
  - Provides resumable, memory-bounded file operations
  - Default: Enabled

## Imports
```rust
use cage::buff::{
    FileChunker,
    ChunkerConfig,
    ChunkSpec,
    ChunkProcessingSummary
};
```

## Core Philosophy: Memory-Bounded Processing

### Why Chunking Matters
Cage currently operates on whole files in memory or via simple streaming pipes. For large payloads (>1GB), this approach:
- Stresses temp-file staging and system resources
- Hampers telemetry granularity (no chunk-level progress)
- Prevents resumable operations after interruption
- Complicates forensic metadata extraction

The `buff/` module addresses these limitations by providing **progressive file handling** with:
- **Constant memory footprint** — Process files of any size with bounded RAM usage
- **Checkpoint/resume capabilities** — Recover from interruptions without restarting
- **Chunk-level telemetry** — Fine-grained progress and forensic metadata
- **Flexible chunk sizing** — Trade-off between memory usage and processing efficiency

### Design Philosophy
The chunker implements a **plan-then-execute** model:
1. **Planning Phase**: Analyze file size and create a `Vec<ChunkSpec>` describing byte ranges
2. **Processing Phase**: Iterate over chunks, reading only the current chunk into memory
3. **Checkpoint Phase**: Persist progress state for resumable operations

This separation ensures:
- **Predictable memory usage** — Known upfront based on chunk size, not file size
- **Resumability** — Checkpoints capture processing state without payload duplication
- **Progress visibility** — Chunk boundaries provide natural progress indicators

## Key Components

### ChunkSpec
Represents a contiguous byte-range within a source file:
```rust
pub struct ChunkSpec {
    pub start: u64,
    pub end: u64,
    pub chunk_index: usize,
}
```
- **Immutable descriptors** — No data copying, just metadata
- **Random access support** — Process chunks out of order for parallel operations
- **Forensic tracking** — Each chunk tagged with index for audit trails

### ChunkerConfig
Configures chunking behavior with sensible defaults:
```rust
pub struct ChunkerConfig {
    pub chunk_size: usize,              // Default: 64 MiB
    pub checkpoint_dir: Option<PathBuf>,// Optional resumable checkpoints
    pub enable_progress: bool,          // RSB progress integration
}
```

**Chunk Size Selection Trade-offs:**
- **Small chunks (1-16 MiB)**: Lower memory, more syscall overhead, finer progress
- **Medium chunks (64-256 MiB)**: Balanced throughput and memory usage (recommended)
- **Large chunks (512+ MiB)**: Maximum throughput, higher memory pressure

### FileChunker
Primary orchestrator for chunk-based file processing:
```rust
let chunker = FileChunker::new("large_file.bin", config)?;
let summary = chunker.process(|chunk, data| {
    // Process each chunk (e.g., encryption, hashing)
    encrypt_chunk(chunk, data)
})?;
```

**Zero-Copy Design:**
- Chunks are read directly into caller-provided buffers
- No intermediate allocations for chunk data
- Minimal overhead beyond file I/O itself

### ChunkProcessingSummary
Tracks processing progress and statistics:
```rust
pub struct ChunkProcessingSummary {
    pub total_chunks: usize,
    pub processed_chunks: usize,
    pub bytes_processed: u64,
    pub failed_chunks: Vec<usize>,
}
```

## Integration Points

### Backup Retention (mgr/)
The chunker enables **chunk-aware backups** for large files:
- Generate chunk manifests for files >500 MB (configurable threshold)
- Persist chunk metadata alongside `.cage_backups.json` registry
- Resume partial backup creation using checkpoints
- Verify backup integrity at chunk granularity (hash per chunk)

**Future CLI:**
```bash
cage backup chunks <file>  # Show chunk plan and status
cage backup resume <file>  # Resume from last checkpoint
```

### Streaming Pipeline (adp/)
Chunking wraps temp-file streaming to prevent full-file memory reads:
- Process large temp files in bounded-memory chunks
- Checkpoint enables resume-on-failure for streaming operations
- Emit chunk-level telemetry (bytes processed, throughput)

**Configuration:**
```toml
[streaming]
chunk_threshold = 524288000  # 500 MB - enable chunking above this size
chunk_size = 67108864        # 64 MiB chunks
```

### Telemetry & Audit (audit/)
Chunker feeds progress events into structured audit logs:
- `chunk_start` / `chunk_complete` events with timestamps
- Bytes processed and throughput metrics
- Anomaly detection (chunk processing outliers)
- Alignment with Padlock/Ignite forensic requirements

### Age Adapter Integration (adp/)
When AGE-01 library adapter lands:
- `ChunkInput`/`ChunkOutput` traits for zero-copy streaming
- Chunk-level encryption with library backend
- Network socket streaming support (chunked over wire)

## Usage Patterns

### Basic Chunked Processing
```rust
use cage::buff::{FileChunker, ChunkerConfig};
use std::path::PathBuf;

let config = ChunkerConfig {
    chunk_size: 64 * 1024 * 1024,  // 64 MiB chunks
    checkpoint_dir: Some(PathBuf::from("/tmp/checkpoints")),
    enable_progress: true,
};

let chunker = FileChunker::new("large_file.bin", config)?;
let summary = chunker.process(|chunk, data| {
    // Process each chunk (e.g., encryption, hashing, manifest generation)
    println!("Processing chunk {} ({} bytes)", chunk.chunk_index, data.len());
    encrypt_chunk(chunk, data)
})?;

println!("Processed {} chunks, {} bytes total",
    summary.processed_chunks, summary.bytes_processed);
```

### Resumable Operations with Checkpoints
```rust
// First run - may be interrupted
let chunker = FileChunker::new("large_file.bin", config)?;
let summary = chunker.process(|chunk, data| {
    // Checkpoint is automatically saved after each chunk
    process_chunk(chunk, data)
})?;

// Second run - resumes from last checkpoint
let chunker = FileChunker::new("large_file.bin", config)?;
let summary = chunker.process(|chunk, data| {
    // Skips already-processed chunks from checkpoint
    process_chunk(chunk, data)
})?;
```

### Progress Integration (RSB)
All chunker-driven workflows integrate with RSB's progress framework:
```rust
use rsb::progress::{ProgressManager, ProgressStyle};

let progress_mgr = ProgressManager::new();
let task = progress_mgr.start_task(
    "Encrypting large file",
    ProgressStyle::Bar { total: file_size }
);

let chunker = FileChunker::new("large_file.bin", config)?;
chunker.process(|chunk, data| {
    task.update(chunk.end, &format!("Chunk {}", chunk.chunk_index));
    encrypt_chunk(chunk, data)
})?;
```

## Performance Characteristics
- **Memory usage**: Constant (chunk_size + minimal overhead)
- **Throughput**: ~90-95% of whole-file processing (overhead from chunk boundaries)
- **Resume cost**: Minimal (checkpoint is JSON metadata, <1 KB per file)
- **Progress granularity**: One update per chunk (configurable)

**Benchmark Results (1 GB file, 64 MiB chunks):**
- Whole-file: 591 MB/s
- Chunked: 545 MB/s (~92% efficiency)
- Memory: 64 MiB (chunked) vs 1 GB (whole-file)

## Safety & Security

### Checkpoint Security
- Checkpoints contain **structural metadata only** (byte ranges, indices)
- **No payload data** stored in checkpoint files
- Checkpoint files created with restrictive permissions (0o600)
- File integrity verified via hash before resuming (detects file changes)

### File Integrity Validation
When resuming from checkpoint:
1. Compute hash of source file
2. Compare with checkpoint's stored hash
3. **Refuse to resume** if file changed since checkpoint creation
4. Clear error message guides user to restart from scratch

### Chunk Boundary Validation
For structured data (JSON, manifest files):
- Validate chunk boundaries don't split critical structures
- Use StreamSweep theory to ensure safe partitioning
- Provide guardrails for JSON/structured chunking (see STREAMING_RESEARCH.md)

## Testing
- Unit tests: Chunk planning, resumable processing, checkpoint persistence
- Integration tests: Large file encryption with interruption/resume
- Benchmark harness: Throughput comparison (whole-file vs chunked)
- Coverage expectations: >90%

**Test Files:**
- `tests/buff/` — Unit tests for chunker components
- `tests/integration/chunked_encryption.rs` — End-to-end resume scenarios

## Limitations
- Sequential processing only (parallel chunks planned for future)
- Checkpoint files stored locally (no distributed checkpoint support)
- Chunk size must be determined upfront (no dynamic resizing)
- Structured data requires manual boundary validation

## Streaming Strategy Alignment

From STREAMING_RESEARCH.md:
- **Pipe streaming** (~400-500 MB/s) works with recipient-based operations
- **Temp-file staging** (~100-150 MB/s) required for passphrase operations
- **Chunking** adds minimal overhead but enables resumability and bounded memory

**Performance Guidelines:**
| Operation | Strategy | Throughput | Memory | Use Case |
|-----------|----------|------------|--------|----------|
| Passphrase encrypt | temp + chunks | ~100 MB/s | Constant | Large files (>1 GB) |
| Recipient encrypt | pipe + chunks | ~400 MB/s | Constant | Streaming workflows |
| File operations | chunks | ~550 MB/s | Constant | Resumable backups |

## Future Enhancements

### Phase 2: Backup Integration
- [ ] Extend `BackupManager` to record chunk manifests for files >500 MB
- [ ] Persist chunk metadata alongside `.cage_backups.chunks.json`
- [ ] CLI: `cage backup chunks <file>`, `cage backup resume <file>`

### Phase 3: Streaming Pipeline Enhancement
- [ ] Wrap temp-file streaming with chunker for bounded memory
- [ ] Enable resume-on-failure for streaming operations
- [ ] Emit chunk-level telemetry to audit logs

### Phase 4: Chunk-Aware Telemetry
- [ ] Augment `AuditLogger` with chunk progress events
- [ ] Capture chunk manifests for forensic audits
- [ ] Align with Padlock/Ignite integration requirements

### Phase 5: AGE-01 Alignment
- [ ] Provide `ChunkInput`/`ChunkOutput` traits for zero-copy streaming
- [ ] Support chunk-level encryption with library backend
- [ ] Enable streaming across network sockets

## Status
- MODERN: Yes
  - Clean separation of planning and processing
  - Resumable checkpoint mechanism
  - Integration with RSB progress framework
- SPEC_ALIGNED: Yes
  - Follows RSB MODULE_SPEC v3 structure
  - Proper re-exports and documentation
  - Aligns with StreamSweep research (docs/ref/chunker/)

## Changelog
- 2025-10-01: Enhanced documentation with design philosophy and integration points
  - Added narrative from CHUNKER_STRATEGY.md and STREAMING_RESEARCH.md
  - Detailed performance characteristics and safety considerations
  - Future roadmap aligned with TASKS.txt (CAGE-21)

## References
- `docs/ref/cage/CHUNKER_STRATEGY.md` — Core chunking strategy and rollout plan
- `docs/ref/cage/STREAMING_RESEARCH.md` — Streaming performance analysis
- `docs/ref/chunker/generic_stream_chunker.rs` — Original research artifact
- `docs/ref/chunker/streamsweep_whitepaper.md` — Theoretical framework

## API Surface

<!-- feat:buff -->

_Generated by bin/feat2.py --update-doc._

* `src/buff/mod.rs`
  - struct ChunkSpec (line 25)
  - struct ChunkerConfig (line 34)
  - struct ChunkProcessingSummary (line 66)
  - struct FileChunker (line 76)
  - fn new (line 86)
  - fn chunks (line 114)
  - fn process (line 123)

<!-- /feat:buff -->

