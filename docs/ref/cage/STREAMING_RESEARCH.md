# Streaming Performance & Limitations (CAGE-12a/12b)

_Last updated: 2025-09-30_

## Overview

This document captures the current state of Cage's streaming implementation, observed performance
characteristics, and constraints inherited from the upstream `age` CLI. It summarizes the findings from:
- `.analysis/streaming_benchmark_results.md`
- `tests/test_streaming_benchmark.rs`
- Source analysis of `src/cage/adp/v2.rs` and `src/cage/adp/v2_pipe_passphrase.rs`

## Current Implementation

- `CrudManager::stream_with_request()` routes streaming operations through `ShellAdapterV2`
  (`src/cage/lifecycle/crud_manager.rs:1027`).
- For recipients, `ShellAdapterV2::encrypt_stream()` spawns the `age` CLI and copies data through its
  stdin/stdout (`src/cage/adp/v2.rs:982`). Buffering relies on `std::io::copy` (default 8 KB chunks)
  plus a scoped thread to drain stdout.
- Passphrase “pipe” support defers to `encrypt_stream_pipe_passphrase`, but that routine intentionally
  returns `InvalidOperation` because `age` demands to read secrets from a controlling TTY. The adapter
  therefore falls back to `encrypt_stream_temp()`, writing the entire payload to disk and running the
  PTY automator (`src/cage/adp/v2_pipe_passphrase.rs`).
- Streaming strategies are selected via `CAGE_STREAMING_STRATEGY` (`tempfile`, `pipe`, `auto`). For
  passphrases, `CAGE_PASSPHRASE_PIPE=1` is required even to attempt pipe mode (`src/cage/adp/v2.rs:557`).

## Benchmark Results (1 GB Test)

| Mode                       | Throughput | Duration | Notes                                      |
|---------------------------|-----------:|---------:|-------------------------------------------|
| File-based encryption     | 591 MB/s   |   1.73 s | Direct file I/O via `age` CLI             |
| File-based decryption     | 462 MB/s   |   2.22 s | Direct file I/O                           |
| Streaming (pipe/temp mix) | 119 MB/s   |   8.58 s | Uses pipe for recipients, temp file for passphrase |

(Source: `.analysis/streaming_benchmark_results.md`)

Key takeaways:
- Streaming achieves ~20–25 % of file-based throughput on 1 GB payloads.
- Integrity checks pass; functionality is correct.
- Memory usage is bounded (constant-size buffers), but disk staging occurs whenever passphrases are involved.

## Root Causes of Performance Gap

1. **CLI I/O model** – `std::io::copy` with an 8 KB buffer monopolizes stdin before draining stdout,
   leading to repeated flushes and context switches. File mode lets `age` handle both sides optimally.
2. **Extra data copies** – Data flows disk → Rust buffer → child stdin → `age` → Rust buffer → caller.
   File mode moves bytes directly between files under the CLI’s control.
3. **Process spawn overhead** – Each stream spawns a new `age` process. Until the AGE-01 library adapter
   lands, we cannot reuse an encryptor/decryptor across requests.
4. **Passphrase TTY requirement** – True pipe streaming is incompatible with passphrase mode because `age`
   insists on `/dev/tty`. Our temp-file fallback adds disk I/O and hashes the full payload twice.

## Constraints & Limitations

- Pipe streaming works only when explicit recipients or identities are provided. Passphrase flows always
  fall back to temp files.
- `age` does not support reading passphrases from stdin or environment variables; PTY automation is the
  secure workaround.
- Streaming benchmarks write output into an in-memory `Vec`, so throughput numbers reflect adapter/CLI
  overhead rather than disk speed.

## Mitigation Options (Near Term)

1. **Larger buffers** – Replace `std::io::copy` with a loop using a larger reusable buffer (e.g., 256 KB–1 MB).
   We already ship `StreamBuffer`; wiring it into `encrypt_stream_pipe`/`decrypt_stream_pipe` would reduce
   syscall overhead.
2. **Concurrent pumping** – Spawn dedicated threads (or async tasks) to read stdin and stdout concurrently,
   rather than serializing the operations. This keeps the `age` process busy and minimizes stalls.
3. **Configurable buffer size** – Introduce a configuration or env knob (`CAGE_STREAMING_BUFFER`) so ops teams
   can tune throughput based on workload.
4. **Enhanced logging** – Surface when the adapter falls back to temp-file mode (e.g., in verbose output) so
   operators understand the performance characteristics they’re seeing.

## Medium-Term Improvements

1. **AGE-01 (Rust adapter)** – Integrate the `age` crate directly. That removes the CLI bottleneck,
   enables true streaming with fine-grained control over buffering, and handles passphrases programmatically.
2. **Upstream engagement** – Request CLI support for passphrase input via file descriptor or env var,
   enabling true pipe streaming without PTY.
3. **Strategy heuristics** – Auto-select temp-file vs. pipe based on payload size or configured thresholds.
4. **Zero-copy techniques** – Explore `splice`/`sendfile` on Linux to cut the copy count when staying on CLI.

## Usage Guidance (as of 2025-09-30)

- **<100 MB files** – Streaming is acceptable; lower memory footprint outweighs speed.
- **>100 MB passphrase files** – Prefer file-based operations; temp-file staging equals file mode anyway.
- **Recipient-based streaming** – Pipe mode is viable and achieves reasonable throughput when buffers are tuned.
- **Automation** – Always set `CAGE_STREAMING_STRATEGY` explicitly in scripts if predictable performance
  matters (`tempfile`, `pipe`, or `auto`).

## Chunker Integration (CAGE-21)

- The new `cage::buff::FileChunker` provides chunk planning, resumable checkpoints, and progress reporting
  via `rsb::progress`. Streaming, backups, and manifests should layer on this module to cap memory usage and
  support resume-on-failure scenarios.
- For streaming encrypt/decrypt, wrap the temp-file staging path with chunked reads/writes so each chunk can be
  retried independently, emit chunk-level telemetry, and keep dashboards responsive.
- Backup retention can store chunk manifests alongside `.cage_backups.json` for richer restore tooling.
- See `docs/ref/cage/CHUNKER_STRATEGY.md` for the full rollout plan.

## Next Steps for CAGE-12a

1. Prototype larger buffer usage in `encrypt_stream_pipe`/`decrypt_stream_pipe` and re-run the 1 GB benchmark.
2. Profile the streaming path with `perf`/`dtrace` to quantify time spent in copies vs. waiting on the child.
3. Add a README note summarizing performance trade-offs (stream vs. file).
4. Integrate `FileChunker` into streaming/backups to provide resumable chunked processing and align telemetry.
5. Track AGE-01 progress as the definitive solution for high-throughput streaming.

## References
- `.analysis/streaming_benchmark_results.md`
- `tests/test_streaming_benchmark.rs`
- `src/cage/adp/v2.rs`
- `src/cage/adp/v2_pipe_passphrase.rs`
