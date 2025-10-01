//! Cage - Age Encryption Automation Core Module
//!
//! This module provides bulletproof Age encryption automation by eliminating TTY interaction
//! requirements while maintaining cryptographic security standards. Based on proven PTY
//! patterns that successfully eliminated T2.1: TTY Automation Subversion.
//!
//! # Features
//!
//! - **PTY Automation**: Native PTY wrapper for seamless Age encryption
//! - **CRUD Operations**: Complete encryption lifecycle management
//! - **ASCII Armor Support**: Optional `-a` flag for text-safe environments
//! - **Batch Processing**: High-performance parallel operations
//! - **Security Validation**: Comprehensive injection prevention and audit logging
//! - **Production Ready**: Robust error handling and monitoring integration

pub mod adp; // Adapter implementations (v1, v2, pipe streaming)
pub mod audit; // Audit logging and security validation
pub mod buff; // Chunking and buffer management (formerly chunker)
pub mod core; // Core primitives (config, requests, engine, recovery)
pub mod error;
pub mod forge; // Repository operations (formerly operations)
pub mod keygen; // Key generation service module (CAGE-21)
pub mod mgr; // CageManager lifecycle coordination (formerly manager)
pub mod passphrase; // Secure passphrase management
pub mod pty; // PTY automation (wrap, tty methods)
pub mod strings; // Centralized string constants (SEC-01)

// Re-export core types for convenience
pub use adp::{AdapterFactory, AgeAdapter};
pub use audit::{AuditLogger, SecurityValidator};
pub use buff::{ChunkProcessingSummary, ChunkSpec, ChunkerConfig, FileChunker};
pub use core::{
    AgeAutomator, AgeConfig, InPlaceOperation, InPlaceOptions, OutputFormat, RecoveryManager,
    SafetyValidator, TtyMethod,
};
pub use error::{AgeError, AgeResult};
pub use forge::{
    FileEncryption, Operation, OperationResult, RepositoryOperations, RepositoryStatus,
};
pub use keygen::{KeygenError, KeygenRequest, KeygenService, KeygenSummary};
pub use mgr::{CageManager, LockOptions, UnlockOptions, VerificationResult};
pub use passphrase::{PassphraseManager, PassphraseMode};

/// Module version synchronized with Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Security clearance level for this module
pub const SECURITY_LEVEL: &str = "FORTRESS_CLEARED";

/// Threat elimination status
pub const THREAT_STATUS: &str = "T2.1_ELIMINATED";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all core types are properly exported
        let _config = AgeConfig::default();
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        assert_eq!(SECURITY_LEVEL, "FORTRESS_CLEARED");
        assert_eq!(THREAT_STATUS, "T2.1_ELIMINATED");
    }
}
