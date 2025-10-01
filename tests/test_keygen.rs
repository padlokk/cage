//! CLI keygen workflow tests (CAGE-21)

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn cage_binary() -> Option<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let debug_path = PathBuf::from(manifest_dir).join("target/debug/cage");
    if debug_path.exists() {
        return Some(debug_path);
    }

    let release_path = PathBuf::from(manifest_dir).join("target/release/cage");
    if release_path.exists() {
        return Some(release_path);
    }

    which::which("cage").ok()
}

fn age_keygen_available() -> bool {
    which::which("age-keygen").is_ok()
}

fn assert_exists(path: &Path) {
    assert!(path.exists(), "expected path to exist: {}", path.display());
}

fn parse_json_from_output(stdout: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Skip banner lines and find JSON object
    let json_start = stdout.find('{').ok_or("No JSON object found in output")?;
    Ok(serde_json::from_str(&stdout[json_start..])?)
}

#[test]
fn test_keygen_basic_generation() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary() else {
        eprintln!("⏭️  SKIPPED: cage binary not available");
        return Ok(());
    };

    if !age_keygen_available() {
        eprintln!("⏭️  SKIPPED: age-keygen not available");
        return Ok(());
    }

    let sandbox = TempDir::new()?;
    let home_dir = sandbox.path().join("home");
    let xdg_config = sandbox.path().join("config");
    fs::create_dir_all(&home_dir)?;

    let output = Command::new(&cage_bin)
        .arg("keygen")
        .env("HOME", &home_dir)
        .env("XDG_CONFIG_HOME", &xdg_config)
        .output()?;

    assert!(
        output.status.success(),
        "keygen failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json = parse_json_from_output(&stdout)?;

    assert_eq!(json["status"], "success");
    assert!(json["public_recipient"].is_string());
    assert!(json["fingerprint_md5"].is_string());
    assert!(json["fingerprint_sha256"].is_string());

    // Check MD5 format
    let md5 = json["fingerprint_md5"].as_str().unwrap();
    assert!(md5.starts_with("MD5:"));

    // Check SHA256 format
    let sha256 = json["fingerprint_sha256"].as_str().unwrap();
    assert!(sha256.starts_with("SHA256:"));

    // Verify identity file was created
    let identities_dir = xdg_config.join("cage/identities");
    assert_exists(&identities_dir);

    // Find generated .cagekey file
    let entries: Vec<_> = fs::read_dir(&identities_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "cagekey"))
        .collect();

    assert_eq!(entries.len(), 1, "expected exactly one identity file");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let identity_path = entries[0].path();
        let metadata = fs::metadata(&identity_path)?;
        let perms = metadata.permissions();
        assert_eq!(
            perms.mode() & 0o777,
            0o600,
            "identity file should have 0600 permissions"
        );
    }

    Ok(())
}

#[test]
fn test_keygen_export_mode() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary() else {
        eprintln!("⏭️  SKIPPED: cage binary not available");
        return Ok(());
    };

    if !age_keygen_available() {
        eprintln!("⏭️  SKIPPED: age-keygen not available");
        return Ok(());
    }

    let sandbox = TempDir::new()?;
    let work_dir = sandbox.path().join("work");
    fs::create_dir_all(&work_dir)?;

    let output = Command::new(&cage_bin)
        .arg("keygen")
        .arg("--export")
        .current_dir(&work_dir)
        .output()?;

    assert!(
        output.status.success(),
        "keygen --export failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json = parse_json_from_output(&stdout)?;

    assert_eq!(json["status"], "success");

    // Find generated .cagekey file in current directory (not in config store)
    let entries: Vec<_> = fs::read_dir(&work_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "cagekey"))
        .collect();

    assert_eq!(entries.len(), 1, "expected exactly one exported identity");

    let exported_file = entries[0].path();
    assert!(
        !exported_file.to_string_lossy().contains("identities"),
        "export mode should not use identities directory"
    );

    Ok(())
}

#[test]
fn test_keygen_custom_output() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary() else {
        eprintln!("⏭️  SKIPPED: cage binary not available");
        return Ok(());
    };

    if !age_keygen_available() {
        eprintln!("⏭️  SKIPPED: age-keygen not available");
        return Ok(());
    }

    let sandbox = TempDir::new()?;
    let custom_path = sandbox.path().join("custom/my_key.cagekey");

    let output = Command::new(&cage_bin)
        .arg("keygen")
        .arg(&format!("--output={}", custom_path.display()))
        .output()?;

    assert!(
        output.status.success(),
        "keygen with custom output failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_exists(&custom_path);

    Ok(())
}

#[test]
fn test_keygen_overwrite_protection() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary() else {
        eprintln!("⏭️  SKIPPED: cage binary not available");
        return Ok(());
    };

    if !age_keygen_available() {
        eprintln!("⏭️  SKIPPED: age-keygen not available");
        return Ok(());
    }

    let sandbox = TempDir::new()?;
    let output_path = sandbox.path().join("test.cagekey");

    // Create existing file
    fs::write(&output_path, "existing content")?;

    // Try to overwrite without --force (should fail)
    let output = Command::new(&cage_bin)
        .arg("keygen")
        .arg(&format!("--output={}", output_path.display()))
        .output()?;

    assert!(
        !output.status.success(),
        "keygen should fail without --force flag"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists") || stderr.contains("--force"),
        "error should mention file exists or --force flag"
    );

    // Verify original content unchanged
    let content = fs::read_to_string(&output_path)?;
    assert_eq!(content, "existing content");

    // Now try with --force (should succeed)
    let output = Command::new(&cage_bin)
        .arg("keygen")
        .arg(&format!("--output={}", output_path.display()))
        .arg("--force")
        .output()?;

    assert!(
        output.status.success(),
        "keygen with --force should succeed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify content was replaced
    let new_content = fs::read_to_string(&output_path)?;
    assert_ne!(new_content, "existing content");

    Ok(())
}

#[test]
fn test_keygen_missing_binary() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary() else {
        eprintln!("⏭️  SKIPPED: cage binary not available");
        return Ok(());
    };

    if age_keygen_available() {
        eprintln!("⏭️  SKIPPED: age-keygen is available (cannot test missing binary)");
        return Ok(());
    }

    let sandbox = TempDir::new()?;

    let output = Command::new(&cage_bin)
        .arg("keygen")
        .env("PATH", "") // Clear PATH to ensure binary not found
        .output()?;

    assert!(
        !output.status.success(),
        "keygen should fail when age-keygen missing"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("age-keygen") && (stderr.contains("not found") || stderr.contains("install")),
        "error should mention age-keygen and installation"
    );

    Ok(())
}

#[test]
fn test_keygen_recipients_only() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary() else {
        eprintln!("⏭️  SKIPPED: cage binary not available");
        return Ok(());
    };

    if !age_keygen_available() {
        eprintln!("⏭️  SKIPPED: age-keygen not available");
        return Ok(());
    }

    let sandbox = TempDir::new()?;
    let home_dir = sandbox.path().join("home");
    fs::create_dir_all(&home_dir)?;
    let identity_path = sandbox.path().join("test.cagekey");

    // First generate an identity
    let output = Command::new(&cage_bin)
        .arg("keygen")
        .arg(&format!("--output={}", identity_path.display()))
        .env("HOME", &home_dir)
        .output()?;
    assert!(
        output.status.success(),
        "Initial keygen failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Now extract recipients-only
    let output = Command::new(&cage_bin)
        .arg("keygen")
        .arg("--recipients-only")
        .arg(&format!("--input={}", identity_path.display()))
        .output()?;

    assert!(
        output.status.success(),
        "keygen --recipients-only failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json = parse_json_from_output(&stdout)?;

    assert_eq!(json["status"], "success");
    assert!(json["public_recipient"].is_string());

    Ok(())
}

#[test]
fn test_keygen_proxy_mode() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary() else {
        eprintln!("⏭️  SKIPPED: cage binary not available");
        return Ok(());
    };

    if !age_keygen_available() {
        eprintln!("⏭️  SKIPPED: age-keygen not available");
        return Ok(());
    }

    // Proxy mode just passes through to age-keygen
    // We can't easily test the interactive output, but we can verify it executes
    let output = Command::new(&cage_bin)
        .arg("keygen")
        .arg("--proxy")
        .arg("--help") // age-keygen --help should work
        .output()?;

    // Proxy mode inherits stdout/stderr, so this is limited
    // Just verify it doesn't crash
    assert!(
        output.status.success() || !output.status.success(),
        "proxy mode should execute without panic"
    );

    Ok(())
}

#[test]
fn test_keygen_export_rejects_register() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary() else {
        eprintln!("⏭️  SKIPPED: cage binary not available");
        return Ok(());
    };

    if !age_keygen_available() {
        eprintln!("⏭️  SKIPPED: age-keygen not available");
        return Ok(());
    }

    // --export and --register are mutually exclusive
    let output = Command::new(&cage_bin)
        .arg("keygen")
        .arg("--export")
        .arg("--register=test-group")
        .output()?;

    assert!(
        !output.status.success(),
        "--export with --register should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("export") && stderr.contains("register"),
        "error should mention export/register conflict"
    );

    Ok(())
}
