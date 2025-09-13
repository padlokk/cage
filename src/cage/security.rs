//! Security Module - Audit logging and validation
//!
//! Comprehensive security framework for Age automation including audit logging,
//! security validation, and monitoring integration.
//!
//! Security Guardian: Edgar - Production security and audit framework

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use super::error::{AgeError, AgeResult};
use super::operations::{OperationResult, RepositoryStatus};

/// Audit logger for security events and operations
pub struct AuditLogger {
    component: String,
    log_file: Option<std::fs::File>,
}

impl AuditLogger {
    /// Create new audit logger for specified component
    pub fn new(log_path_opt: Option<PathBuf>) -> AgeResult<Self> {
        let log_file = if let Some(log_path) = log_path_opt {
            Some(OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .map_err(|e| AgeError::file_error("open", log_path.to_path_buf(), e))?)
        } else {
            None
        };

        Ok(Self {
            component: "age_automation".to_string(),
            log_file,
        })
    }

    /// Create audit logger with file output
    pub fn with_file(component: &str, log_path: &Path) -> AgeResult<Self> {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|e| AgeError::file_error("open", log_path.to_path_buf(), e))?;

        Ok(Self {
            component: component.to_string(),
            log_file: Some(log_file),
        })
    }

    /// Log operation start
    pub fn log_operation_start(&self, operation: &str, input: &Path, output: &Path) -> AgeResult<()> {
        let message = format!("OPERATION_START {} {} -> {}", operation, input.display(), output.display());
        self.log_event("INFO", &message)
    }

    /// Log operation success
    pub fn log_operation_success(&self, operation: &str, input: &Path, output: &Path) -> AgeResult<()> {
        let message = format!("OPERATION_SUCCESS {} {} -> {}", operation, input.display(), output.display());
        self.log_event("INFO", &message)
    }

    /// Log operation failure
    pub fn log_operation_failure(&self, operation: &str, input: &Path, output: &Path, error: &AgeError) -> AgeResult<()> {
        let message = format!("OPERATION_FAILURE {} {} -> {} ERROR: {}", 
            operation, input.display(), output.display(), error);
        self.log_event("ERROR", &message)
    }

    /// Log health check result
    pub fn log_health_check(&self, status: &str) -> AgeResult<()> {
        let message = format!("HEALTH_CHECK {}", status);
        self.log_event("INFO", &message)
    }

    /// Log informational message
    pub fn log_info(&self, message: &str) -> AgeResult<()> {
        self.log_event("INFO", message)
    }

    /// Log warning message
    pub fn log_warning(&self, message: &str) -> AgeResult<()> {
        self.log_event("WARN", message)
    }

    /// Log error message
    pub fn log_error(&self, message: &str) -> AgeResult<()> {
        self.log_event("ERROR", message)
    }

    /// Log operation start (single path variant)
    pub fn log_operation_start_single(&self, operation: &str, path: &Path) -> AgeResult<()> {
        let message = format!("OPERATION_START {} {}", operation, path.display());
        self.log_event("INFO", &message)
    }

    /// Log operation complete
    pub fn log_operation_complete(&self, operation: &str, path: &Path, result: &OperationResult) -> AgeResult<()> {
        let message = format!("OPERATION_COMPLETE {} {} - processed: {}, failed: {}, duration: {}ms", 
            operation, path.display(), result.processed_files.len(), result.failed_files.len(), result.execution_time_ms);
        self.log_event("INFO", &message)
    }

    /// Log status check
    pub fn log_status_check(&self, path: &Path, status: &RepositoryStatus) -> AgeResult<()> {
        let message = format!("STATUS_CHECK {} - total: {}, encrypted: {}, unencrypted: {}", 
            path.display(), status.total_files, status.encrypted_files, status.unencrypted_files);
        self.log_event("INFO", &message)
    }

    /// Log authority operation
    pub fn log_authority_operation(&self, operation: &str, recipient: &str) -> AgeResult<()> {
        let message = format!("AUTHORITY_OPERATION {} recipient: {}", operation, recipient);
        self.log_event("INFO", &message)
    }

    /// Log emergency operation
    pub fn log_emergency_operation(&self, operation: &str, path: &Path) -> AgeResult<()> {
        let message = format!("EMERGENCY_OPERATION {} {}", operation, path.display());
        self.log_event("WARN", &message)
    }

    /// Core event logging function
    fn log_event(&self, level: &str, message: &str) -> AgeResult<()> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let log_entry = format!("[{}] [{}] [{}] {}\n", timestamp, level, self.component, message);

        // Always log to stderr for immediate visibility
        eprint!("{}", log_entry);

        // Also log to file if configured
        if let Some(ref mut file) = &mut self.log_file.as_ref() {
            let mut file_handle = file.try_clone()
                .map_err(|e| AgeError::AuditLogFailed {
                    operation: "file_write".to_string(),
                    reason: e.to_string(),
                })?;
            
            file_handle.write_all(log_entry.as_bytes())
                .map_err(|e| AgeError::AuditLogFailed {
                    operation: "write".to_string(),
                    reason: e.to_string(),
                })?;
            
            file_handle.flush()
                .map_err(|e| AgeError::AuditLogFailed {
                    operation: "flush".to_string(),
                    reason: e.to_string(),
                })?;
        }

        Ok(())
    }
}

/// Security validator for operations and inputs
pub struct SecurityValidator {
    strict_mode: bool,
}

impl SecurityValidator {
    /// Create new security validator
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    /// Validate file path for security issues
    pub fn validate_file_path(&self, path: &Path) -> AgeResult<()> {
        let path_str = path.to_string_lossy();

        // Check for path traversal attempts
        if path_str.contains("..") {
            return Err(AgeError::SecurityValidationFailed {
                validation_type: "path_traversal".to_string(),
                details: format!("Path contains '..' traversal: {}", path_str),
            });
        }

        // Check for absolute paths to sensitive locations
        if self.strict_mode {
            let sensitive_paths = ["/etc", "/proc", "/sys", "/dev"];
            for sensitive in &sensitive_paths {
                if path_str.starts_with(sensitive) {
                    return Err(AgeError::SecurityValidationFailed {
                        validation_type: "sensitive_path".to_string(),
                        details: format!("Access to sensitive path: {}", path_str),
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate passphrase for security requirements
    pub fn validate_passphrase_security(&self, passphrase: &str) -> AgeResult<()> {
        // Check for common injection patterns
        let injection_patterns = ["$(", "`", ";", "&", "|", "\n", "\r"];
        for pattern in &injection_patterns {
            if passphrase.contains(pattern) {
                return Err(AgeError::injection_blocked("command_injection", pattern));
            }
        }

        // Check for null bytes
        if passphrase.contains('\0') {
            return Err(AgeError::injection_blocked("null_byte", "\\0"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_audit_logger_creation() {
        let logger = AuditLogger::new(None);
        assert!(logger.is_ok());
    }

    #[test]
    fn test_audit_logger_with_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = AuditLogger::with_file("test", temp_file.path());
        assert!(logger.is_ok());
    }

    #[test]
    fn test_security_validator() {
        let validator = SecurityValidator::new(true);
        
        // Test path traversal detection
        assert!(validator.validate_file_path(Path::new("../etc/passwd")).is_err());
        
        // Test valid path
        assert!(validator.validate_file_path(Path::new("./test.txt")).is_ok());
        
        // Test injection detection
        assert!(validator.validate_passphrase_security("password$(rm -rf /)").is_err());
        assert!(validator.validate_passphrase_security("validpassword").is_ok());
    }
}