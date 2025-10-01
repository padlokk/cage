//! CLI init workflow tests (CAGE-20)

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

fn assert_exists(path: &Path) {
    assert!(path.exists(), "expected path to exist: {}", path.display());
}

#[test]
fn test_cage_init_creates_xdg_structure() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cage_bin) = cage_binary() else {
        eprintln!("⏭️  SKIPPED: cage binary not available");
        return Ok(());
    };

    let sandbox = TempDir::new()?;
    let home_dir = sandbox.path().join("home");
    fs::create_dir_all(&home_dir)?;

    let config_path = sandbox.path().join("config/custom_cage.toml");
    let xdg_data = sandbox.path().join("xdg_data");
    let xdg_cache = sandbox.path().join("xdg_cache");

    let status = Command::new(&cage_bin)
        .arg("init")
        .env("HOME", &home_dir)
        .env("CAGE_CONFIG", &config_path)
        .env("XDG_DATA_HOME", &xdg_data)
        .env("XDG_CACHE_HOME", &xdg_cache)
        .status()?;
    assert!(status.success(), "cage init failed (status: {:?})", status);

    let config_parent = config_path.parent().unwrap();
    assert_exists(config_parent);

    let data_dir = xdg_data.join("cage");
    let cache_dir = xdg_cache.join("cage");
    let backup_dir = data_dir.join("backups");

    for dir in [&data_dir, &cache_dir, &backup_dir] {
        assert_exists(dir);
    }

    assert_exists(&config_path);
    let config_contents = fs::read_to_string(&config_path)?;
    let expected_backup = backup_dir.canonicalize().unwrap_or(backup_dir.clone());
    assert!(
        config_contents.contains(&format!("directory = \"{}\"", expected_backup.display())),
        "config should reference backup directory"
    );

    // Modify config and re-run without --force to ensure idempotence
    fs::write(&config_path, "custom = true\n")?;
    let status = Command::new(&cage_bin)
        .arg("init")
        .env("HOME", &home_dir)
        .env("CAGE_CONFIG", &config_path)
        .env("XDG_DATA_HOME", &xdg_data)
        .env("XDG_CACHE_HOME", &xdg_cache)
        .status()?;
    assert!(status.success());
    let contents_after = fs::read_to_string(&config_path)?;
    assert_eq!(contents_after, "custom = true\n");

    // Force regeneration
    let status = Command::new(&cage_bin)
        .arg("init")
        .arg("--force")
        .env("HOME", &home_dir)
        .env("CAGE_CONFIG", &config_path)
        .env("XDG_DATA_HOME", &xdg_data)
        .env("XDG_CACHE_HOME", &xdg_cache)
        .status()?;
    assert!(status.success());
    let forced_contents = fs::read_to_string(&config_path)?;
    assert!(forced_contents.contains("# Cage configuration generated"));

    Ok(())
}
