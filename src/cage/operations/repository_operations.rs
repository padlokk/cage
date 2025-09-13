//! Repository Operations - Batch encryption/decryption for entire repositories
//!
//! Implements repository-level operations for encrypting/decrypting entire directories
//! and managing repository encryption status. Provides comprehensive batch processing
//! with progress tracking and error recovery.
//!
//! Security Guardian: Edgar - Production repository operation framework

use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;
use super::super::adapter::AgeAdapter;
use super::super::config::OutputFormat;
use super::super::error::{AgeError, AgeResult};
use super::super::security::{AuditLogger, SecurityValidator};
use super::{Operation, RepositoryOperations, RepositoryStatus, OperationResult, FileEncryption};
use super::file_operations::FileOperationsManager;

/// Repository encryption operation with comprehensive batch processing
pub struct RepositoryEncryptOperation {
    adapter: Box<dyn AgeAdapter>,
    repo_path: PathBuf,
    passphrase: String,
    format: OutputFormat,
    audit_logger: AuditLogger,
    validator: SecurityValidator,
    file_manager: FileOperationsManager,
}

impl RepositoryEncryptOperation {
    /// Create new repository encryption operation
    pub fn new(
        adapter: Box<dyn AgeAdapter>,
        repo_path: &Path,
        passphrase: &str,
        format: OutputFormat,
    ) -> AgeResult<Self> {
        let audit_logger = AuditLogger::new(None)?;
        let validator = SecurityValidator::new(true);
        let file_manager = FileOperationsManager::new(adapter.clone_box())?;
        
        Ok(Self {
            adapter,
            repo_path: repo_path.to_path_buf(),
            passphrase: passphrase.to_string(),
            format,
            audit_logger,
            validator,
            file_manager,
        })
    }
    
    /// Get all files in repository that can be encrypted
    fn discover_files(&self) -> AgeResult<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        fn visit_dir(dir: &Path, files: &mut Vec<PathBuf>) -> AgeResult<()> {
            let entries = fs::read_dir(dir)
                .map_err(|e| AgeError::file_error("read_dir", dir.to_path_buf(), e))?;
            
            for entry in entries {
                let entry = entry
                    .map_err(|e| AgeError::file_error("read_entry", dir.to_path_buf(), e))?;
                let path = entry.path();
                
                if path.is_file() {
                    // Skip already encrypted files
                    if let Some(ext) = path.extension() {
                        if ext == "age" {
                            continue;
                        }
                    }
                    files.push(path);
                } else if path.is_dir() {
                    // Skip hidden directories and common non-data directories
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with('.') || 
                           name_str == "target" || 
                           name_str == "node_modules" ||
                           name_str == ".git" {
                            continue;
                        }
                    }
                    visit_dir(&path, files)?;
                }
            }
            Ok(())
        }
        
        visit_dir(&self.repo_path, &mut files)?;
        Ok(files)
    }
}

impl Operation for RepositoryEncryptOperation {
    fn operation_name(&self) -> &'static str {
        "repository_encrypt"
    }
    
    fn validate_preconditions(&self) -> AgeResult<()> {
        // Validate repository path exists and is directory
        if !self.repo_path.exists() {
            return Err(AgeError::file_error("read", self.repo_path.clone(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Repository path not found")));
        }
        
        if !self.repo_path.is_dir() {
            return Err(AgeError::file_error("read", self.repo_path.clone(),
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Repository path is not a directory")));
        }
        
        // Validate repository path security
        self.validator.validate_file_path(&self.repo_path)?;
        
        // Validate passphrase security
        self.validator.validate_passphrase_security(&self.passphrase)?;
        
        // Validate adapter health
        self.adapter.health_check()?;
        
        self.audit_logger.log_info(&format!("Repository encryption preconditions validated: {}", self.repo_path.display()))?;
        Ok(())
    }
    
    fn execute(&self) -> AgeResult<()> {
        let start_time = Instant::now();
        self.audit_logger.log_info(&format!("Starting repository encryption: {}", self.repo_path.display()))?;
        
        // Discover files to encrypt
        let files = self.discover_files()?;
        self.audit_logger.log_info(&format!("Discovered {} files for encryption", files.len()))?;
        
        let mut processed = 0;
        let mut failed = 0;
        
        for file_path in files {
            let output_path = file_path.with_extension(
                format!("{}.age", file_path.extension().unwrap_or_default().to_string_lossy())
            );
            
            match self.file_manager.encrypt_file(&file_path, &output_path, &self.passphrase, self.format) {
                Ok(_) => {
                    processed += 1;
                    self.audit_logger.log_info(&format!("Encrypted: {} -> {}", 
                        file_path.display(), output_path.display()))?;
                }
                Err(e) => {
                    failed += 1;
                    self.audit_logger.log_error(&format!("Failed to encrypt {}: {}", 
                        file_path.display(), e))?;
                }
            }
        }
        
        let duration = start_time.elapsed();
        self.audit_logger.log_info(&format!(
            "Repository encryption completed: {} processed, {} failed, duration: {:?}",
            processed, failed, duration
        ))?;
        
        if failed > 0 {
            return Err(AgeError::RepositoryOperationFailed {
                operation: "encrypt".to_string(),
                repository: self.repo_path.clone(),
                reason: format!("Processed: {}, Failed: {}", processed, failed),
            });
        }
        
        Ok(())
    }
    
    fn validate_postconditions(&self) -> AgeResult<()> {
        // Verify that encrypted files were created
        let files = self.discover_files()?;
        let mut encrypted_count = 0;
        
        for file_path in files {
            let output_path = file_path.with_extension(
                format!("{}.age", file_path.extension().unwrap_or_default().to_string_lossy())
            );
            
            if output_path.exists() {
                encrypted_count += 1;
            }
        }
        
        self.audit_logger.log_info(&format!("Repository encryption postconditions: {} encrypted files", encrypted_count))?;
        Ok(())
    }
}

/// Repository decryption operation with comprehensive batch processing
pub struct RepositoryDecryptOperation {
    adapter: Box<dyn AgeAdapter>,
    repo_path: PathBuf,
    passphrase: String,
    audit_logger: AuditLogger,
    validator: SecurityValidator,
    file_manager: FileOperationsManager,
}

impl RepositoryDecryptOperation {
    /// Create new repository decryption operation
    pub fn new(
        adapter: Box<dyn AgeAdapter>,
        repo_path: &Path,
        passphrase: &str,
    ) -> AgeResult<Self> {
        let audit_logger = AuditLogger::new(None)?;
        let validator = SecurityValidator::new(true);
        let file_manager = FileOperationsManager::new(adapter.clone_box())?;
        
        Ok(Self {
            adapter,
            repo_path: repo_path.to_path_buf(),
            passphrase: passphrase.to_string(),
            audit_logger,
            validator,
            file_manager,
        })
    }
    
    /// Get all encrypted files in repository
    fn discover_encrypted_files(&self) -> AgeResult<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        fn visit_dir(dir: &Path, files: &mut Vec<PathBuf>) -> AgeResult<()> {
            let entries = fs::read_dir(dir)
                .map_err(|e| AgeError::file_error("read_dir", dir.to_path_buf(), e))?;
            
            for entry in entries {
                let entry = entry
                    .map_err(|e| AgeError::file_error("read_entry", dir.to_path_buf(), e))?;
                let path = entry.path();
                
                if path.is_file() {
                    // Check if file is encrypted
                    if let Some(ext) = path.extension() {
                        if ext == "age" {
                            files.push(path);
                        }
                    }
                } else if path.is_dir() {
                    // Skip hidden directories
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if !name_str.starts_with('.') {
                            visit_dir(&path, files)?;
                        }
                    }
                }
            }
            Ok(())
        }
        
        visit_dir(&self.repo_path, &mut files)?;
        Ok(files)
    }
}

impl Operation for RepositoryDecryptOperation {
    fn operation_name(&self) -> &'static str {
        "repository_decrypt"
    }
    
    fn validate_preconditions(&self) -> AgeResult<()> {
        // Validate repository path exists and is directory
        if !self.repo_path.exists() {
            return Err(AgeError::file_error("read", self.repo_path.clone(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Repository path not found")));
        }
        
        if !self.repo_path.is_dir() {
            return Err(AgeError::file_error("read", self.repo_path.clone(),
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Repository path is not a directory")));
        }
        
        // Validate repository path security
        self.validator.validate_file_path(&self.repo_path)?;
        
        // Validate passphrase security
        self.validator.validate_passphrase_security(&self.passphrase)?;
        
        // Validate adapter health
        self.adapter.health_check()?;
        
        self.audit_logger.log_info(&format!("Repository decryption preconditions validated: {}", self.repo_path.display()))?;
        Ok(())
    }
    
    fn execute(&self) -> AgeResult<()> {
        let start_time = Instant::now();
        self.audit_logger.log_info(&format!("Starting repository decryption: {}", self.repo_path.display()))?;
        
        // Discover encrypted files to decrypt
        let files = self.discover_encrypted_files()?;
        self.audit_logger.log_info(&format!("Discovered {} encrypted files for decryption", files.len()))?;
        
        let mut processed = 0;
        let mut failed = 0;
        
        for file_path in files {
            // Remove .age extension for output
            let output_path = file_path.with_extension("");
            
            match self.file_manager.decrypt_file(&file_path, &output_path, &self.passphrase) {
                Ok(_) => {
                    processed += 1;
                    self.audit_logger.log_info(&format!("Decrypted: {} -> {}", 
                        file_path.display(), output_path.display()))?;
                }
                Err(e) => {
                    failed += 1;
                    self.audit_logger.log_error(&format!("Failed to decrypt {}: {}", 
                        file_path.display(), e))?;
                }
            }
        }
        
        let duration = start_time.elapsed();
        self.audit_logger.log_info(&format!(
            "Repository decryption completed: {} processed, {} failed, duration: {:?}",
            processed, failed, duration
        ))?;
        
        if failed > 0 {
            return Err(AgeError::RepositoryOperationFailed {
                operation: "decrypt".to_string(),
                repository: self.repo_path.clone(),
                reason: format!("Processed: {}, Failed: {}", processed, failed),
            });
        }
        
        Ok(())
    }
    
    fn validate_postconditions(&self) -> AgeResult<()> {
        // Verify that decrypted files were created
        let files = self.discover_encrypted_files()?;
        let mut decrypted_count = 0;
        
        for file_path in files {
            let output_path = file_path.with_extension("");
            
            if output_path.exists() {
                decrypted_count += 1;
            }
        }
        
        self.audit_logger.log_info(&format!("Repository decryption postconditions: {} decrypted files", decrypted_count))?;
        Ok(())
    }
}

/// Repository operations manager implementing RepositoryOperations trait
pub struct RepositoryOperationsManager {
    adapter: Box<dyn AgeAdapter>,
    audit_logger: AuditLogger,
    validator: SecurityValidator,
    file_manager: FileOperationsManager,
}

impl RepositoryOperationsManager {
    /// Create new repository operations manager
    pub fn new(adapter: Box<dyn AgeAdapter>) -> AgeResult<Self> {
        let audit_logger = AuditLogger::new(None)?;
        let validator = SecurityValidator::new(true);
        let file_manager = FileOperationsManager::new(adapter.clone_box())?;
        
        Ok(Self {
            adapter,
            audit_logger,
            validator,
            file_manager,
        })
    }
    
    /// Perform repository encryption with full operation framework
    pub fn encrypt_with_validation(&self, repo_path: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<OperationResult> {
        let start_time = Instant::now();
        let mut result = OperationResult::new();
        
        let operation = RepositoryEncryptOperation::new(
            self.adapter.clone_box(),
            repo_path,
            passphrase,
            format,
        )?;
        
        match operation.perform() {
            Ok(_) => {
                result.add_success(repo_path.to_string_lossy().to_string());
                self.audit_logger.log_info(&format!("Repository encryption completed: {}", repo_path.display()))?;
            }
            Err(e) => {
                result.add_failure(repo_path.to_string_lossy().to_string());
                self.audit_logger.log_error(&format!("Repository encryption failed: {} - {}", repo_path.display(), e))?;
                return Err(e);
            }
        }
        
        result.finalize(start_time);
        Ok(result)
    }
    
    /// Perform repository decryption with full operation framework
    pub fn decrypt_with_validation(&self, repo_path: &Path, passphrase: &str) -> AgeResult<OperationResult> {
        let start_time = Instant::now();
        let mut result = OperationResult::new();
        
        let operation = RepositoryDecryptOperation::new(
            self.adapter.clone_box(),
            repo_path,
            passphrase,
        )?;
        
        match operation.perform() {
            Ok(_) => {
                result.add_success(repo_path.to_string_lossy().to_string());
                self.audit_logger.log_info(&format!("Repository decryption completed: {}", repo_path.display()))?;
            }
            Err(e) => {
                result.add_failure(repo_path.to_string_lossy().to_string());
                self.audit_logger.log_error(&format!("Repository decryption failed: {} - {}", repo_path.display(), e))?;
                return Err(e);
            }
        }
        
        result.finalize(start_time);
        Ok(result)
    }
}

impl RepositoryOperations for RepositoryOperationsManager {
    fn encrypt_repository(&self, repo_path: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()> {
        self.encrypt_with_validation(repo_path, passphrase, format)?;
        Ok(())
    }
    
    fn decrypt_repository(&self, repo_path: &Path, passphrase: &str) -> AgeResult<()> {
        self.decrypt_with_validation(repo_path, passphrase)?;
        Ok(())
    }
    
    fn repository_status(&self, repo_path: &Path) -> AgeResult<RepositoryStatus> {
        let mut status = RepositoryStatus::new();
        
        fn count_files(dir: &Path, status: &mut RepositoryStatus, file_manager: &FileOperationsManager) -> AgeResult<()> {
            let entries = fs::read_dir(dir)
                .map_err(|e| AgeError::file_error("read_dir", dir.to_path_buf(), e))?;
            
            for entry in entries {
                let entry = entry
                    .map_err(|e| AgeError::file_error("read_entry", dir.to_path_buf(), e))?;
                let path = entry.path();
                
                if path.is_file() {
                    status.total_files += 1;
                    
                    match file_manager.is_encrypted_file(&path) {
                        Ok(true) => status.encrypted_files += 1,
                        Ok(false) => status.unencrypted_files += 1,
                        Err(_) => status.failed_files.push(path.to_string_lossy().to_string()),
                    }
                } else if path.is_dir() {
                    // Skip hidden directories
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if !name_str.starts_with('.') && 
                           name_str != "target" && 
                           name_str != "node_modules" {
                            count_files(&path, status, file_manager)?;
                        }
                    }
                }
            }
            Ok(())
        }
        
        count_files(repo_path, &mut status, &self.file_manager)?;
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use tempfile::TempDir;
    use super::super::adapter::ShellAdapter;
    
    #[test]
    fn test_repository_operations_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = Box::new(ShellAdapter::new(temp_dir.path()).unwrap());
        let manager = RepositoryOperationsManager::new(adapter);
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_repository_status_calculation() {
        let mut status = RepositoryStatus::new();
        status.total_files = 10;
        status.encrypted_files = 7;
        status.unencrypted_files = 3;
        
        assert_eq!(status.encryption_percentage(), 70.0);
        assert!(!status.is_fully_encrypted());
        assert!(!status.is_fully_decrypted());
        
        status.encrypted_files = 10;
        status.unencrypted_files = 0;
        assert!(status.is_fully_encrypted());
        
        status.encrypted_files = 0;
        status.unencrypted_files = 10;
        assert!(status.is_fully_decrypted());
    }
}