//! Enhanced Age Adapter Interface with Streaming Support (CAGE-12)
//!
//! This module extends the adapter pattern to support both file and streaming operations,
//! providing a unified trait for all encryption backends with enhanced capabilities.

use std::path::Path;
use std::io::{Read, Write};
use std::fs::File;
use std::sync::Arc;
use super::error::{AgeError, AgeResult};
use super::config::OutputFormat;
use super::requests::{Identity, Recipient};
use super::pty_wrap::PtyAgeAutomator;
use tempfile::tempdir;

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
        _identity: &Identity,
        recipients: Option<&[Recipient]>,
        format: OutputFormat,
    ) -> AgeResult<u64>; // Returns bytes processed

    /// Decrypt from reader to writer
    fn decrypt_stream(
        &self,
        input: &mut dyn Read,
        output: &mut dyn Write,
        _identity: &Identity,
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
pub struct AdapterV1Compat {
    inner: Arc<dyn AgeAdapterV2>,
}

impl AdapterV1Compat {
    /// Create a new compatibility wrapper
    pub fn new(adapter: impl AgeAdapterV2 + 'static) -> Self {
        Self { inner: Arc::new(adapter) }
    }
}

impl super::adapter::AgeAdapter for AdapterV1Compat {
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
        Box::new(AdapterV1Compat { inner: Arc::clone(&self.inner) })
    }
}

// ============================================================================
// SHELL ADAPTER V2 IMPLEMENTATION
// ============================================================================

#[derive(Clone, Default)]
pub struct ShellAdapterV2;

impl ShellAdapterV2 {
    pub fn new() -> AgeResult<Self> {
        // Validate age binary availability early
        let automator = PtyAgeAutomator::new()?;
        automator.check_age_binary()?;
        Ok(Self)
    }
}

impl AgeAdapterV2 for ShellAdapterV2 {
    fn encrypt_file(
        &self,
        input: &Path,
        output: &Path,
        identity: &Identity,
        _recipients: Option<&[Recipient]>,
        format: OutputFormat,
    ) -> AgeResult<()> {
        let pass = match identity {
            Identity::Passphrase(p) => p.clone(),
            Identity::PromptPassphrase => {
                return Err(AgeError::AdapterNotImplemented("PromptPassphrase not supported in ShellAdapterV2".into()));
            }
            Identity::IdentityFile(_) | Identity::SshKey(_) => {
                return Err(AgeError::AdapterNotImplemented("Identity-based encryption not yet implemented".into()));
            }
        };

        let automator = PtyAgeAutomator::new()?;
        automator.encrypt(input, output, &pass, format)
    }

    fn decrypt_file(
        &self,
        input: &Path,
        output: &Path,
        identity: &Identity,
    ) -> AgeResult<()> {
        let pass = match identity {
            Identity::Passphrase(p) => p.clone(),
            Identity::PromptPassphrase => {
                return Err(AgeError::AdapterNotImplemented("PromptPassphrase not supported in ShellAdapterV2".into()));
            }
            Identity::IdentityFile(_) | Identity::SshKey(_) => {
                return Err(AgeError::AdapterNotImplemented("Identity-based decryption not yet implemented".into()));
            }
        };

        let automator = PtyAgeAutomator::new()?;
        automator.decrypt(input, output, &pass)
    }

    fn encrypt_stream(
        &self,
        input: &mut dyn Read,
        output: &mut dyn Write,
        identity: &Identity,
        recipients: Option<&[Recipient]>,
        format: OutputFormat,
    ) -> AgeResult<u64> {
        if let Some(recips) = recipients {
            if !recips.is_empty() {
                return Err(AgeError::AdapterNotImplemented("Public-key streaming not yet implemented".into()));
            }
        }

        let pass = match identity {
            Identity::Passphrase(p) => p.clone(),
            _ => return Err(AgeError::AdapterNotImplemented("Streaming requires passphrase identity".into())),
        };

        let temp_dir = tempdir().map_err(|e| AgeError::TemporaryResourceError {
            resource_type: "dir".into(),
            operation: "create".into(),
            reason: e.to_string(),
        })?;

        let input_path = temp_dir.path().join("stream_input");
        let mut temp_in = File::create(&input_path).map_err(|e| AgeError::file_error("create", input_path.clone(), e))?;
        let bytes_copied = std::io::copy(input, &mut temp_in).map_err(|e| AgeError::IoError {
            operation: "stream_copy".into(),
            context: "encrypt_stream".into(),
            source: e,
        })?;
        temp_in.flush().map_err(|e| AgeError::IoError {
            operation: "flush".into(),
            context: "encrypt_stream".into(),
            source: e,
        })?;

        let output_path = temp_dir.path().join("stream_output");

        let automator = PtyAgeAutomator::new()?;
        automator.encrypt(&input_path, &output_path, &pass, format)?;

        let mut encrypted = File::open(&output_path).map_err(|e| AgeError::file_error("open", output_path.clone(), e))?;
        std::io::copy(&mut encrypted, output).map_err(|e| AgeError::IoError {
            operation: "stream_copy".into(),
            context: "encrypt_stream".into(),
            source: e,
        })?;

        Ok(bytes_copied)
    }

    fn decrypt_stream(
        &self,
        input: &mut dyn Read,
        output: &mut dyn Write,
        identity: &Identity,
    ) -> AgeResult<u64> {
        let pass = match identity {
            Identity::Passphrase(p) => p.clone(),
            _ => return Err(AgeError::AdapterNotImplemented("Streaming requires passphrase identity".into())),
        };

        let temp_dir = tempdir().map_err(|e| AgeError::TemporaryResourceError {
            resource_type: "dir".into(),
            operation: "create".into(),
            reason: e.to_string(),
        })?;

        let input_path = temp_dir.path().join("stream_input.cage");
        let mut temp_in = File::create(&input_path).map_err(|e| AgeError::file_error("create", input_path.clone(), e))?;
        let bytes_copied = std::io::copy(input, &mut temp_in).map_err(|e| AgeError::IoError {
            operation: "stream_copy".into(),
            context: "decrypt_stream".into(),
            source: e,
        })?;
        temp_in.flush().map_err(|e| AgeError::IoError {
            operation: "flush".into(),
            context: "decrypt_stream".into(),
            source: e,
        })?;

        let output_path = temp_dir.path().join("stream_output");

        let automator = PtyAgeAutomator::new()?;
        automator.decrypt(&input_path, &output_path, &pass)?;

        let mut decrypted = File::open(&output_path).map_err(|e| AgeError::file_error("open", output_path.clone(), e))?;
        std::io::copy(&mut decrypted, output).map_err(|e| AgeError::IoError {
            operation: "stream_copy".into(),
            context: "decrypt_stream".into(),
            source: e,
        })?;

        Ok(bytes_copied)
    }

    fn validate_identity(&self, identity: &Identity) -> AgeResult<()> {
        match identity {
            Identity::Passphrase(pass) => {
                if pass.is_empty() {
                    Err(AgeError::InvalidOperation { operation: "validate_identity".into(), reason: "Empty passphrase".into() })
                } else {
                    Ok(())
                }
            }
            _ => Err(AgeError::AdapterNotImplemented("Identity validation not implemented".into())),
        }
    }

    fn validate_recipients(&self, _recipients: &[Recipient]) -> AgeResult<()> {
        Err(AgeError::AdapterNotImplemented("Recipient validation not implemented".into()))
    }

    fn generate_identity(&self) -> AgeResult<(String, String)> {
        Err(AgeError::AdapterNotImplemented("Identity generation not implemented".into()))
    }

    fn ssh_to_recipient(&self, _ssh_pubkey: &str) -> AgeResult<String> {
        Err(AgeError::AdapterNotImplemented("SSH recipient conversion not implemented".into()))
    }

    fn verify_file(&self, _file: &Path, _identity: Option<&Identity>) -> AgeResult<VerificationResult> {
        Err(AgeError::AdapterNotImplemented("verify_file not implemented".into()))
    }

    fn inspect_file(&self, _file: &Path) -> AgeResult<FileMetadata> {
        Err(AgeError::AdapterNotImplemented("inspect_file not implemented".into()))
    }

    fn is_encrypted(&self, file: &Path) -> bool {
        file.extension().map_or(false, |e| e == "cage")
    }

    fn health_check(&self) -> AgeResult<HealthStatus> {
        let age_available = PtyAgeAutomator::new()?.check_age_binary().is_ok();
        Ok(HealthStatus {
            healthy: age_available,
            age_binary: age_available,
            age_version: None,
            can_encrypt: age_available,
            can_decrypt: age_available,
            streaming_available: age_available,
            errors: if age_available { vec![] } else { vec!["Age binary not available".into()] },
        })
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            passphrase: true,
            public_key: false,
            identity_files: false,
            ssh_recipients: false,
            streaming: true,
            ascii_armor: true,
            hardware_keys: false,
            key_derivation: false,
            max_file_size: None,
        }
    }

    fn adapter_name(&self) -> &'static str {
        "ShellAdapterV2"
    }

    fn adapter_version(&self) -> String {
        format!("shell-v2-{}", super::VERSION)
    }

    fn clone_box(&self) -> Box<dyn AgeAdapterV2> {
        Box::new(Self)
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

    #[test]
    fn test_shell_adapter_v2_stream_round_trip() {
        if which::which("age").is_err() {
            println!("Streaming test skipped: age binary not available");
            return;
        }

        let adapter = ShellAdapterV2::new().expect("Failed to create ShellAdapterV2");

        let mut plaintext = std::io::Cursor::new(b"streaming round trip".to_vec());
        let mut encrypted = Vec::new();

        adapter
            .encrypt_stream(
                &mut plaintext,
                &mut encrypted,
                &Identity::Passphrase("passphrase123".to_string()),
                None,
                OutputFormat::Binary,
            )
            .expect("Streaming encrypt failed");

        let mut encrypted_cursor = std::io::Cursor::new(encrypted);
        let mut decrypted = Vec::new();

        adapter
            .decrypt_stream(
                &mut encrypted_cursor,
                &mut decrypted,
                &Identity::Passphrase("passphrase123".to_string()),
            )
            .expect("Streaming decrypt failed");

        assert_eq!(decrypted, b"streaming round trip");
    }
}
