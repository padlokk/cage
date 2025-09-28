//! Enhanced Age Adapter Interface with Streaming Support (CAGE-12)
//!
//! This module extends the adapter pattern to support both file and streaming operations,
//! providing a unified trait for all encryption backends with enhanced capabilities.

use std::path::Path;
use std::io::{Read, Write};
use super::error::{AgeError, AgeResult};
use super::config::OutputFormat;
use super::requests::{Identity, Recipient};

// ============================================================================
// CORE ADAPTER TRAITS
// ============================================================================

/// Enhanced Age operations interface with streaming support
pub trait AgeAdapterV2: Send + Sync {
    // ========================================================================
    // FILE-BASED OPERATIONS (Original Interface)
    // ========================================================================

    /// Encrypt a file with the given identity
    fn encrypt_file(
        &self,
        input: &Path,
        output: &Path,
        identity: &Identity,
        recipients: Option<&[Recipient]>,
        format: OutputFormat,
    ) -> AgeResult<()>;

    /// Decrypt a file with the given identity
    fn decrypt_file(
        &self,
        input: &Path,
        output: &Path,
        identity: &Identity,
    ) -> AgeResult<()>;

    // ========================================================================
    // STREAMING OPERATIONS (New Interface)
    // ========================================================================

    /// Encrypt from reader to writer
    fn encrypt_stream(
        &self,
        input: &mut dyn Read,
        output: &mut dyn Write,
        identity: &Identity,
        recipients: Option<&[Recipient]>,
        format: OutputFormat,
    ) -> AgeResult<u64>; // Returns bytes processed

    /// Decrypt from reader to writer
    fn decrypt_stream(
        &self,
        input: &mut dyn Read,
        output: &mut dyn Write,
        identity: &Identity,
    ) -> AgeResult<u64>; // Returns bytes processed

    // ========================================================================
    // IDENTITY & RECIPIENT OPERATIONS
    // ========================================================================

    /// Validate an identity (passphrase, key file, etc.)
    fn validate_identity(&self, identity: &Identity) -> AgeResult<()>;

    /// Validate recipients
    fn validate_recipients(&self, recipients: &[Recipient]) -> AgeResult<()>;

    /// Generate a new identity
    fn generate_identity(&self) -> AgeResult<(String, String)>; // (private, public)

    /// Convert SSH key to Age recipient
    fn ssh_to_recipient(&self, ssh_pubkey: &str) -> AgeResult<String>;

    // ========================================================================
    // VERIFICATION & INSPECTION
    // ========================================================================

    /// Verify encrypted file integrity (without decrypting fully)
    fn verify_file(&self, file: &Path, identity: Option<&Identity>) -> AgeResult<VerificationResult>;

    /// Get metadata from encrypted file
    fn inspect_file(&self, file: &Path) -> AgeResult<FileMetadata>;

    /// Check if a file is encrypted with Age
    fn is_encrypted(&self, file: &Path) -> bool;

    // ========================================================================
    // HEALTH & DIAGNOSTICS
    // ========================================================================

    /// Validate adapter is functional
    fn health_check(&self) -> AgeResult<HealthStatus>;

    /// Get adapter capabilities
    fn capabilities(&self) -> AdapterCapabilities;

    /// Get adapter name
    fn adapter_name(&self) -> &'static str;

    /// Get adapter version
    fn adapter_version(&self) -> String;

    /// Clone this adapter into a boxed trait object
    fn clone_box(&self) -> Box<dyn AgeAdapterV2>;
}

// ============================================================================
// SUPPORT STRUCTURES
// ============================================================================

/// Verification result for encrypted files
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// File is properly formatted
    pub format_valid: bool,

    /// Header is intact
    pub header_valid: bool,

    /// Can be decrypted with provided identity
    pub decryptable: Option<bool>,

    /// File size
    pub size_bytes: u64,

    /// Detected format
    pub format: DetectedFormat,
}

/// Detected encryption format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetectedFormat {
    /// Age binary format
    AgeBinary,
    /// Age ASCII armor format
    AgeArmor,
    /// Unknown format
    Unknown,
}

/// Metadata from encrypted file
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// Number of recipients (if detectable)
    pub recipient_count: Option<usize>,

    /// File format
    pub format: DetectedFormat,

    /// Encrypted size
    pub encrypted_size: u64,

    /// Creation timestamp (if available)
    pub created: Option<std::time::SystemTime>,
}

/// Health check status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Overall health
    pub healthy: bool,

    /// Age binary available
    pub age_binary: bool,

    /// Version of Age binary
    pub age_version: Option<String>,

    /// Can perform encryption
    pub can_encrypt: bool,

    /// Can perform decryption
    pub can_decrypt: bool,

    /// Streaming support available
    pub streaming_available: bool,

    /// Error messages if unhealthy
    pub errors: Vec<String>,
}

/// Adapter capabilities
#[derive(Debug, Clone)]
pub struct AdapterCapabilities {
    /// Supports passphrase encryption
    pub passphrase: bool,

    /// Supports public key encryption
    pub public_key: bool,

    /// Supports identity files
    pub identity_files: bool,

    /// Supports SSH recipients
    pub ssh_recipients: bool,

    /// Supports streaming operations
    pub streaming: bool,

    /// Supports ASCII armor format
    pub ascii_armor: bool,

    /// Supports hardware keys (e.g., YubiKey)
    pub hardware_keys: bool,

    /// Supports key derivation
    pub key_derivation: bool,

    /// Maximum file size (None for unlimited)
    pub max_file_size: Option<u64>,
}

// ============================================================================
// BACKWARD COMPATIBILITY WRAPPER
// ============================================================================

/// Wrapper to adapt V2 interface to original AgeAdapter trait
pub struct AdapterV1Compat<T: AgeAdapterV2 + Clone> {
    inner: T,
}

impl<T: AgeAdapterV2 + Clone> AdapterV1Compat<T> {
    /// Create a new compatibility wrapper
    pub fn new(adapter: T) -> Self {
        Self { inner: adapter }
    }
}

impl<T: AgeAdapterV2 + Clone + 'static> super::adapter::AgeAdapter for AdapterV1Compat<T> {
    fn encrypt(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()> {
        let identity = Identity::Passphrase(passphrase.to_string());
        self.inner.encrypt_file(input, output, &identity, None, format)
    }

    fn decrypt(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()> {
        let identity = Identity::Passphrase(passphrase.to_string());
        self.inner.decrypt_file(input, output, &identity)
    }

    fn health_check(&self) -> AgeResult<()> {
        let status = self.inner.health_check()?;
        if status.healthy {
            Ok(())
        } else {
            Err(AgeError::HealthCheckFailed(
                status.errors.join(", ")
            ))
        }
    }

    fn adapter_name(&self) -> &'static str {
        self.inner.adapter_name()
    }

    fn adapter_version(&self) -> String {
        self.inner.adapter_version()
    }

    fn clone_box(&self) -> Box<dyn super::adapter::AgeAdapter> {
        // For now, return a placeholder error
        unimplemented!("Clone not implemented for AdapterV1Compat")
    }
}

// ============================================================================
// STREAMING UTILITIES
// ============================================================================

/// Buffer size for streaming operations
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

/// Helper for buffered streaming
pub struct StreamBuffer {
    buffer: Vec<u8>,
    position: usize,
    capacity: usize,
}

impl StreamBuffer {
    /// Create a new stream buffer
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            position: 0,
            capacity,
        }
    }

    /// Create with default size
    pub fn default() -> Self {
        Self::new(DEFAULT_BUFFER_SIZE)
    }

    /// Get mutable buffer slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buffer[..self.capacity]
    }

    /// Get buffer slice up to position
    pub fn filled(&self) -> &[u8] {
        &self.buffer[..self.position]
    }

    /// Update position after write
    pub fn advance(&mut self, count: usize) {
        self.position = (self.position + count).min(self.capacity);
    }

    /// Reset buffer
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_buffer() {
        let mut buffer = StreamBuffer::new(1024);
        assert_eq!(buffer.capacity, 1024);
        assert_eq!(buffer.position, 0);

        buffer.advance(512);
        assert_eq!(buffer.position, 512);

        buffer.reset();
        assert_eq!(buffer.position, 0);
    }

    #[test]
    fn test_adapter_capabilities() {
        let caps = AdapterCapabilities {
            passphrase: true,
            public_key: true,
            identity_files: true,
            ssh_recipients: false,
            streaming: true,
            ascii_armor: true,
            hardware_keys: false,
            key_derivation: false,
            max_file_size: Some(1024 * 1024 * 1024), // 1GB
        };

        assert!(caps.passphrase);
        assert!(caps.streaming);
        assert_eq!(caps.max_file_size, Some(1_073_741_824));
    }
}