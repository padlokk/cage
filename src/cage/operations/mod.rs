//! Operations Module - Core operation traits and implementations
//!
//! Provides structured operation framework for Age automation including file operations,
//! repository operations, and lifecycle management following CRUD patterns.
//!
//! Security Guardian: Edgar - Production operations framework

pub mod file_operations;
pub mod repository_operations;

use std::path::Path;
use super::error::AgeResult;
use super::config::OutputFormat;

/// Core operation trait defining common operation behavior
pub trait Operation {
    /// Get operation name for logging
    fn operation_name(&self) -> &'static str;
    
    /// Validate operation preconditions
    fn validate_preconditions(&self) -> AgeResult<()>;
    
    /// Execute the operation
    fn execute(&self) -> AgeResult<()>;
    
    /// Validate operation postconditions
    fn validate_postconditions(&self) -> AgeResult<()>;
    
    /// Full operation with validation
    fn perform(&self) -> AgeResult<()> {
        self.validate_preconditions()?;
        self.execute()?;
        self.validate_postconditions()?;
        Ok(())
    }
}

/// Trait for file-based encryption operations
pub trait FileEncryption {
    /// Encrypt a single file
    fn encrypt_file(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()>;
    
    /// Decrypt a single file
    fn decrypt_file(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()>;
    
    /// Check if file is encrypted (basic heuristic)
    fn is_encrypted_file(&self, path: &Path) -> AgeResult<bool>;
}

/// Trait for repository-level operations
pub trait RepositoryOperations {
    /// Encrypt all files in directory
    fn encrypt_repository(&self, repo_path: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()>;
    
    /// Decrypt all encrypted files in directory
    fn decrypt_repository(&self, repo_path: &Path, passphrase: &str) -> AgeResult<()>;
    
    /// Get repository encryption status
    fn repository_status(&self, repo_path: &Path) -> AgeResult<RepositoryStatus>;
}

/// Repository encryption status information
#[derive(Debug, Clone)]
pub struct RepositoryStatus {
    pub total_files: usize,
    pub encrypted_files: usize,
    pub unencrypted_files: usize,
    pub failed_files: Vec<String>,
}

impl RepositoryStatus {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            encrypted_files: 0,
            unencrypted_files: 0,
            failed_files: Vec::new(),
        }
    }
    
    pub fn is_fully_encrypted(&self) -> bool {
        self.total_files > 0 && self.encrypted_files == self.total_files && self.failed_files.is_empty()
    }
    
    pub fn is_fully_decrypted(&self) -> bool {
        self.encrypted_files == 0 && self.failed_files.is_empty()
    }
    
    pub fn encryption_percentage(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            (self.encrypted_files as f64 / self.total_files as f64) * 100.0
        }
    }
}

/// Operation result with detailed information
#[derive(Debug)]
pub struct OperationResult {
    pub success: bool,
    pub processed_files: Vec<String>,
    pub failed_files: Vec<String>,
    pub total_processed: usize,
    pub execution_time_ms: u64,
}

impl OperationResult {
    pub fn new() -> Self {
        Self {
            success: false,
            processed_files: Vec::new(),
            failed_files: Vec::new(),
            total_processed: 0,
            execution_time_ms: 0,
        }
    }
    
    pub fn add_success(&mut self, file_path: String) {
        self.processed_files.push(file_path);
        self.total_processed += 1;
    }
    
    pub fn add_failure(&mut self, file_path: String) {
        self.failed_files.push(file_path);
    }
    
    pub fn finalize(&mut self, start_time: std::time::Instant) {
        self.execution_time_ms = start_time.elapsed().as_millis() as u64;
        self.success = self.failed_files.is_empty() && self.total_processed > 0;
    }
    
    pub fn success_rate(&self) -> f64 {
        let total = self.processed_files.len() + self.failed_files.len();
        if total == 0 {
            0.0
        } else {
            (self.processed_files.len() as f64 / total as f64) * 100.0
        }
    }
}