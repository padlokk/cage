//! Age Automation Configuration
//!
//! Configuration management for Age automation module, including output formats,
//! TTY automation methods, and security settings.
//!
//! Security Guardian: Edgar - Production-ready configuration management

use super::error::{AgeError, AgeResult};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

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

#[derive(Debug, Clone)]
pub enum RetentionPolicyConfig {
    KeepAll,
    KeepDays(u32),
    KeepLast(usize),
    KeepLastAndDays { last: usize, days: u32 },
}

impl Default for RetentionPolicyConfig {
    fn default() -> Self {
        RetentionPolicyConfig::KeepLast(3)
    }
}

/// Age automation configuration
#[derive(Debug, Clone)]
pub struct AgeConfig {
    /// Path where the configuration was loaded from (if any)
    pub source_path: Option<PathBuf>,

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

    /// File extension for encrypted files (default: "cage")
    pub encrypted_file_extension: String,

    /// Delete backups after successful operations
    pub backup_cleanup: bool,

    /// Optional default backup directory
    pub backup_directory: Option<String>,

    /// Retention policy applied to backups
    pub backup_retention: RetentionPolicyConfig,

    /// Default streaming strategy (temp, pipe, auto)
    pub streaming_strategy: Option<String>,
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

        if let Some(strategy) = &self.streaming_strategy {
            match strategy.as_str() {
                "temp" | "pipe" | "auto" => {}
                other => {
                    return Err(AgeError::ConfigurationError {
                        parameter: "streaming_strategy".to_string(),
                        value: other.to_string(),
                        reason: "Valid values: temp, pipe, auto".to_string(),
                    });
                }
            }
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

    /// Set encrypted file extension
    pub fn with_extension<S: Into<String>>(mut self, extension: S) -> Self {
        self.encrypted_file_extension = extension.into();
        self
    }

    /// Create configuration for padlock integration
    pub fn for_padlock() -> Self {
        Self {
            encrypted_file_extension: "padlock".to_string(),
            audit_logging: true,
            security_validation: true,
            health_checks: true,
            secure_deletion: true,
            ..Default::default()
        }
    }

    /// Get file extension with dot prefix
    pub fn extension_with_dot(&self) -> String {
        if self.encrypted_file_extension.starts_with('.') {
            self.encrypted_file_extension.clone()
        } else {
            format!(".{}", self.encrypted_file_extension)
        }
    }

    /// Check if a file has the configured encrypted extension
    pub fn is_encrypted_file(&self, path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return ext_str == self.encrypted_file_extension;
            }
        }
        false
    }

    pub fn load_default() -> AgeResult<Self> {
        for path in default_config_paths() {
            if path.exists() {
                return Self::load_from_path(&path);
            }
        }

        Ok(AgeConfig::default())
    }

    /// Get the paths that will be checked for configuration files
    pub fn get_config_search_paths() -> Vec<PathBuf> {
        default_config_paths()
    }

    /// Get a formatted string showing the configuration layers
    pub fn format_layers(&self) -> String {
        let paths = Self::get_config_search_paths();
        let mut layers = String::new();

        layers.push_str("Configuration search order (highest priority first):\n");
        for (i, path) in paths.iter().enumerate() {
            let status = if path.exists() {
                if self.source_path.as_ref() == Some(path) {
                    " [LOADED]"
                } else {
                    " [exists]"
                }
            } else {
                " [not found]"
            };

            let source = if i == 0 && std::env::var("CAGE_CONFIG").is_ok() {
                "env:CAGE_CONFIG"
            } else if path.to_string_lossy().contains("XDG_CONFIG_HOME")
                || path.to_string_lossy().contains(".config")
            {
                "XDG"
            } else if path.to_string_lossy().contains("cage.toml") {
                "local"
            } else {
                "unknown"
            };

            layers.push_str(&format!(
                "  {}. {} -> {}{}\n",
                i + 1,
                source,
                path.display(),
                status
            ));
        }

        if self.source_path.is_none() {
            layers.push_str("\nUsing default configuration (no config file found)");
        }

        layers
    }

    fn load_from_path(path: &Path) -> AgeResult<Self> {
        let contents = fs::read_to_string(path).map_err(|e| AgeError::ConfigurationError {
            parameter: "config_file".to_string(),
            value: path.display().to_string(),
            reason: e.to_string(),
        })?;

        let file: AgeConfigFile =
            toml::from_str(&contents).map_err(|e| AgeError::ConfigurationError {
                parameter: "config_file".to_string(),
                value: path.display().to_string(),
                reason: e.to_string(),
            })?;

        let mut config = AgeConfig::default();
        config.source_path = Some(path.to_path_buf());

        if let Some(backup_cfg) = file.backup {
            if let Some(cleanup) = backup_cfg.cleanup_on_success {
                config.backup_cleanup = cleanup;
            }
            if let Some(dir) = backup_cfg.directory {
                config.backup_directory = Some(dir);
            }
            if let Some(retention) = backup_cfg.retention {
                config.backup_retention = parse_retention_policy(&retention)?;
            }
        }

        if let Some(streaming_cfg) = file.streaming {
            if let Some(strategy) = streaming_cfg.strategy {
                config.streaming_strategy = Some(strategy);
            }
        }

        config.validate()?;
        Ok(config)
    }
}

impl Default for AgeConfig {
    fn default() -> Self {
        Self {
            source_path: None,
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
            encrypted_file_extension: "cage".to_string(),
            backup_cleanup: true,
            backup_directory: None,
            backup_retention: RetentionPolicyConfig::default(),
            streaming_strategy: None,
        }
    }
}

#[derive(Default, Deserialize)]
struct AgeConfigFile {
    backup: Option<BackupConfigSection>,
    streaming: Option<StreamingConfigSection>,
}

#[derive(Default, Deserialize)]
struct BackupConfigSection {
    cleanup_on_success: Option<bool>,
    directory: Option<String>,
    retention: Option<String>,
}

#[derive(Default, Deserialize)]
struct StreamingConfigSection {
    strategy: Option<String>,
}

fn default_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(custom) = env::var("CAGE_CONFIG") {
        if !custom.is_empty() {
            paths.push(PathBuf::from(custom));
        }
    }

    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(xdg);
        path.push("cage/config.toml");
        paths.push(path);
    } else if let Ok(home) = env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".config/cage/config.toml");
        paths.push(path);
    }

    paths.push(PathBuf::from("cage.toml"));

    paths
}

fn parse_retention_policy(value: &str) -> AgeResult<RetentionPolicyConfig> {
    let trimmed = value.trim();
    let lower = trimmed.to_lowercase();

    if lower == "keep_all" {
        return Ok(RetentionPolicyConfig::KeepAll);
    }

    if let Some(rest) = lower.strip_prefix("keep_days:") {
        let days: u32 = rest
            .trim()
            .parse()
            .map_err(|_| AgeError::ConfigurationError {
                parameter: "backup.retention".to_string(),
                value: trimmed.to_string(),
                reason: "Expected keep_days:<u32>".to_string(),
            })?;
        return Ok(RetentionPolicyConfig::KeepDays(days));
    }

    if let Some(rest) = lower.strip_prefix("keep_last:") {
        let count: usize = rest
            .trim()
            .parse()
            .map_err(|_| AgeError::ConfigurationError {
                parameter: "backup.retention".to_string(),
                value: trimmed.to_string(),
                reason: "Expected keep_last:<usize>".to_string(),
            })?;
        return Ok(RetentionPolicyConfig::KeepLast(count));
    }

    if let Some(rest) = lower.strip_prefix("keep_last_and_days:") {
        let parts: Vec<&str> = rest.split(',').collect();
        if parts.len() != 2 {
            return Err(AgeError::ConfigurationError {
                parameter: "backup.retention".to_string(),
                value: trimmed.to_string(),
                reason: "Expected keep_last_and_days:<usize>,<u32>".to_string(),
            });
        }
        let last: usize = parts[0]
            .trim()
            .parse()
            .map_err(|_| AgeError::ConfigurationError {
                parameter: "backup.retention".to_string(),
                value: trimmed.to_string(),
                reason: "Invalid last parameter".to_string(),
            })?;
        let days: u32 = parts[1]
            .trim()
            .parse()
            .map_err(|_| AgeError::ConfigurationError {
                parameter: "backup.retention".to_string(),
                value: trimmed.to_string(),
                reason: "Invalid days parameter".to_string(),
            })?;
        return Ok(RetentionPolicyConfig::KeepLastAndDays { last, days });
    }

    Err(AgeError::ConfigurationError {
        parameter: "backup.retention".to_string(),
        value: trimmed.to_string(),
        reason: "Valid values: keep_all, keep_days:<u32>, keep_last:<usize>, keep_last_and_days:<usize>,<u32>".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

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

    #[test]
    fn test_parse_retention_policy_strings() {
        assert!(matches!(
            parse_retention_policy("keep_all").unwrap(),
            RetentionPolicyConfig::KeepAll
        ));
        assert!(matches!(
            parse_retention_policy("keep_days:7").unwrap(),
            RetentionPolicyConfig::KeepDays(7)
        ));
        assert!(matches!(
            parse_retention_policy("keep_last:5").unwrap(),
            RetentionPolicyConfig::KeepLast(5)
        ));
        assert!(matches!(
            parse_retention_policy("keep_last_and_days:2,30").unwrap(),
            RetentionPolicyConfig::KeepLastAndDays { last: 2, days: 30 }
        ));

        assert!(parse_retention_policy("invalid").is_err());
    }

    #[test]
    fn test_load_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            "[backup]\ncleanup_on_success=false\ndirectory='backups'\nretention='keep_days:5'\n\n[streaming]\nstrategy='pipe'\n",
        )
        .unwrap();

        let config = AgeConfig::load_from_path(&config_path).unwrap();
        assert!(!config.backup_cleanup);
        assert_eq!(config.backup_directory.as_deref(), Some("backups"));
        assert!(matches!(
            config.backup_retention,
            RetentionPolicyConfig::KeepDays(5)
        ));
        assert_eq!(config.streaming_strategy.as_deref(), Some("pipe"));
    }
}
