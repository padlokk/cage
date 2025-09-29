//! Request Structs for Cage Operations (CAGE-11)
//!
//! This module provides typed request structs to unify CLI and library entry points,
//! enabling a clean API for all encryption operations while maintaining backward compatibility.

use crate::cage::config::{AgeConfig, OutputFormat};
use std::path::PathBuf;

// ============================================================================
// COMMON REQUEST OPTIONS
// ============================================================================

/// Common options shared across multiple request types
#[derive(Debug, Clone, Default)]
pub struct CommonOptions {
    /// Enable verbose output
    pub verbose: bool,

    /// Enable quiet mode (suppress non-error output)
    pub quiet: bool,

    /// Enable dry-run mode (preview without executing)
    pub dry_run: bool,

    /// Force operation without confirmations
    pub force: bool,

    /// Custom configuration override
    pub config: Option<AgeConfig>,
}

/// Identity configuration for encryption/decryption operations
#[derive(Debug, Clone)]
pub enum Identity {
    /// Use passphrase-based encryption
    Passphrase(String),

    /// Use identity file (age -i flag)
    IdentityFile(PathBuf),

    /// Use SSH key as identity
    SshKey(PathBuf),

    /// Prompt for passphrase interactively
    PromptPassphrase,
}

/// Recipient configuration for encryption operations
#[derive(Debug, Clone)]
pub enum Recipient {
    /// Single recipient public key
    PublicKey(String),

    /// Multiple recipient public keys
    MultipleKeys(Vec<String>),

    /// Recipients from file
    RecipientsFile(PathBuf),

    /// SSH recipients
    SshRecipients(Vec<String>),

    /// Self (encrypt to own identity)
    SelfRecipient,
}

// ============================================================================
// LOCK REQUEST (ENCRYPTION)
// ============================================================================

/// Request structure for lock (encryption) operations
#[derive(Debug, Clone)]
pub struct LockRequest {
    /// Target file or directory to encrypt
    pub target: PathBuf,

    /// Identity/passphrase for encryption
    pub identity: Identity,

    /// Recipients for public key encryption (optional)
    pub recipients: Option<Vec<Recipient>>,

    /// Output format (Binary or ASCII armor)
    pub format: OutputFormat,

    /// Process directories recursively
    pub recursive: bool,

    /// File pattern filter (glob patterns)
    pub pattern: Option<String>,

    /// Create backup before locking
    pub backup: bool,

    /// Custom backup directory
    pub backup_dir: Option<PathBuf>,

    /// In-place encryption (overwrite original)
    pub in_place: bool,

    /// Common options
    pub common: CommonOptions,
}

impl LockRequest {
    /// Create a new lock request with minimal required fields
    pub fn new(target: PathBuf, identity: Identity) -> Self {
        Self {
            target,
            identity,
            recipients: None,
            format: OutputFormat::Binary,
            recursive: false,
            pattern: None,
            backup: true,
            backup_dir: None,
            in_place: false,
            common: CommonOptions::default(),
        }
    }

    /// Builder method to set recipients
    pub fn with_recipients(mut self, recipients: Vec<Recipient>) -> Self {
        self.recipients = Some(recipients);
        self
    }

    /// Builder method to enable recursive mode
    pub fn recursive(mut self, enabled: bool) -> Self {
        self.recursive = enabled;
        self
    }

    /// Builder method to set pattern filter
    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = Some(pattern);
        self
    }

    /// Builder method to set output format
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }
}

// ============================================================================
// UNLOCK REQUEST (DECRYPTION)
// ============================================================================

/// Request structure for unlock (decryption) operations
#[derive(Debug, Clone)]
pub struct UnlockRequest {
    /// Target file or directory to decrypt
    pub target: PathBuf,

    /// Identity/passphrase for decryption
    pub identity: Identity,

    /// Process directories recursively
    pub recursive: bool,

    /// File pattern filter
    pub pattern: Option<String>,

    /// Verify integrity before unlocking
    pub verify_first: bool,

    /// Selective unlock (skip invalid files)
    pub selective: bool,

    /// Preserve encrypted files after unlock
    pub preserve_encrypted: bool,

    /// In-place decryption
    pub in_place: bool,

    /// Common options
    pub common: CommonOptions,
}

impl UnlockRequest {
    /// Create a new unlock request with minimal required fields
    pub fn new(target: PathBuf, identity: Identity) -> Self {
        Self {
            target,
            identity,
            recursive: false,
            pattern: None,
            verify_first: true,
            selective: false,
            preserve_encrypted: false,
            in_place: false,
            common: CommonOptions::default(),
        }
    }

    /// Builder method for recursive mode
    pub fn recursive(mut self, enabled: bool) -> Self {
        self.recursive = enabled;
        self
    }

    /// Builder method for selective unlock
    pub fn selective(mut self, enabled: bool) -> Self {
        self.selective = enabled;
        self
    }

    /// Builder method to preserve encrypted files
    pub fn preserve_encrypted(mut self, enabled: bool) -> Self {
        self.preserve_encrypted = enabled;
        self
    }

    /// Builder method to set pattern filter
    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = Some(pattern);
        self
    }
}

// ============================================================================
// ROTATE REQUEST (KEY ROTATION)
// ============================================================================

/// Request structure for key rotation operations
#[derive(Debug, Clone)]
pub struct RotateRequest {
    /// Target file or directory for rotation
    pub target: PathBuf,

    /// Current identity/passphrase
    pub current_identity: Identity,

    /// New identity/passphrase
    pub new_identity: Identity,

    /// New recipients (if changing to public key encryption)
    pub new_recipients: Option<Vec<Recipient>>,

    /// Process directories recursively
    pub recursive: bool,

    /// File pattern filter
    pub pattern: Option<String>,

    /// Create backup before rotation
    pub backup: bool,

    /// Atomic rotation (all-or-nothing)
    pub atomic: bool,

    /// Common options
    pub common: CommonOptions,
}

impl RotateRequest {
    /// Create a new rotation request
    pub fn new(target: PathBuf, current: Identity, new: Identity) -> Self {
        Self {
            target,
            current_identity: current,
            new_identity: new,
            new_recipients: None,
            recursive: false,
            pattern: None,
            backup: true,
            atomic: true,
            common: CommonOptions::default(),
        }
    }

    /// Builder method to set new recipients
    pub fn with_new_recipients(mut self, recipients: Vec<Recipient>) -> Self {
        self.new_recipients = Some(recipients);
        self
    }

    /// Builder method for atomic mode
    pub fn atomic(mut self, enabled: bool) -> Self {
        self.atomic = enabled;
        self
    }
}

// ============================================================================
// VERIFY REQUEST (INTEGRITY CHECK)
// ============================================================================

/// Request structure for verification operations
#[derive(Debug, Clone)]
pub struct VerifyRequest {
    /// Target file or directory to verify
    pub target: PathBuf,

    /// Identity for verification (optional - may just check format)
    pub identity: Option<Identity>,

    /// Process directories recursively
    pub recursive: bool,

    /// File pattern filter
    pub pattern: Option<String>,

    /// Deep verification (attempt decryption)
    pub deep_verify: bool,

    /// Report format
    pub report_format: ReportFormat,

    /// Common options
    pub common: CommonOptions,
}

/// Report format for verification results
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    /// Simple text output
    Simple,
    /// Detailed text report
    Detailed,
    /// JSON format for machine parsing
    Json,
    /// CSV format for spreadsheets
    Csv,
}

impl VerifyRequest {
    /// Create a new verification request
    pub fn new(target: PathBuf) -> Self {
        Self {
            target,
            identity: None,
            recursive: false,
            pattern: None,
            deep_verify: false,
            report_format: ReportFormat::Simple,
            common: CommonOptions::default(),
        }
    }

    /// Builder method for deep verification
    pub fn deep_verify(mut self, identity: Identity) -> Self {
        self.identity = Some(identity);
        self.deep_verify = true;
        self
    }

    /// Builder method for report format
    pub fn with_report_format(mut self, format: ReportFormat) -> Self {
        self.report_format = format;
        self
    }
}

// ============================================================================
// STATUS REQUEST (REPOSITORY STATUS)
// ============================================================================

/// Request structure for status operations
#[derive(Debug, Clone)]
pub struct StatusRequest {
    /// Target directory to check
    pub target: PathBuf,

    /// Include subdirectories
    pub recursive: bool,

    /// File pattern filter
    pub pattern: Option<String>,

    /// Show detailed information
    pub detailed: bool,

    /// Report format
    pub report_format: ReportFormat,

    /// Common options
    pub common: CommonOptions,
}

impl StatusRequest {
    /// Create a new status request
    pub fn new(target: PathBuf) -> Self {
        Self {
            target,
            recursive: false,
            pattern: None,
            detailed: false,
            report_format: ReportFormat::Simple,
            common: CommonOptions::default(),
        }
    }

    /// Builder method for detailed mode
    pub fn detailed(mut self, enabled: bool) -> Self {
        self.detailed = enabled;
        self
    }
}

// ============================================================================
// STREAM REQUEST (STREAMING OPERATIONS)
// ============================================================================

/// Request structure for streaming encryption/decryption
#[derive(Debug, Clone)]
pub struct StreamRequest {
    /// Operation type (encrypt or decrypt)
    pub operation: StreamOperation,

    /// Identity for the operation
    pub identity: Identity,

    /// Recipients (for encryption)
    pub recipients: Option<Vec<Recipient>>,

    /// Output format (for encryption)
    pub format: OutputFormat,

    /// Buffer size for streaming
    pub buffer_size: usize,

    /// Common options
    pub common: CommonOptions,
}

/// Stream operation type
#[derive(Debug, Clone, Copy)]
pub enum StreamOperation {
    /// Stream encryption
    Encrypt,
    /// Stream decryption
    Decrypt,
}

impl StreamRequest {
    /// Create a new stream encryption request
    pub fn encrypt(identity: Identity) -> Self {
        Self {
            operation: StreamOperation::Encrypt,
            identity,
            recipients: None,
            format: OutputFormat::Binary,
            buffer_size: 8192,
            common: CommonOptions::default(),
        }
    }

    /// Create a new stream decryption request
    pub fn decrypt(identity: Identity) -> Self {
        Self {
            operation: StreamOperation::Decrypt,
            identity,
            recipients: None,
            format: OutputFormat::Binary,
            buffer_size: 8192,
            common: CommonOptions::default(),
        }
    }
}

// ============================================================================
// CONVERSION HELPERS
// ============================================================================

/// Trait for converting CLI arguments to request structs
/// Note: Requires clap dependency when implemented
pub trait FromCliArgs {
    /// The request type to convert to
    type Request;

    /// Convert from CLI arguments (requires clap crate)
    /// Using generic parameter to avoid hard dependency
    fn from_cli_args(args: &impl std::any::Any) -> Result<Self::Request, String>;
}

/// Trait for converting request structs to operation parameters
pub trait ToOperationParams {
    /// Convert to parameters suitable for the underlying operation
    fn to_params(&self) -> Result<serde_json::Value, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_request_builder() {
        let request = LockRequest::new(
            PathBuf::from("/test/file.txt"),
            Identity::Passphrase("test123".to_string()),
        )
        .recursive(true)
        .with_pattern("*.txt".to_string())
        .with_format(OutputFormat::AsciiArmor);

        assert!(request.recursive);
        assert_eq!(request.pattern, Some("*.txt".to_string()));
        assert_eq!(request.format, OutputFormat::AsciiArmor);
    }

    #[test]
    fn test_unlock_request_builder() {
        let request =
            UnlockRequest::new(PathBuf::from("/test/file.cage"), Identity::PromptPassphrase)
                .selective(true)
                .preserve_encrypted(true);

        assert!(request.selective);
        assert!(request.preserve_encrypted);
    }

    #[test]
    fn test_identity_variants() {
        let _pass = Identity::Passphrase("secret".to_string());
        let _file = Identity::IdentityFile(PathBuf::from("~/.age/key.txt"));
        let _ssh = Identity::SshKey(PathBuf::from("~/.ssh/id_rsa"));
        let _prompt = Identity::PromptPassphrase;
    }
}
