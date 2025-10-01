//! Key generation service API.
//!
//! This stub captures the contract that the CLI and future library callers will
//! rely on. Implementation work is tracked under task CAGE-21 (CLI workflow) and
//! CAGE-22 (adapter-native identities). See `docs/ref/cage/KEYGEN_STRATEGY.md`
//! for the authoritative specification.

use crate::cage::core::AgeConfig;
use crate::cage::keygen::error::KeygenError;
use std::path::PathBuf;

/// Request payload accepted by the key generation service.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct KeygenRequest {
    /// Optional explicit output path supplied by the caller.
    pub output_path: Option<PathBuf>,
    /// Optional input identity when operating in recipients-only mode.
    pub input_path: Option<PathBuf>,
    /// Recipient groups to register the generated public key with.
    pub register_groups: Vec<String>,
    /// When true, convert an existing identity to recipients-only output.
    pub recipients_only: bool,
    /// Permit overwriting an existing output file.
    pub force: bool,
    /// Skip writing to disk and emit results on stdout only.
    pub stdout_only: bool,
    /// Emit structured JSON summary.
    pub json_output: bool,
    /// Force proxy mode (direct passthrough to `age-keygen`).
    pub proxy_mode: bool,
}

/// Result summary returned by the key generation workflow.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct KeygenSummary {
    /// Path where the identity (or recipients) was written, when applicable.
    pub output_path: Option<PathBuf>,
    /// Derived public recipient (if available).
    pub public_recipient: Option<String>,
    /// Collected MD5 fingerprint of the private key (if applicable).
    pub fingerprint_md5: Option<String>,
    /// Collected SHA256 fingerprint of the private key (if applicable).
    pub fingerprint_sha256: Option<String>,
    /// Recipient groups that were updated as part of the request.
    pub registered_groups: Vec<String>,
}

/// Primary key generation service entry point.
#[derive(Debug, Clone, Default)]
pub struct KeygenService {
    config: Option<AgeConfig>,
}

impl KeygenService {
    /// Create a new service with the provided configuration context.
    pub fn new(config: Option<AgeConfig>) -> Self {
        Self { config }
    }

    /// Generate an identity according to `request`.
    pub fn generate(&self, _request: &KeygenRequest) -> Result<KeygenSummary, KeygenError> {
        // Implementation to be provided in CAGE-21 / CAGE-22.
        Err(KeygenError::NotImplemented)
    }

    /// Access the underlying configuration (when available).
    pub fn config(&self) -> Option<&AgeConfig> {
        self.config.as_ref()
    }
}
