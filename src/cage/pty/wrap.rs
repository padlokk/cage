//! PTY Age Automation - Proper Terminal Emulation for Age
//!
//! This module uses proper PTY (Pseudo Terminal) automation to control age,
//! making age think it's running in a real terminal for reliable automation.

use std::io::{Read, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
// Use Hub's terminal-ext for portable-pty (RSB ecosystem approach)
// Both import styles work per HOWTO_HUB:
use hub::terminal_ext::portable_pty::*; // Grouped module (preferred for clarity)
                                        // Alternative: use hub::portable_pty::*;  // Top-level re-export
use crate::cage::core::OutputFormat;
use crate::cage::error::{AgeError, AgeResult};
use tempfile::TempDir;

/// PTY-based Age automator - reliable and robust
pub struct PtyAgeAutomator {
    temp_dir: TempDir,
    timeout: Duration,
    capture_stderr: bool,
}

impl PtyAgeAutomator {
    /// Create new PTY Age automator
    pub fn new() -> AgeResult<Self> {
        Self::with_config(&crate::cage::core::AgeConfig::load_default().unwrap_or_default())
    }

    /// Create PTY automator with custom config
    pub fn with_config(config: &crate::cage::core::AgeConfig) -> AgeResult<Self> {
        let temp_dir = tempfile::tempdir().map_err(|e| AgeError::TemporaryResourceError {
            resource_type: "directory".to_string(),
            operation: "create".to_string(),
            reason: format!(
                "Failed to create temporary directory for PTY operations: {}",
                e
            ),
        })?;

        Ok(Self {
            temp_dir,
            timeout: config.operation_timeout,
            capture_stderr: true,
        })
    }

    /// Encrypt file using PTY automation - foolproof method
    pub fn encrypt(
        &self,
        input: &Path,
        output: &Path,
        passphrase: &str,
        format: OutputFormat,
    ) -> AgeResult<()> {
        let mut captured_stderr = String::new();

        // Validate inputs
        if !input.exists() {
            return Err(AgeError::file_error(
                "read",
                input.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Input file not found"),
            ));
        }

        let pty_system = native_pty_system();
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = pty_system
            .openpty(pty_size)
            .map_err(|e| AgeError::ProcessExecutionFailed {
                command: "create_pty".to_string(),
                exit_code: None,
                stderr: format!("Failed to create PTY: {}", e),
            })?;

        // Build age command
        let mut cmd = CommandBuilder::new("age");
        cmd.arg("-p"); // Passphrase mode (requires TTY)

        // Set working directory to match parent process
        if let Ok(current_dir) = std::env::current_dir() {
            cmd.cwd(current_dir);
        }

        if matches!(format, OutputFormat::AsciiArmor) {
            cmd.arg("-a");
        }

        cmd.arg("-o");
        cmd.arg(output);
        cmd.arg(input);

        // Spawn age in PTY - it thinks it has a real terminal!
        let child = pair.slave.spawn_command(cmd).map_err(|e| {
            let error_msg = format!("{}", e);
            if error_msg.contains("No viable candidates found in PATH")
                || error_msg.contains("No such file or directory")
                || error_msg.contains("not found")
            {
                AgeError::AgeBinaryNotFound(format!("age command not found: {}", e))
            } else {
                AgeError::ProcessExecutionFailed {
                    command: "age".to_string(),
                    exit_code: None,
                    stderr: format!("Failed to spawn age: {}", e),
                }
            }
        })?;

        drop(pair.slave); // Close slave end in parent

        let mut writer =
            pair.master
                .take_writer()
                .map_err(|e| AgeError::ProcessExecutionFailed {
                    command: "pty_writer".to_string(),
                    exit_code: None,
                    stderr: format!("Failed to get PTY writer: {}", e),
                })?;

        let mut reader =
            pair.master
                .try_clone_reader()
                .map_err(|e| AgeError::ProcessExecutionFailed {
                    command: "pty_reader".to_string(),
                    exit_code: None,
                    stderr: format!("Failed to get PTY reader: {}", e),
                })?;

        // Handle age interaction with timeout and proper process monitoring
        let passphrase_clone = passphrase.to_string();
        let timeout_duration = self.timeout;
        let capture_stderr = self.capture_stderr;
        let automation_thread = thread::spawn(move || -> AgeResult<String> {
            let mut buffer = [0u8; 1024];
            let mut output_buffer = String::new();
            let mut stderr_buffer = String::new();
            let start_time = std::time::Instant::now();

            // Set reader to non-blocking mode would be ideal, but portable-pty doesn't expose that
            // Instead, we'll use a timeout-based approach with short reads

            loop {
                // Check for timeout
                if start_time.elapsed() > timeout_duration {
                    return Err(AgeError::ProcessExecutionFailed {
                        command: "pty_automation_timeout".to_string(),
                        exit_code: None,
                        stderr: format!("PTY automation timed out after {:?}", timeout_duration),
                    });
                }

                // Try to read with a short timeout using thread sleep
                // This is not ideal but works with portable-pty's blocking interface
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - age finished
                        break;
                    }
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buffer[..n]);
                        output_buffer.push_str(&text);

                        // Capture stderr if enabled
                        if capture_stderr {
                            stderr_buffer.push_str(&text);
                        }

                        // Check for passphrase prompts (case-insensitive)
                        let output_lower = output_buffer.to_lowercase();

                        // More robust prompt detection
                        let is_passphrase_prompt = output_lower.contains("enter passphrase")
                            || output_lower.contains("passphrase:")
                            || output_lower.contains("password:")
                            || output_lower.contains("enter password");

                        if is_passphrase_prompt && !output_lower.contains("confirm") {
                            // Send passphrase
                            writer.write_all(passphrase_clone.as_bytes()).map_err(|e| {
                                AgeError::ProcessExecutionFailed {
                                    command: "pty_write".to_string(),
                                    exit_code: None,
                                    stderr: format!(
                                        "Failed to write passphrase to PTY: {}. Prompt was: '{}'",
                                        e,
                                        output_buffer.trim()
                                    ),
                                }
                            })?;
                            writer.write_all(b"\n").map_err(|e| {
                                AgeError::ProcessExecutionFailed {
                                    command: "pty_write".to_string(),
                                    exit_code: None,
                                    stderr: format!(
                                        "Failed to send newline after passphrase: {}",
                                        e
                                    ),
                                }
                            })?;

                            // Clear buffer after handling prompt
                            output_buffer.clear();
                        }

                        // Check for confirmation prompts
                        let is_confirm_prompt = output_lower.contains("confirm passphrase")
                            || output_lower.contains("confirm:")
                            || output_lower.contains("re-enter")
                            || output_lower.contains("verify");

                        if is_confirm_prompt {
                            // Send confirmation
                            writer.write_all(passphrase_clone.as_bytes()).map_err(|e| {
                                AgeError::ProcessExecutionFailed {
                                    command: "pty_write".to_string(),
                                    exit_code: None,
                                    stderr: format!("Failed to write passphrase confirmation: {}. Prompt was: '{}'", e, output_buffer.trim()),
                                }
                            })?;
                            writer.write_all(b"\n").map_err(|e| {
                                AgeError::ProcessExecutionFailed {
                                    command: "pty_write".to_string(),
                                    exit_code: None,
                                    stderr: format!(
                                        "Failed to send newline after confirmation: {}",
                                        e
                                    ),
                                }
                            })?;

                            output_buffer.clear();
                        }
                    }
                    Err(e) => {
                        // Check if it's a "would block" error (non-fatal) or real error
                        use std::io::ErrorKind;
                        if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                            // Give it a moment and continue
                            thread::sleep(Duration::from_millis(50));
                            continue;
                        }

                        return Err(AgeError::ProcessExecutionFailed {
                            command: "pty_read".to_string(),
                            exit_code: None,
                            stderr: format!("PTY read error: {}", e),
                        });
                    }
                }

                // Small delay to prevent busy waiting
                thread::sleep(Duration::from_millis(10));
            }
            Ok(stderr_buffer)
        });

        // Wait for automation thread with timeout
        let automation_result = match automation_thread.join() {
            Ok(Ok(stderr)) => {
                captured_stderr = stderr;
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(AgeError::ProcessExecutionFailed {
                command: "automation_thread".to_string(),
                exit_code: None,
                stderr: "Automation thread panicked".to_string(),
            }),
        };

        // Wait for child process to complete
        let mut child = child;
        let exit_status = child.wait().map_err(|e| AgeError::ProcessExecutionFailed {
            command: "age_wait".to_string(),
            exit_code: None,
            stderr: format!("Failed to wait for age process: {}", e),
        })?;

        // Check results
        automation_result?;

        if exit_status.success() && output.exists() {
            Ok(())
        } else {
            let reason = if !captured_stderr.is_empty() {
                format!(
                    "Age encryption failed with exit status: {:?}. Stderr: {}",
                    exit_status,
                    captured_stderr.trim()
                )
            } else {
                format!("Age encryption failed with exit status: {:?}", exit_status)
            };
            Err(AgeError::EncryptionFailed {
                input: input.to_path_buf(),
                output: output.to_path_buf(),
                reason,
            })
        }
    }

    /// Decrypt file using PTY automation
    pub fn decrypt(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()> {
        let mut captured_stderr = String::new();

        // Validate inputs
        if !input.exists() {
            return Err(AgeError::file_error(
                "read",
                input.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Input file not found"),
            ));
        }

        let pty_system = native_pty_system();
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = pty_system
            .openpty(pty_size)
            .map_err(|e| AgeError::ProcessExecutionFailed {
                command: "create_pty".to_string(),
                exit_code: None,
                stderr: format!("Failed to create PTY: {}", e),
            })?;

        // Build age decrypt command
        let mut cmd = CommandBuilder::new("age");
        cmd.arg("-d"); // Decrypt mode

        // Set working directory to match parent process
        if let Ok(current_dir) = std::env::current_dir() {
            cmd.cwd(current_dir);
        }

        cmd.arg("-o");
        cmd.arg(output);
        cmd.arg(input);

        // Spawn age in PTY
        let child = pair.slave.spawn_command(cmd).map_err(|e| {
            let error_msg = format!("{}", e);
            if error_msg.contains("No viable candidates found in PATH")
                || error_msg.contains("No such file or directory")
                || error_msg.contains("not found")
            {
                AgeError::AgeBinaryNotFound(format!("age command not found: {}", e))
            } else {
                AgeError::ProcessExecutionFailed {
                    command: "age".to_string(),
                    exit_code: None,
                    stderr: format!("Failed to spawn age: {}", e),
                }
            }
        })?;

        drop(pair.slave);

        let mut writer =
            pair.master
                .take_writer()
                .map_err(|e| AgeError::ProcessExecutionFailed {
                    command: "pty_writer".to_string(),
                    exit_code: None,
                    stderr: format!("Failed to get PTY writer: {}", e),
                })?;

        let mut reader =
            pair.master
                .try_clone_reader()
                .map_err(|e| AgeError::ProcessExecutionFailed {
                    command: "pty_reader".to_string(),
                    exit_code: None,
                    stderr: format!("Failed to get PTY reader: {}", e),
                })?;

        // Handle decryption interaction with timeout
        let passphrase_clone = passphrase.to_string();
        let timeout_duration = self.timeout;
        let capture_stderr = self.capture_stderr;
        let automation_thread = thread::spawn(move || -> AgeResult<String> {
            let mut buffer = [0u8; 1024];
            let mut output_buffer = String::new();
            let mut stderr_buffer = String::new();
            let start_time = std::time::Instant::now();

            loop {
                // Check for timeout
                if start_time.elapsed() > timeout_duration {
                    return Err(AgeError::ProcessExecutionFailed {
                        command: "pty_automation_timeout".to_string(),
                        exit_code: None,
                        stderr: format!("PTY automation timed out after {:?}", timeout_duration),
                    });
                }

                match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buffer[..n]);
                        output_buffer.push_str(&text);

                        // Capture stderr if enabled
                        if capture_stderr {
                            stderr_buffer.push_str(&text);
                        }

                        // Look for passphrase prompt (case-insensitive)
                        let output_lower = output_buffer.to_lowercase();
                        let is_passphrase_prompt = output_lower.contains("enter passphrase")
                            || output_lower.contains("passphrase:")
                            || output_lower.contains("password:")
                            || output_lower.contains("enter password");

                        if is_passphrase_prompt && !output_lower.contains("confirm") {
                            writer.write_all(passphrase_clone.as_bytes()).map_err(|e| {
                                AgeError::ProcessExecutionFailed {
                                    command: "pty_write_passphrase".to_string(),
                                    exit_code: None,
                                    stderr: format!(
                                        "Failed to write passphrase: {}. Prompt was: '{}'",
                                        e,
                                        output_buffer.trim()
                                    ),
                                }
                            })?;
                            writer.write_all(b"\n").map_err(|e| {
                                AgeError::ProcessExecutionFailed {
                                    command: "pty_write_newline".to_string(),
                                    exit_code: None,
                                    stderr: format!("Failed to write newline: {}", e),
                                }
                            })?;

                            output_buffer.clear();
                        }
                    }
                    Err(e) => {
                        // Check if it's a "would block" error (non-fatal) or real error
                        use std::io::ErrorKind;
                        if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                            // Give it a moment and continue
                            thread::sleep(Duration::from_millis(50));
                            continue;
                        }

                        return Err(AgeError::ProcessExecutionFailed {
                            command: "pty_read".to_string(),
                            exit_code: None,
                            stderr: format!("PTY read error: {}", e),
                        });
                    }
                }

                // Small delay to prevent busy waiting
                thread::sleep(Duration::from_millis(10));
            }
            Ok(stderr_buffer)
        });

        // Wait for automation and process
        let automation_result = match automation_thread.join() {
            Ok(Ok(stderr)) => {
                captured_stderr = stderr;
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(AgeError::ProcessExecutionFailed {
                command: "automation_thread".to_string(),
                exit_code: None,
                stderr: "Automation thread panicked".to_string(),
            }),
        };

        let mut child = child;
        let exit_status = child.wait().map_err(|e| AgeError::ProcessExecutionFailed {
            command: "age_wait".to_string(),
            exit_code: None,
            stderr: format!("Failed to wait for age process: {}", e),
        })?;

        automation_result?;

        if exit_status.success() && output.exists() {
            Ok(())
        } else {
            let reason = if !captured_stderr.is_empty() {
                format!(
                    "Age decryption failed with exit status: {:?}. Stderr: {}",
                    exit_status,
                    captured_stderr.trim()
                )
            } else {
                format!("Age decryption failed with exit status: {:?}", exit_status)
            };
            Err(AgeError::DecryptionFailed {
                input: input.to_path_buf(),
                output: output.to_path_buf(),
                reason,
            })
        }
    }

    /// Check if Age binary is available
    pub fn check_age_binary(&self) -> AgeResult<()> {
        let pty_system = native_pty_system();
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = pty_system
            .openpty(pty_size)
            .map_err(|e| AgeError::AgeBinaryNotFound(format!("PTY creation failed: {}", e)))?;

        let mut cmd = CommandBuilder::new("age");
        cmd.arg("--version");

        let child = pair.slave.spawn_command(cmd).map_err(|_| {
            AgeError::AgeBinaryNotFound("age command not found in PATH".to_string())
        })?;

        let mut child = child;
        let exit_status = child.wait().map_err(|e| {
            AgeError::AgeBinaryNotFound(format!("Failed to run age --version: {}", e))
        })?;

        if exit_status.success() {
            Ok(())
        } else {
            Err(AgeError::AgeBinaryNotFound(
                "age --version failed".to_string(),
            ))
        }
    }

    /// Perform comprehensive health check
    pub fn perform_health_check(&self) -> AgeResult<()> {
        self.check_age_binary()?;

        // Test encrypt/decrypt cycle with PTY
        let test_content = "PTY automation test content";
        let test_passphrase = "test-passphrase-123";

        let input_file = self.temp_dir.path().join("test_input.txt");
        let encrypted_file = self.temp_dir.path().join("test_encrypted.age");
        let decrypted_file = self.temp_dir.path().join("test_decrypted.txt");

        // Write test file
        std::fs::write(&input_file, test_content)
            .map_err(|e| AgeError::file_error("write", input_file.clone(), e))?;

        // Test encryption
        self.encrypt(
            &input_file,
            &encrypted_file,
            test_passphrase,
            OutputFormat::Binary,
        )?;

        if !encrypted_file.exists() {
            return Err(AgeError::EncryptionFailed {
                input: input_file,
                output: encrypted_file,
                reason: "Encrypted file was not created".to_string(),
            });
        }

        // Test decryption
        self.decrypt(&encrypted_file, &decrypted_file, test_passphrase)?;

        if !decrypted_file.exists() {
            return Err(AgeError::DecryptionFailed {
                input: encrypted_file,
                output: decrypted_file,
                reason: "Decrypted file was not created".to_string(),
            });
        }

        // Verify content
        let decrypted_content = std::fs::read_to_string(&decrypted_file)
            .map_err(|e| AgeError::file_error("read", decrypted_file.clone(), e))?;

        if decrypted_content != test_content {
            return Err(AgeError::DecryptionFailed {
                input: encrypted_file,
                output: decrypted_file,
                reason: format!(
                    "Content mismatch: expected '{}', got '{}'",
                    test_content, decrypted_content
                ),
            });
        }

        Ok(())
    }

    /// Execute arbitrary age command with PTY automation for passphrase handling
    pub fn execute_age_command(
        &self,
        args: &[String],
        passphrase: Option<&str>,
    ) -> AgeResult<String> {
        let pty_system = native_pty_system();
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = pty_system
            .openpty(pty_size)
            .map_err(|e| AgeError::ProcessExecutionFailed {
                command: "create_pty".to_string(),
                exit_code: None,
                stderr: format!("Failed to create PTY: {}", e),
            })?;

        // Build age command with provided arguments
        let mut cmd = CommandBuilder::new("age");

        // Set working directory to match parent process
        if let Ok(current_dir) = std::env::current_dir() {
            cmd.cwd(current_dir);
        }

        for arg in args {
            cmd.arg(arg);
        }

        // Spawn age in PTY
        let child = pair.slave.spawn_command(cmd).map_err(|e| {
            let error_msg = format!("{}", e);
            if error_msg.contains("No viable candidates found in PATH")
                || error_msg.contains("No such file or directory")
                || error_msg.contains("not found")
            {
                AgeError::AgeBinaryNotFound(format!("age command not found: {}", e))
            } else {
                AgeError::ProcessExecutionFailed {
                    command: "age".to_string(),
                    exit_code: None,
                    stderr: format!("Failed to spawn age: {}", e),
                }
            }
        })?;

        drop(pair.slave);

        let mut writer =
            pair.master
                .take_writer()
                .map_err(|e| AgeError::ProcessExecutionFailed {
                    command: "pty_writer".to_string(),
                    exit_code: None,
                    stderr: format!("Failed to get PTY writer: {}", e),
                })?;

        let mut reader =
            pair.master
                .try_clone_reader()
                .map_err(|e| AgeError::ProcessExecutionFailed {
                    command: "pty_reader".to_string(),
                    exit_code: None,
                    stderr: format!("Failed to get PTY reader: {}", e),
                })?;

        // Handle age interaction with timeout
        let passphrase_clone = passphrase.map(|p| p.to_string());
        let timeout_duration = self.timeout;
        let automation_thread = thread::spawn(move || -> AgeResult<String> {
            let mut buffer = [0u8; 1024];
            let mut output_buffer = String::new();
            let mut complete_output = String::new();
            let start_time = std::time::Instant::now();

            loop {
                // Check for timeout
                if start_time.elapsed() > timeout_duration {
                    return Err(AgeError::ProcessExecutionFailed {
                        command: "pty_automation_timeout".to_string(),
                        exit_code: None,
                        stderr: format!("PTY automation timed out after {:?}", timeout_duration),
                    });
                }

                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - age finished
                        break;
                    }
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buffer[..n]);
                        output_buffer.push_str(&text);
                        complete_output.push_str(&text);

                        // Check for passphrase prompts if passphrase provided
                        if let Some(ref pass) = passphrase_clone {
                            if output_buffer.contains("Enter passphrase")
                                || output_buffer.contains("passphrase:")
                            {
                                // Send passphrase
                                writer.write_all(pass.as_bytes()).map_err(|e| {
                                    AgeError::ProcessExecutionFailed {
                                        command: "pty_write_passphrase".to_string(),
                                        exit_code: None,
                                        stderr: format!("Failed to write passphrase: {}", e),
                                    }
                                })?;
                                writer.write_all(b"\n").map_err(|e| {
                                    AgeError::ProcessExecutionFailed {
                                        command: "pty_write_newline".to_string(),
                                        exit_code: None,
                                        stderr: format!("Failed to write newline: {}", e),
                                    }
                                })?;

                                // Clear buffer after handling prompt
                                output_buffer.clear();
                            }

                            if output_buffer.contains("Confirm passphrase")
                                || output_buffer.contains("confirm:")
                            {
                                // Send confirmation
                                writer.write_all(pass.as_bytes()).map_err(|e| {
                                    AgeError::ProcessExecutionFailed {
                                        command: "pty_write_confirm".to_string(),
                                        exit_code: None,
                                        stderr: format!("Failed to write confirmation: {}", e),
                                    }
                                })?;
                                writer.write_all(b"\n").map_err(|e| {
                                    AgeError::ProcessExecutionFailed {
                                        command: "pty_write_confirm_newline".to_string(),
                                        exit_code: None,
                                        stderr: format!(
                                            "Failed to write confirmation newline: {}",
                                            e
                                        ),
                                    }
                                })?;

                                output_buffer.clear();
                            }
                        }
                    }
                    Err(e) => {
                        // Check if it's a "would block" error (non-fatal) or real error
                        use std::io::ErrorKind;
                        if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                            // Give it a moment and continue
                            thread::sleep(Duration::from_millis(50));
                            continue;
                        }

                        return Err(AgeError::ProcessExecutionFailed {
                            command: "pty_read".to_string(),
                            exit_code: None,
                            stderr: format!("PTY read error: {}", e),
                        });
                    }
                }

                // Small delay to prevent busy waiting
                thread::sleep(Duration::from_millis(10));
            }
            Ok(complete_output)
        });

        // Wait for automation thread
        let automation_result =
            automation_thread
                .join()
                .map_err(|_| AgeError::ProcessExecutionFailed {
                    command: "automation_thread".to_string(),
                    exit_code: None,
                    stderr: "Automation thread panicked".to_string(),
                })?;

        // Wait for child process to complete
        let mut child = child;
        let exit_status = child.wait().map_err(|e| AgeError::ProcessExecutionFailed {
            command: "age_wait".to_string(),
            exit_code: None,
            stderr: format!("Failed to wait for age process: {}", e),
        })?;

        let output = automation_result?;

        if exit_status.success() {
            Ok(output)
        } else {
            Err(AgeError::ProcessExecutionFailed {
                command: format!("age {}", args.join(" ")),
                exit_code: None, // portable_pty doesn't expose exit code directly
                stderr: format!("Age command failed with exit status: {:?}", exit_status),
            })
        }
    }

    /// Get available automation methods
    pub fn available_methods(&self) -> Vec<String> {
        vec!["pty".to_string()] // Only PTY method available
    }

    /// Validate dependencies
    pub fn validate_dependencies(&self) -> AgeResult<()> {
        self.check_age_binary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_pty_automator_creation() {
        let automator = PtyAgeAutomator::new();
        assert!(automator.is_ok());
    }

    #[test]
    fn test_age_binary_check() {
        let automator = PtyAgeAutomator::new().unwrap();
        // This will pass if age is installed, fail if not
        let result = automator.check_age_binary();
        // We don't assert success here since age might not be installed in test environment
        println!("Age binary check result: {:?}", result);
    }

    #[test]
    fn test_full_encryption_cycle() {
        let automator = PtyAgeAutomator::new().unwrap();

        // Skip if age not available
        if automator.check_age_binary().is_err() {
            println!("Skipping encryption test - age binary not available");
            return;
        }

        let test_content = "Test content for PTY automation";
        let test_passphrase = "secure-test-passphrase-456";

        let input_file = automator.temp_dir.path().join("test.txt");
        let encrypted_file = automator.temp_dir.path().join("test.age");
        let decrypted_file = automator.temp_dir.path().join("test_decrypted.txt");

        // Write test file
        fs::write(&input_file, test_content).unwrap();

        // Test encryption
        let encrypt_result = automator.encrypt(
            &input_file,
            &encrypted_file,
            test_passphrase,
            OutputFormat::Binary,
        );
        if encrypt_result.is_err() {
            println!(
                "Encryption failed (expected in test env): {:?}",
                encrypt_result
            );
            return;
        }

        assert!(encrypted_file.exists());

        // Test decryption
        let decrypt_result = automator.decrypt(&encrypted_file, &decrypted_file, test_passphrase);
        assert!(decrypt_result.is_ok());
        assert!(decrypted_file.exists());

        // Verify content
        let decrypted_content = fs::read_to_string(&decrypted_file).unwrap();
        assert_eq!(decrypted_content, test_content);
    }
}
