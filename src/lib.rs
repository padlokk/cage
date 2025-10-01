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
//! let mut crud_manager = CageManager::with_defaults()?;
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

// Core cage modules - flattened from src/cage/ to src/
pub mod adp; // Adapter implementations (v1, v2, pipe streaming)
pub mod audit; // Audit logging and security validation
pub mod buff; // Chunking and buffer management
pub mod core; // Core primitives (config, requests, engine, recovery)
pub mod error;
pub mod forge; // Repository operations
pub mod keygen; // Key generation service module
pub mod mgr; // CageManager lifecycle coordination
pub mod passphrase; // Secure passphrase management
pub mod pty; // PTY automation (wrap, tty methods)

// Supporting modules
pub mod deps;
pub mod lang;
pub mod prelude;

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
