//! Cage Prelude - Public API Surface
//!
//! This module provides a curated set of commonly used types and traits
//! from the Cage library. Import this module to get started quickly:
//!
//! ```rust
//! use cage::prelude::*;
//! ```
//!
//! # Included Types
//!
//! - **Core Management**: `CrudManager` - Main entry point for encryption operations
//! - **Request API**: `LockRequest`, `UnlockRequest`, `RotateRequest` - Typed operation builders
//! - **Configuration**: `AgeConfig`, `OutputFormat`, `TtyMethod` - Runtime configuration
//! - **Options**: `LockOptions`, `UnlockOptions` - Operation-specific settings
//! - **Results**: `AgeResult`, `AgeError`, `OperationResult` - Error handling types
//! - **Adapters**: `AgeAdapter`, `AgeAdapterV2` - Core adapter traits
//! - **Security**: `SecurityValidator`, `AuditLogger` - Security components
//! - **Progress**: `ProgressManager`, `ProgressReporter` - Progress tracking

// Core types from the cage module
pub use crate::cage::{
    adapter_v2::{AgeAdapterV2, ShellAdapterV2},

    // Request API (CAGE-11)
    requests::{LockRequest, RotateRequest, UnlockRequest},

    AdapterFactory,
    // Adapters
    AgeAdapter,
    // Configuration
    AgeConfig,
    AgeError,
    // Results and Errors
    AgeResult,
    AuditLogger,

    // Management
    CrudManager,

    FileEncryption,
    // Options
    LockOptions,
    // Operations
    Operation,
    OperationResult,

    OutputFormat,
    // Passphrase
    PassphraseManager,
    PassphraseMode,

    RepositoryOperations,
    RepositoryStatus,
    // Security
    SecurityValidator,
    TtyMethod,

    UnlockOptions,
    VerificationResult,
};

// Constants
pub use crate::{FEATURES, SECURITY_LEVEL, VERSION};
