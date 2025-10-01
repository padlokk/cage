//! Core primitives and configuration for Cage encryption operations.
//!
//! This module provides the fundamental building blocks for the Cage encryption
//! system, including configuration management, request types, the age engine
//! interface, and recovery/in-place operation handling.
//!
//! # Submodules
//!
//! - `config` - Configuration types (AgeConfig, OutputFormat, SecurityLevel, etc.)
//! - `requests` - Request structures for encryption operations (Lock, Unlock, Rotate, etc.)
//! - `engine` - Age encryption engine automation interface
//! - `recovery` - In-place operation recovery and safety validation

pub mod config;
pub mod engine;
pub mod recovery;
pub mod requests;

// Re-export commonly used types
pub use config::{
    AgeConfig, OutputFormat, RetentionPolicyConfig, SecurityLevel, TelemetryFormat, TtyMethod,
};
pub use engine::AgeAutomator;
pub use recovery::{InPlaceOperation, InPlaceOptions, RecoveryManager, SafetyValidator};
pub use requests::{
    AuthorityTier, BatchOperation, BatchRequest, CommonOptions, FromCliArgs, Identity,
    LockRequest, MultiRecipientConfig, Recipient, RecipientGroup, ReportFormat, RotateRequest,
    StatusRequest, StreamOperation, StreamRequest, ToOperationParams, UnlockRequest,
    VerifyRequest,
};
