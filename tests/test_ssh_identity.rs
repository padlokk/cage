//! Test SSH identity support for age encryption
//!
//! Tests both SSH recipient (public key) and SSH identity (private key) functionality

use cage::cage::{
    adapter_v2::{AgeAdapterV2, ShellAdapterV2},
    config::{AgeConfig, OutputFormat},
    requests::{Identity, Recipient},
};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Test SSH recipient encryption and SSH identity decryption
#[test]
fn test_ssh_encryption_decryption() {
    // Skip if age is not available
    if Command::new("age").arg("--version").output().is_err() {
        eprintln!("Skipping SSH test: age binary not found");
        return;
    }

    // Skip if ssh-keygen is not available
    if Command::new("ssh-keygen").arg("-h").output().is_err() {
        eprintln!("Skipping SSH test: ssh-keygen not found");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let key_path = temp_dir.path().join("test_ssh_key");
    let pub_key_path = temp_dir.path().join("test_ssh_key.pub");

    // Generate SSH key pair
    let keygen = Command::new("ssh-keygen")
        .args(&["-t", "ed25519"])
        .args(&["-f", key_path.to_str().unwrap()])
        .args(&["-N", ""]) // No passphrase
        .args(&["-q"]) // Quiet mode
        .output()
        .expect("Failed to generate SSH key");

    assert!(keygen.status.success(), "SSH key generation failed");
    assert!(pub_key_path.exists(), "Public key not created");
    assert!(key_path.exists(), "Private key not created");

    // Read the public key
    let pub_key = fs::read_to_string(&pub_key_path)
        .expect("Failed to read public key")
        .trim()
        .to_string();

    // Create test data
    let input_path = temp_dir.path().join("test_data.txt");
    let encrypted_path = temp_dir.path().join("test_data.txt.age");
    let decrypted_path = temp_dir.path().join("test_data_decrypted.txt");
    let test_content = "SSH encryption test data";

    fs::write(&input_path, test_content).expect("Failed to write test data");

    // Test encryption with SSH recipient
    let adapter = match ShellAdapterV2::new() {
        Ok(a) => a,
        Err(_) => {
            eprintln!("Skipping SSH test: Failed to create adapter (PTY/age not available)");
            return;
        }
    };

    // First verify the SSH key is valid for use as recipient
    let age_recipient = adapter.ssh_to_recipient(&pub_key)
        .expect("Failed to validate SSH key as recipient");
    // For CLI usage, SSH keys are passed directly to age
    assert_eq!(age_recipient, pub_key, "SSH key should be returned as-is");

    // Encrypt with SSH recipient
    let result = adapter.encrypt_file(
        &input_path,
        &encrypted_path,
        &Identity::PromptPassphrase, // Not used for recipient encryption
        Some(&[Recipient::SshRecipients(vec![pub_key.clone()])]),
        OutputFormat::Binary,
    );

    assert!(result.is_ok(), "Encryption failed: {:?}", result);
    assert!(encrypted_path.exists(), "Encrypted file not created");

    // Decrypt with SSH identity
    let result = adapter.decrypt_file(
        &encrypted_path,
        &decrypted_path,
        &Identity::SshKey(key_path.clone()),
    );

    assert!(result.is_ok(), "Decryption failed: {:?}", result);
    assert!(decrypted_path.exists(), "Decrypted file not created");

    // Verify content
    let decrypted_content = fs::read_to_string(&decrypted_path)
        .expect("Failed to read decrypted content");
    assert_eq!(decrypted_content, test_content, "Content mismatch after decryption");
}

/// Test that invalid SSH keys are rejected
#[test]
fn test_invalid_ssh_key_rejection() {
    let adapter = match ShellAdapterV2::new() {
        Ok(a) => a,
        Err(_) => {
            eprintln!("Skipping SSH rejection test: Failed to create adapter (PTY/age not available)");
            return;
        }
    };

    // Test invalid SSH key format
    let invalid_keys = vec![
        "not-an-ssh-key",
        "age1somekey", // Age key, not SSH
        "rsa key data", // Missing ssh- prefix
    ];

    for invalid_key in invalid_keys {
        let result = adapter.ssh_to_recipient(invalid_key);
        assert!(
            result.is_err(),
            "Invalid key '{}' should be rejected",
            invalid_key
        );
    }
}

/// Test SSH identity file validation
#[test]
fn test_ssh_identity_file_validation() {
    let adapter = match ShellAdapterV2::new() {
        Ok(a) => a,
        Err(_) => {
            eprintln!("Skipping SSH identity validation test: Failed to create adapter (PTY/age not available)");
            return;
        }
    };
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test with non-existent file
    let non_existent = temp_dir.path().join("non_existent_key");
    let identity = Identity::SshKey(non_existent.clone());
    let result = adapter.validate_identity(&identity);

    assert!(
        result.is_err(),
        "Non-existent SSH key file should fail validation"
    );

    // Test with existing file
    let existing_file = temp_dir.path().join("existing_key");
    fs::write(&existing_file, "dummy content").expect("Failed to create dummy file");
    let identity = Identity::SshKey(existing_file);
    let result = adapter.validate_identity(&identity);

    assert!(
        result.is_ok(),
        "Existing SSH key file should pass validation"
    );
}