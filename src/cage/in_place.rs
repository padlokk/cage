//! Safe In-Place File Operations
//!
//! Implements secure in-place file replacement with multiple layers of safety protection
//! to prevent data loss during encryption/decryption operations.
//!
//! Safety Architecture:
//! - Layer 1: Explicit --in-place flag required
//! - Layer 2: Recovery file creation (default)
//! - Layer 3: --danger-mode with confirmation
//! - Layer 4: DANGER_MODE=1 environment variable
//! - Layer 5: --i-am-sure automation override

use std::path::{Path, PathBuf};
use std::io::{self, Write};
use chrono::Utc;
use crate::cage::strings::fmt_warning;
use crate::cage::error::{AgeError, AgeResult};

/// Recovery file manager for creating and managing .tmp.recover files
pub struct RecoveryManager {
    create_recovery: bool,
    danger_mode: bool,
}

impl RecoveryManager {
    pub fn new(create_recovery: bool, danger_mode: bool) -> Self {
        Self {
            create_recovery,
            danger_mode,
        }
    }

    /// Create recovery file with passphrase and instructions
    pub fn create_recovery_file(&self, original: &Path, passphrase: &str, operation: &str) -> AgeResult<PathBuf> {
        if !self.create_recovery || self.danger_mode {
            return Err(AgeError::InvalidOperation {
                operation: "create_recovery_file".to_string(),
                reason: "Recovery file creation disabled".to_string(),
            });
        }

        let recovery_path = original.with_extension("tmp.recover");
        let content = format!(r#"# CAGE RECOVERY INFORMATION
# Generated: {}
# Original: {}
# Operation: {}
# Passphrase: {}
#
# TO RECOVER YOUR FILE:
# cage unlock {} {}
#
# DELETE THIS FILE ONCE YOU'VE VERIFIED YOUR ENCRYPTION!
# This file contains your passphrase and is a security risk if left around.
"#,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            original.display(),
            operation,
            passphrase,
            original.display(),
            passphrase
        );

        // Create with restrictive permissions (600)
        std::fs::write(&recovery_path, content)
            .map_err(|e| AgeError::file_error("create_recovery", recovery_path.clone(), e))?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&recovery_path, perms)
                .map_err(|e| AgeError::file_error("set_permissions", recovery_path.clone(), e))?;
        }

        Ok(recovery_path)
    }
}

/// Safety validator for in-place operations
pub struct SafetyValidator {
    danger_mode: bool,
    i_am_sure: bool,
    env_danger: bool,
}

impl SafetyValidator {
    pub fn new(danger_mode: bool, i_am_sure: bool) -> Self {
        let env_danger = std::env::var("DANGER_MODE").map(|v| v == "1").unwrap_or(false);

        Self {
            danger_mode,
            i_am_sure,
            env_danger,
        }
    }

    /// Validate in-place operation safety requirements
    pub fn validate_in_place_operation(&self, file: &Path) -> AgeResult<()> {
        // Check if file exists and will be replaced
        if !file.exists() {
            return Err(AgeError::FileError {
                operation: "in-place".to_string(),
                path: file.to_path_buf(),
                source: io::Error::new(io::ErrorKind::NotFound, "File not found"),
            });
        }

        // Danger mode validation
        if self.danger_mode {
            if !self.env_danger {
                return Err(AgeError::InvalidOperation {
                    operation: "in-place-danger".to_string(),
                    reason: "DANGER_MODE=1 environment variable required with --danger-mode".to_string(),
                });
            }

            if !self.i_am_sure {
                // Prompt for confirmation
                eprintln!("{}", fmt_warning("DANGER MODE: This action is UNRECOVERABLE!"));
                eprintln!("   File: {}", file.display());
                eprintln!("   No recovery file will be created.");
                eprintln!("   If encryption fails or you forget the passphrase, your file is LOST FOREVER.");
                eprintln!();
                eprint!("Type 'DELETE MY FILE' to confirm this unrecoverable action: ");

                io::stderr().flush().map_err(|e| AgeError::IoError {
                    operation: "flush_stderr".to_string(),
                    context: "confirmation_prompt".to_string(),
                    source: e,
                })?;

                let mut input = String::new();
                io::stdin().read_line(&mut input).map_err(|e| AgeError::IoError {
                    operation: "read_line".to_string(),
                    context: "confirmation_input".to_string(),
                    source: e,
                })?;

                if input.trim() != "DELETE MY FILE" {
                    return Err(AgeError::InvalidOperation {
                        operation: "in-place-danger".to_string(),
                        reason: "User cancelled dangerous operation".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Atomic in-place operation manager
pub struct InPlaceOperation {
    original: PathBuf,
    temp_encrypted: PathBuf,
    recovery_file: Option<PathBuf>,
    completed: bool,
}

impl InPlaceOperation {
    pub fn new(file: &Path) -> Self {
        Self {
            original: file.to_path_buf(),
            temp_encrypted: file.with_extension("tmp.cage"),
            recovery_file: None,
            completed: false,
        }
    }

    /// Execute in-place lock operation
    pub fn execute_lock<F>(&mut self, passphrase: &str, danger_mode: bool, encrypt_fn: F) -> AgeResult<()>
    where
        F: FnOnce(&Path, &Path, &str) -> AgeResult<()>,
    {
        // 1. Create recovery file if not in danger mode
        if !danger_mode {
            let recovery_manager = RecoveryManager::new(true, false);
            self.recovery_file = Some(recovery_manager.create_recovery_file(
                &self.original,
                passphrase,
                "encrypt"
            )?);
        }

        // 2. Encrypt original -> temp
        encrypt_fn(&self.original, &self.temp_encrypted, passphrase)?;

        // 3. Verify temp file exists and is readable
        if !self.temp_encrypted.exists() {
            return Err(AgeError::FileError {
                operation: "verify_temp".to_string(),
                path: self.temp_encrypted.clone(),
                source: io::Error::new(io::ErrorKind::NotFound, "Encrypted temp file not created"),
            });
        }

        // 4. Preserve metadata
        self.copy_metadata(&self.original, &self.temp_encrypted)?;

        // 5. Atomic replace (this is the dangerous moment)
        std::fs::rename(&self.temp_encrypted, &self.original)
            .map_err(|e| AgeError::file_error("atomic_replace", self.original.clone(), e))?;

        self.completed = true;
        Ok(())
    }

    /// Copy metadata from source to destination
    fn copy_metadata(&self, from: &Path, to: &Path) -> AgeResult<()> {
        let metadata = std::fs::metadata(from)
            .map_err(|e| AgeError::file_error("read_metadata", from.to_path_buf(), e))?;

        let permissions = metadata.permissions();
        std::fs::set_permissions(to, permissions)
            .map_err(|e| AgeError::file_error("set_permissions", to.to_path_buf(), e))?;

        // Set modification time (if possible)
        #[cfg(unix)]
        {
            if let Ok(_modified) = metadata.modified() {
                use std::os::unix::fs::MetadataExt;
                let _atime = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(metadata.atime() as u64);

                // Use filetime crate if available, or ignore if not critical
                // This is optional metadata preservation
                let _ = std::process::Command::new("touch")
                    .arg("-r")
                    .arg(from)
                    .arg(to)
                    .output();
            }
        }

        Ok(())
    }
}

impl Drop for InPlaceOperation {
    fn drop(&mut self) {
        if !self.completed {
            // Rollback: remove temp file if operation failed
            if self.temp_encrypted.exists() {
                let _ = std::fs::remove_file(&self.temp_encrypted);
            }

            // Remove recovery file if operation failed
            if let Some(ref recovery) = self.recovery_file {
                if recovery.exists() {
                    let _ = std::fs::remove_file(recovery);
                }
            }
        }
    }
}

/// Configuration for in-place operations
#[derive(Debug, Clone)]
pub struct InPlaceOptions {
    pub enabled: bool,
    pub danger_mode: bool,
    pub i_am_sure: bool,
}

impl Default for InPlaceOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            danger_mode: false,
            i_am_sure: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_recovery_manager_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let recovery_manager = RecoveryManager::new(true, false);
        let recovery_path = recovery_manager.create_recovery_file(
            &test_file,
            "testpass",
            "encrypt"
        ).unwrap();

        assert!(recovery_path.exists());
        let content = std::fs::read_to_string(&recovery_path).unwrap();
        assert!(content.contains("testpass"));
        assert!(content.contains("RECOVERY INFORMATION"));
    }

    #[test]
    fn test_safety_validator_blocks_without_env() {
        let validator = SafetyValidator::new(true, false); // danger mode but no env
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "content").unwrap();

        let result = validator.validate_in_place_operation(&test_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DANGER_MODE=1"));
    }

    #[test]
    fn test_in_place_operation_cleanup_on_drop() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "content").unwrap();

        {
            let mut op = InPlaceOperation::new(&test_file);
            // Create temp file to simulate partial operation
            std::fs::write(&op.temp_encrypted, "temp content").unwrap();
            // Drop without completing - should clean up
        }

        // Temp file should be cleaned up
        let temp_path = test_file.with_extension("tmp.cage");
        assert!(!temp_path.exists());
    }
}
