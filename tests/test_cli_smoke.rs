//! CLI Smoke Tests - End-to-End Testing (QA-02)
//!
//! Comprehensive CLI integration tests covering:
//! - BUG-01: Extension preservation (.txt -> .txt.cage, not .cage)
//! - BUG-02: Recursive directory operations
//! - BUG-03: Glob pattern matching (*.txt, not substring)
//! - BUG-04: Unlock options (preserve_encrypted, selective)
//! - .padlock extension support for Padlock integration
//!
//! These tests require the `age` binary in PATH and will skip gracefully if unavailable.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Check if the cage binary is available
fn cage_binary_available() -> Option<PathBuf> {
    // Try to find the cage binary in target/debug or target/release
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let debug_path = PathBuf::from(manifest_dir).join("target/debug/cage");
    if debug_path.exists() {
        return Some(debug_path);
    }

    let release_path = PathBuf::from(manifest_dir).join("target/release/cage");
    if release_path.exists() {
        return Some(release_path);
    }

    // Try which
    if let Ok(path) = which::which("cage") {
        return Some(path);
    }

    None
}

/// Check if age binary is available
fn age_available() -> bool {
    which::which("age").is_ok()
}

/// Check if age-keygen binary is available
fn age_keygen_available() -> bool {
    which::which("age-keygen").is_ok()
}

/// Skip test if required binaries aren't available
fn check_test_requirements() -> Option<PathBuf> {
    if !age_available() {
        println!("⏭️  SKIPPED: age binary not available in PATH");
        println!("   Install age: https://github.com/FiloSottile/age");
        return None;
    }

    if !age_keygen_available() {
        println!("⏭️  SKIPPED: age-keygen binary not available in PATH");
        println!("   Some systems split age and age-keygen into separate packages");
        return None;
    }

    let cage_bin = cage_binary_available()?;
    if !cage_bin.exists() {
        println!("⏭️  SKIPPED: cage binary not found");
        println!("   Run: cargo build");
        return None;
    }

    Some(cage_bin)
}

/// Generate a test age identity for encryption tests
fn generate_test_identity(temp_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let identity_path = temp_dir.join("test_identity.txt");

    let output = Command::new("age-keygen")
        .arg("-o")
        .arg(&identity_path)
        .output()?;

    if !output.status.success() {
        return Err(format!("Failed to generate age identity: {}",
            String::from_utf8_lossy(&output.stderr)).into());
    }

    Ok(identity_path)
}

/// Extract public key from identity file
fn extract_public_key(identity_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(identity_path)?;

    for line in content.lines() {
        if line.starts_with("# public key: ") {
            return Ok(line.strip_prefix("# public key: ").unwrap().to_string());
        }
    }

    Err("No public key found in identity file".into())
}

// ============================================================================
// BUG-01: Extension Preservation Tests
// ============================================================================

#[test]
fn test_extension_preservation_single_file() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = check_test_requirements() else {
        return Ok(());
    };

    println!("🧪 TEST: BUG-01 regression - Extension preservation");

    let temp_dir = TempDir::new()?;
    let identity_path = generate_test_identity(temp_dir.path())?;
    let recipient = extract_public_key(&identity_path)?;

    // Create test file with specific extension
    let test_file = temp_dir.path().join("document.txt");
    fs::write(&test_file, "Secret content")?;

    // Lock the file
    let output = Command::new(&cage_bin)
        .arg("lock")
        .arg(&test_file)
        .arg(format!("--recipient={}", recipient))
        .output()?;

    if !output.status.success() {
        eprintln!("Lock failed: {}", String::from_utf8_lossy(&output.stderr));
        return Err("Lock command failed".into());
    }

    // Verify encrypted file has correct extension: document.txt.cage (NOT document.cage)
    let expected_encrypted = temp_dir.path().join("document.txt.cage");
    assert!(expected_encrypted.exists(),
        "BUG-01: Encrypted file should be document.txt.cage, not document.cage");

    // Verify original is preserved by default (current behavior)
    assert!(test_file.exists(), "Original file should be preserved by default");

    println!("✅ Extension preserved: document.txt -> document.txt.cage");

    Ok(())
}

#[test]
fn test_extension_preservation_unlock() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = check_test_requirements() else {
        return Ok(());
    };

    println!("🧪 TEST: BUG-01 regression - Extension preservation on unlock");

    let temp_dir = TempDir::new()?;
    let identity_path = generate_test_identity(temp_dir.path())?;
    let recipient = extract_public_key(&identity_path)?;

    // Create and encrypt file
    let test_file = temp_dir.path().join("data.json");
    fs::write(&test_file, r#"{"key": "value"}"#)?;

    Command::new(&cage_bin)
        .arg("lock")
        .arg(&test_file)
        .arg(format!("--recipient={}", recipient))
        .output()?;

    let encrypted_file = temp_dir.path().join("data.json.cage");
    assert!(encrypted_file.exists());

    // Unlock the file
    let output = Command::new(&cage_bin)
        .arg("unlock")
        .arg(&encrypted_file)
        .arg(format!("--identity={}", identity_path.display()))
        .output()?;

    if !output.status.success() {
        eprintln!("Unlock failed: {}", String::from_utf8_lossy(&output.stderr));
        return Err("Unlock command failed".into());
    }

    // Verify decrypted file has original extension restored
    let decrypted_file = temp_dir.path().join("data.json");
    assert!(decrypted_file.exists(),
        "BUG-01: Decrypted file should be data.json (original extension restored)");

    let content = fs::read_to_string(&decrypted_file)?;
    assert!(content.contains("\"key\""), "Decrypted content should match original");

    println!("✅ Extension restored on unlock: data.json.cage -> data.json");

    Ok(())
}

// ============================================================================
// .padlock Extension Support (Padlock Integration)
// ============================================================================

#[test]
fn test_padlock_extension_lock() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = check_test_requirements() else {
        return Ok(());
    };

    println!("🧪 TEST: .padlock extension support for Padlock integration");

    let temp_dir = TempDir::new()?;
    let identity_path = generate_test_identity(temp_dir.path())?;
    let recipient = extract_public_key(&identity_path)?;

    // Create test file
    let test_file = temp_dir.path().join("secret.txt");
    fs::write(&test_file, "Padlock vault data")?;

    // Lock with custom .padlock extension (future feature - may need --extension flag)
    // For now, just verify standard .cage works and document .padlock readiness
    let output = Command::new(&cage_bin)
        .arg("lock")
        .arg(&test_file)
        .arg(format!("--recipient={}", recipient))
        .output()?;

    assert!(output.status.success(), "Lock should succeed");

    let encrypted_file = temp_dir.path().join("secret.txt.cage");
    assert!(encrypted_file.exists());

    println!("✅ Encryption works (cage extension)");
    println!("   Note: .padlock extension support tracked in config (padlock_extension_support=true)");

    Ok(())
}

// ============================================================================
// BUG-02: Recursive Directory Operations
// ============================================================================

#[test]
#[ignore = "BUG-02: Recursive operations not yet fully implemented - awaiting CLI recursion support"]
fn test_recursive_directory_lock() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = check_test_requirements() else {
        return Ok(());
    };

    println!("🧪 TEST: BUG-02 regression - Recursive directory operations");

    let temp_dir = TempDir::new()?;
    let identity_path = generate_test_identity(temp_dir.path())?;
    let recipient = extract_public_key(&identity_path)?;

    // Create directory structure with files in subdirectories
    let subdir1 = temp_dir.path().join("folder1");
    let subdir2 = temp_dir.path().join("folder1/nested");
    fs::create_dir_all(&subdir2)?;

    fs::write(temp_dir.path().join("root.txt"), "root file")?;
    fs::write(subdir1.join("file1.txt"), "subfolder file")?;
    fs::write(subdir2.join("file2.txt"), "nested file")?;

    // Lock recursively - this MUST succeed for BUG-02 to be considered fixed
    let output = Command::new(&cage_bin)
        .arg("lock")
        .arg(temp_dir.path())
        .arg("--recursive")
        .arg(format!("--recipient={}", recipient))
        .output()?;

    assert!(output.status.success(),
        "BUG-02: Recursive lock must succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr));

    // Verify files in subdirectories were encrypted
    let root_encrypted = temp_dir.path().join("root.txt.cage").exists();
    let sub_encrypted = subdir1.join("file1.txt.cage").exists();
    let nested_encrypted = subdir2.join("file2.txt.cage").exists();

    assert!(root_encrypted, "BUG-02: Root file must be encrypted");
    assert!(sub_encrypted, "BUG-02: Subfolder file must be encrypted");
    assert!(nested_encrypted, "BUG-02: Nested file must be encrypted");

    println!("✅ BUG-02 FIXED: Recursive lock processed all subdirectories");

    Ok(())
}

// ============================================================================
// BUG-03: Glob Pattern Matching
// ============================================================================

#[test]
#[ignore = "BUG-03: Glob pattern CLI flags not yet implemented - awaiting --include/--exclude support"]
fn test_glob_pattern_matching() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = check_test_requirements() else {
        return Ok(());
    };

    println!("🧪 TEST: BUG-03 regression - Glob pattern matching");

    let temp_dir = TempDir::new()?;
    let identity_path = generate_test_identity(temp_dir.path())?;
    let recipient = extract_public_key(&identity_path)?;

    // Create mix of files
    fs::write(temp_dir.path().join("data.txt"), "text file")?;
    fs::write(temp_dir.path().join("data.json"), "json file")?;
    fs::write(temp_dir.path().join("readme.txt"), "readme")?;
    fs::write(temp_dir.path().join("config.yaml"), "config")?;

    // Lock with glob pattern - should only match .txt files
    let output = Command::new(&cage_bin)
        .arg("lock")
        .arg(temp_dir.path())
        .arg("--include=*.txt")
        .arg(format!("--recipient={}", recipient))
        .output()?;

    assert!(output.status.success(),
        "BUG-03: Glob-filtered lock must succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr));

    // Verify only .txt files were encrypted
    assert!(temp_dir.path().join("data.txt.cage").exists(), "BUG-03: *.txt should match data.txt");
    assert!(temp_dir.path().join("readme.txt.cage").exists(), "BUG-03: *.txt should match readme.txt");
    assert!(!temp_dir.path().join("data.json.cage").exists(), "BUG-03: *.txt should NOT match data.json");
    assert!(!temp_dir.path().join("config.yaml.cage").exists(), "BUG-03: *.txt should NOT match config.yaml");

    println!("✅ BUG-03 FIXED: Glob patterns correctly filter files");

    Ok(())
}

// ============================================================================
// BUG-04: Unlock Options
// ============================================================================

#[test]
fn test_unlock_preserve_option() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = check_test_requirements() else {
        return Ok(());
    };

    println!("🧪 TEST: BUG-04 regression - Unlock --preserve option");

    let temp_dir = TempDir::new()?;
    let identity_path = generate_test_identity(temp_dir.path())?;
    let recipient = extract_public_key(&identity_path)?;

    // Create and encrypt file
    let test_file = temp_dir.path().join("preserve_test.txt");
    fs::write(&test_file, "content")?;

    let lock_output = Command::new(&cage_bin)
        .arg("lock")
        .arg(&test_file)
        .arg(format!("--recipient={}", recipient))
        .output()?;

    assert!(lock_output.status.success(),
        "Lock must succeed. stderr: {}",
        String::from_utf8_lossy(&lock_output.stderr));

    let encrypted_file = temp_dir.path().join("preserve_test.txt.cage");
    assert!(encrypted_file.exists(), "Encrypted file must exist after lock");

    // Unlock with --preserve (CLI flag from src/bin/cli_age.rs:251,1070)
    let output = Command::new(&cage_bin)
        .arg("unlock")
        .arg(&encrypted_file)
        .arg(format!("--identity={}", identity_path.display()))
        .arg("--preserve")
        .output()?;

    assert!(output.status.success(),
        "BUG-04: Unlock with --preserve must succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr));

    // Verify both files exist
    let decrypted_file = temp_dir.path().join("preserve_test.txt");
    assert!(decrypted_file.exists(), "BUG-04: Decrypted file should exist");
    assert!(encrypted_file.exists(), "BUG-04: Encrypted file should be preserved with --preserve flag");

    println!("✅ BUG-04: --preserve option correctly preserves encrypted file");

    Ok(())
}

// ============================================================================
// Basic CLI Smoke Tests
// ============================================================================

#[test]
fn test_cli_version() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary_available() else {
        println!("⏭️  SKIPPED: cage binary not found");
        return Ok(());
    };

    println!("🧪 TEST: CLI version command");

    let output = Command::new(&cage_bin)
        .arg("version")
        .output()?;

    assert!(output.status.success(), "cage version should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.") || stdout.contains("Cage"),
        "Version output should contain version info");

    println!("✅ CLI version command works");

    Ok(())
}

#[test]
fn test_cli_help() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary_available() else {
        println!("⏭️  SKIPPED: cage binary not found");
        return Ok(());
    };

    println!("🧪 TEST: CLI help command");

    let output = Command::new(&cage_bin)
        .arg("help")
        .output()?;

    assert!(output.status.success() || output.status.code() == Some(0),
        "cage help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lock") || stdout.contains("unlock") || stdout.contains("COMMANDS"),
        "Help output should contain command information");

    println!("✅ CLI help command works");

    Ok(())
}

#[test]
fn test_cli_config_show() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary_available() else {
        println!("⏭️  SKIPPED: cage binary not found");
        return Ok(());
    };

    println!("🧪 TEST: CLI config show command");

    let output = Command::new(&cage_bin)
        .arg("config")
        .arg("show")
        .output()?;

    // Config show should work even if no config file exists
    assert!(output.status.success() || output.status.code() == Some(0),
        "cage config show should not fail");

    println!("✅ CLI config show command works");

    Ok(())
}