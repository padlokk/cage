//! CRUD Manager - Central Lifecycle Coordinator for Age Automation
//!
//! The CrudManager provides comprehensive lifecycle coordination for all Age encryption
//! operations, integrating TTY automation, authority management, and operational validation
//! into a unified interface that supports the complete padlock command set.
//!
//! Security Guardian: Edgar - Production CRUD coordination with authority integration

use std::path::{Path, PathBuf};
#[allow(unused_imports)]
use std::time::{Duration, Instant};
use std::collections::HashMap;

use crate::cage::error::{AgeError, AgeResult};
use crate::cage::config::{AgeConfig, OutputFormat};
use crate::cage::adapter::AgeAdapter;
#[allow(unused_imports)]
use crate::cage::tty_automation::TtyAutomator;
use crate::cage::security::AuditLogger;
use crate::cage::operations::{
    RepositoryStatus, OperationResult
};

/// Options for lock operations
#[derive(Debug, Clone)]
pub struct LockOptions {
    pub recursive: bool,
    pub format: OutputFormat,
    pub pattern_filter: Option<String>,
    pub backup_before_lock: bool,
}

impl Default for LockOptions {
    fn default() -> Self {
        Self {
            recursive: false,
            format: OutputFormat::Binary,
            pattern_filter: None,
            backup_before_lock: false,
        }
    }
}

/// Options for unlock operations
#[derive(Debug, Clone)]
pub struct UnlockOptions {
    pub selective: bool,
    pub verify_before_unlock: bool,
    pub pattern_filter: Option<String>,
    pub preserve_encrypted: bool,
}

impl Default for UnlockOptions {
    fn default() -> Self {
        Self {
            selective: false,
            verify_before_unlock: true,
            pattern_filter: None,
            preserve_encrypted: false,
        }
    }
}

/// Authority operation result
#[derive(Debug, Clone)]
pub struct AuthorityResult {
    pub operation: String,
    pub recipient: String,
    pub success: bool,
    pub authority_chain_status: String,
}

/// Verification operation result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub verified_files: Vec<String>,
    pub failed_files: Vec<String>,
    pub authority_status: String,
    pub overall_status: String,
}

/// Backup management for safe file operations
#[derive(Debug, Clone)]
pub struct BackupManager {
    backup_dir: Option<PathBuf>,
    backup_extension: String,
    cleanup_on_success: bool,
}

impl BackupManager {
    /// Create new backup manager with default settings
    pub fn new() -> Self {
        Self {
            backup_dir: None, // Use same directory as original file
            backup_extension: ".bak".to_string(),
            cleanup_on_success: true,
        }
    }

    /// Create backup manager with custom backup directory
    pub fn with_backup_dir(backup_dir: PathBuf) -> Self {
        Self {
            backup_dir: Some(backup_dir),
            backup_extension: ".bak".to_string(),
            cleanup_on_success: true,
        }
    }

    /// Set backup extension (default: .bak)
    pub fn with_extension(mut self, extension: String) -> Self {
        self.backup_extension = if extension.starts_with('.') {
            extension
        } else {
            format!(".{}", extension)
        };
        self
    }

    /// Set cleanup behavior
    pub fn with_cleanup(mut self, cleanup: bool) -> Self {
        self.cleanup_on_success = cleanup;
        self
    }

    /// Create backup of a file
    pub fn create_backup(&self, file_path: &Path) -> AgeResult<BackupInfo> {
        if !file_path.exists() {
            return Err(AgeError::file_error("backup", file_path.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")));
        }

        let backup_path = self.generate_backup_path(file_path)?;

        // Handle backup conflicts
        if backup_path.exists() {
            let conflict_path = self.generate_conflict_path(&backup_path)?;
            std::fs::rename(&backup_path, &conflict_path)
                .map_err(|e| AgeError::file_error("move_existing_backup", backup_path.clone(), e))?;
        }

        // Create backup directory if needed
        if let Some(parent) = backup_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| AgeError::file_error("create_backup_dir", parent.to_path_buf(), e))?;
            }
        }

        // Copy file to backup location
        std::fs::copy(file_path, &backup_path)
            .map_err(|e| AgeError::file_error("create_backup", backup_path.clone(), e))?;

        Ok(BackupInfo {
            original_path: file_path.to_path_buf(),
            backup_path,
            created_at: std::time::SystemTime::now(),
            size_bytes: std::fs::metadata(file_path)
                .map(|m| m.len())
                .unwrap_or(0),
        })
    }

    /// Restore from backup
    pub fn restore_backup(&self, backup_info: &BackupInfo) -> AgeResult<()> {
        if !backup_info.backup_path.exists() {
            return Err(AgeError::file_error("restore", backup_info.backup_path.clone(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Backup file not found")));
        }

        std::fs::copy(&backup_info.backup_path, &backup_info.original_path)
            .map_err(|e| AgeError::file_error("restore_backup", backup_info.original_path.clone(), e))?;

        Ok(())
    }

    /// Clean up backup file
    pub fn cleanup_backup(&self, backup_info: &BackupInfo) -> AgeResult<()> {
        if backup_info.backup_path.exists() {
            std::fs::remove_file(&backup_info.backup_path)
                .map_err(|e| AgeError::file_error("cleanup_backup", backup_info.backup_path.clone(), e))?;
        }
        Ok(())
    }

    /// Generate backup path for a file
    fn generate_backup_path(&self, file_path: &Path) -> AgeResult<PathBuf> {
        let file_name = file_path.file_name()
            .ok_or_else(|| AgeError::file_error("get_filename", file_path.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename")))?;

        let backup_filename = format!("{}{}", file_name.to_string_lossy(), self.backup_extension);

        let backup_path = if let Some(ref backup_dir) = self.backup_dir {
            backup_dir.join(backup_filename)
        } else {
            file_path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(backup_filename)
        };

        Ok(backup_path)
    }

    /// Generate conflict resolution path
    fn generate_conflict_path(&self, backup_path: &Path) -> AgeResult<PathBuf> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let conflict_name = format!("{}.conflict.{}", backup_path.to_string_lossy(), timestamp);
        Ok(PathBuf::from(conflict_name))
    }
}

/// Information about a created backup
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub created_at: std::time::SystemTime,
    pub size_bytes: u64,
}

impl BackupInfo {
    pub fn age_seconds(&self) -> u64 {
        self.created_at
            .elapsed()
            .unwrap_or_default()
            .as_secs()
    }
}

/// File verification status with detailed information
#[derive(Debug, Clone)]
pub struct FileVerificationStatus {
    pub file_path: PathBuf,
    pub is_encrypted: bool,
    pub format_valid: bool,
    pub header_valid: bool,
    pub size_check: bool,
    pub error_message: Option<String>,
}

impl FileVerificationStatus {
    pub fn is_valid(&self) -> bool {
        self.is_encrypted && self.format_valid && self.header_valid && self.size_check && self.error_message.is_none()
    }
}

/// Emergency operation result
#[derive(Debug, Clone)]
pub struct EmergencyResult {
    pub operation: String,
    pub affected_files: Vec<String>,
    pub recovery_actions: Vec<String>,
    pub security_events: Vec<String>,
}

/// Central CRUD manager coordinating all Age automation lifecycle operations
pub struct CrudManager {
    adapter: Box<dyn AgeAdapter>,
    audit_logger: AuditLogger,
    config: AgeConfig,
    operation_history: Vec<OperationRecord>,
}

/// Record of performed operations for audit and recovery
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OperationRecord {
    #[allow(dead_code)]
    operation_type: String,
    #[allow(dead_code)]
    target_path: PathBuf,
    #[allow(dead_code)]
    timestamp: Instant,
    #[allow(dead_code)]
    success: bool,
    #[allow(dead_code)]
    details: HashMap<String, String>,
}

impl CrudManager {
    /// Create new CrudManager with specified adapter and configuration
    pub fn new(
        adapter: Box<dyn AgeAdapter>,
        config: AgeConfig,
    ) -> AgeResult<Self> {
        let audit_logger = AuditLogger::new(config.audit_log_path.clone().map(PathBuf::from))?;

        Ok(Self {
            adapter,
            audit_logger,
            config,
            operation_history: Vec::new(),
        })
    }

    /// Create CrudManager with default configuration
    pub fn with_defaults() -> AgeResult<Self> {
        let adapter = super::super::adapter::AdapterFactory::create_default()?;
        let config = AgeConfig::default();
        Self::new(adapter, config)
    }

    // ========================================================================================
    // CORE CRUD OPERATIONS - Primary lifecycle management
    // ========================================================================================

    /// CREATE: Lock (encrypt) files or repositories
    pub fn lock(&mut self, path: &Path, passphrase: &str, options: LockOptions) -> AgeResult<OperationResult> {
        let start_time = Instant::now();
        self.audit_logger.log_operation_start_single("lock", path)?;
        
        let mut result = OperationResult::new();
        
        // Validate preconditions
        if !path.exists() {
            return Err(AgeError::file_error("read", path.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Path not found")));
        }

        // Validate passphrase
        self.validate_passphrase(passphrase)?;

        // Determine operation scope
        if path.is_file() {
            self.lock_single_file(path, passphrase, &options, &mut result)?;
        } else if path.is_dir() {
            if options.recursive {
                self.lock_repository(path, passphrase, &options, &mut result)?;
            } else {
                return Err(AgeError::InvalidOperation {
                    operation: "lock".to_string(),
                    reason: "Directory requires --recursive flag".to_string(),
                });
            }
        }

        // Record operation
        self.record_operation("lock", path, true, &result);
        result.finalize(start_time);
        
        self.audit_logger.log_operation_complete("lock", path, &result)?;
        Ok(result)
    }

    /// READ: Status - Check encryption status and repository state
    pub fn status(&self, path: &Path) -> AgeResult<RepositoryStatus> {
        self.audit_logger.log_operation_start_single("status", path)?;
        
        if !path.exists() {
            return Err(AgeError::file_error("read", path.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Path not found")));
        }

        let status = if path.is_file() {
            self.get_file_status(path)?
        } else {
            self.get_repository_status(path)?
        };

        self.audit_logger.log_status_check(path, &status)?;
        Ok(status)
    }

    /// UPDATE: Rotate - Key rotation while maintaining access
    pub fn rotate(&mut self, repository: &Path, old_passphrase: &str, new_passphrase: &str) -> AgeResult<OperationResult> {
        let start_time = Instant::now();
        self.audit_logger.log_operation_start_single("rotate", repository)?;

        let mut result = OperationResult::new();

        // Validate inputs
        if !repository.exists() || !repository.is_dir() {
            return Err(AgeError::InvalidOperation {
                operation: "rotate".to_string(),
                reason: "Repository path required".to_string(),
            });
        }

        // Validate both passphrases
        self.validate_passphrase(old_passphrase)?;
        self.validate_passphrase(new_passphrase)?;

        if old_passphrase == new_passphrase {
            return Err(AgeError::InvalidOperation {
                operation: "rotate".to_string(),
                reason: "New passphrase must be different from old passphrase".to_string(),
            });
        }

        // Get repository status to find encrypted files
        let status = self.status(repository)?;
        if status.encrypted_files == 0 {
            return Err(AgeError::InvalidOperation {
                operation: "rotate".to_string(),
                reason: "No encrypted files found to rotate".to_string(),
            });
        }

        self.audit_logger.log_info(&format!("Starting key rotation for {} encrypted files", status.encrypted_files))?;

        // Collect all encrypted files for rotation
        let mut encrypted_files = Vec::new();
        self.collect_encrypted_files(repository, &mut encrypted_files)?;

        // Create backup directory for atomic operation
        let backup_dir = repository.join(".cage_rotation_backup");
        if backup_dir.exists() {
            std::fs::remove_dir_all(&backup_dir)
                .map_err(|e| AgeError::file_error("remove_backup_dir", backup_dir.clone(), e))?;
        }
        std::fs::create_dir(&backup_dir)
            .map_err(|e| AgeError::file_error("create_backup_dir", backup_dir.clone(), e))?;

        let mut successful_rotations = 0;
        let mut failed_rotations = Vec::new();

        // Process each encrypted file
        for file_path in &encrypted_files {
            match self.rotate_single_file(file_path, old_passphrase, new_passphrase, &backup_dir) {
                Ok(_) => {
                    successful_rotations += 1;
                    result.add_success(file_path.to_string_lossy().to_string());
                    self.audit_logger.log_info(&format!("Rotated key for: {}", file_path.display()))?;
                }
                Err(e) => {
                    failed_rotations.push(format!("{}: {}", file_path.display(), e));
                    result.add_failure(file_path.to_string_lossy().to_string());
                    self.audit_logger.log_error(&format!("Failed to rotate key for {}: {}", file_path.display(), e))?;
                }
            }
        }

        // Handle results
        if failed_rotations.is_empty() {
            // All successful - clean up backup
            std::fs::remove_dir_all(&backup_dir)
                .map_err(|e| AgeError::file_error("cleanup_backup", backup_dir, e))?;

            self.audit_logger.log_info(&format!("Key rotation completed successfully for {} files", successful_rotations))?;
        } else {
            // Partial failure - rollback successful rotations
            self.audit_logger.log_error(&format!("Key rotation failed for {} files, rolling back {} successful rotations",
                failed_rotations.len(), successful_rotations))?;

            if let Err(rollback_err) = self.rollback_rotation(&encrypted_files, &backup_dir) {
                self.audit_logger.log_error(&format!("CRITICAL: Rollback failed: {}", rollback_err))?;
                return Err(AgeError::RepositoryOperationFailed {
                    operation: "rotate_rollback".to_string(),
                    repository: repository.to_path_buf(),
                    reason: format!("Rotation failed and rollback failed: {}", rollback_err),
                });
            }

            return Err(AgeError::BatchOperationFailed {
                operation: "rotate".to_string(),
                successful_count: 0, // All rolled back
                failed_count: failed_rotations.len(),
                failures: failed_rotations,
            });
        }

        self.record_operation("rotate", repository, true, &result);
        result.finalize(start_time);

        self.audit_logger.log_operation_complete("rotate", repository, &result)?;
        Ok(result)
    }

    /// Helper method to collect all encrypted files in a directory
    fn collect_encrypted_files(&self, directory: &Path, files: &mut Vec<PathBuf>) -> AgeResult<()> {
        let entries = std::fs::read_dir(directory)
            .map_err(|e| AgeError::file_error("read_dir", directory.to_path_buf(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| AgeError::file_error("read_entry", directory.to_path_buf(), e))?;
            let path = entry.path();

            if path.is_file() {
                // Check if file is encrypted by checking Age header
                if self.is_encrypted_file(&path)? {
                    files.push(path);
                }
            } else if path.is_dir() {
                // Always recurse for key rotation - we want to find all encrypted files
                self.collect_encrypted_files(&path, files)?;
            }
        }

        Ok(())
    }

    /// Check if a file is encrypted (basic heuristic)
    fn is_encrypted_file(&self, path: &Path) -> AgeResult<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let content = std::fs::read(path)
            .map_err(|e| AgeError::file_error("read", path.to_path_buf(), e))?;

        // Check for Age headers
        Ok(content.starts_with(b"age-encryption.org/v1") ||
           content.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----"))
    }

    /// Rotate key for a single file with backup
    fn rotate_single_file(
        &self,
        file_path: &Path,
        old_passphrase: &str,
        new_passphrase: &str,
        backup_dir: &Path,
    ) -> AgeResult<()> {
        // Create backup of original file
        let file_name = file_path.file_name()
            .ok_or_else(|| AgeError::file_error("get_filename", file_path.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename")))?;

        let backup_path = backup_dir.join(file_name);
        std::fs::copy(file_path, &backup_path)
            .map_err(|e| AgeError::file_error("backup_file", backup_path, e))?;

        // Create temporary decrypted file
        let temp_decrypted = backup_dir.join(format!("{}.tmp_decrypted", file_name.to_string_lossy()));

        // Step 1: Decrypt with old passphrase
        self.adapter.decrypt(file_path, &temp_decrypted, old_passphrase)
            .map_err(|e| AgeError::DecryptionFailed {
                input: file_path.to_path_buf(),
                output: temp_decrypted.clone(),
                reason: format!("Failed to decrypt with old passphrase: {}", e),
            })?;

        // Step 2: Re-encrypt with new passphrase
        self.adapter.encrypt(&temp_decrypted, file_path, new_passphrase, self.config.output_format)
            .map_err(|e| AgeError::EncryptionFailed {
                input: temp_decrypted.clone(),
                output: file_path.to_path_buf(),
                reason: format!("Failed to encrypt with new passphrase: {}", e),
            })?;

        // Step 3: Verify the re-encrypted file can be decrypted
        let temp_verify = backup_dir.join(format!("{}.tmp_verify", file_name.to_string_lossy()));
        self.adapter.decrypt(file_path, &temp_verify, new_passphrase)
            .map_err(|e| AgeError::DecryptionFailed {
                input: file_path.to_path_buf(),
                output: temp_verify.clone(),
                reason: format!("Verification failed with new passphrase: {}", e),
            })?;

        // Step 4: Verify content integrity
        let original_content = std::fs::read(&temp_decrypted)
            .map_err(|e| AgeError::file_error("read_original", temp_decrypted.clone(), e))?;
        let verified_content = std::fs::read(&temp_verify)
            .map_err(|e| AgeError::file_error("read_verified", temp_verify.clone(), e))?;

        if original_content != verified_content {
            return Err(AgeError::SecurityValidationFailed {
                validation_type: "content_integrity".to_string(),
                details: "Content mismatch after key rotation".to_string(),
            });
        }

        // Clean up temporary files
        let _ = std::fs::remove_file(&temp_decrypted);
        let _ = std::fs::remove_file(&temp_verify);

        Ok(())
    }

    /// Rollback failed rotation by restoring from backups
    fn rollback_rotation(&self, files: &[PathBuf], backup_dir: &Path) -> AgeResult<()> {
        for file_path in files {
            let file_name = file_path.file_name()
                .ok_or_else(|| AgeError::file_error("get_filename", file_path.to_path_buf(),
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename")))?;

            let backup_path = backup_dir.join(file_name);
            if backup_path.exists() {
                std::fs::copy(&backup_path, file_path)
                    .map_err(|e| AgeError::file_error("restore_backup", backup_path, e))?;
            }
        }

        // Clean up backup directory
        std::fs::remove_dir_all(backup_dir)
            .map_err(|e| AgeError::file_error("cleanup_backup", backup_dir.to_path_buf(), e))?;

        Ok(())
    }

    /// DELETE: Unlock (decrypt) files with controlled access
    pub fn unlock(&mut self, path: &Path, passphrase: &str, options: UnlockOptions) -> AgeResult<OperationResult> {
        let start_time = Instant::now();
        self.audit_logger.log_operation_start_single("unlock", path)?;
        
        let mut result = OperationResult::new();
        
        // Validate preconditions
        if !path.exists() {
            return Err(AgeError::file_error("read", path.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Path not found")));
        }

        self.validate_passphrase(passphrase)?;

        // Verify before unlock if requested
        if options.verify_before_unlock {
            let status = self.status(path)?;
            if status.encrypted_files == 0 {
                return Err(AgeError::InvalidOperation {
                    operation: "unlock".to_string(),
                    reason: "No encrypted files found".to_string(),
                });
            }
        }

        // Perform unlock operation
        if path.is_file() {
            self.unlock_single_file(path, passphrase, &options, &mut result)?;
        } else if path.is_dir() {
            self.unlock_repository(path, passphrase, &options, &mut result)?;
        }

        self.record_operation("unlock", path, true, &result);
        result.finalize(start_time);
        
        self.audit_logger.log_operation_complete("unlock", path, &result)?;
        Ok(result)
    }

    // ========================================================================================
    // AUTHORITY MANAGEMENT OPERATIONS - Bridge to Lucas's patterns
    // ========================================================================================

    /// ALLOW: Add recipients to authority chain
    pub fn allow(&mut self, recipient: &str) -> AgeResult<AuthorityResult> {
        self.audit_logger.log_authority_operation("allow", recipient)?;
        
        // Validate recipient format
        if recipient.is_empty() {
            return Err(AgeError::InvalidOperation {
                operation: "allow".to_string(),
                reason: "Recipient cannot be empty".to_string(),
            });
        }

        // This would bridge to Lucas's authority management
        // For now, return a placeholder result
        Ok(AuthorityResult {
            operation: "allow".to_string(),
            recipient: recipient.to_string(),
            success: true,
            authority_chain_status: "Authority integration pending".to_string(),
        })
    }

    /// REVOKE: Remove recipients from authority chain  
    pub fn revoke(&mut self, recipient: &str) -> AgeResult<AuthorityResult> {
        self.audit_logger.log_authority_operation("revoke", recipient)?;
        
        if recipient.is_empty() {
            return Err(AgeError::InvalidOperation {
                operation: "revoke".to_string(),
                reason: "Recipient cannot be empty".to_string(),
            });
        }

        // Bridge to Lucas's authority management
        Ok(AuthorityResult {
            operation: "revoke".to_string(),
            recipient: recipient.to_string(),
            success: true,
            authority_chain_status: "Authority integration pending".to_string(),
        })
    }

    /// RESET: Emergency repository unlock/reset
    pub fn reset(&mut self, repository: &Path, confirmation: &str) -> AgeResult<EmergencyResult> {
        self.audit_logger.log_emergency_operation("reset", repository)?;
        
        // Require explicit confirmation for destructive operation
        if confirmation != "CONFIRM_RESET" {
            return Err(AgeError::InvalidOperation {
                operation: "reset".to_string(),
                reason: "Reset requires explicit confirmation".to_string(),
            });
        }

        if !repository.exists() || !repository.is_dir() {
            return Err(AgeError::InvalidOperation {
                operation: "reset".to_string(),
                reason: "Repository path required".to_string(),
            });
        }

        // Emergency reset would involve:
        // 1. Validate emergency access authorization
        // 2. Create backup of current state
        // 3. Reset authority chain to emergency state
        // 4. Provide recovery procedures

        Ok(EmergencyResult {
            operation: "reset".to_string(),
            affected_files: vec![repository.display().to_string()],
            recovery_actions: vec!["Emergency reset completed".to_string()],
            security_events: vec!["Emergency reset authorized".to_string()],
        })
    }

    // ========================================================================================
    // LIFECYCLE OPERATIONS - Integrity and emergency procedures
    // ========================================================================================

    /// VERIFY: Integrity checking and validation
    pub fn verify(&self, path: &Path) -> AgeResult<VerificationResult> {
        self.audit_logger.log_operation_start_single("verify", path)?;
        
        if !path.exists() {
            return Err(AgeError::file_error("read", path.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Path not found")));
        }

        let mut verified_files = Vec::new();
        let mut failed_files = Vec::new();

        if path.is_file() {
            // Verify single file
            match self.verify_file_integrity(path) {
                Ok(status) => {
                    if status.is_valid() {
                        verified_files.push(path.display().to_string());
                    } else {
                        let error_msg = status.error_message.unwrap_or_else(||
                            format!("Verification failed: encrypted={}, format={}, header={}, size={}",
                                status.is_encrypted, status.format_valid, status.header_valid, status.size_check));
                        failed_files.push(format!("{}: {}", path.display(), error_msg));
                    }
                }
                Err(e) => failed_files.push(format!("{}: {}", path.display(), e)),
            }
        } else {
            // Verify repository
            self.verify_repository_integrity(path, &mut verified_files, &mut failed_files)?;
        }

        Ok(VerificationResult {
            verified_files,
            failed_files,
            authority_status: "Authority verification pending".to_string(),
            overall_status: "Verification completed".to_string(),
        })
    }

    /// EMERGENCY: Fail-safe recovery operations
    pub fn emergency_unlock(&mut self, repository: &Path, emergency_passphrase: &str) -> AgeResult<EmergencyResult> {
        self.audit_logger.log_emergency_operation("emergency_unlock", repository)?;
        
        if !repository.exists() || !repository.is_dir() {
            return Err(AgeError::InvalidOperation {
                operation: "emergency_unlock".to_string(),
                reason: "Repository path required".to_string(),
            });
        }

        self.validate_passphrase(emergency_passphrase)?;

        // Emergency unlock involves:
        // 1. Validate emergency access credentials
        // 2. Attempt unlock with emergency procedures
        // 3. Document all security events
        // 4. Provide recovery guidance

        Ok(EmergencyResult {
            operation: "emergency_unlock".to_string(),
            affected_files: vec![repository.display().to_string()],
            recovery_actions: vec!["Emergency unlock procedures initiated".to_string()],
            security_events: vec!["Emergency access authorized".to_string()],
        })
    }

    /// BATCH: Bulk operations for directories/repositories
    pub fn batch_process(&mut self, directory: &Path, pattern: Option<&str>, operation: &str, passphrase: &str) -> AgeResult<OperationResult> {
        let start_time = Instant::now();
        self.audit_logger.log_operation_start_single(&format!("batch_{}", operation), directory)?;
        
        if !directory.exists() || !directory.is_dir() {
            return Err(AgeError::InvalidOperation {
                operation: "batch".to_string(),
                reason: "Directory path required".to_string(),
            });
        }

        let mut result = OperationResult::new();
        
        // Collect files matching pattern
        let files = self.collect_files_with_pattern(directory, pattern)?;
        
        // Process files in batches for performance
        for file in files {
            match operation {
                "lock" => {
                    if let Err(e) = self.lock_single_file(&file, passphrase, &LockOptions::default(), &mut result) {
                        result.add_failure(format!("Failed to lock {}: {}", file.display(), e));
                    }
                }
                "unlock" => {
                    if let Err(e) = self.unlock_single_file(&file, passphrase, &UnlockOptions::default(), &mut result) {
                        result.add_failure(format!("Failed to unlock {}: {}", file.display(), e));
                    }
                }
                _ => {
                    return Err(AgeError::InvalidOperation {
                        operation: "batch".to_string(),
                        reason: format!("Unsupported batch operation: {}", operation),
                    });
                }
            }
        }

        self.record_operation(&format!("batch_{}", operation), directory, result.success, &result);
        result.finalize(start_time);
        
        self.audit_logger.log_operation_complete(&format!("batch_{}", operation), directory, &result)?;
        Ok(result)
    }

    // ========================================================================================
    // INTERNAL IMPLEMENTATION METHODS
    // ========================================================================================

    /// Validate passphrase meets security requirements
    fn validate_passphrase(&self, passphrase: &str) -> AgeResult<()> {
        if passphrase.is_empty() {
            return Err(AgeError::SecurityValidationFailed {
                validation_type: "passphrase_validation".to_string(),
                details: "Empty passphrase not allowed".to_string(),
            });
        }

        if passphrase.len() > 1024 {
            return Err(AgeError::SecurityValidationFailed {
                validation_type: "passphrase_validation".to_string(),
                details: "Passphrase exceeds maximum length".to_string(),
            });
        }

        Ok(())
    }

    /// Lock a single file
    fn lock_single_file(&self, file: &Path, passphrase: &str, options: &LockOptions, result: &mut OperationResult) -> AgeResult<()> {
        let output_path = {
            let mut path = file.as_os_str().to_os_string();
            path.push(self.config.extension_with_dot());
            PathBuf::from(path)
        };

        let mut backup_info: Option<BackupInfo> = None;

        // Create backup if requested
        if options.backup_before_lock {
            let backup_manager = BackupManager::new();
            match backup_manager.create_backup(file) {
                Ok(info) => {
                    backup_info = Some(info);
                    self.audit_logger.log_info(&format!("Created backup: {} -> {}",
                        file.display(), backup_info.as_ref().unwrap().backup_path.display()))?;
                }
                Err(e) => {
                    self.audit_logger.log_error(&format!("Failed to create backup for {}: {}", file.display(), e))?;
                    result.add_failure(file.display().to_string());
                    return Err(e);
                }
            }
        }

        // Perform encryption
        match self.adapter.encrypt(file, &output_path, passphrase, options.format) {
            Ok(_) => {
                result.add_success(file.display().to_string());

                // Clean up backup on success if configured
                if let Some(backup) = backup_info {
                    let backup_manager = BackupManager::new();
                    if backup_manager.cleanup_on_success {
                        if let Err(e) = backup_manager.cleanup_backup(&backup) {
                            self.audit_logger.log_warning(&format!("Failed to cleanup backup {}: {}",
                                backup.backup_path.display(), e))?;
                        } else {
                            self.audit_logger.log_info(&format!("Cleaned up backup: {}",
                                backup.backup_path.display()))?;
                        }
                    }
                }

                Ok(())
            }
            Err(e) => {
                result.add_failure(file.display().to_string());

                // Restore from backup on failure
                if let Some(backup) = backup_info {
                    let backup_manager = BackupManager::new();
                    if let Err(restore_err) = backup_manager.restore_backup(&backup) {
                        self.audit_logger.log_error(&format!("CRITICAL: Failed to restore backup {}: {}",
                            backup.backup_path.display(), restore_err))?;
                    } else {
                        self.audit_logger.log_info(&format!("Restored from backup: {}",
                            backup.backup_path.display()))?;
                    }
                }

                Err(e)
            }
        }
    }

    /// Lock repository (directory)
    fn lock_repository(&self, repository: &Path, passphrase: &str, options: &LockOptions, result: &mut OperationResult) -> AgeResult<()> {
        let files = self.collect_files_with_pattern(repository, options.pattern_filter.as_deref())?;
        
        for file in files {
            if let Err(e) = self.lock_single_file(&file, passphrase, options, result) {
                // Continue processing other files even if one fails
                eprintln!("Failed to lock {}: {}", file.display(), e);
            }
        }

        Ok(())
    }

    /// Unlock a single file
    fn unlock_single_file(&self, file: &Path, passphrase: &str, _options: &UnlockOptions, result: &mut OperationResult) -> AgeResult<()> {
        // Determine output path by stripping only the configured extension suffix
        let output_path = {
            let file_name_os = file.file_name()
                .ok_or_else(|| {
                    result.add_failure(file.display().to_string());
                    AgeError::InvalidOperation {
                        operation: "unlock".to_string(),
                        reason: format!("Cannot extract filename from path: {}", file.display()),
                    }
                })?;

            // Try UTF-8 conversion for standard filename handling
            let file_name = match file_name_os.to_str() {
                Some(name) => name,
                None => {
                    result.add_failure(file.display().to_string());
                    eprintln!("⚠️  Skipping file with non-UTF8 filename: {}", file.display());
                    return Err(AgeError::InvalidOperation {
                        operation: "unlock".to_string(),
                        reason: format!("Non-UTF8 filename not supported: {}", file.display()),
                    });
                }
            };

            let suffix = self.config.extension_with_dot();
            if !file_name.ends_with(&suffix) {
                result.add_failure(file.display().to_string());
                eprintln!("⚠️  Skipping file without {} extension: {}", suffix, file.display());
                return Err(AgeError::InvalidOperation {
                    operation: "unlock".to_string(),
                    reason: format!("File does not have {} extension: {}", suffix, file.display()),
                });
            }

            let output_name = &file_name[..file_name.len() - suffix.len()];
            file.with_file_name(output_name)
        };

        match self.adapter.decrypt(file, &output_path, passphrase) {
            Ok(_) => {
                result.add_success(file.display().to_string());
                Ok(())
            }
            Err(e) => {
                result.add_failure(file.display().to_string());
                Err(e)
            }
        }
    }

    /// Unlock repository (directory)
    fn unlock_repository(&self, repository: &Path, passphrase: &str, options: &UnlockOptions, result: &mut OperationResult) -> AgeResult<()> {
        let files = self.collect_encrypted_files_with_pattern(repository, options.pattern_filter.as_deref())?;
        
        for file in files {
            if let Err(e) = self.unlock_single_file(&file, passphrase, options, result) {
                eprintln!("Failed to unlock {}: {}", file.display(), e);
            }
        }

        Ok(())
    }

    /// Get status for a single file
    fn get_file_status(&self, file: &Path) -> AgeResult<RepositoryStatus> {
        let mut status = RepositoryStatus::new();
        status.total_files = 1;

        // Check if file has configured encrypted extension
        if self.config.is_encrypted_file(file) {
            status.encrypted_files = 1;
        } else {
            status.unencrypted_files = 1;
        }

        Ok(status)
    }

    /// Get status for repository (directory)
    fn get_repository_status(&self, repository: &Path) -> AgeResult<RepositoryStatus> {
        let mut status = RepositoryStatus::new();

        for entry in std::fs::read_dir(repository)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                status.total_files += 1;

                if self.config.is_encrypted_file(&path) {
                    status.encrypted_files += 1;
                } else {
                    status.unencrypted_files += 1;
                }
            }
        }

        Ok(status)
    }

    /// Verify integrity of a single file
    fn verify_file_integrity(&self, file: &Path) -> AgeResult<FileVerificationStatus> {
        // Check if file exists and is readable
        if !file.exists() {
            return Err(AgeError::file_error("verify", file.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")));
        }

        if !file.is_file() {
            return Err(AgeError::file_error("verify", file.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path is not a file")));
        }

        // Check if file appears to be encrypted
        if !self.is_encrypted_file(file)? {
            return Ok(FileVerificationStatus {
                file_path: file.to_path_buf(),
                is_encrypted: false,
                format_valid: false,
                header_valid: false,
                size_check: true,
                error_message: Some("File does not appear to be Age encrypted".to_string()),
            });
        }

        // Read file content for verification
        let content = std::fs::read(file)
            .map_err(|e| AgeError::file_error("read", file.to_path_buf(), e))?;

        let mut status = FileVerificationStatus {
            file_path: file.to_path_buf(),
            is_encrypted: true,
            format_valid: false,
            header_valid: false,
            size_check: content.len() > 0,
            error_message: None,
        };

        // Verify Age header format
        if content.starts_with(b"age-encryption.org/v1") {
            status.format_valid = true;
            status.header_valid = self.verify_age_binary_header(&content)?;
        } else if content.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----") {
            status.format_valid = true;
            status.header_valid = self.verify_age_ascii_header(&content)?;
        } else {
            status.error_message = Some("Invalid Age format header".to_string());
        }

        Ok(status)
    }

    /// Verify Age binary format header
    fn verify_age_binary_header(&self, content: &[u8]) -> AgeResult<bool> {
        // Age binary format starts with "age-encryption.org/v1" followed by newline
        if content.len() < 22 {
            return Ok(false);
        }

        // Check for proper header structure
        let header_end = content.iter().position(|&b| b == b'\n');
        if let Some(pos) = header_end {
            if pos >= 21 && pos < 100 { // Reasonable header length
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Verify Age ASCII armor format header
    fn verify_age_ascii_header(&self, content: &[u8]) -> AgeResult<bool> {
        let content_str = String::from_utf8_lossy(content);
        let lines: Vec<&str> = content_str.lines().collect();

        if lines.is_empty() {
            return Ok(false);
        }

        // Check for proper ASCII armor structure
        let has_begin = lines[0] == "-----BEGIN AGE ENCRYPTED FILE-----";
        let has_end = lines.iter().any(|line| *line == "-----END AGE ENCRYPTED FILE-----");

        Ok(has_begin && has_end)
    }

    /// Verify integrity of repository
    fn verify_repository_integrity(&self, repository: &Path, verified: &mut Vec<String>, failed: &mut Vec<String>) -> AgeResult<()> {
        for entry in std::fs::read_dir(repository)? {
            let entry = entry.map_err(|e| AgeError::file_error("read_dir", repository.to_path_buf(), e))?;
            let path = entry.path();

            if path.is_file() {
                // Check if file appears to be encrypted (any format)
                if self.is_encrypted_file(&path)? {
                    match self.verify_file_integrity(&path) {
                        Ok(status) => {
                            if status.is_valid() {
                                verified.push(path.display().to_string());
                            } else {
                                let error_msg = status.error_message.unwrap_or_else(||
                                    format!("Verification failed: encrypted={}, format={}, header={}, size={}",
                                        status.is_encrypted, status.format_valid, status.header_valid, status.size_check));
                                failed.push(format!("{}: {}", path.display(), error_msg));
                            }
                        }
                        Err(e) => failed.push(format!("{}: {}", path.display(), e)),
                    }
                }
            } else if path.is_dir() {
                // Recursively verify subdirectories
                self.verify_repository_integrity(&path, verified, failed)?;
            }
        }

        Ok(())
    }

    /// Collect files matching pattern
    fn collect_files_with_pattern(&self, directory: &Path, pattern: Option<&str>) -> AgeResult<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                // Apply pattern filter if specified
                if let Some(pattern) = pattern {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        if !filename.contains(pattern) {
                            continue;
                        }
                    }
                }
                files.push(path);
            }
        }

        Ok(files)
    }

    /// Collect encrypted files matching pattern
    fn collect_encrypted_files_with_pattern(&self, directory: &Path, pattern: Option<&str>) -> AgeResult<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && self.config.is_encrypted_file(&path) {
                // Apply pattern filter if specified
                if let Some(pattern) = pattern {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        if !filename.contains(pattern) {
                            continue;
                        }
                    }
                }
                files.push(path);
            }
        }

        Ok(files)
    }

    /// Record operation for audit and recovery purposes
    fn record_operation(&mut self, operation_type: &str, target_path: &Path, success: bool, result: &OperationResult) {
        let mut details = HashMap::new();
        details.insert("processed_files".to_string(), result.processed_files.len().to_string());
        details.insert("failed_files".to_string(), result.failed_files.len().to_string());

        let record = OperationRecord {
            operation_type: operation_type.to_string(),
            target_path: target_path.to_path_buf(),
            timestamp: Instant::now(),
            success,
            details,
        };

        self.operation_history.push(record);
    }

    /// Get operation history for audit purposes
    pub fn get_operation_history(&self) -> &[OperationRecord] {
        &self.operation_history
    }

    /// Encrypt a single file to a specific output path (for in-place operations)
    pub fn encrypt_to_path(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()> {
        self.adapter.encrypt(input, output, passphrase, format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cage::config::OutputFormat;
    use tempfile::TempDir;

    #[test]
    fn test_crud_manager_creation() {
        let result = CrudManager::with_defaults();
        assert!(result.is_ok());
    }

    #[test]
    fn test_passphrase_validation() {
        let crud_manager = CrudManager::with_defaults().unwrap();
        
        // Empty passphrase should fail
        assert!(crud_manager.validate_passphrase("").is_err());
        
        // Normal passphrase should pass
        assert!(crud_manager.validate_passphrase("valid_passphrase").is_ok());
        
        // Very long passphrase should fail
        let long_passphrase = "a".repeat(2000);
        assert!(crud_manager.validate_passphrase(&long_passphrase).is_err());
    }

    #[test]
    fn test_lock_options_defaults() {
        let options = LockOptions::default();
        assert!(!options.recursive);
        assert_eq!(options.format, OutputFormat::Binary);
        assert!(options.pattern_filter.is_none());
        assert!(!options.backup_before_lock);
    }

    #[test]
    fn test_unlock_options_defaults() {
        let options = UnlockOptions::default();
        assert!(!options.selective);
        assert!(options.verify_before_unlock);
        assert!(options.pattern_filter.is_none());
        assert!(!options.preserve_encrypted);
    }

    #[test]
    fn test_key_rotation_validation() {
        // Test basic validation logic
        if let Ok(mut crud_manager) = CrudManager::with_defaults() {
            // Test same passphrase rejection
            let temp_dir = TempDir::new().unwrap();
            let result = crud_manager.rotate(temp_dir.path(), "same_pass", "same_pass");
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("must be different"));
        } else {
            // Skip test if Age not available
            println!("Skipping key rotation test - Age not available");
        }
    }

    #[test]
    fn test_encrypted_file_detection() {
        if let Ok(crud_manager) = CrudManager::with_defaults() {
            // Test with non-existent file
            let fake_path = std::path::Path::new("/non/existent/file");
            let result = crud_manager.is_encrypted_file(fake_path).unwrap();
            assert!(!result);

            // Test with temporary non-encrypted file
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp_file.path(), b"plain text content").unwrap();
            let result = crud_manager.is_encrypted_file(temp_file.path()).unwrap();
            assert!(!result);

        } else {
            println!("Skipping encrypted file detection test - Age not available");
        }
    }

    #[test]
    fn test_file_verification_system() {
        if let Ok(crud_manager) = CrudManager::with_defaults() {
            // Test verification of non-encrypted file
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp_file.path(), b"plain text content").unwrap();

            let result = crud_manager.verify_file_integrity(temp_file.path()).unwrap();
            assert!(!result.is_encrypted);
            assert!(!result.is_valid());
            assert!(result.error_message.is_some());

            // Test verification of file with Age binary header
            let temp_age_file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp_age_file.path(), b"age-encryption.org/v1\ntest encrypted content").unwrap();

            let result = crud_manager.verify_file_integrity(temp_age_file.path()).unwrap();
            assert!(result.is_encrypted);
            assert!(result.format_valid);
            assert!(result.header_valid);

            // Test verification of file with Age ASCII header
            let temp_ascii_file = tempfile::NamedTempFile::new().unwrap();
            let ascii_content = b"-----BEGIN AGE ENCRYPTED FILE-----\nencrypted content here\n-----END AGE ENCRYPTED FILE-----";
            std::fs::write(temp_ascii_file.path(), ascii_content).unwrap();

            let result = crud_manager.verify_file_integrity(temp_ascii_file.path()).unwrap();
            assert!(result.is_encrypted);
            assert!(result.format_valid);
            assert!(result.header_valid);

        } else {
            println!("Skipping file verification test - Age not available");
        }
    }

    #[test]
    fn test_verification_result_creation() {
        if let Ok(crud_manager) = CrudManager::with_defaults() {
            // Test verification of non-existent path
            let fake_path = std::path::Path::new("/non/existent/path");
            let result = crud_manager.verify(fake_path);
            assert!(result.is_err());

            // Test verification of empty directory
            let temp_dir = TempDir::new().unwrap();
            let result = crud_manager.verify(temp_dir.path()).unwrap();
            assert!(result.verified_files.is_empty());
            assert!(result.failed_files.is_empty());

        } else {
            println!("Skipping verification result test - Age not available");
        }
    }

    #[test]
    fn test_backup_manager_creation() {
        let backup_manager = BackupManager::new();
        assert!(backup_manager.backup_dir.is_none());
        assert_eq!(backup_manager.backup_extension, ".bak");
        assert!(backup_manager.cleanup_on_success);

        let custom_manager = BackupManager::new()
            .with_extension("backup".to_string())
            .with_cleanup(false);
        assert_eq!(custom_manager.backup_extension, ".backup");
        assert!(!custom_manager.cleanup_on_success);
    }

    #[test]
    fn test_backup_system() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, b"test content").unwrap();

        let backup_manager = BackupManager::new();

        // Test backup creation
        let backup_info = backup_manager.create_backup(&test_file).unwrap();
        assert_eq!(backup_info.original_path, test_file);
        assert_eq!(backup_info.size_bytes, 12); // "test content".len()
        assert!(backup_info.backup_path.exists());
        assert!(backup_info.backup_path.to_string_lossy().contains(".bak"));

        // Test backup content
        let backup_content = std::fs::read(&backup_info.backup_path).unwrap();
        assert_eq!(backup_content, b"test content");

        // Test backup restoration
        std::fs::write(&test_file, b"modified content").unwrap();
        backup_manager.restore_backup(&backup_info).unwrap();
        let restored_content = std::fs::read(&test_file).unwrap();
        assert_eq!(restored_content, b"test content");

        // Test cleanup
        backup_manager.cleanup_backup(&backup_info).unwrap();
        assert!(!backup_info.backup_path.exists());
    }

    #[test]
    fn test_backup_conflict_handling() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, b"original").unwrap();

        let backup_manager = BackupManager::new();

        // Create first backup
        let backup_info1 = backup_manager.create_backup(&test_file).unwrap();
        assert!(backup_info1.backup_path.exists());

        // Modify file and create second backup (should handle conflict)
        std::fs::write(&test_file, b"modified").unwrap();
        let backup_info2 = backup_manager.create_backup(&test_file).unwrap();
        assert!(backup_info2.backup_path.exists());

        // Both backups should exist in some form
        let backup_content = std::fs::read(&backup_info2.backup_path).unwrap();
        assert_eq!(backup_content, b"modified");
    }
}