# CAGE-12b: Passphrase Pipe Streaming Investigation

**Date**: 2025-09-29
**Status**: Investigation Complete
**Conclusion**: Not Feasible with Current Age Implementation

## Executive Summary

Investigation into implementing true pipe streaming for passphrase-based encryption revealed fundamental architectural limitations in the age binary that prevent this optimization.

## Technical Findings

### How Age Handles Passphrases

1. **TTY Requirement**: Age reads passphrases from `/dev/tty` (the controlling terminal), not from stdin
2. **Security by Design**: This is intentional - age maintainers want passphrases to be interactive to discourage insecure practices
3. **No Environment Variables**: Age doesn't support `AGE_PASSPHRASE` or similar environment variables
4. **No File Descriptor Support**: Unlike GPG's `--passphrase-fd`, age has no option to read passphrases from alternative sources

### PTY Limitations

When using PTY (Pseudo-Terminal) automation:
- PTY connects stdin/stdout to the pseudo-terminal
- Cannot simultaneously use PTY for passphrase AND separate pipes for data
- The stdin/stdout become part of the PTY, not available for data streaming

### Performance Impact

Current implementation (temp file with PTY):
- **File-based encryption**: ~600 MB/s
- **Temp file streaming**: ~100-150 MB/s
- **Performance ratio**: ~20-25% of file-based speed

This is acceptable for most use cases and is a necessary trade-off for security.

## Implementation Details

### Module Created
- `src/cage/adapter_v2_pipe_passphrase.rs` - Documents the investigation and provides stub implementations

### Key Code Changes
1. Made `StreamingStrategy` enum public for future extensibility
2. Added passphrase pipe check in `encrypt_stream` (returns error appropriately)
3. Documented limitations in module and function documentation

## Recommendations

### Short-term (Current)
1. Continue using temp file approach with PTY automation
2. Document the performance characteristics for users
3. Recommend file-based operations for files > 100MB

### Long-term Possibilities
1. Monitor age project for potential future support:
   - PR #641 proposes reading passphrase from stdin when not used for data
   - Plugin system may allow alternative passphrase input methods
2. Consider using `rage` (Rust implementation) which might have more flexibility
3. For high-performance needs, consider recipient-based encryption (supports true pipe streaming)

## Lessons Learned

1. **Security vs Performance**: Age prioritizes security (interactive passphrases) over automation convenience
2. **PTY Complexity**: PTY automation is powerful but has fundamental limitations when mixing control and data channels
3. **Documentation Value**: Even failed experiments provide valuable documentation for future developers

## Code Artifacts

- Module: `src/cage/adapter_v2_pipe_passphrase.rs`
- Benchmark: `tests/test_streaming_benchmark.rs`
- Results: `.analysis/streaming_benchmark_results.md`

## References

- [Age Discussion #275](https://github.com/FiloSottile/age/discussions/275) - Non-interactive passphrase input
- [Age PR #641](https://github.com/FiloSottile/age/pull/641) - Proposal for stdin passphrase reading
- [Age Issue #603](https://github.com/FiloSottile/age/issues/603) - Can't pass password via stdin

## Conclusion

While true pipe streaming with passphrases is not currently feasible, the investigation has:
1. Clarified the technical limitations
2. Documented the optimal current approach
3. Established performance baselines
4. Prepared the codebase for future enhancements if age evolves

The temp file approach with PTY automation remains the best solution, balancing security, reliability, and acceptable performance.