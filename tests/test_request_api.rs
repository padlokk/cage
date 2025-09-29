//! Test the new request API (CAGE-11)
//! Demonstrates that the unified request structs are properly wired into CrudManager

use cage::cage::adapter::ShellAdapter;
use cage::cage::config::{AgeConfig, OutputFormat};
use cage::cage::lifecycle::crud_manager::CrudManager;
use cage::cage::requests::{Identity, LockRequest, UnlockRequest};
use std::fs;
use tempfile::TempDir;

fn age_available() -> bool {
    which::which("age").is_ok()
}

fn setup_test_manager(temp_dir: &TempDir) -> CrudManager {
    let adapter = Box::new(ShellAdapter::new().expect("Failed to create adapter"));
    let mut config = AgeConfig::default();
    config.audit_log_path = Some(temp_dir.path().join("audit.log").display().to_string());

    CrudManager::new(adapter, config).expect("Failed to create CrudManager")
}

#[test]
fn test_lock_with_request_api() -> Result<(), Box<dyn std::error::Error>> {
    if !age_available() {
        println!("SKIPPED: Age binary not found in PATH");
        return Ok(());
    }

    println!("TEST: Lock operation using request API");
    println!("========================================");

    let temp_dir = TempDir::new()?;
    let mut manager = setup_test_manager(&temp_dir);

    // Create test file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content for request API")?;

    // Create lock request
    let lock_request = LockRequest::new(
        test_file.clone(),
        Identity::Passphrase("test_password_123".to_string()),
    )
    .with_format(OutputFormat::Binary);

    // Lock using request API
    let lock_result = manager.lock_with_request(&lock_request)?;
    assert_eq!(
        lock_result.processed_files.len(),
        1,
        "Should process one file"
    );

    // Verify encrypted file exists
    let encrypted_file = test_file.with_extension("txt.cage");
    assert!(encrypted_file.exists(), "Encrypted file should exist");

    println!("[PASS] Lock with request API works correctly");
    Ok(())
}

#[test]
fn test_unlock_with_request_api() -> Result<(), Box<dyn std::error::Error>> {
    if !age_available() {
        println!("SKIPPED: Age binary not found in PATH");
        return Ok(());
    }

    println!("TEST: Unlock operation using request API");
    println!("==========================================");

    let temp_dir = TempDir::new()?;
    let mut manager = setup_test_manager(&temp_dir);

    // Create and lock test file first
    let test_file = temp_dir.path().join("unlock_test.txt");
    let original_content = "Content to unlock via request API";
    fs::write(&test_file, original_content)?;

    // Lock it first
    let lock_request = LockRequest::new(
        test_file.clone(),
        Identity::Passphrase("unlock_pass_456".to_string()),
    );
    manager.lock_with_request(&lock_request)?;

    // Now unlock using request API
    let encrypted_file = test_file.with_extension("txt.cage");
    let unlock_request = UnlockRequest::new(
        encrypted_file.clone(),
        Identity::Passphrase("unlock_pass_456".to_string()),
    )
    .selective(true)
    .preserve_encrypted(true);

    let unlock_result = manager.unlock_with_request(&unlock_request)?;
    assert_eq!(
        unlock_result.processed_files.len(),
        1,
        "Should unlock one file"
    );

    // Verify content
    let unlocked_content = fs::read_to_string(&test_file)?;
    assert_eq!(unlocked_content, original_content, "Content should match");

    // Verify encrypted file preserved
    assert!(
        encrypted_file.exists(),
        "Encrypted file should be preserved"
    );

    println!("[PASS] Unlock with request API works correctly");
    Ok(())
}

#[test]
fn test_request_api_with_pattern_filter() -> Result<(), Box<dyn std::error::Error>> {
    if !age_available() {
        println!("SKIPPED: Age binary not found in PATH");
        return Ok(());
    }

    println!("TEST: Request API with pattern filtering");
    println!("==========================================");

    let temp_dir = TempDir::new()?;
    let mut manager = setup_test_manager(&temp_dir);

    // Create multiple files
    fs::write(temp_dir.path().join("file1.txt"), "Content 1")?;
    fs::write(temp_dir.path().join("file2.txt"), "Content 2")?;
    fs::write(temp_dir.path().join("file3.doc"), "Content 3")?;

    // Lock only .txt files using pattern
    let lock_request = LockRequest::new(
        temp_dir.path().to_path_buf(),
        Identity::Passphrase("pattern_pass".to_string()),
    )
    .recursive(true)
    .with_pattern("*.txt".to_string());

    let lock_result = manager.lock_with_request(&lock_request)?;

    // Should lock 2 .txt files, not the .doc file
    assert_eq!(
        lock_result.processed_files.len(),
        2,
        "Should lock only .txt files"
    );

    // Verify .txt files are encrypted
    assert!(temp_dir.path().join("file1.txt.cage").exists());
    assert!(temp_dir.path().join("file2.txt.cage").exists());
    assert!(!temp_dir.path().join("file3.doc.cage").exists());

    println!("[PASS] Request API with pattern filtering works correctly");
    Ok(())
}
