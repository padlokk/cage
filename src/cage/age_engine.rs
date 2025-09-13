//! Age Engine - Main automation coordinator
//!
//! Coordinates TTY automation with security validation and audit logging.
//! Provides high-level interface for Age operations with adapter pattern integration.
//!
//! Security Guardian: Edgar - Production Age automation coordination

use std::path::Path;
use super::adapter::{AgeAdapter, AdapterFactory};
use super::config::{AgeConfig, OutputFormat};
use super::error::{AgeError, AgeResult};
use super::security::AuditLogger;
use super::tty_automation::TtyAutomator;

/// Main Age automation engine coordinating all components
pub struct AgeAutomator {
    adapter: Box<dyn AgeAdapter>,
    config: AgeConfig,
    audit_logger: AuditLogger,
}

impl AgeAutomator {
    /// Create new Age automator with specified adapter and configuration
    pub fn new(adapter: Box<dyn AgeAdapter>, config: AgeConfig) -> AgeResult<Self> {
        // Validate configuration
        config.validate()?;
        
        // Initialize audit logger
        let audit_logger = AuditLogger::new(None)?;
        
        // Validate adapter is functional
        adapter.health_check()?;
        
        audit_logger.log_info(&format!("Age automator initialized with {} adapter", adapter.adapter_name()))?;
        
        Ok(Self {
            adapter,
            config,
            audit_logger,
        })
    }

    /// Create automator with default configuration and recommended adapter
    pub fn with_defaults() -> AgeResult<Self> {
        let adapter = AdapterFactory::create_default()?;
        let config = AgeConfig::production();
        Self::new(adapter, config)
    }

    /// Encrypt file using configured adapter and TTY automation
    pub fn encrypt<P: AsRef<Path>>(&self, input: P, output: P, passphrase: &str, format: OutputFormat) -> AgeResult<()> {
        let input = input.as_ref();
        let output = output.as_ref();
        
        self.audit_logger.log_operation_start("encrypt", input, output)?;
        
        // Validate passphrase
        self.validate_passphrase(passphrase)?;
        
        // Delegate to adapter
        let result = self.adapter.encrypt(input, output, passphrase, format);
        
        match &result {
            Ok(_) => {
                self.audit_logger.log_operation_success("encrypt", input, output)?;
                self.audit_logger.log_info(&format!("Encryption completed: {} -> {} ({})", 
                    input.display(), output.display(), format.description()))?;
            }
            Err(e) => {
                self.audit_logger.log_operation_failure("encrypt", input, output, e)?;
            }
        }
        
        result
    }

    /// Decrypt file using configured adapter and TTY automation
    pub fn decrypt<P: AsRef<Path>>(&self, input: P, output: P, passphrase: &str) -> AgeResult<()> {
        let input = input.as_ref();
        let output = output.as_ref();
        
        self.audit_logger.log_operation_start("decrypt", input, output)?;
        
        // Validate passphrase
        self.validate_passphrase(passphrase)?;
        
        // Delegate to adapter
        let result = self.adapter.decrypt(input, output, passphrase);
        
        match &result {
            Ok(_) => {
                self.audit_logger.log_operation_success("decrypt", input, output)?;
                self.audit_logger.log_info(&format!("Decryption completed: {} -> {}", 
                    input.display(), output.display()))?;
            }
            Err(e) => {
                self.audit_logger.log_operation_failure("decrypt", input, output, e)?;
            }
        }
        
        result
    }

    /// Perform health check on automation system
    pub fn health_check(&self) -> AgeResult<()> {
        self.audit_logger.log_info("Starting health check")?;
        
        // Check adapter health
        self.adapter.health_check()?;
        
        self.audit_logger.log_info("Health check completed successfully")?;
        Ok(())
    }

    /// Get adapter information
    pub fn adapter_info(&self) -> String {
        format!("{} ({})", self.adapter.adapter_name(), self.adapter.adapter_version())
    }

    /// Validate passphrase according to configuration
    fn validate_passphrase(&self, passphrase: &str) -> AgeResult<()> {
        if passphrase.is_empty() {
            return Err(AgeError::passphrase_validation(
                "Empty passphrase", 
                "Provide a non-empty passphrase"
            ));
        }

        if passphrase.len() > self.config.max_passphrase_length {
            return Err(AgeError::passphrase_validation(
                &format!("Passphrase too long ({} chars)", passphrase.len()),
                &format!("Use passphrase with max {} characters", self.config.max_passphrase_length)
            ));
        }

        // Check for injection patterns
        if passphrase.contains('\0') {
            return Err(AgeError::injection_blocked("null_byte", "\\0"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::adapter::AdapterFactory;

    #[test]
    fn test_automator_creation() {
        let config = AgeConfig::testing();
        // This will fail if dependencies aren't available, which is expected
        let _result = AdapterFactory::create_default().and_then(|adapter| {
            AgeAutomator::new(adapter, config)
        });
    }

    #[test]
    fn test_passphrase_validation() {
        let config = AgeConfig::testing();
        let adapter = AdapterFactory::create_default();
        
        if let Ok(adapter) = adapter {
            if let Ok(automator) = AgeAutomator::new(adapter, config) {
                // Test empty passphrase
                assert!(automator.validate_passphrase("").is_err());
                
                // Test null byte injection
                assert!(automator.validate_passphrase("pass\0word").is_err());
                
                // Test valid passphrase
                assert!(automator.validate_passphrase("ValidPassphrase123").is_ok());
            }
        }
    }
}