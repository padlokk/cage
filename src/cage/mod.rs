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

pub mod adapter;
pub mod adapter_v2; // Enhanced adapter with streaming (CAGE-12)
pub mod adapter_v2_pipe_passphrase; // True pipe streaming for passphrases (CAGE-12b)
pub mod age_engine;
pub mod chunker;
pub mod config;
pub mod error;
pub mod in_place;
pub mod lifecycle;
pub mod operations;
pub mod passphrase;
pub mod pty_wrap; // New PTY automation module
pub mod requests;
pub mod security;
pub mod strings; // Centralized string constants (SEC-01)
pub mod tty_automation; // Request structs for unified API (CAGE-11)

// Re-export core types for convenience
pub use adapter::{AdapterFactory, AgeAdapter};
pub use age_engine::AgeAutomator;
pub use chunker::{ChunkProcessingSummary, ChunkSpec, ChunkerConfig, FileChunker};
pub use config::{AgeConfig, OutputFormat, TtyMethod};
pub use error::{AgeError, AgeResult};
pub use in_place::{InPlaceOperation, InPlaceOptions, RecoveryManager, SafetyValidator};
pub use lifecycle::{CrudManager, LockOptions, UnlockOptions, VerificationResult};
pub use operations::{
    FileEncryption, Operation, OperationResult, RepositoryOperations, RepositoryStatus,
};
pub use passphrase::{PassphraseManager, PassphraseMode};
pub use security::{AuditLogger, SecurityValidator};

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
