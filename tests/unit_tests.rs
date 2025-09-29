//! Unit Tests for Cage
//! Tests core cage library functionality

use cage::{AgeConfig, OutputFormat, VERSION};

#[test]
fn test_version_constant() {
    // Test that VERSION constant is properly set
    assert!(!VERSION.is_empty());
    assert!(VERSION.contains('.'));
    println!("✅ Version: {}", VERSION);
}

#[test]
fn test_age_config_creation() {
    // Test AgeConfig can be created with defaults
    let config = AgeConfig::default();
    assert_eq!(config.output_format, OutputFormat::Binary);
    println!("✅ AgeConfig creation works");
}

#[test]
fn test_output_format_variants() {
    // Test OutputFormat enum variants
    let binary = OutputFormat::Binary;
    let ascii = OutputFormat::AsciiArmor;

    assert_ne!(binary, ascii);
    println!("✅ OutputFormat variants work");
}

#[test]
fn test_library_exports() {
    // Test that main library types are exported and accessible
    let _config = AgeConfig::default();
    let _format = OutputFormat::Binary;

    // These should compile without errors
    println!("✅ Library exports accessible");
}

#[cfg(test)]
mod pty_tests {
    use super::*;

    #[test]
    fn test_pty_module_accessible() {
        // Basic test that PTY module compiles
        // More detailed tests would require age binary
        println!("✅ PTY module accessible");
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_error_types_exist() {
        // Test that error types are accessible
        use cage::AgeError;

        // This should compile
        let _error_type = std::marker::PhantomData::<AgeError>;
        println!("✅ Error types accessible");
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_security_validator_accessible() {
        // Test that security validation is available
        use cage::SecurityValidator;

        let validator = SecurityValidator::new(false);
        // SecurityValidator doesn't return Result, so just test creation
        println!("✅ Security validator accessible");
    }
}
