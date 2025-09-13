//! Age Automation Error Types
//!
//! Comprehensive error handling covering all failure scenarios identified in the pilot
//! implementation. Provides clear error messages with actionable guidance for operators.
//!
//! Security Guardian: Edgar - Bulletproof error handling for production deployment

use std::fmt;
use std::error::Error;
use std::io;
use std::path::PathBuf;

/// Result type alias for Age automation operations
pub type AgeResult<T> = Result<T, AgeError>;

/// Comprehensive error types for Age automation
#[derive(Debug)]
pub enum AgeError {
    /// Age binary not found or not executable
    AgeBinaryNotFound(String),
    
    /// TTY automation method unavailable
    TtyMethodUnavailable {
        method: String,
        reason: String,
    },
    
    /// All TTY automation methods failed
    AllTtyMethodsFailed(Vec<String>),
    
    /// File operation errors
    FileError {
        operation: String,
        path: PathBuf,
        source: io::Error,
    },
    
    /// Passphrase validation errors
    PassphraseValidation {
        reason: String,
        guidance: String,
    },
    
    /// Encryption operation failed
    EncryptionFailed {
        input: PathBuf,
        output: PathBuf,
        reason: String,
    },
    
    /// Decryption operation failed
    DecryptionFailed {
        input: PathBuf,
        output: PathBuf,
        reason: String,
    },
    
    /// Output verification failed
    OutputVerificationFailed {
        expected_path: PathBuf,
        verification_type: String,
    },
    
    /// Security validation failed
    SecurityValidationFailed {
        validation_type: String,
        details: String,
    },
    
    /// Injection attack detected
    InjectionAttemptBlocked {
        attack_type: String,
        detected_pattern: String,
    },
    
    /// Audit logging failed
    AuditLogFailed {
        operation: String,
        reason: String,
    },
    
    /// Configuration error
    ConfigurationError {
        parameter: String,
        value: String,
        reason: String,
    },
    
    /// Adapter-related errors
    AdapterNotImplemented(String),
    InvalidAdapter(String),
    AdapterInitializationFailed {
        adapter_name: String,
        reason: String,
    },
    
    /// Batch operation errors
    BatchOperationFailed {
        operation: String,
        successful_count: usize,
        failed_count: usize,
        failures: Vec<String>,
    },
    
    /// System dependency errors
    DependencyMissing {
        dependency: String,
        installation_guide: String,
    },
    
    /// Temporary file/directory errors
    TemporaryResourceError {
        resource_type: String,
        operation: String,
        reason: String,
    },
    
    /// Process execution errors
    ProcessExecutionFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    
    /// Timeout errors
    OperationTimeout {
        operation: String,
        timeout_seconds: u64,
    },
    
    /// Permission errors
    PermissionDenied {
        operation: String,
        path: PathBuf,
        suggestion: String,
    },
    
    /// Generic I/O errors with context
    IoError {
        operation: String,
        context: String,
        source: io::Error,
    },
    
    /// Repository operation failed
    RepositoryOperationFailed {
        operation: String,
        repository: PathBuf,
        reason: String,
    },
    
    /// Invalid operation attempted
    InvalidOperation {
        operation: String,
        reason: String,
    },
}

impl fmt::Display for AgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgeError::AgeBinaryNotFound(details) => {
                write!(f, "Age binary not found: {}. Please install Age encryption tool.", details)
            }
            
            AgeError::TtyMethodUnavailable { method, reason } => {
                write!(f, "TTY automation method '{}' unavailable: {}", method, reason)
            }
            
            AgeError::AllTtyMethodsFailed(methods) => {
                write!(f, "All TTY automation methods failed: {}. Check dependencies: expect, util-linux", methods.join(", "))
            }
            
            AgeError::FileError { operation, path, source } => {
                write!(f, "File operation '{}' failed on '{}': {}", operation, path.display(), source)
            }
            
            AgeError::PassphraseValidation { reason, guidance } => {
                write!(f, "Passphrase validation failed: {}. Guidance: {}", reason, guidance)
            }
            
            AgeError::EncryptionFailed { input, output, reason } => {
                write!(f, "Encryption failed: {} -> {}: {}", input.display(), output.display(), reason)
            }
            
            AgeError::DecryptionFailed { input, output, reason } => {
                write!(f, "Decryption failed: {} -> {}: {}", input.display(), output.display(), reason)
            }
            
            AgeError::OutputVerificationFailed { expected_path, verification_type } => {
                write!(f, "Output verification '{}' failed for: {}", verification_type, expected_path.display())
            }
            
            AgeError::SecurityValidationFailed { validation_type, details } => {
                write!(f, "Security validation '{}' failed: {}", validation_type, details)
            }
            
            AgeError::InjectionAttemptBlocked { attack_type, detected_pattern } => {
                write!(f, "Injection attack '{}' blocked. Pattern: {}", attack_type, detected_pattern)
            }
            
            AgeError::AuditLogFailed { operation, reason } => {
                write!(f, "Audit logging failed for operation '{}': {}", operation, reason)
            }
            
            AgeError::ConfigurationError { parameter, value, reason } => {
                write!(f, "Configuration error: parameter '{}' value '{}': {}", parameter, value, reason)
            }
            
            AgeError::AdapterNotImplemented(details) => {
                write!(f, "Adapter not implemented: {}", details)
            }
            
            AgeError::InvalidAdapter(adapter_name) => {
                write!(f, "Invalid adapter: '{}'. Available: shell, rage", adapter_name)
            }
            
            AgeError::AdapterInitializationFailed { adapter_name, reason } => {
                write!(f, "Adapter '{}' initialization failed: {}", adapter_name, reason)
            }
            
            AgeError::BatchOperationFailed { operation, successful_count, failed_count, failures } => {
                write!(f, "Batch operation '{}' completed with {} successes, {} failures. Failures: {}", 
                       operation, successful_count, failed_count, failures.join("; "))
            }
            
            AgeError::DependencyMissing { dependency, installation_guide } => {
                write!(f, "Missing dependency '{}'. Install with: {}", dependency, installation_guide)
            }
            
            AgeError::TemporaryResourceError { resource_type, operation, reason } => {
                write!(f, "Temporary {} error during '{}': {}", resource_type, operation, reason)
            }
            
            AgeError::ProcessExecutionFailed { command, exit_code, stderr } => {
                write!(f, "Process '{}' failed with exit code {:?}: {}", command, exit_code, stderr)
            }
            
            AgeError::OperationTimeout { operation, timeout_seconds } => {
                write!(f, "Operation '{}' timed out after {} seconds", operation, timeout_seconds)
            }
            
            AgeError::PermissionDenied { operation, path, suggestion } => {
                write!(f, "Permission denied for operation '{}' on '{}'. Suggestion: {}", 
                       operation, path.display(), suggestion)
            }
            
            AgeError::IoError { operation, context, source } => {
                write!(f, "I/O error during '{}' ({}): {}", operation, context, source)
            }
            
            AgeError::RepositoryOperationFailed { operation, repository, reason } => {
                write!(f, "Repository operation '{}' failed for {}: {}", operation, repository.display(), reason)
            }
            
            AgeError::InvalidOperation { operation, reason } => {
                write!(f, "Invalid operation '{}': {}", operation, reason)
            }
        }
    }
}

impl Error for AgeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AgeError::FileError { source, .. } => Some(source),
            AgeError::IoError { source, .. } => Some(source),
            _ => None,
        }
    }
}

// Convenience implementations for common error conversions
impl From<io::Error> for AgeError {
    fn from(err: io::Error) -> Self {
        AgeError::IoError {
            operation: "unknown".to_string(),
            context: "I/O operation".to_string(),
            source: err,
        }
    }
}

/// Helper functions for creating specific error types
impl AgeError {
    /// Create a file operation error with context
    pub fn file_error(operation: &str, path: PathBuf, source: io::Error) -> Self {
        AgeError::FileError {
            operation: operation.to_string(),
            path,
            source,
        }
    }
    
    /// Create a passphrase validation error with guidance
    pub fn passphrase_validation(reason: &str, guidance: &str) -> Self {
        AgeError::PassphraseValidation {
            reason: reason.to_string(),
            guidance: guidance.to_string(),
        }
    }
    
    /// Create an encryption failure error
    pub fn encryption_failed(input: PathBuf, output: PathBuf, reason: &str) -> Self {
        AgeError::EncryptionFailed {
            input,
            output,
            reason: reason.to_string(),
        }
    }
    
    /// Create a decryption failure error
    pub fn decryption_failed(input: PathBuf, output: PathBuf, reason: &str) -> Self {
        AgeError::DecryptionFailed {
            input,
            output,
            reason: reason.to_string(),
        }
    }
    
    /// Create a dependency missing error with installation guidance
    pub fn dependency_missing(dependency: &str, installation_guide: &str) -> Self {
        AgeError::DependencyMissing {
            dependency: dependency.to_string(),
            installation_guide: installation_guide.to_string(),
        }
    }
    
    /// Create an injection attempt blocked error
    pub fn injection_blocked(attack_type: &str, pattern: &str) -> Self {
        AgeError::InjectionAttemptBlocked {
            attack_type: attack_type.to_string(),
            detected_pattern: pattern.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    
    #[test]
    fn test_error_display() {
        let err = AgeError::AgeBinaryNotFound("not in PATH".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Age binary not found"));
        assert!(display.contains("Please install Age encryption tool"));
    }
    
    #[test]
    fn test_error_helper_functions() {
        let path = PathBuf::from("/test/path");
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        
        let err = AgeError::file_error("read", path.clone(), io_err);
        match err {
            AgeError::FileError { operation, path: err_path, .. } => {
                assert_eq!(operation, "read");
                assert_eq!(err_path, path);
            }
            _ => panic!("Expected FileError"),
        }
    }
    
    #[test]
    fn test_passphrase_validation_error() {
        let err = AgeError::passphrase_validation(
            "too long", 
            "use shorter passphrase"
        );
        let display = format!("{}", err);
        assert!(display.contains("too long"));
        assert!(display.contains("use shorter passphrase"));
    }
    
    #[test]
    fn test_injection_blocked_error() {
        let err = AgeError::injection_blocked("command_injection", "; rm -rf /");
        let display = format!("{}", err);
        assert!(display.contains("command_injection"));
        assert!(display.contains("; rm -rf /"));
    }
}