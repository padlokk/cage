//! Secure Passphrase Management
//!
//! Handles secure passphrase input with various modes:
//! - Interactive terminal prompting (secure, hidden input)
//! - Stdin passphrase mode for automation
//! - Environment variable fallback
//! - Command line argument detection and warnings

use crate::cage::error::{AgeError, AgeResult};
use crate::lang::{fmt_info, fmt_warning};
use rpassword::read_password;
use rsb::visual::glyphs::glyph;
use std::io::{self, Write};

/// Passphrase input modes for different scenarios
#[derive(Debug, Clone, PartialEq)]
pub enum PassphraseMode {
    /// Interactive prompt (secure, hidden input)
    Interactive,
    /// Read from stdin (for piped automation)
    Stdin,
    /// Use environment variable
    Environment(String),
    /// Command line argument (insecure, warn user)
    CommandLine(String),
}

/// Secure passphrase manager with multiple input methods
pub struct PassphraseManager {
    tty_available: bool,
    #[allow(dead_code)]
    stdin_is_tty: bool,
}

impl Default for PassphraseManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PassphraseManager {
    /// Create new passphrase manager with TTY detection
    pub fn new() -> Self {
        Self {
            tty_available: Self::detect_tty(),
            stdin_is_tty: Self::detect_stdin_tty(),
        }
    }

    /// Detect if TTY is available for interactive prompts
    fn detect_tty() -> bool {
        // Check if we can access /dev/tty (Unix) or have interactive terminal
        #[cfg(unix)]
        {
            use std::fs::File;
            File::open("/dev/tty").is_ok()
        }
        #[cfg(windows)]
        {
            // Windows TTY detection would go here
            true // Assume available for now
        }
    }

    /// Detect if stdin is a TTY (not redirected/piped)
    fn detect_stdin_tty() -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = std::io::stdin().as_raw_fd();
            unsafe { libc::isatty(fd) == 1 }
        }
        #[cfg(windows)]
        {
            // Windows stdin TTY detection would go here
            true // Assume TTY for now
        }
    }

    /// Get passphrase securely with automatic mode detection
    pub fn get_passphrase(&self, prompt: &str, confirm: bool) -> AgeResult<String> {
        let mode = self.detect_best_mode()?;
        self.get_passphrase_with_mode(prompt, confirm, mode)
    }

    /// Get passphrase with explicit mode
    pub fn get_passphrase_with_mode(
        &self,
        prompt: &str,
        confirm: bool,
        mode: PassphraseMode,
    ) -> AgeResult<String> {
        match mode {
            PassphraseMode::Interactive => self.prompt_interactive(prompt, confirm),
            PassphraseMode::Stdin => self.read_from_stdin(),
            PassphraseMode::Environment(var) => self.read_from_env(&var),
            PassphraseMode::CommandLine(pass) => {
                self.warn_insecure_usage();
                Ok(pass)
            }
        }
    }

    /// Detect the best passphrase input mode based on environment
    fn detect_best_mode(&self) -> AgeResult<PassphraseMode> {
        // Check for explicit environment variable
        if let Ok(_pass) = std::env::var("CAGE_PASSPHRASE") {
            return Ok(PassphraseMode::Environment("CAGE_PASSPHRASE".to_string()));
        }

        // Check for stdin passphrase flag in environment
        if std::env::var("CAGE_STDIN_PASSPHRASE").is_ok() {
            return Ok(PassphraseMode::Stdin);
        }

        // Default to interactive if TTY available
        if self.tty_available {
            Ok(PassphraseMode::Interactive)
        } else {
            Err(AgeError::PassphraseError {
                message: "No TTY available for interactive input. Use CAGE_PASSPHRASE env var or --stdin-passphrase".to_string(),
            })
        }
    }

    /// Prompt for passphrase interactively with secure hidden input
    fn prompt_interactive(&self, prompt: &str, confirm: bool) -> AgeResult<String> {
        if !self.tty_available {
            return Err(AgeError::PassphraseError {
                message: "TTY not available for interactive prompt".to_string(),
            });
        }

        // Print prompt to stderr to avoid interfering with stdout
        eprint!("{} {}: ", glyph("lock"), prompt);
        io::stderr()
            .flush()
            .map_err(|e| AgeError::PassphraseError {
                message: format!("Failed to flush stderr: {}", e),
            })?;

        let passphrase = read_password().map_err(|e| AgeError::PassphraseError {
            message: format!("Failed to read passphrase: {}", e),
        })?;

        if passphrase.is_empty() {
            return Err(AgeError::PassphraseError {
                message: "Empty passphrase not allowed".to_string(),
            });
        }

        // Confirmation for critical operations
        if confirm {
            eprint!("{} Confirm {}: ", glyph("lock"), prompt);
            io::stderr()
                .flush()
                .map_err(|e| AgeError::PassphraseError {
                    message: format!("Failed to flush stderr: {}", e),
                })?;

            let confirmation = read_password().map_err(|e| AgeError::PassphraseError {
                message: format!("Failed to read confirmation: {}", e),
            })?;

            if passphrase != confirmation {
                return Err(AgeError::PassphraseError {
                    message: "Passphrases do not match".to_string(),
                });
            }
        }

        // Validate passphrase strength
        self.validate_passphrase_strength(&passphrase)?;

        Ok(passphrase)
    }

    /// Read passphrase from stdin (for scripting/automation)
    fn read_from_stdin(&self) -> AgeResult<String> {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| AgeError::PassphraseError {
                message: format!("Failed to read from stdin: {}", e),
            })?;

        let passphrase = input.trim().to_string();
        if passphrase.is_empty() {
            return Err(AgeError::PassphraseError {
                message: "Empty passphrase from stdin".to_string(),
            });
        }

        Ok(passphrase)
    }

    /// Read passphrase from environment variable
    fn read_from_env(&self, var_name: &str) -> AgeResult<String> {
        std::env::var(var_name).map_err(|_| AgeError::PassphraseError {
            message: format!("Environment variable {} not found", var_name),
        })
    }

    /// Warn about insecure command line usage
    fn warn_insecure_usage(&self) {
        eprintln!(
            "{}",
            fmt_warning("WARNING: Passphrase provided on command line!")
        );
        eprintln!("   This is insecure and visible in process list and shell history.");
        eprintln!("   Use interactive prompt or CAGE_PASSPHRASE environment variable instead.");
        eprintln!("   For automation, use --stdin-passphrase flag.");
    }

    /// Validate passphrase strength and provide recommendations
    fn validate_passphrase_strength(&self, passphrase: &str) -> AgeResult<()> {
        if passphrase.len() < 8 {
            eprintln!("{}", fmt_warning("Passphrase is less than 8 characters."));
            eprintln!("   Consider using a longer passphrase for better security.");
        }

        if passphrase.len() < 12 && !passphrase.chars().any(|c| c.is_ascii_punctuation()) {
            eprintln!(
                "{}",
                fmt_info("Tip: Consider adding special characters for stronger security.")
            );
        }

        if passphrase.to_lowercase() == passphrase {
            eprintln!(
                "{}",
                fmt_info("Tip: Mix of uppercase and lowercase letters improves security.")
            );
        }

        // Don't fail on weak passwords, just warn
        Ok(())
    }

    /// Check if passphrase was provided insecurely via command line
    pub fn detect_insecure_usage(args: &[String]) -> Option<String> {
        // Look for common passphrase patterns in command line arguments
        for (i, arg) in args.iter().enumerate() {
            if arg == "--passphrase" || arg == "-p" {
                if let Some(pass) = args.get(i + 1) {
                    return Some(pass.clone());
                }
            }
            if arg.starts_with("--passphrase=") {
                if let Some(pass) = arg.split('=').nth(1) {
                    return Some(pass.to_string());
                }
            }
        }
        None
    }

    /// Create recovery information for dangerous operations
    pub fn create_recovery_hint(&self, file_path: &str, operation: &str) -> String {
        format!(
            "# Cage Recovery Information\n\
             # File: {}\n\
             # Operation: {} at {}\n\
             # To recover: cage unlock {} <passphrase>\n\
             # Delete this file once you've verified successful operation\n",
            file_path,
            operation,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            file_path
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passphrase_manager_creation() {
        let manager = PassphraseManager::new();
        // Just test that it doesn't panic
        assert!(manager.tty_available || !manager.tty_available);
    }

    #[test]
    fn test_insecure_usage_detection() {
        let args1 = vec![
            "cage".to_string(),
            "lock".to_string(),
            "--passphrase".to_string(),
            "secret123".to_string(),
        ];
        let detected = PassphraseManager::detect_insecure_usage(&args1);
        assert_eq!(detected, Some("secret123".to_string()));

        let args2 = vec![
            "cage".to_string(),
            "lock".to_string(),
            "--passphrase=mysecret".to_string(),
        ];
        let detected = PassphraseManager::detect_insecure_usage(&args2);
        assert_eq!(detected, Some("mysecret".to_string()));

        let args3 = vec![
            "cage".to_string(),
            "lock".to_string(),
            "file.txt".to_string(),
        ];
        let detected = PassphraseManager::detect_insecure_usage(&args3);
        assert_eq!(detected, None);
    }

    #[test]
    fn test_recovery_hint_creation() {
        let manager = PassphraseManager::new();
        let hint = manager.create_recovery_hint("/path/to/file.txt", "encrypt");
        assert!(hint.contains("/path/to/file.txt"));
        assert!(hint.contains("encrypt"));
        assert!(hint.contains("cage unlock"));
    }

    #[test]
    fn test_passphrase_mode_detection() {
        let manager = PassphraseManager::new();

        // Test environment variable detection
        std::env::set_var("CAGE_PASSPHRASE", "test123");
        let mode = manager.detect_best_mode().unwrap();
        assert_eq!(
            mode,
            PassphraseMode::Environment("CAGE_PASSPHRASE".to_string())
        );
        std::env::remove_var("CAGE_PASSPHRASE");

        // Test stdin mode detection
        std::env::set_var("CAGE_STDIN_PASSPHRASE", "1");
        let mode = manager.detect_best_mode().unwrap();
        assert_eq!(mode, PassphraseMode::Stdin);
        std::env::remove_var("CAGE_STDIN_PASSPHRASE");
    }
}
