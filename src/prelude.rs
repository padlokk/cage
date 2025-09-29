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
    // Management
    CrudManager,

    // Request API (CAGE-11)
    requests::{LockRequest, UnlockRequest, RotateRequest},

    // Configuration
    AgeConfig, OutputFormat, TtyMethod,

    // Options
    LockOptions, UnlockOptions, VerificationResult,

    // Results and Errors
    AgeResult, AgeError, OperationResult,

    // Adapters
    AgeAdapter, AdapterFactory,
    adapter_v2::{AgeAdapterV2, ShellAdapterV2},

    // Security
    SecurityValidator, AuditLogger,

    // Progress
    ProgressManager, ProgressReporter,

    // Passphrase
    PassphraseManager, PassphraseMode,

    // Operations
    Operation, FileEncryption, RepositoryOperations, RepositoryStatus,
};

// Constants
pub use crate::{VERSION, SECURITY_LEVEL, FEATURES};