//! Security Module - Audit logging and validation
//!
//! Comprehensive security framework for Age automation including audit logging,
//! security validation, and monitoring integration.
//!
//! Security Guardian: Edgar - Production security and audit framework

use super::config::TelemetryFormat;
use super::error::{AgeError, AgeResult};
use super::operations::{OperationResult, RepositoryStatus};
#[allow(unused_imports)]
use chrono::{DateTime, Utc};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Audit logger for security events and operations
pub struct AuditLogger {
    component: String,
    log_file: Option<std::fs::File>,
    telemetry_format: TelemetryFormat,
}

impl AuditLogger {
    /// Create new audit logger for specified component
    pub fn new(log_path_opt: Option<PathBuf>) -> AgeResult<Self> {
        let log_file = if let Some(log_path) = log_path_opt {
            Some(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                    .map_err(|e| AgeError::file_error("open", log_path.to_path_buf(), e))?,
            )
        } else {
            None
        };

        Ok(Self {
            component: "cage_automation".to_string(),
            log_file,
            telemetry_format: TelemetryFormat::default(),
        })
    }

    /// Create audit logger with file output
    pub fn with_file(_component: &str, log_path: &Path) -> AgeResult<Self> {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|e| AgeError::file_error("open", log_path.to_path_buf(), e))?;

        Ok(Self {
            component: "cage_automation".to_string(),
            log_file: Some(log_file),
            telemetry_format: TelemetryFormat::default(),
        })
    }

    /// Create audit logger with specific telemetry format
    pub fn with_format(log_path_opt: Option<PathBuf>, format: TelemetryFormat) -> AgeResult<Self> {
        let mut logger = Self::new(log_path_opt)?;
        logger.telemetry_format = format;
        Ok(logger)
    }

    /// Log operation start
    pub fn log_operation_start(
        &self,
        operation: &str,
        input: &Path,
        output: &Path,
    ) -> AgeResult<()> {
        let message = format!(
            "OPERATION_START {} {} -> {}",
            operation,
            input.display(),
            output.display()
        );
        self.log_event("INFO", &message)
    }

    /// Log operation success
    pub fn log_operation_success(
        &self,
        operation: &str,
        input: &Path,
        output: &Path,
    ) -> AgeResult<()> {
        let message = format!(
            "OPERATION_SUCCESS {} {} -> {}",
            operation,
            input.display(),
            output.display()
        );
        self.log_event("INFO", &message)
    }

    /// Log operation failure
    pub fn log_operation_failure(
        &self,
        operation: &str,
        input: &Path,
        output: &Path,
        error: &AgeError,
    ) -> AgeResult<()> {
        let message = format!(
            "OPERATION_FAILURE {} {} -> {} ERROR: {}",
            operation,
            input.display(),
            output.display(),
            error
        );
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
        if matches!(self.telemetry_format, TelemetryFormat::Json) {
            let event = json!({
                "event_type": "operation_start",
                "operation": operation,
                "path": path.display().to_string(),
            });
            self.log_json_event("INFO", event)
        } else {
            let message = format!("OPERATION_START {} {}", operation, path.display());
            self.log_event("INFO", &message)
        }
    }

    /// Log operation complete
    pub fn log_operation_complete(
        &self,
        operation: &str,
        path: &Path,
        result: &OperationResult,
    ) -> AgeResult<()> {
        if matches!(self.telemetry_format, TelemetryFormat::Json) {
            let event = json!({
                "event_type": "operation_complete",
                "operation": operation,
                "path": path.display().to_string(),
                "processed_count": result.processed_files.len(),
                "failed_count": result.failed_files.len(),
                "execution_time_ms": result.execution_time_ms,
                "processed_files": result.processed_files.clone(),
            });
            self.log_json_event("INFO", event)
        } else {
            let message = format!(
                "OPERATION_COMPLETE {} {} - processed: {}, failed: {}, duration: {}ms",
                operation,
                path.display(),
                result.processed_files.len(),
                result.failed_files.len(),
                result.execution_time_ms
            );
            self.log_event("INFO", &message)
        }
    }

    /// Log status check
    pub fn log_status_check(&self, path: &Path, status: &RepositoryStatus) -> AgeResult<()> {
        if matches!(self.telemetry_format, TelemetryFormat::Json) {
            let event = json!({
                "event_type": "status_check",
                "path": path.display().to_string(),
                "total_files": status.total_files,
                "encrypted_files": status.encrypted_files,
                "unencrypted_files": status.unencrypted_files,
            });
            self.log_json_event("INFO", event)
        } else {
            let message = format!(
                "STATUS_CHECK {} - total: {}, encrypted: {}, unencrypted: {}",
                path.display(),
                status.total_files,
                status.encrypted_files,
                status.unencrypted_files
            );
            self.log_event("INFO", &message)
        }
    }

    /// Log authority operation with structured metadata
    pub fn log_authority_operation(&self, operation: &str, recipient: &str) -> AgeResult<()> {
        if matches!(self.telemetry_format, TelemetryFormat::Json) {
            // Redact sensitive recipient data - only log hash for audit trail
            let recipient_hash = format!("{:x}", md5::compute(recipient.as_bytes()));
            let event = json!({
                "event_type": "authority_operation",
                "operation": operation,
                "recipient_hash": recipient_hash,
            });
            self.log_json_event("INFO", event)
        } else {
            let message = format!("AUTHORITY_OPERATION {} recipient: {}", operation, recipient);
            self.log_event("INFO", &message)
        }
    }

    /// Log structured encryption event with metadata
    ///
    /// # Arguments
    /// * `path` - File path being encrypted
    /// * `recipients` - Optional list of recipient public keys
    /// * `identity_type` - Type of identity (passphrase, age_identity, ssh_identity)
    /// * `success` - Whether the operation succeeded
    /// * `streaming_strategy` - Optional streaming strategy used (auto, file_staging, pipe)
    /// * `authority_tier` - Optional Ignition authority tier (X, M, R, I, D)
    pub fn log_encryption_event_extended(
        &self,
        path: &Path,
        recipients: Option<Vec<String>>,
        identity_type: &str,
        success: bool,
        streaming_strategy: Option<&str>,
        authority_tier: Option<&str>,
    ) -> AgeResult<()> {
        if matches!(self.telemetry_format, TelemetryFormat::Json) {
            let recipient_hash = recipients
                .as_ref()
                .map(|r| {
                    let mut sorted = r.clone();
                    sorted.sort();
                    format!("{:x}", md5::compute(sorted.join(",").as_bytes()))
                });

            let mut event = json!({
                "event_type": "encryption",
                "path": path.display().to_string(),
                "identity_type": identity_type,
                "recipient_count": recipients.as_ref().map(|r| r.len()).unwrap_or(0),
                "recipient_group_hash": recipient_hash,
                "success": success,
            });

            // Add optional metadata
            if let Some(obj) = event.as_object_mut() {
                if let Some(strategy) = streaming_strategy {
                    obj.insert("streaming_strategy".to_string(), json!(strategy));
                }
                if let Some(tier) = authority_tier {
                    obj.insert("authority_tier".to_string(), json!(tier));
                }
            }

            self.log_json_event("INFO", event)
        } else {
            let msg = if success {
                format!("ENCRYPTION {} identity:{} recipients:{} {}{}",
                    path.display(),
                    identity_type,
                    recipients.as_ref().map(|r| r.len()).unwrap_or(0),
                    streaming_strategy.map(|s| format!("strategy:{} ", s)).unwrap_or_default(),
                    authority_tier.map(|t| format!("tier:{}", t)).unwrap_or_default())
            } else {
                format!("ENCRYPTION_FAILED {} identity:{}", path.display(), identity_type)
            };
            self.log_event("INFO", &msg)
        }
    }

    /// Log structured encryption event with metadata (simplified version for backwards compat)
    pub fn log_encryption_event(
        &self,
        path: &Path,
        recipients: Option<Vec<String>>,
        identity_type: &str,
        success: bool,
    ) -> AgeResult<()> {
        self.log_encryption_event_extended(path, recipients, identity_type, success, None, None)
    }

    /// Log structured decryption event with extended metadata
    ///
    /// # Arguments
    /// * `path` - File path being decrypted
    /// * `identity_type` - Type of identity used (passphrase, age_identity, ssh_identity)
    /// * `success` - Whether the operation succeeded
    /// * `streaming_strategy` - Optional streaming strategy used
    pub fn log_decryption_event_extended(
        &self,
        path: &Path,
        identity_type: &str,
        success: bool,
        streaming_strategy: Option<&str>,
    ) -> AgeResult<()> {
        if matches!(self.telemetry_format, TelemetryFormat::Json) {
            let mut event = json!({
                "event_type": "decryption",
                "path": path.display().to_string(),
                "identity_type": identity_type,
                "success": success,
            });

            // Add optional metadata
            if let Some(obj) = event.as_object_mut() {
                if let Some(strategy) = streaming_strategy {
                    obj.insert("streaming_strategy".to_string(), json!(strategy));
                }
            }

            self.log_json_event("INFO", event)
        } else {
            let msg = if success {
                format!("DECRYPTION {} identity:{} {}",
                    path.display(),
                    identity_type,
                    streaming_strategy.map(|s| format!("strategy:{}", s)).unwrap_or_default())
            } else {
                format!("DECRYPTION_FAILED {} identity:{}", path.display(), identity_type)
            };
            self.log_event("INFO", &msg)
        }
    }

    /// Log structured decryption event with metadata (simplified version for backwards compat)
    pub fn log_decryption_event(
        &self,
        path: &Path,
        identity_type: &str,
        success: bool,
    ) -> AgeResult<()> {
        self.log_decryption_event_extended(path, identity_type, success, None)
    }

    /// Log structured JSON event
    fn log_json_event(&self, level: &str, mut event: serde_json::Value) -> AgeResult<()> {
        // Add common fields
        if let Some(obj) = event.as_object_mut() {
            obj.insert("timestamp".to_string(), json!(Utc::now().to_rfc3339()));
            obj.insert("level".to_string(), json!(level));
            obj.insert("component".to_string(), json!(self.component));
        }

        let log_entry = format!("{}\n", event.to_string());

        // Output
        eprint!("{}", log_entry);

        // Also log to file if configured
        if let Some(ref mut file) = &mut self.log_file.as_ref() {
            let mut file_handle = file.try_clone().map_err(|e| AgeError::AuditLogFailed {
                operation: "file_write".to_string(),
                reason: e.to_string(),
            })?;

            file_handle
                .write_all(log_entry.as_bytes())
                .map_err(|e| AgeError::AuditLogFailed {
                    operation: "write".to_string(),
                    reason: e.to_string(),
                })?;

            file_handle.flush().map_err(|e| AgeError::AuditLogFailed {
                operation: "flush".to_string(),
                reason: e.to_string(),
            })?;
        }

        Ok(())
    }

    /// Log emergency operation
    pub fn log_emergency_operation(&self, operation: &str, path: &Path) -> AgeResult<()> {
        let message = format!("EMERGENCY_OPERATION {} {}", operation, path.display());
        self.log_event("WARN", &message)
    }

    /// Core event logging function
    fn log_event(&self, level: &str, message: &str) -> AgeResult<()> {
        let timestamp = Utc::now();

        let log_entry = match self.telemetry_format {
            TelemetryFormat::Text => {
                format!(
                    "[{}] [{}] [{}] {}\n",
                    timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                    level,
                    self.component,
                    message
                )
            }
            TelemetryFormat::Json => {
                let json_event = json!({
                    "timestamp": timestamp.to_rfc3339(),
                    "level": level,
                    "component": self.component,
                    "message": message,
                });
                format!("{}\n", json_event.to_string())
            }
        };

        // Always log to stderr for immediate visibility
        eprint!("{}", log_entry);

        // Also log to file if configured
        if let Some(ref mut file) = &mut self.log_file.as_ref() {
            let mut file_handle = file.try_clone().map_err(|e| AgeError::AuditLogFailed {
                operation: "file_write".to_string(),
                reason: e.to_string(),
            })?;

            file_handle
                .write_all(log_entry.as_bytes())
                .map_err(|e| AgeError::AuditLogFailed {
                    operation: "write".to_string(),
                    reason: e.to_string(),
                })?;

            file_handle.flush().map_err(|e| AgeError::AuditLogFailed {
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
        assert!(validator
            .validate_file_path(Path::new("../etc/passwd"))
            .is_err());

        // Test valid path
        assert!(validator
            .validate_file_path(Path::new("./test.txt"))
            .is_ok());

        // Test injection detection
        assert!(validator
            .validate_passphrase_security("password$(rm -rf /)")
            .is_err());
        assert!(validator
            .validate_passphrase_security("validpassword")
            .is_ok());
    }

    #[test]
    fn test_json_telemetry_format() {
        use std::fs;
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Create logger with JSON format
        let logger = AuditLogger::with_format(Some(temp_path.clone()), TelemetryFormat::Json).unwrap();

        // Log an operation
        logger.log_operation_start_single("test_op", Path::new("/test/path")).unwrap();

        // Read the log file
        let log_content = fs::read_to_string(&temp_path).unwrap();

        // Verify JSON format
        assert!(log_content.contains("\"event_type\":\"operation_start\""));
        assert!(log_content.contains("\"operation\":\"test_op\""));
        assert!(log_content.contains("\"path\":\"/test/path\""));
        assert!(log_content.contains("\"component\":\"cage_automation\""));
    }

    #[test]
    fn test_encryption_event_json() {
        use std::fs;
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        let logger = AuditLogger::with_format(Some(temp_path.clone()), TelemetryFormat::Json).unwrap();

        let recipients = vec!["age1abc123".to_string(), "age1xyz789".to_string()];
        logger.log_encryption_event_extended(
            Path::new("/secret.txt"),
            Some(recipients),
            "age_identity",
            true,
            Some("pipe"),
            Some("M")
        ).unwrap();

        let log_content = fs::read_to_string(&temp_path).unwrap();

        // Verify JSON structure
        assert!(log_content.contains("\"event_type\":\"encryption\""));
        assert!(log_content.contains("\"identity_type\":\"age_identity\""));
        assert!(log_content.contains("\"recipient_count\":2"));
        assert!(log_content.contains("\"streaming_strategy\":\"pipe\""));
        assert!(log_content.contains("\"authority_tier\":\"M\""));
        assert!(log_content.contains("\"success\":true"));

        // Verify sensitive recipient_group_hash is present (MD5 audit hash)
        assert!(log_content.contains("\"recipient_group_hash\":"));
    }

    #[test]
    fn test_operation_complete_json() {
        use std::fs;
        use crate::cage::operations::OperationResult;

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        let logger = AuditLogger::with_format(Some(temp_path.clone()), TelemetryFormat::Json).unwrap();

        let mut result = OperationResult::new();
        result.processed_files.push("file1.txt".to_string());
        result.processed_files.push("file2.txt".to_string());
        result.execution_time_ms = 150;

        logger.log_operation_complete("lock", Path::new("/repo"), &result).unwrap();

        let log_content = fs::read_to_string(&temp_path).unwrap();

        // Verify JSON structure
        assert!(log_content.contains("\"event_type\":\"operation_complete\""));
        assert!(log_content.contains("\"operation\":\"lock\""));
        assert!(log_content.contains("\"processed_count\":2"));
        assert!(log_content.contains("\"failed_count\":0"));
        assert!(log_content.contains("\"execution_time_ms\":150"));
        assert!(log_content.contains("\"processed_files\":[\"file1.txt\",\"file2.txt\"]"));
    }

    #[test]
    fn test_text_telemetry_format() {
        use std::fs;
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Create logger with Text format (default)
        let logger = AuditLogger::with_format(Some(temp_path.clone()), TelemetryFormat::Text).unwrap();

        // Log an operation
        logger.log_operation_start_single("test_op", Path::new("/test/path")).unwrap();

        // Read the log file
        let log_content = fs::read_to_string(&temp_path).unwrap();

        // Verify text format (not JSON)
        assert!(log_content.contains("OPERATION_START"));
        assert!(log_content.contains("test_op"));
        assert!(log_content.contains("/test/path"));
        assert!(!log_content.contains("\"event_type\""));
    }
}
