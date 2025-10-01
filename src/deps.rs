//! Cage Dependencies Re-export Module
//!
//! Standard RSB pattern for re-exporting external dependencies that downstream
//! users might need when working with Cage. This centralizes dependency management
//! and makes it clear which external crates are part of the public API surface.
//!
//! # Re-exported Crates
//!
//! - **age** (0.11.1): Core Age encryption library (for Identity/Recipient types)
//! - **rsb** (git): Rebel String-Biased framework (for CLI macros and utilities)
//! - **hub** (git): Terminal utilities and PTY support
//!
//! Note: rsb and hub are currently git dependencies tracking their main branches.
//!
//! # Intentionally Excluded
//!
//! The following crates are internal implementation details and not re-exported:
//! - `tempfile`: Internal temporary file management
//! - `which`: Binary detection utility
//! - `rayon`: Parallel processing implementation
//! - `walkdir`: Directory traversal implementation
//! - `indicatif`: Progress bar implementation detail

/// Age encryption library types
///
/// Provides access to Identity and Recipient types that users might need
/// when working with Cage's encryption APIs.
pub use age;

/// Rebel String-Biased framework
///
/// Core RSB utilities including:
/// - String manipulation macros (echo!, die!, etc.)
/// - Color formatting (green!, red!, yellow!)
/// - CLI building blocks
pub use rsb;

/// Terminal and PTY utilities
///
/// Provides portable PTY support and terminal handling functionality.
/// Primarily used for advanced PTY automation scenarios.
pub use hub;

// Optional re-exports for common types users might need
pub mod common {
    /// Re-export commonly used Age types
    pub use age::{Identity, Recipient};

    /// Re-export PTY types for advanced users
    pub use hub::portable_pty::{CommandBuilder, PtySize};
}
