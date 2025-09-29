//! Selective Unlock Integration Tests (BUG-04 Regression Coverage)
//! Tests the selective unlock feature that pre-verifies files before unlocking
//!
//! Note: These tests require the Age binary to be installed and available in PATH.
//! Tests will be skipped if Age is not found.

use cage::cage::adapter::ShellAdapter;
use cage::cage::config::{AgeConfig, OutputFormat};
use cage::cage::lifecycle::crud_manager::{CrudManager, LockOptions, UnlockOptions};
use std::fs;
use tempfile::TempDir;

fn age_available() -> bool {
    which::which("age").is_ok()
}

fn setup_test_manager(temp_dir: &TempDir) -> CrudManager {
    if which::which("age").is_err() {
        println!("{}", cage::cage::strings::TEST_SKIP_NO_AGE);
        panic!("skip");
    }
    let adapter = Box::new(ShellAdapter::new().expect("Failed to create adapter"));
    let mut config = AgeConfig::default();
    config.audit_log_path = Some(temp_dir.path().join("audit.log").display().to_string());

    CrudManager::new(adapter, config).expect("Failed to create CrudManager")
}

#[test]
fn test_selective_unlock_skips_invalid_files() -> Result<(), Box<dyn std::error::Error>> {
    if !age_available() {
        println!("{}", cage::cage::strings::TEST_SKIP_NO_AGE);
        return Ok(());
    }

    println!("TEST: Selective unlock skips invalid files");
    println!("===========================================");

    let temp_dir = TempDir::new()?;
    let mut manager = setup_test_manager(&temp_dir);

    // Create valid encrypted file
    let test_file = temp_dir.path().join("valid.txt");
    fs::write(&test_file, "Test content")?;

    // Lock the file first
    let lock_options = LockOptions {
        format: OutputFormat::Binary,
        recursive: false,
        pattern_filter: None,
        backup_before_lock: false,
    };
    let passphrase = "test_password_123";
    let lock_result = manager.lock(&test_file, passphrase, lock_options)?;
    assert!(
        lock_result.processed_files.len() > 0,
        "Failed to lock test file"
    );

    // Create an invalid file with .cage extension
    let invalid_file = temp_dir.path().join("invalid.txt.cage");
    fs::write(&invalid_file, "This is not a valid Age encrypted file")?;

    // Unlock valid file - should succeed
    let valid_encrypted = test_file.with_extension("txt.cage");
    let unlock_options = UnlockOptions {
        selective: true,
        verify_before_unlock: true,
        pattern_filter: None,
        preserve_encrypted: true,
    };
    let unlock_result = manager.unlock(&valid_encrypted, passphrase, unlock_options)?;
    assert_eq!(
        unlock_result.processed_files.len(),
        1,
        "Selective unlock should succeed for valid file"
    );

    // Try to unlock invalid file - should skip
    let unlock_options2 = UnlockOptions {
        selective: true,
        verify_before_unlock: true,
        pattern_filter: None,
        preserve_encrypted: true,
    };
    let unlock_invalid_result = manager.unlock(&invalid_file, passphrase, unlock_options2)?;
    assert_eq!(
        unlock_invalid_result.processed_files.len(),
        0,
        "Selective unlock should skip invalid file"
    );
    assert_eq!(
        unlock_invalid_result.failed_files.len(),
        1,
        "Invalid file should be marked as failure"
    );

    println!("[PASS] Selective unlock correctly skips invalid files");
    Ok(())
}

#[test]
fn test_non_selective_unlock_attempts_all_files() -> Result<(), Box<dyn std::error::Error>> {
    if !age_available() {
        println!("{}", cage::cage::strings::TEST_SKIP_NO_AGE);
        return Ok(());
    }

    println!("TEST: Non-selective unlock attempts all files");
    println!("=================================================");

    let temp_dir = TempDir::new()?;
    let mut manager = setup_test_manager(&temp_dir);

    // Create valid encrypted file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content")?;

    let lock_options = LockOptions {
        format: OutputFormat::Binary,
        recursive: false,
        pattern_filter: None,
        backup_before_lock: false,
    };
    let passphrase = "test_password_123";
    let lock_result = manager.lock(&test_file, passphrase, lock_options)?;
    assert!(lock_result.processed_files.len() > 0);

    // Unlock with non-selective mode (default behavior)
    let unlock_options = UnlockOptions {
        selective: false,
        verify_before_unlock: true,
        pattern_filter: None,
        preserve_encrypted: true,
    };

    let encrypted_file = test_file.with_extension("txt.cage");
    let unlock_result = manager.unlock(&encrypted_file, passphrase, unlock_options)?;
    assert_eq!(
        unlock_result.processed_files.len(),
        1,
        "Non-selective unlock should succeed for valid file"
    );

    println!("[PASS] Non-selective unlock works correctly");
    Ok(())
}

#[test]
fn test_selective_unlock_with_verify_before_unlock() -> Result<(), Box<dyn std::error::Error>> {
    if !age_available() {
        println!("{}", cage::cage::strings::TEST_SKIP_NO_AGE);
        return Ok(());
    }

    println!("TEST: Selective unlock with verify_before_unlock");
    println!("===================================================");

    let temp_dir = TempDir::new()?;
    let mut manager = setup_test_manager(&temp_dir);

    // Create and encrypt a file
    let test_file = temp_dir.path().join("verified.txt");
    fs::write(&test_file, "Content to verify")?;

    let lock_options = LockOptions {
        format: OutputFormat::Binary,
        recursive: false,
        pattern_filter: None,
        backup_before_lock: false,
    };
    let passphrase = "secure_pass_456";
    let lock_result = manager.lock(&test_file, passphrase, lock_options)?;
    assert!(lock_result.processed_files.len() > 0);

    // Unlock with both selective and verify_before_unlock enabled
    let unlock_options = UnlockOptions {
        selective: true,
        verify_before_unlock: true,
        pattern_filter: None,
        preserve_encrypted: true,
    };

    let encrypted_file = test_file.with_extension("txt.cage");
    let unlock_result = manager.unlock(&encrypted_file, passphrase, unlock_options)?;

    assert_eq!(
        unlock_result.processed_files.len(),
        1,
        "Unlock should succeed for verified file"
    );

    // Verify the unlocked content
    let unlocked_content = fs::read_to_string(&test_file)?;
    assert_eq!(unlocked_content, "Content to verify");

    println!("[PASS] Selective unlock with verification works correctly");
    Ok(())
}

#[test]
fn test_selective_unlock_directory_with_mixed_files() -> Result<(), Box<dyn std::error::Error>> {
    if !age_available() {
        println!("{}", cage::cage::strings::TEST_SKIP_NO_AGE);
        return Ok(());
    }

    println!("TEST: Selective unlock directory with mixed valid/invalid files");
    println!("===================================================================");

    let temp_dir = TempDir::new()?;
    let mut manager = setup_test_manager(&temp_dir);

    // Create multiple files - some valid, some invalid
    let valid1 = temp_dir.path().join("valid1.txt");
    let valid2 = temp_dir.path().join("valid2.txt");
    fs::write(&valid1, "Valid content 1")?;
    fs::write(&valid2, "Valid content 2")?;

    // Encrypt valid files
    let lock_options = LockOptions {
        format: OutputFormat::Binary,
        recursive: false,
        pattern_filter: None,
        backup_before_lock: false,
    };
    let passphrase = "test_pass_789";
    manager.lock(&valid1, passphrase, lock_options.clone())?;
    manager.lock(&valid2, passphrase, lock_options)?;

    // Create invalid .cage files
    let invalid1 = temp_dir.path().join("invalid1.txt.cage");
    let invalid2 = temp_dir.path().join("invalid2.txt.cage");
    fs::write(&invalid1, "Not encrypted 1")?;
    fs::write(&invalid2, "Not encrypted 2")?;

    // Unlock directory with selective mode
    let unlock_options = UnlockOptions {
        selective: true,
        verify_before_unlock: true,
        pattern_filter: None,
        preserve_encrypted: true,
    };

    let unlock_result = manager.unlock(temp_dir.path(), passphrase, unlock_options)?;

    // Should unlock 2 valid files and skip 2 invalid files
    assert_eq!(
        unlock_result.processed_files.len(),
        2,
        "Should unlock 2 valid files"
    );
    assert_eq!(
        unlock_result.failed_files.len(),
        2,
        "Should skip 2 invalid files"
    );

    println!("[PASS] Selective unlock handles mixed directories correctly");
    Ok(())
}

#[test]
fn test_preserve_encrypted_with_selective() -> Result<(), Box<dyn std::error::Error>> {
    if !age_available() {
        println!("SKIPPED: Age binary not found in PATH");
        return Ok(());
    }

    println!("TEST: preserve_encrypted works with selective unlock");
    println!("========================================================");

    let temp_dir = TempDir::new()?;
    let mut manager = setup_test_manager(&temp_dir);

    let test_file = temp_dir.path().join("preserve_test.txt");
    fs::write(&test_file, "Content to preserve")?;

    let lock_options = LockOptions {
        format: OutputFormat::Binary,
        recursive: false,
        pattern_filter: None,
        backup_before_lock: false,
    };
    let passphrase = "preserve_pass_101";
    manager.lock(&test_file, passphrase, lock_options)?;

    let encrypted_file = test_file.with_extension("txt.cage");

    // Unlock with preserve_encrypted and selective
    let unlock_options = UnlockOptions {
        selective: true,
        verify_before_unlock: true,
        pattern_filter: None,
        preserve_encrypted: true,
    };

    let unlock_result = manager.unlock(&encrypted_file, passphrase, unlock_options)?;
    assert_eq!(unlock_result.processed_files.len(), 1);

    // Verify both files exist
    assert!(test_file.exists(), "Unlocked file should exist");
    assert!(
        encrypted_file.exists(),
        "Encrypted file should be preserved"
    );

    println!("[PASS] preserve_encrypted works correctly with selective mode");
    Ok(())
}
