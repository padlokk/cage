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
pub mod age_engine;
pub mod audit; // Audit logging and security validation
pub mod chunker;
pub mod config;
pub mod error;
pub mod in_place;
pub mod keygen; // Key generation service module (CAGE-21)
pub mod manager; // CageManager lifecycle coordination
pub mod operations;
pub mod passphrase;
pub mod pty; // PTY automation (wrap and tty methods)
pub mod requests;
pub mod strings; // Centralized string constants (SEC-01)

// Re-export core types for convenience
pub use adp::{AdapterFactory, AgeAdapter};
pub use age_engine::AgeAutomator;
pub use chunker::{ChunkProcessingSummary, ChunkSpec, ChunkerConfig, FileChunker};
pub use config::{AgeConfig, OutputFormat, TtyMethod};
pub use error::{AgeError, AgeResult};
pub use in_place::{InPlaceOperation, InPlaceOptions, RecoveryManager, SafetyValidator};
pub use keygen::{KeygenError, KeygenRequest, KeygenService, KeygenSummary};
pub use manager::{CageManager, LockOptions, UnlockOptions, VerificationResult};
pub use operations::{
    FileEncryption, Operation, OperationResult, RepositoryOperations, RepositoryStatus,
};
pub use passphrase::{PassphraseManager, PassphraseMode};
pub use audit::{AuditLogger, SecurityValidator};

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
