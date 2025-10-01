//! PTY Automation Module - Terminal Emulation for Age CLI
//!
//! This module provides multiple methods for automating the Age CLI tool through
//! pseudo-terminal (PTY) interfaces, enabling non-interactive encryption/decryption
//! operations while maintaining security and reliability.
//!
//! # Submodules
//!
//! - `wrap` - PTY-based Age automation using portable-pty (primary method)
//! - `tty` - TTY automation using script/expect methods (fallback/alternative)
//!
//! # Primary Method: PTY Wrapper
//!
//! The PTY wrapper (`wrap`) provides the most reliable and portable method for
//! Age automation using the portable-pty library. This is the recommended approach
//! for most use cases.
//!
//! # Alternative Method: TTY Automation
//!
//! The TTY automation (`tty`) provides proven script/expect-based methods as a
//! fallback or alternative approach. This was part of the original pilot that
//! eliminated T2.1: TTY Automation Subversion.
//!
//! # Security Considerations
//!
//! All PTY automation methods handle passphrases securely:
//! - No passphrase leakage to process lists
//! - Secure temporary file handling
//! - Proper cleanup on success and failure
//! - Timeout protection against hanging processes

pub mod tty;
pub mod wrap;

// Re-export primary types for convenience
pub use tty::TtyAutomator;
pub use wrap::PtyAgeAutomator;
