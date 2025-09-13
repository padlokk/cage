//! File Operations - Individual file encryption/decryption operations
//!
//! Implements file-level encryption and decryption operations using the proven TTY automation
//! methods. Provides structured operation framework with validation and audit logging.
//!
//! Security Guardian: Edgar - Production file operation framework

use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;
use super::super::adapter::AgeAdapter;
use super::super::config::OutputFormat;
use super::super::error::{AgeError, AgeResult};
use super::super::security::{AuditLogger, SecurityValidator};
use super::{Operation, FileEncryption, OperationResult};

/// File encryption operation with comprehensive validation
pub struct FileEncryptOperation {
    adapter: Box<dyn AgeAdapter>,
    input_path: PathBuf,
    output_path: PathBuf,
    passphrase: String,
    format: OutputFormat,
    audit_logger: AuditLogger,
    validator: SecurityValidator,
}

impl FileEncryptOperation {
    /// Create new file encryption operation
    pub fn new(
        adapter: Box<dyn AgeAdapter>,
        input: &Path,
        output: &Path,
        passphrase: &str,
        format: OutputFormat,
    ) -> AgeResult<Self> {
        let audit_logger = AuditLogger::new(None)?;
        let validator = SecurityValidator::new(true);
        
        Ok(Self {
            adapter,
            input_path: input.to_path_buf(),
            output_path: output.to_path_buf(),
            passphrase: passphrase.to_string(),
            format,
            audit_logger,
            validator,
        })
    }
    
    /// Create with audit file logging
    pub fn with_audit_file(
        adapter: Box<dyn AgeAdapter>,
        input: &Path,
        output: &Path,
        passphrase: &str,
        format: OutputFormat,
        audit_path: &Path,
    ) -> AgeResult<Self> {
        let audit_logger = AuditLogger::with_file("file_encrypt", audit_path)?;
        let validator = SecurityValidator::new(true);
        
        Ok(Self {
            adapter,
            input_path: input.to_path_buf(),
            output_path: output.to_path_buf(),
            passphrase: passphrase.to_string(),
            format,
            audit_logger,
            validator,
        })
    }
}

impl Operation for FileEncryptOperation {
    fn operation_name(&self) -> &'static str {
        "file_encrypt"
    }
    
    fn validate_preconditions(&self) -> AgeResult<()> {
        // Validate input file exists and is readable
        if !self.input_path.exists() {
            return Err(AgeError::file_error("read", self.input_path.clone(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Input file not found")));
        }
        
        if !self.input_path.is_file() {
            return Err(AgeError::file_error("read", self.input_path.clone(),
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Input path is not a file")));
        }
        
        // Validate output path security
        self.validator.validate_file_path(&self.output_path)?;
        
        // Validate passphrase security
        self.validator.validate_passphrase_security(&self.passphrase)?;
        
        // Check if output file already exists (warn but allow)
        if self.output_path.exists() {
            self.audit_logger.log_warning(&format!("Output file already exists: {}", self.output_path.display()))?;
        }
        
        // Validate adapter health
        self.adapter.health_check()?;
        
        self.audit_logger.log_info("File encryption preconditions validated")?;
        Ok(())
    }
    
    fn execute(&self) -> AgeResult<()> {
        self.audit_logger.log_operation_start("encrypt", &self.input_path, &self.output_path)?;
        
        let result = self.adapter.encrypt(&self.input_path, &self.output_path, &self.passphrase, self.format);
        
        match &result {
            Ok(_) => {
                self.audit_logger.log_operation_success("encrypt", &self.input_path, &self.output_path)?;
            }
            Err(e) => {
                self.audit_logger.log_operation_failure("encrypt", &self.input_path, &self.output_path, e)?;
            }
        }
        
        result
    }
    
    fn validate_postconditions(&self) -> AgeResult<()> {
        // Verify output file was created
        if !self.output_path.exists() {
            return Err(AgeError::EncryptionFailed {
                input: self.input_path.clone(),
                output: self.output_path.clone(),
                reason: "Output file was not created".to_string(),
            });
        }
        
        // Verify output file has content
        let metadata = fs::metadata(&self.output_path)
            .map_err(|e| AgeError::file_error("stat", self.output_path.clone(), e))?;
        
        if metadata.len() == 0 {
            return Err(AgeError::EncryptionFailed {
                input: self.input_path.clone(),
                output: self.output_path.clone(),
                reason: "Output file is empty".to_string(),
            });
        }
        
        self.audit_logger.log_info("File encryption postconditions validated")?;
        Ok(())
    }
}

/// File decryption operation with comprehensive validation
pub struct FileDecryptOperation {
    adapter: Box<dyn AgeAdapter>,
    input_path: PathBuf,
    output_path: PathBuf,
    passphrase: String,
    audit_logger: AuditLogger,
    validator: SecurityValidator,
}

impl FileDecryptOperation {
    /// Create new file decryption operation
    pub fn new(
        adapter: Box<dyn AgeAdapter>,
        input: &Path,
        output: &Path,
        passphrase: &str,
    ) -> AgeResult<Self> {
        let audit_logger = AuditLogger::new(None)?;
        let validator = SecurityValidator::new(true);
        
        Ok(Self {
            adapter,
            input_path: input.to_path_buf(),
            output_path: output.to_path_buf(),
            passphrase: passphrase.to_string(),
            audit_logger,
            validator,
        })
    }
    
    /// Create with audit file logging
    pub fn with_audit_file(
        adapter: Box<dyn AgeAdapter>,
        input: &Path,
        output: &Path,
        passphrase: &str,
        audit_path: &Path,
    ) -> AgeResult<Self> {
        let audit_logger = AuditLogger::with_file("file_decrypt", audit_path)?;
        let validator = SecurityValidator::new(true);
        
        Ok(Self {
            adapter,
            input_path: input.to_path_buf(),
            output_path: output.to_path_buf(),
            passphrase: passphrase.to_string(),
            audit_logger,
            validator,
        })
    }
}

impl Operation for FileDecryptOperation {
    fn operation_name(&self) -> &'static str {
        "file_decrypt"
    }
    
    fn validate_preconditions(&self) -> AgeResult<()> {
        // Validate input file exists and appears to be encrypted
        if !self.input_path.exists() {
            return Err(AgeError::file_error("read", self.input_path.clone(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Input file not found")));
        }
        
        // Basic heuristic to check if file might be encrypted
        let content = fs::read(&self.input_path)
            .map_err(|e| AgeError::file_error("read", self.input_path.clone(), e))?;
        
        // Check for Age header
        if !content.starts_with(b"age-encryption.org/v1") && !content.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----") {
            self.audit_logger.log_warning("Input file does not appear to be Age encrypted")?;
        }
        
        // Validate output path security
        self.validator.validate_file_path(&self.output_path)?;
        
        // Validate passphrase security
        self.validator.validate_passphrase_security(&self.passphrase)?;
        
        // Validate adapter health
        self.adapter.health_check()?;
        
        self.audit_logger.log_info("File decryption preconditions validated")?;
        Ok(())
    }
    
    fn execute(&self) -> AgeResult<()> {
        self.audit_logger.log_operation_start("decrypt", &self.input_path, &self.output_path)?;
        
        let result = self.adapter.decrypt(&self.input_path, &self.output_path, &self.passphrase);
        
        match &result {
            Ok(_) => {
                self.audit_logger.log_operation_success("decrypt", &self.input_path, &self.output_path)?;
            }
            Err(e) => {
                self.audit_logger.log_operation_failure("decrypt", &self.input_path, &self.output_path, e)?;
            }
        }
        
        result
    }
    
    fn validate_postconditions(&self) -> AgeResult<()> {
        // Verify output file was created
        if !self.output_path.exists() {
            return Err(AgeError::DecryptionFailed {
                input: self.input_path.clone(),
                output: self.output_path.clone(),
                reason: "Output file was not created".to_string(),
            });
        }
        
        // Verify output file has content
        let metadata = fs::metadata(&self.output_path)
            .map_err(|e| AgeError::file_error("stat", self.output_path.clone(), e))?;
        
        if metadata.len() == 0 {
            return Err(AgeError::DecryptionFailed {
                input: self.input_path.clone(),
                output: self.output_path.clone(),
                reason: "Output file is empty".to_string(),
            });
        }
        
        self.audit_logger.log_info("File decryption postconditions validated")?;
        Ok(())
    }
}

/// File operations manager implementing FileEncryption trait
pub struct FileOperationsManager {
    adapter: Box<dyn AgeAdapter>,
    audit_logger: AuditLogger,
    validator: SecurityValidator,
}

impl FileOperationsManager {
    /// Create new file operations manager
    pub fn new(adapter: Box<dyn AgeAdapter>) -> AgeResult<Self> {
        let audit_logger = AuditLogger::new(None)?;
        let validator = SecurityValidator::new(true);
        
        Ok(Self {
            adapter,
            audit_logger,
            validator,
        })
    }
    
    /// Perform file encryption with full operation framework
    pub fn encrypt_with_validation(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<OperationResult> {
        let start_time = Instant::now();
        let mut result = OperationResult::new();
        
        // Create and execute operation
        let operation = FileEncryptOperation::new(
            // Note: In production, we'd need to clone the adapter properly
            // For now, this shows the structure
            self.adapter.clone_box(),
            input,
            output,
            passphrase,
            format,
        )?;
        
        match operation.perform() {
            Ok(_) => {
                result.add_success(input.to_string_lossy().to_string());
                self.audit_logger.log_info(&format!("File encryption completed: {}", input.display()))?;
            }
            Err(e) => {
                result.add_failure(input.to_string_lossy().to_string());
                self.audit_logger.log_error(&format!("File encryption failed: {} - {}", input.display(), e))?;
                return Err(e);
            }
        }
        
        result.finalize(start_time);
        Ok(result)
    }
    
    /// Perform file decryption with full operation framework
    pub fn decrypt_with_validation(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<OperationResult> {
        let start_time = Instant::now();
        let mut result = OperationResult::new();
        
        // Create and execute operation
        let operation = FileDecryptOperation::new(
            self.adapter.clone_box(),
            input,
            output,
            passphrase,
        )?;
        
        match operation.perform() {
            Ok(_) => {
                result.add_success(input.to_string_lossy().to_string());
                self.audit_logger.log_info(&format!("File decryption completed: {}", input.display()))?;
            }
            Err(e) => {
                result.add_failure(input.to_string_lossy().to_string());
                self.audit_logger.log_error(&format!("File decryption failed: {} - {}", input.display(), e))?;
                return Err(e);
            }
        }
        
        result.finalize(start_time);
        Ok(result)
    }
}

impl FileEncryption for FileOperationsManager {
    fn encrypt_file(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()> {
        self.encrypt_with_validation(input, output, passphrase, format)?;
        Ok(())
    }
    
    fn decrypt_file(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()> {
        self.decrypt_with_validation(input, output, passphrase)?;
        Ok(())
    }
    
    fn is_encrypted_file(&self, path: &Path) -> AgeResult<bool> {
        if !path.exists() {
            return Ok(false);
        }
        
        let content = fs::read(path)
            .map_err(|e| AgeError::file_error("read", path.to_path_buf(), e))?;
        
        // Check for Age headers
        Ok(content.starts_with(b"age-encryption.org/v1") || 
           content.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----"))
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use tempfile::{NamedTempFile, TempDir};
    use super::super::adapter::ShellAdapter;
    
    #[test]
    fn test_file_operations_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = Box::new(ShellAdapter::new(temp_dir.path()).unwrap());
        let manager = FileOperationsManager::new(adapter);
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_encrypted_file_detection() {
        let manager = create_test_manager();
        
        // Test with non-existent file
        assert!(!manager.is_encrypted_file(Path::new("/nonexistent")).unwrap());
        
        // Test with Age binary header
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), b"age-encryption.org/v1\n...encrypted data...").unwrap();
        assert!(manager.is_encrypted_file(temp_file.path()).unwrap());
        
        // Test with ASCII armor header
        let temp_file2 = NamedTempFile::new().unwrap();
        std::fs::write(temp_file2.path(), b"-----BEGIN AGE ENCRYPTED FILE-----\n...").unwrap();
        assert!(manager.is_encrypted_file(temp_file2.path()).unwrap());
        
        // Test with regular file
        let temp_file3 = NamedTempFile::new().unwrap();
        std::fs::write(temp_file3.path(), b"regular file content").unwrap();
        assert!(!manager.is_encrypted_file(temp_file3.path()).unwrap());
    }
    
    fn create_test_manager() -> FileOperationsManager {
        let temp_dir = TempDir::new().unwrap();
        let adapter = Box::new(ShellAdapter::new(temp_dir.path()).unwrap());
        FileOperationsManager::new(adapter).unwrap()
    }
}