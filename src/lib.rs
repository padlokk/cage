//! Cage - Age Encryption Automation Library
//!
//! Cage provides bulletproof Age encryption automation by eliminating TTY interaction
//! requirements while maintaining cryptographic security standards. Features production-grade
//! PTY automation with comprehensive error handling and security validation.
//!
//! # Features
//!
//! - **PTY Automation**: Native PTY wrapper for Age encryption with timeout handling
//! - **CRUD Operations**: Complete encryption lifecycle management
//! - **ASCII Armor Support**: Optional `-a` flag for text-safe environments
//! - **Batch Processing**: High-performance parallel operations
//! - **Security Validation**: Comprehensive injection prevention and audit logging
//! - **Production Ready**: Robust error handling and monitoring integration
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use cage::prelude::*;
//! use std::path::Path;
//!
//! # fn main() -> AgeResult<()> {
//! // Create CRUD manager with defaults
//! let mut crud_manager = CrudManager::with_defaults()?;
//!
//! // Lock (encrypt) a file
//! let options = LockOptions::default();
//! crud_manager.lock(Path::new("input.txt"), "passphrase", options)?;
//!
//! // Check status
//! let status = crud_manager.status(Path::new("."))?;
//! println!("Encrypted files: {}", status.encrypted_files);
//! # Ok(())
//! # }
//! ```

pub mod cage;
pub mod deps;
pub mod prelude;

// Re-export core types for convenience
pub use cage::{
    AdapterFactory, AgeAdapter, AgeAutomator, AgeConfig, AgeError, AgeResult, AuditLogger,
    CrudManager, FileEncryption, LockOptions, Operation, OperationResult, OutputFormat,
    PassphraseManager, PassphraseMode, RepositoryOperations, RepositoryStatus, SecurityValidator,
    TtyMethod, UnlockOptions, VerificationResult,
};

/// Library version - synchronized with Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Security clearance level for this library
pub const SECURITY_LEVEL: &str = "PRODUCTION_READY";

/// Features provided by this library
pub const FEATURES: &[&str] = &[
    "PTY_AUTOMATION",
    "AGE_ENCRYPTION",
    "BATCH_PROCESSING",
    "SECURITY_VALIDATION",
    "ASCII_ARMOR_SUPPORT",
];
