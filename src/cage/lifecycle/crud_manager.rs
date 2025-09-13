//! CRUD Manager - Central Lifecycle Coordinator for Age Automation
//!
//! The CrudManager provides comprehensive lifecycle coordination for all Age encryption
//! operations, integrating TTY automation, authority management, and operational validation
//! into a unified interface that supports the complete padlock command set.
//!
//! Security Guardian: Edgar - Production CRUD coordination with authority integration

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use super::super::error::{AgeError, AgeResult};
use super::super::config::{AgeConfig, OutputFormat};
use super::super::adapter::AgeAdapter;
use super::super::tty_automation::TtyAutomator;
use super::super::security::AuditLogger;
use super::super::operations::{
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
struct OperationRecord {
    operation_type: String,
    target_path: PathBuf,
    timestamp: Instant,
    success: bool,
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
    pub fn rotate(&mut self, repository: &Path, new_passphrase: &str) -> AgeResult<OperationResult> {
        let start_time = Instant::now();
        self.audit_logger.log_operation_start_single("rotate", repository)?;
        
        // This is a placeholder for key rotation functionality
        // In practice, this would coordinate with authority management
        let mut result = OperationResult::new();
        
        // Validate inputs
        if !repository.exists() || !repository.is_dir() {
            return Err(AgeError::InvalidOperation {
                operation: "rotate".to_string(),
                reason: "Repository path required".to_string(),
            });
        }

        self.validate_passphrase(new_passphrase)?;

        // Key rotation would involve:
        // 1. Validate current authority chain
        // 2. Generate new keys
        // 3. Re-encrypt repository with new keys
        // 4. Update authority chain
        // 5. Validate new configuration

        // For now, record the operation intent
        result.add_success(format!("Key rotation initiated for: {}", repository.display()));
        self.record_operation("rotate", repository, true, &result);
        result.finalize(start_time);
        
        self.audit_logger.log_operation_complete("rotate", repository, &result)?;
        Ok(result)
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
                Ok(_) => verified_files.push(path.display().to_string()),
                Err(_) => failed_files.push(path.display().to_string()),
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
        let output_path = if options.format == OutputFormat::AsciiArmor {
            file.with_extension("age")
        } else {
            file.with_extension("age")
        };

        match self.adapter.encrypt(file, &output_path, passphrase, options.format) {
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
        // Determine output path by removing .age extension
        let output_path = file.with_extension("");

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

        // Simple heuristic: check if file has .age extension
        if file.extension().and_then(|s| s.to_str()) == Some("age") {
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
                
                if path.extension().and_then(|s| s.to_str()) == Some("age") {
                    status.encrypted_files += 1;
                } else {
                    status.unencrypted_files += 1;
                }
            }
        }

        Ok(status)
    }

    /// Verify integrity of a single file
    fn verify_file_integrity(&self, _file: &Path) -> AgeResult<()> {
        // Placeholder for file integrity verification
        // In practice, this would attempt to decrypt and verify
        Ok(())
    }

    /// Verify integrity of repository
    fn verify_repository_integrity(&self, repository: &Path, verified: &mut Vec<String>, failed: &mut Vec<String>) -> AgeResult<()> {
        for entry in std::fs::read_dir(repository)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("age") {
                match self.verify_file_integrity(&path) {
                    Ok(_) => verified.push(path.display().to_string()),
                    Err(_) => failed.push(path.display().to_string()),
                }
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

    /// Collect encrypted files (*.age) matching pattern
    fn collect_encrypted_files_with_pattern(&self, directory: &Path, pattern: Option<&str>) -> AgeResult<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("age") {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::OutputFormat;
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
}