//! Age Automation Configuration
//!
//! Configuration management for Age automation module, including output formats,
//! TTY automation methods, and security settings.
//!
//! Security Guardian: Edgar - Production-ready configuration management

use std::time::Duration;
use super::error::{AgeError, AgeResult};

/// Output format for Age encryption
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Binary output (.age files) - default and most efficient
    Binary,
    /// ASCII armor output (-a flag) - text-safe for various environments
    AsciiArmor,
}

impl OutputFormat {
    /// Get the Age command line flag for this format
    pub fn age_flag(&self) -> Option<&'static str> {
        match self {
            OutputFormat::Binary => None,
            OutputFormat::AsciiArmor => Some("-a"),
        }
    }
    
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            OutputFormat::Binary => "Binary format (efficient, smallest size)",
            OutputFormat::AsciiArmor => "ASCII armor (text-safe, larger size)",
        }
    }
    
    /// Detect format from file extension or content
    pub fn detect_from_path(path: &std::path::Path) -> OutputFormat {
        if let Some(ext) = path.extension() {
            if ext == "txt" || ext == "asc" || ext == "armor" {
                return OutputFormat::AsciiArmor;
            }
        }
        // Default to binary format
        OutputFormat::Binary
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Binary
    }
}

/// TTY automation method preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtyMethod {
    /// Use script command for TTY emulation (fastest, most compatible)
    Script,
    /// Use expect for pattern-based automation (most reliable)
    Expect,
    /// Automatically choose best available method
    Auto,
}

impl TtyMethod {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            TtyMethod::Script => "Script command TTY emulation (fast, compatible)",
            TtyMethod::Expect => "Expect pattern automation (reliable, requires expect)",
            TtyMethod::Auto => "Automatic method selection with fallback",
        }
    }
    
    /// Get dependency requirements
    pub fn dependencies(&self) -> Vec<&'static str> {
        match self {
            TtyMethod::Script => vec!["script", "util-linux"],
            TtyMethod::Expect => vec!["expect"],
            TtyMethod::Auto => vec!["script OR expect"],
        }
    }
}

impl Default for TtyMethod {
    fn default() -> Self {
        TtyMethod::Auto
    }
}

/// Security validation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Basic security validation
    Basic,
    /// Standard security validation (recommended)
    Standard,
    /// Paranoid security validation (maximum security)
    Paranoid,
}

impl SecurityLevel {
    /// Get validation timeout based on security level
    pub fn validation_timeout(&self) -> Duration {
        match self {
            SecurityLevel::Basic => Duration::from_secs(5),
            SecurityLevel::Standard => Duration::from_secs(10),
            SecurityLevel::Paranoid => Duration::from_secs(30),
        }
    }
}

impl Default for SecurityLevel {
    fn default() -> Self {
        SecurityLevel::Standard
    }
}

/// Age automation configuration
#[derive(Debug, Clone)]
pub struct AgeConfig {
    /// Preferred output format
    pub output_format: OutputFormat,
    
    /// Preferred TTY automation method
    pub tty_method: TtyMethod,
    
    /// Security validation level
    pub security_level: SecurityLevel,
    
    /// Maximum passphrase length (characters)
    pub max_passphrase_length: usize,
    
    /// Operation timeout (seconds)
    pub operation_timeout: Duration,
    
    /// Path to Age binary (None for auto-detection)
    pub age_binary_path: Option<String>,
    
    /// Path to script binary (None for auto-detection)
    pub script_binary_path: Option<String>,
    
    /// Path to expect binary (None for auto-detection)
    pub expect_binary_path: Option<String>,
    
    /// Enable comprehensive audit logging
    pub audit_logging: bool,
    
    /// Audit log file path (None for stderr)
    pub audit_log_path: Option<String>,
    
    /// Enable security validation
    pub security_validation: bool,
    
    /// Enable health checks before operations
    pub health_checks: bool,
    
    /// Maximum number of retry attempts
    pub max_retries: u32,
    
    /// Delay between retry attempts
    pub retry_delay: Duration,
    
    /// Enable temporary file shredding
    pub secure_deletion: bool,
    
    /// Temporary directory override (None for system default)
    pub temp_dir_override: Option<String>,
}

impl AgeConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create configuration for production environment
    pub fn production() -> Self {
        Self {
            security_level: SecurityLevel::Standard,
            audit_logging: true,
            security_validation: true,
            health_checks: true,
            secure_deletion: true,
            max_retries: 3,
            ..Default::default()
        }
    }
    
    /// Create configuration for development environment
    pub fn development() -> Self {
        Self {
            security_level: SecurityLevel::Basic,
            audit_logging: false,
            security_validation: true,
            health_checks: false,
            secure_deletion: false,
            max_retries: 1,
            ..Default::default()
        }
    }
    
    /// Create configuration for testing environment
    pub fn testing() -> Self {
        Self {
            security_level: SecurityLevel::Paranoid,
            audit_logging: true,
            security_validation: true,
            health_checks: true,
            secure_deletion: true,
            max_retries: 0, // No retries in tests for fast failure
            operation_timeout: Duration::from_secs(10),
            ..Default::default()
        }
    }
    
    /// Validate configuration settings
    pub fn validate(&self) -> AgeResult<()> {
        // Validate passphrase length
        if self.max_passphrase_length == 0 {
            return Err(AgeError::ConfigurationError {
                parameter: "max_passphrase_length".to_string(),
                value: "0".to_string(),
                reason: "Must be greater than 0".to_string(),
            });
        }
        
        if self.max_passphrase_length > 10_000 {
            return Err(AgeError::ConfigurationError {
                parameter: "max_passphrase_length".to_string(),
                value: self.max_passphrase_length.to_string(),
                reason: "Unreasonably large, maximum 10,000 characters".to_string(),
            });
        }
        
        // Validate timeout
        if self.operation_timeout.as_secs() == 0 {
            return Err(AgeError::ConfigurationError {
                parameter: "operation_timeout".to_string(),
                value: "0".to_string(),
                reason: "Must be greater than 0 seconds".to_string(),
            });
        }
        
        if self.operation_timeout.as_secs() > 3600 {
            return Err(AgeError::ConfigurationError {
                parameter: "operation_timeout".to_string(),
                value: format!("{} seconds", self.operation_timeout.as_secs()),
                reason: "Unreasonably large, maximum 1 hour".to_string(),
            });
        }
        
        // Validate retry settings
        if self.max_retries > 10 {
            return Err(AgeError::ConfigurationError {
                parameter: "max_retries".to_string(),
                value: self.max_retries.to_string(),
                reason: "Maximum 10 retries allowed".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Set output format
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }
    
    /// Set TTY method
    pub fn with_tty_method(mut self, method: TtyMethod) -> Self {
        self.tty_method = method;
        self
    }
    
    /// Set security level
    pub fn with_security_level(mut self, level: SecurityLevel) -> Self {
        self.security_level = level;
        self
    }
    
    /// Set operation timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.operation_timeout = timeout;
        self
    }
    
    /// Enable or disable audit logging
    pub fn with_audit_logging(mut self, enabled: bool) -> Self {
        self.audit_logging = enabled;
        self
    }
    
    /// Set audit log file path
    pub fn with_audit_log_path<P: Into<String>>(mut self, path: P) -> Self {
        self.audit_log_path = Some(path.into());
        self
    }
    
    /// Set Age binary path
    pub fn with_age_binary<P: Into<String>>(mut self, path: P) -> Self {
        self.age_binary_path = Some(path.into());
        self
    }
}

impl Default for AgeConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::default(),
            tty_method: TtyMethod::default(),
            security_level: SecurityLevel::default(),
            max_passphrase_length: 1024,
            operation_timeout: Duration::from_secs(120),
            age_binary_path: None,
            script_binary_path: None,
            expect_binary_path: None,
            audit_logging: true,
            audit_log_path: None,
            security_validation: true,
            health_checks: true,
            max_retries: 2,
            retry_delay: Duration::from_secs(1),
            secure_deletion: true,
            temp_dir_override: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    
    #[test]
    fn test_output_format() {
        assert_eq!(OutputFormat::Binary.age_flag(), None);
        assert_eq!(OutputFormat::AsciiArmor.age_flag(), Some("-a"));
        
        assert_eq!(OutputFormat::default(), OutputFormat::Binary);
    }
    
    #[test]
    fn test_output_format_detection() {
        assert_eq!(
            OutputFormat::detect_from_path(Path::new("test.age")),
            OutputFormat::Binary
        );
        assert_eq!(
            OutputFormat::detect_from_path(Path::new("test.txt")),
            OutputFormat::AsciiArmor
        );
        assert_eq!(
            OutputFormat::detect_from_path(Path::new("test.asc")),
            OutputFormat::AsciiArmor
        );
    }
    
    #[test]
    fn test_tty_method() {
        assert_eq!(TtyMethod::default(), TtyMethod::Auto);
        assert!(TtyMethod::Script.dependencies().contains(&"script"));
        assert!(TtyMethod::Expect.dependencies().contains(&"expect"));
    }
    
    #[test]
    fn test_config_validation() {
        let config = AgeConfig::default();
        assert!(config.validate().is_ok());
        
        let bad_config = AgeConfig {
            max_passphrase_length: 0,
            ..Default::default()
        };
        assert!(bad_config.validate().is_err());
    }
    
    #[test]
    fn test_config_builders() {
        let config = AgeConfig::production()
            .with_output_format(OutputFormat::AsciiArmor)
            .with_tty_method(TtyMethod::Script)
            .with_timeout(Duration::from_secs(60));
        
        assert_eq!(config.output_format, OutputFormat::AsciiArmor);
        assert_eq!(config.tty_method, TtyMethod::Script);
        assert_eq!(config.operation_timeout, Duration::from_secs(60));
        assert!(config.audit_logging);
    }
    
    #[test]
    fn test_environment_configs() {
        let prod = AgeConfig::production();
        assert!(prod.audit_logging);
        assert!(prod.security_validation);
        assert_eq!(prod.max_retries, 3);
        
        let dev = AgeConfig::development();
        assert!(!dev.audit_logging);
        assert_eq!(dev.max_retries, 1);
        
        let test = AgeConfig::testing();
        assert_eq!(test.max_retries, 0);
        assert_eq!(test.security_level, SecurityLevel::Paranoid);
    }
}