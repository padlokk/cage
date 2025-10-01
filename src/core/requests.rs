//! Request Structs for Cage Operations (CAGE-11)
//!
//! This module provides typed request structs to unify CLI and library entry points,
//! enabling a clean API for all encryption operations while maintaining backward compatibility.

use crate::core::{AgeConfig, OutputFormat};
use md5;
use serde::{Deserialize, Serialize};
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

/// Authority tier in the Ignite X/M/R/I/D hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AuthorityTier {
    /// X - Skull (top-level authority)
    Skull,
    /// M - Master (operational authority)
    Master,
    /// R - Repository (per-repo authority)
    Repository,
    /// I - Ignition (automation authority)
    Ignition,
    /// D - Distro (distribution authority)
    Distro,
}

impl AuthorityTier {
    /// Get tier designation as string
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthorityTier::Skull => "X",
            AuthorityTier::Master => "M",
            AuthorityTier::Repository => "R",
            AuthorityTier::Ignition => "I",
            AuthorityTier::Distro => "D",
        }
    }

    /// Parse tier from string designation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "X" => Some(AuthorityTier::Skull),
            "M" => Some(AuthorityTier::Master),
            "R" => Some(AuthorityTier::Repository),
            "I" => Some(AuthorityTier::Ignition),
            "D" => Some(AuthorityTier::Distro),
            _ => None,
        }
    }
}

/// Recipient group representing a collection of recipients with tier metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientGroup {
    /// Group identifier/name
    pub name: String,

    /// Recipients in this group
    pub recipients: Vec<String>,

    /// Authority tier this group represents
    pub tier: Option<AuthorityTier>,

    /// Group metadata (fingerprints, creation time, etc.)
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl RecipientGroup {
    /// Create a new recipient group
    pub fn new(name: String) -> Self {
        Self {
            name,
            recipients: Vec::new(),
            tier: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a group with tier designation
    pub fn with_tier(name: String, tier: AuthorityTier) -> Self {
        let mut group = Self::new(name);
        group.tier = Some(tier);
        group
    }

    /// Add recipient to group
    pub fn add_recipient(&mut self, recipient: String) {
        if !self.recipients.contains(&recipient) {
            self.recipients.push(recipient);
        }
    }

    /// Remove recipient from group
    pub fn remove_recipient(&mut self, recipient: &str) -> bool {
        if let Some(pos) = self.recipients.iter().position(|r| r == recipient) {
            self.recipients.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if group contains recipient
    pub fn contains_recipient(&self, recipient: &str) -> bool {
        self.recipients.contains(&String::from(recipient))
    }

    /// Get recipient count
    pub fn len(&self) -> usize {
        self.recipients.len()
    }

    /// Check if group is empty
    pub fn is_empty(&self) -> bool {
        self.recipients.is_empty()
    }

    /// Set authority tier for this group
    pub fn set_tier(&mut self, tier: Option<AuthorityTier>) {
        self.tier = tier;
    }

    /// Get group hash for audit logging (with stable ordering)
    pub fn group_hash(&self) -> String {
        let mut sorted = self.recipients.clone();
        sorted.sort();
        format!("{:x}", md5::compute(sorted.join(",").as_bytes()))
    }

    /// Set metadata field
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata field
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Multi-recipient configuration for operations
#[derive(Debug, Clone)]
pub struct MultiRecipientConfig {
    /// Primary recipient group
    pub primary_group: Option<RecipientGroup>,

    /// Additional recipient groups to include
    pub additional_groups: Vec<RecipientGroup>,

    /// Whether to validate recipient authority proofs
    pub validate_authority: bool,

    /// Whether to enforce tier hierarchy
    pub enforce_hierarchy: bool,
}

impl MultiRecipientConfig {
    /// Create new multi-recipient config
    pub fn new() -> Self {
        Self {
            primary_group: None,
            additional_groups: Vec::new(),
            validate_authority: false,
            enforce_hierarchy: false,
        }
    }

    /// Set primary recipient group
    pub fn with_primary_group(mut self, group: RecipientGroup) -> Self {
        self.primary_group = Some(group);
        self
    }

    /// Add additional recipient group
    pub fn add_group(mut self, group: RecipientGroup) -> Self {
        self.additional_groups.push(group);
        self
    }

    /// Enable authority validation
    pub fn with_authority_validation(mut self, enabled: bool) -> Self {
        self.validate_authority = enabled;
        self
    }

    /// Enable hierarchy enforcement
    pub fn with_hierarchy_enforcement(mut self, enabled: bool) -> Self {
        self.enforce_hierarchy = enabled;
        self
    }

    /// Flatten all groups into a single recipient list
    pub fn flatten_recipients(&self) -> Vec<String> {
        let mut all_recipients = Vec::new();

        if let Some(ref primary) = self.primary_group {
            all_recipients.extend(primary.recipients.clone());
        }

        for group in &self.additional_groups {
            for recipient in &group.recipients {
                if !all_recipients.contains(recipient) {
                    all_recipients.push(recipient.clone());
                }
            }
        }

        all_recipients
    }

    /// Get total recipient count (deduplicated)
    pub fn total_recipients(&self) -> usize {
        self.flatten_recipients().len()
    }

    /// Get all groups
    pub fn all_groups(&self) -> Vec<&RecipientGroup> {
        let mut groups = Vec::new();
        if let Some(ref primary) = self.primary_group {
            groups.push(primary);
        }
        groups.extend(self.additional_groups.iter());
        groups
    }
}

impl Default for MultiRecipientConfig {
    fn default() -> Self {
        Self::new()
    }
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

    /// Multi-recipient configuration for advanced scenarios
    pub multi_recipient_config: Option<MultiRecipientConfig>,

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
            multi_recipient_config: None,
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

    /// Builder method to set multi-recipient configuration
    pub fn with_multi_recipient_config(mut self, config: MultiRecipientConfig) -> Self {
        self.multi_recipient_config = Some(config);
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
// BATCH REQUEST (BULK OPERATIONS)
// ============================================================================

/// Batch operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchOperation {
    /// Batch encrypt (lock)
    Lock,
    /// Batch decrypt (unlock)
    Unlock,
}

/// Request structure for batch directory operations
#[derive(Debug, Clone)]
pub struct BatchRequest {
    /// Target directory for the batch operation
    pub target: PathBuf,

    /// Operation (lock or unlock)
    pub operation: BatchOperation,

    /// Identity/passphrase used for the operation
    pub identity: Identity,

    /// Recipients for encryption workflows
    pub recipients: Option<Vec<Recipient>>,

    /// File pattern filter (glob)
    pub pattern: Option<String>,

    /// Recurse into sub-directories
    pub recursive: bool,

    /// Output format for encryption operations
    pub format: OutputFormat,

    /// Create backups before encrypting
    pub backup: bool,

    /// Unlock option: preserve encrypted file after decrypting
    pub preserve_encrypted: bool,

    /// Unlock option: verify before attempting decrypt
    pub verify_before_unlock: bool,

    /// Common request options (verbosity, dry-run, etc.)
    pub common: CommonOptions,
}

impl BatchRequest {
    /// Create a new batch request with required fields
    pub fn new(target: PathBuf, operation: BatchOperation, identity: Identity) -> Self {
        Self {
            target,
            operation,
            identity,
            recipients: None,
            pattern: None,
            recursive: true,
            format: OutputFormat::Binary,
            backup: false,
            preserve_encrypted: false,
            verify_before_unlock: true,
            common: CommonOptions::default(),
        }
    }

    /// Builder: set recipients for encryption operations
    pub fn with_recipients(mut self, recipients: Vec<Recipient>) -> Self {
        self.recipients = Some(recipients);
        self
    }

    /// Builder: apply glob pattern filter
    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = Some(pattern);
        self
    }

    /// Builder: enable/disable recursive traversal
    pub fn recursive(mut self, enabled: bool) -> Self {
        self.recursive = enabled;
        self
    }

    /// Builder: set output format for encryption operations
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Builder: enable backup before lock operations
    pub fn backup(mut self, enabled: bool) -> Self {
        self.backup = enabled;
        self
    }

    /// Builder: configure unlock preservation behaviour
    pub fn preserve_encrypted(mut self, enabled: bool) -> Self {
        self.preserve_encrypted = enabled;
        self
    }

    /// Builder: configure unlock verification behaviour
    pub fn verify_before_unlock(mut self, enabled: bool) -> Self {
        self.verify_before_unlock = enabled;
        self
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
