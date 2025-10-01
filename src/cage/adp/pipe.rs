//! CAGE-12b: Passphrase Pipe Streaming Investigation
//!
//! This module investigated true pipe streaming for passphrase-based encryption.
//!
//! FINDINGS: True pipe streaming with passphrases is not feasible because:
//! 1. Age reads passphrases from /dev/tty (controlling terminal), not stdin
//! 2. PTY wrapping connects stdin/stdout to the PTY, preventing separate data pipes
//! 3. We cannot simultaneously use PTY for passphrase AND pipes for data
//!
//! The current temp file approach with PTY automation is the optimal solution,
//! providing security (passphrase never in CLI/env) and reliability.
//!
//! Performance measurements show:
//! - File-based encryption: ~600 MB/s
//! - Temp file streaming: ~100-150 MB/s (acceptable for most use cases)
//!
//! This module is kept for documentation and may be useful if age adds
//! support for reading passphrases from environment variables or file descriptors.

use std::io::{Read, Write};

use crate::cage::adp::v2::ShellAdapterV2;
use crate::cage::core::OutputFormat;
use crate::cage::error::{AgeError, AgeResult};

impl ShellAdapterV2 {
    /// Encrypt stream using pipes with passphrase (CAGE-12b)
    ///
    /// This method is not feasible with current age implementation.
    /// Age requires passphrase from TTY, preventing true pipe streaming.
    pub(crate) fn encrypt_stream_pipe_passphrase(
        &self,
        _input: &mut (dyn Read + Send),
        _output: &mut (dyn Write + Send),
        _passphrase: &str,
        _format: OutputFormat,
    ) -> AgeResult<u64> {
        // NOTE: This implementation is kept for documentation purposes.
        // True pipe streaming with passphrases is not feasible - see module docs.
        // The temp file approach in encrypt_stream_temp is the correct solution.

        Err(AgeError::InvalidOperation {
            operation: "encrypt_stream_pipe_passphrase".into(),
            reason: "Passphrase pipe streaming not feasible - use temp file strategy instead"
                .into(),
        })
    }

    /// Decrypt stream using pipes with passphrase (CAGE-12b)
    ///
    /// This method is not feasible with current age implementation.
    /// Age requires passphrase from TTY, preventing true pipe streaming.
    pub(crate) fn decrypt_stream_pipe_passphrase(
        &self,
        _input: &mut (dyn Read + Send),
        _output: &mut (dyn Write + Send),
        _passphrase: &str,
    ) -> AgeResult<u64> {
        Err(AgeError::InvalidOperation {
            operation: "decrypt_stream_pipe_passphrase".into(),
            reason: "Passphrase pipe streaming not feasible - use temp file strategy instead"
                .into(),
        })
    }
}

/// Extension trait for enabling passphrase pipe streaming
///
/// This trait is kept for API compatibility but the feature is not feasible.
pub trait PassphrasePipeStreaming {
    /// Check if passphrase pipe streaming is enabled
    fn is_passphrase_pipe_enabled(&self) -> bool;

    /// Enable passphrase pipe streaming for testing
    fn enable_passphrase_pipe(&mut self, enable: bool);
}

impl PassphrasePipeStreaming for ShellAdapterV2 {
    fn is_passphrase_pipe_enabled(&self) -> bool {
        // Always return false since this feature is not feasible
        false
    }

    fn enable_passphrase_pipe(&mut self, _enable: bool) {
        // No-op since this feature is not feasible
        // Kept for API compatibility
    }
}
