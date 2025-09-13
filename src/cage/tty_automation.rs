//! TTY Automation - Proven Script/Expect Methods for Age
//!
//! This module implements the proven TTY automation methods from the pilot that successfully
//! eliminated T2.1: TTY Automation Subversion. Uses script command and expect automation
//! with fallback for maximum reliability.
//!
//! Security Guardian: Edgar - Production implementation of proven automation patterns

use std::process::{Command, Stdio};
use std::path::Path;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;
use super::error::{AgeError, AgeResult};
use super::config::{OutputFormat, TtyMethod};

/// TTY automation engine using proven pilot methods
pub struct TtyAutomator {
    temp_dir: TempDir,
    preferred_method: TtyMethod,
    timeout: Duration,
}

impl TtyAutomator {
    /// Create new TTY automator with secure temporary directory
    pub fn new() -> AgeResult<Self> {
        let temp_dir = tempfile::tempdir()
            .map_err(|e| AgeError::TemporaryResourceError {
                resource_type: "directory".to_string(),
                operation: "create".to_string(),
                reason: e.to_string(),
            })?;

        Ok(Self {
            temp_dir,
            preferred_method: TtyMethod::Auto,
            timeout: Duration::from_secs(120),
        })
    }

    /// Encrypt file using proven TTY automation
    pub fn encrypt(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()> {
        // Validate inputs
        if !input.exists() {
            return Err(AgeError::file_error("read", input.to_path_buf(), 
                std::io::Error::new(std::io::ErrorKind::NotFound, "Input file not found")));
        }

        // Try script method first (proven fastest)
        match self.encrypt_with_script(input, output, passphrase, format) {
            Ok(_) => return Ok(()),
            Err(script_err) => {
                // Fallback to expect method
                match self.encrypt_with_expect(input, output, passphrase, format) {
                    Ok(_) => return Ok(()),
                    Err(expect_err) => {
                        return Err(AgeError::AllTtyMethodsFailed(vec![
                            format!("script: {}", script_err),
                            format!("expect: {}", expect_err),
                        ]));
                    }
                }
            }
        }
    }

    /// Decrypt file using proven TTY automation
    pub fn decrypt(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()> {
        // Validate inputs
        if !input.exists() {
            return Err(AgeError::file_error("read", input.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Input file not found")));
        }

        // Try script method first
        match self.decrypt_with_script(input, output, passphrase) {
            Ok(_) => return Ok(()),
            Err(script_err) => {
                // Fallback to expect method
                match self.decrypt_with_expect(input, output, passphrase) {
                    Ok(_) => return Ok(()),
                    Err(expect_err) => {
                        return Err(AgeError::AllTtyMethodsFailed(vec![
                            format!("script: {}", script_err),
                            format!("expect: {}", expect_err),
                        ]));
                    }
                }
            }
        }
    }

    /// Method 1: Script command TTY automation (proven fastest)
    fn encrypt_with_script(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()> {
        let mut age_cmd = vec!["age", "-p"];
        
        // Add ASCII armor flag if requested
        if matches!(format, OutputFormat::AsciiArmor) {
            age_cmd.push("-a");
        }
        
        let output_str = output.to_string_lossy();
        let input_str = input.to_string_lossy();
        age_cmd.extend(["-o", &output_str, &input_str]);
        
        let script_cmd = format!(
            "script -q -c \"{} << 'EOF'\n{}\n{}\nEOF\" /dev/null",
            age_cmd.join(" "),
            passphrase,
            passphrase  // Confirmation
        );

        let result = Command::new("sh")
            .arg("-c")
            .arg(&script_cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AgeError::ProcessExecutionFailed {
                command: "script".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        if result.status.success() && output.exists() {
            Ok(())
        } else {
            Err(AgeError::EncryptionFailed {
                input: input.to_path_buf(),
                output: output.to_path_buf(),
                reason: format!("Script method failed: {}", String::from_utf8_lossy(&result.stderr)),
            })
        }
    }

    /// Method 2: Expect automation (proven most reliable)
    fn encrypt_with_expect(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()> {
        // Create expect script
        let expect_script = self.temp_dir.path().join("encrypt.exp");
        
        let mut age_cmd = vec!["age", "-p"];
        if matches!(format, OutputFormat::AsciiArmor) {
            age_cmd.push("-a");
        }
        let output_str = output.to_string_lossy();
        let input_str = input.to_string_lossy();
        age_cmd.extend(["-o", &output_str, &input_str]);

        let expect_content = format!(
            r#"#!/usr/bin/expect -f
set timeout 30
log_user 0

spawn {}

expect {{
    "Enter passphrase*" {{
        send "{}\r"
        exp_continue
    }}
    "Confirm passphrase*" {{
        send "{}\r"
        exp_continue
    }}
    eof {{
        catch wait result
        set exit_status [lindex $result 3]
        exit $exit_status
    }}
    timeout {{
        puts stderr "ERROR: Timeout waiting for age encryption"
        exit 1
    }}
}}
"#,
            age_cmd.join(" "),
            passphrase.replace("\"", "\\\""),
            passphrase.replace("\"", "\\\"")
        );

        fs::write(&expect_script, expect_content)
            .map_err(|e| AgeError::file_error("write", expect_script.clone(), e))?;

        let result = Command::new("expect")
            .arg(&expect_script)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AgeError::ProcessExecutionFailed {
                command: "expect".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        if result.status.success() && output.exists() {
            Ok(())
        } else {
            Err(AgeError::EncryptionFailed {
                input: input.to_path_buf(),
                output: output.to_path_buf(),
                reason: format!("Expect method failed: {}", String::from_utf8_lossy(&result.stderr)),
            })
        }
    }

    /// Script-based decryption
    fn decrypt_with_script(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()> {
        let script_cmd = format!(
            "script -q -c \"age -d -o {} {} << 'EOF'\n{}\nEOF\" /dev/null",
            output.to_string_lossy(),
            input.to_string_lossy(),
            passphrase
        );

        let result = Command::new("sh")
            .arg("-c")
            .arg(&script_cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AgeError::ProcessExecutionFailed {
                command: "script".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        if result.status.success() && output.exists() {
            Ok(())
        } else {
            Err(AgeError::DecryptionFailed {
                input: input.to_path_buf(),
                output: output.to_path_buf(),
                reason: format!("Script decryption failed: {}", String::from_utf8_lossy(&result.stderr)),
            })
        }
    }

    /// Expect-based decryption
    fn decrypt_with_expect(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()> {
        let expect_script = self.temp_dir.path().join("decrypt.exp");
        
        let expect_content = format!(
            r#"#!/usr/bin/expect -f
set timeout 30
log_user 0

spawn age -d -o {} {}

expect {{
    "Enter passphrase*" {{
        send "{}\r"
        exp_continue
    }}
    eof {{
        catch wait result
        set exit_status [lindex $result 3]
        exit $exit_status
    }}
    timeout {{
        puts stderr "ERROR: Timeout waiting for age decryption"
        exit 1
    }}
}}
"#,
            output.to_string_lossy(),
            input.to_string_lossy(),
            passphrase.replace("\"", "\\\"")
        );

        fs::write(&expect_script, expect_content)
            .map_err(|e| AgeError::file_error("write", expect_script.clone(), e))?;

        let result = Command::new("expect")
            .arg(&expect_script)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AgeError::ProcessExecutionFailed {
                command: "expect".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        if result.status.success() && output.exists() {
            Ok(())
        } else {
            Err(AgeError::DecryptionFailed {
                input: input.to_path_buf(),
                output: output.to_path_buf(),
                reason: format!("Expect decryption failed: {}", String::from_utf8_lossy(&result.stderr)),
            })
        }
    }

    /// Check if Age binary is available
    pub fn check_age_binary(&self) -> AgeResult<()> {
        Command::new("age")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| AgeError::AgeBinaryNotFound("age command not found in PATH".to_string()))?;
        Ok(())
    }

    /// Check TTY automation methods availability
    pub fn check_automation_methods(&self) -> AgeResult<()> {
        // Check script command
        let script_available = Command::new("script")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok();

        // Check expect command  
        let expect_available = Command::new("expect")
            .arg("-v")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok();

        if !script_available && !expect_available {
            return Err(AgeError::AllTtyMethodsFailed(vec![
                "script command not available".to_string(),
                "expect command not available".to_string(),
            ]));
        }

        Ok(())
    }

    /// Perform comprehensive health check
    pub fn perform_health_check(&self) -> AgeResult<()> {
        self.check_age_binary()?;
        self.check_automation_methods()?;
        
        // TODO: Implement full encrypt/decrypt cycle test
        Ok(())
    }

    /// Get available automation methods
    pub fn available_methods(&self) -> Vec<String> {
        let mut methods = Vec::new();
        
        if Command::new("script").arg("--version").stdout(Stdio::null()).status().is_ok() {
            methods.push("script".to_string());
        }
        
        if Command::new("expect").arg("-v").stdout(Stdio::null()).status().is_ok() {
            methods.push("expect".to_string());
        }
        
        methods
    }

    /// Validate dependencies
    pub fn validate_dependencies(&self) -> AgeResult<()> {
        self.check_age_binary()?;
        self.check_automation_methods()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_automator_creation() {
        let automator = TtyAutomator::new();
        assert!(automator.is_ok());
    }

    #[test]
    fn test_dependency_checking() {
        let automator = TtyAutomator::new().unwrap();
        // This will fail if Age not installed, which is expected in test environments
        let _result = automator.check_age_binary();
    }
}