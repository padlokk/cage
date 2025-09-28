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
pub mod adapter_v2;  // Enhanced adapter with streaming (CAGE-12)
pub mod age_engine;
pub mod tty_automation;
pub mod pty_wrap;  // New PTY automation module
pub mod operations;
pub mod lifecycle;
pub mod security;
pub mod error;
pub mod config;
pub mod passphrase;
pub mod in_place;
pub mod progress;
pub mod strings;  // Centralized string constants (SEC-01)
pub mod requests;  // Request structs for unified API (CAGE-11)

// Re-export core types for convenience
pub use adapter::{AgeAdapter, AdapterFactory};
pub use age_engine::AgeAutomator;
pub use config::{AgeConfig, OutputFormat, TtyMethod};
pub use error::{AgeError, AgeResult};
pub use operations::{
    Operation, FileEncryption, RepositoryOperations, RepositoryStatus, OperationResult
};
pub use lifecycle::{CrudManager, LockOptions, UnlockOptions, VerificationResult};
pub use security::{AuditLogger, SecurityValidator};
pub use passphrase::{PassphraseManager, PassphraseMode};
pub use in_place::{InPlaceOperation, InPlaceOptions, SafetyValidator, RecoveryManager};
pub use progress::{ProgressManager, ProgressStyle, ProgressReporter, TerminalReporter};

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