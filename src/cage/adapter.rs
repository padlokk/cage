//! Age Adapter Pattern - Clean abstraction for Age implementations
//!
//! This module provides adapter pattern for different Age backends:
//! - ShellAdapter: Uses reliable PTY automation (current implementation)
//! - RageAdapter: Future integration with rage crate (planned)
//!
//! Security Guardian: Edgar - Adapter pattern for clean backend abstraction

use std::path::Path;
use super::error::{AgeError, AgeResult};
use super::config::OutputFormat;

/// Core Age operations interface that all adapters must implement
pub trait AgeAdapter {
    /// Encrypt a file with the given passphrase
    fn encrypt(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()>;
    
    /// Decrypt a file with the given passphrase
    fn decrypt(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()>;
    
    /// Validate adapter is functional and dependencies are available
    fn health_check(&self) -> AgeResult<()>;
    
    /// Get adapter name for logging and diagnostics
    fn adapter_name(&self) -> &'static str;
    
    /// Get adapter version information
    fn adapter_version(&self) -> String;
    
    /// Clone this adapter into a boxed trait object
    fn clone_box(&self) -> Box<dyn AgeAdapter>;
}

/// Shell-based Age adapter using PTY automation methods
pub struct ShellAdapter {
    pty_automator: super::pty_wrap::PtyAgeAutomator,
    audit_logger: super::security::AuditLogger,
}

impl ShellAdapter {
    /// Create new ShellAdapter with PTY automation
    pub fn new() -> AgeResult<Self> {
        let pty_automator = super::pty_wrap::PtyAgeAutomator::new()?;
        let audit_logger = super::security::AuditLogger::new(None)?;

        Ok(Self {
            pty_automator,
            audit_logger,
        })
    }
    
    /// Validate PTY dependencies (age binary)
    pub fn validate_dependencies(&self) -> AgeResult<()> {
        self.pty_automator.validate_dependencies()
    }

    /// Get available automation methods on this system
    pub fn available_methods(&self) -> Vec<String> {
        self.pty_automator.available_methods()
    }
}

impl AgeAdapter for ShellAdapter {
    fn encrypt(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()> {
        self.audit_logger.log_operation_start("encrypt", input, output)?;

        let result = self.pty_automator.encrypt(input, output, passphrase, format);

        match &result {
            Ok(_) => self.audit_logger.log_operation_success("encrypt", input, output)?,
            Err(e) => self.audit_logger.log_operation_failure("encrypt", input, output, e)?,
        }

        result
    }
    
    fn decrypt(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()> {
        self.audit_logger.log_operation_start("decrypt", input, output)?;

        let result = self.pty_automator.decrypt(input, output, passphrase);

        match &result {
            Ok(_) => self.audit_logger.log_operation_success("decrypt", input, output)?,
            Err(e) => self.audit_logger.log_operation_failure("decrypt", input, output, e)?,
        }

        result
    }
    
    fn health_check(&self) -> AgeResult<()> {
        // Check Age binary availability
        self.pty_automator.check_age_binary()?;

        // Perform full encrypt/decrypt cycle test with PTY
        self.pty_automator.perform_health_check()?;

        self.audit_logger.log_health_check("passed")?;
        Ok(())
    }
    
    fn adapter_name(&self) -> &'static str {
        "PtyAdapter"
    }

    fn adapter_version(&self) -> String {
        format!("pty-v{}-portable-pty", super::VERSION)
    }
    
    fn clone_box(&self) -> Box<dyn AgeAdapter> {
        Box::new(ShellAdapter {
            pty_automator: super::pty_wrap::PtyAgeAutomator::new().unwrap(),
            audit_logger: super::security::AuditLogger::new(None).unwrap(),
        })
    }
}

/// Future Rage crate adapter (not yet implemented)
#[derive(Debug)]
#[allow(dead_code)]
pub struct RageAdapter {
    // Future: rage crate integration
    // This provides the same interface but uses rage library directly
}

#[allow(dead_code)]
impl RageAdapter {
    /// Create new RageAdapter (future implementation)
    pub fn new() -> AgeResult<Self> {
        Err(AgeError::AdapterNotImplemented("RageAdapter not yet implemented".to_string()))
    }
}

impl AgeAdapter for RageAdapter {
    fn encrypt(&self, _input: &Path, _output: &Path, _passphrase: &str, _format: OutputFormat) -> AgeResult<()> {
        Err(AgeError::AdapterNotImplemented("RageAdapter encrypt not implemented".to_string()))
    }
    
    fn decrypt(&self, _input: &Path, _output: &Path, _passphrase: &str) -> AgeResult<()> {
        Err(AgeError::AdapterNotImplemented("RageAdapter decrypt not implemented".to_string()))
    }
    
    fn health_check(&self) -> AgeResult<()> {
        Err(AgeError::AdapterNotImplemented("RageAdapter health_check not implemented".to_string()))
    }
    
    fn adapter_name(&self) -> &'static str {
        "RageAdapter"
    }
    
    fn adapter_version(&self) -> String {
        "rage-v0.0.0-future".to_string()
    }
    
    fn clone_box(&self) -> Box<dyn AgeAdapter> {
        Box::new(RageAdapter {})
    }
}

/// Adapter factory for creating the appropriate Age adapter
pub struct AdapterFactory;

impl AdapterFactory {
    /// Create the default adapter (currently ShellAdapter)
    pub fn create_default() -> AgeResult<Box<dyn AgeAdapter>> {
        Ok(Box::new(ShellAdapter::new()?))
    }
    
    /// Create specific adapter by name
    pub fn create_adapter(adapter_type: &str) -> AgeResult<Box<dyn AgeAdapter>> {
        match adapter_type {
            "shell" => Ok(Box::new(ShellAdapter::new()?)),
            "rage" => Ok(Box::new(RageAdapter::new()?)),
            _ => Err(AgeError::InvalidAdapter(format!("Unknown adapter type: {}", adapter_type))),
        }
    }
    
    /// List available adapters
    pub fn available_adapters() -> Vec<&'static str> {
        vec!["shell", "rage"]
    }
    
    /// Get recommended adapter for current environment
    pub fn recommended_adapter() -> &'static str {
        // For now, always recommend shell adapter with proven TTY automation
        "shell"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_adapter_factory() {
        let adapters = AdapterFactory::available_adapters();
        assert!(adapters.contains(&"shell"));
        assert!(adapters.contains(&"rage"));
        
        assert_eq!(AdapterFactory::recommended_adapter(), "shell");
    }
    
    #[test]
    fn test_shell_adapter_creation() {
        // This test will fail if Age is not installed, which is expected
        match ShellAdapter::new() {
            Ok(adapter) => {
                assert_eq!(adapter.adapter_name(), "ShellAdapter");
                assert!(adapter.adapter_version().contains("shell-v"));
            }
            Err(_) => {
                // Expected if Age not installed in test environment
                println!("Shell adapter creation failed (expected if Age not installed)");
            }
        }
    }
    
    #[test]
    fn test_rage_adapter_not_implemented() {
        let result = RageAdapter::new();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgeError::AdapterNotImplemented(_)));
    }
}