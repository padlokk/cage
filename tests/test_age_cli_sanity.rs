//! Age CLI Output Sanity Tests (QA-03)
//!
//! Lightweight tests to verify age CLI is available and produces expected output.
//! These tests help detect breaking changes when upgrading age versions.

use std::process::Command;

/// Test that age --version produces expected output format
#[test]
fn test_age_version_output() {
    // Skip if age is not available
    let version_output = match Command::new("age").arg("--version").output() {
        Ok(output) => output,
        Err(_) => {
            eprintln!("Skipping age version test: age binary not found");
            eprintln!("Install age: cargo install age or apt install age");
            return;
        }
    };

    // Check the command succeeded
    assert!(
        version_output.status.success(),
        "age --version should succeed"
    );

    let version_string = String::from_utf8_lossy(&version_output.stdout);

    // Verify expected version format
    // age outputs just the version number: "1.1.1"
    // Should have a version number (e.g., "1.1.1", "1.0.0", etc.)
    assert!(
        version_string.chars().any(|c| c.is_ascii_digit()),
        "Version output should contain version numbers, got: {}",
        version_string
    );

    // Should contain dots for version separation
    assert!(
        version_string.contains('.'),
        "Version output should contain version dots, got: {}",
        version_string
    );

    println!("✅ Age version detected: {}", version_string.trim());
}

/// Test that age --help produces expected usage information
#[test]
fn test_age_help_output() {
    // Skip if age is not available
    let help_output = match Command::new("age").arg("--help").output() {
        Ok(output) => output,
        Err(_) => {
            eprintln!("Skipping age help test: age binary not found");
            return;
        }
    };

    // Check the command succeeded (help might exit with code 0 or 2)
    // Some versions of age return exit code 2 for help

    // Combine stdout and stderr as help might go to either
    let help_string = if help_output.stdout.is_empty() {
        String::from_utf8_lossy(&help_output.stderr)
    } else {
        String::from_utf8_lossy(&help_output.stdout)
    };

    // Verify expected help content
    // These are core flags that should always be present
    let expected_flags = vec![
        "Usage:",    // Usage line
        "--encrypt", // Encrypt flag
        "--decrypt", // Decrypt flag
        "-p",        // Passphrase flag
        "-i",        // Identity flag
        "-r",        // Recipient flag
        "-o",        // Output flag
        "--armor",   // ASCII armor flag
    ];

    for flag in &expected_flags {
        assert!(
            help_string.contains(flag),
            "Help output should contain '{}', but it was not found",
            flag
        );
    }

    println!("✅ Age help output contains all expected flags");
}

/// Test that age-keygen is available (often bundled with age)
#[test]
fn test_age_keygen_availability() {
    // Skip if age-keygen is not available
    let keygen_output = match Command::new("age-keygen").arg("--version").output() {
        Ok(output) => output,
        Err(_) => {
            eprintln!("Note: age-keygen not found (optional tool)");
            return;
        }
    };

    if keygen_output.status.success() {
        let version_string = String::from_utf8_lossy(&keygen_output.stdout);
        println!("✅ age-keygen detected: {}", version_string.trim());
    }
}

/// Reference documentation for expected age CLI behavior
///
/// As of age 1.1.1, the expected outputs are:
///
/// `age --version`:
/// ```
/// 1.1.1
/// ```
///
/// `age --help` (key sections):
/// ```
/// Usage:
///     age [--encrypt] (-r RECIPIENT | -R PATH)... [--armor] [-o OUTPUT] [INPUT]
///     age [--encrypt] --passphrase [--armor] [-o OUTPUT] [INPUT]
///     age --decrypt [-i PATH]... [-o OUTPUT] [INPUT]
///
/// Options:
///     -e, --encrypt               Encrypt the input to the output. Default if omitted.
///     -d, --decrypt               Decrypt the input to the output.
///     -o, --output OUTPUT         Write the result to the file at path OUTPUT.
///     -a, --armor                 Encrypt to a PEM encoded format.
///     -p, --passphrase            Encrypt with a passphrase.
///     -r, --recipient RECIPIENT   Encrypt to the specified RECIPIENT. Can be repeated.
///     -R, --recipients-file PATH  Encrypt to recipients listed at PATH. Can be repeated.
///     -i, --identity PATH         Use the identity file at PATH. Can be repeated.
/// ```
///
/// Note: Help output may go to stdout or stderr depending on the age version.
/// The version output is just the version number without "age" prefix.
///
/// If these outputs change significantly in future age versions,
/// the tests above will fail, alerting us to potential compatibility issues.
#[test]
fn test_reference_documentation() {
    // This test always passes - it exists to document expected behavior
    println!("📚 Reference: See test module documentation for expected age CLI outputs");
}

/// Test binary detection helper used by other tests
#[test]
fn test_age_binary_detection() {
    let age_available = which::which("age").is_ok();
    let age_keygen_available = which::which("age-keygen").is_ok();

    println!("🔍 Age binary detection results:");
    println!(
        "  - age: {}",
        if age_available {
            "✅ Found"
        } else {
            "❌ Not found"
        }
    );
    println!(
        "  - age-keygen: {}",
        if age_keygen_available {
            "✅ Found"
        } else {
            "❌ Not found (optional)"
        }
    );

    if !age_available {
        println!("\n📦 Installation instructions:");
        println!("  - Cargo: cargo install age");
        println!("  - Ubuntu/Debian: apt install age");
        println!("  - macOS: brew install age");
        println!("  - Other: https://github.com/FiloSottile/age#installation");
    }
}
