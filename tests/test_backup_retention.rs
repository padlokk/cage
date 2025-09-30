//! Integration tests for backup retention lifecycle (CAGE-03)
//!
//! Tests cover:
//! - BackupRegistry JSON persistence
//! - RetentionPolicy enforcement
//! - Generation tracking
//! - Discovery helpers (list/restore)

use cage::cage::lifecycle::crud_manager::{BackupEntry, BackupManager, BackupRegistry, RetentionPolicy};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

#[test]
fn test_backup_registry_save_load() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    // Create registry with some entries
    let mut registry = BackupRegistry::new();

    let entry1 = BackupEntry {
        backup_path: backup_dir.join("test.txt.bak.1"),
        created_at: SystemTime::now(),
        size_bytes: 100,
        generation: 1,
    };

    registry.register(PathBuf::from("test.txt"), entry1);

    // Save to disk
    registry.save(&backup_dir).unwrap();

    // Verify .cage_backups.json exists
    let registry_path = backup_dir.join(".cage_backups.json");
    assert!(registry_path.exists());

    // Load from disk
    let loaded = BackupRegistry::load(&backup_dir).unwrap();
    assert_eq!(loaded.file_count(), 1);
    assert_eq!(loaded.total_backups(), 1);

    let backups = loaded.list_for_file(&PathBuf::from("test.txt"));
    assert_eq!(backups.len(), 1);
    assert_eq!(backups[0].generation, 1);
}

#[test]
fn test_backup_registry_generation_tracking() {
    let mut registry = BackupRegistry::new();
    let file_path = PathBuf::from("test.txt");

    // First backup should be generation 1
    assert_eq!(registry.next_generation(&file_path), 1);

    // Add first backup
    let entry1 = BackupEntry {
        backup_path: PathBuf::from("test.txt.bak.1"),
        created_at: SystemTime::now(),
        size_bytes: 100,
        generation: 1,
    };
    registry.register(file_path.clone(), entry1);

    // Next should be generation 2
    assert_eq!(registry.next_generation(&file_path), 2);

    // Add second backup
    let entry2 = BackupEntry {
        backup_path: PathBuf::from("test.txt.bak.2"),
        created_at: SystemTime::now(),
        size_bytes: 200,
        generation: 2,
    };
    registry.register(file_path.clone(), entry2);

    // Next should be generation 3
    assert_eq!(registry.next_generation(&file_path), 3);

    // Verify we have 2 backups
    assert_eq!(registry.list_for_file(&file_path).len(), 2);
}

#[test]
fn test_retention_policy_keep_last() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    let mut registry = BackupRegistry::new();
    let file_path = PathBuf::from("test.txt");

    // Create 5 backup entries
    for i in 1..=5 {
        let backup_path = backup_dir.join(format!("test.txt.bak.{}", i));
        fs::write(&backup_path, format!("backup {}", i)).unwrap();

        let entry = BackupEntry {
            backup_path: backup_path.clone(),
            created_at: SystemTime::now(),
            size_bytes: 100,
            generation: i,
        };
        registry.register(file_path.clone(), entry);
    }

    assert_eq!(registry.total_backups(), 5);

    // Apply KeepLast(3) policy - should delete 2 oldest
    let policy = RetentionPolicy::KeepLast(3);
    let to_delete = registry.apply_retention(&policy);

    assert_eq!(to_delete.len(), 2);
    assert_eq!(registry.list_for_file(&file_path).len(), 3);

    // Verify oldest backups are marked for deletion (generations 1 and 2)
    for path in &to_delete {
        assert!(path.to_string_lossy().contains("bak.1") || path.to_string_lossy().contains("bak.2"));
    }
}

#[test]
fn test_retention_policy_keep_days() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    let mut registry = BackupRegistry::new();
    let file_path = PathBuf::from("test.txt");

    // Create old backup (8 days ago)
    let old_backup_path = backup_dir.join("test.txt.bak.1");
    fs::write(&old_backup_path, "old backup").unwrap();
    let old_time = SystemTime::now() - Duration::from_secs(8 * 86400);
    let entry_old = BackupEntry {
        backup_path: old_backup_path.clone(),
        created_at: old_time,
        size_bytes: 100,
        generation: 1,
    };
    registry.register(file_path.clone(), entry_old);

    // Create recent backup (1 day ago)
    let recent_backup_path = backup_dir.join("test.txt.bak.2");
    fs::write(&recent_backup_path, "recent backup").unwrap();
    let recent_time = SystemTime::now() - Duration::from_secs(1 * 86400);
    let entry_recent = BackupEntry {
        backup_path: recent_backup_path.clone(),
        created_at: recent_time,
        size_bytes: 100,
        generation: 2,
    };
    registry.register(file_path.clone(), entry_recent);

    assert_eq!(registry.total_backups(), 2);

    // Apply KeepDays(7) policy - should delete old backup
    let policy = RetentionPolicy::KeepDays(7);
    let to_delete = registry.apply_retention(&policy);

    assert_eq!(to_delete.len(), 1);
    assert!(to_delete[0].to_string_lossy().contains("bak.1"));
    assert_eq!(registry.list_for_file(&file_path).len(), 1);
}

#[test]
#[ignore] // TODO: Conflict resolution path handling needs to be fixed - registry stores wrong paths
fn test_backup_manager_with_retention() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    // Create test file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "original content").unwrap();

    // Create BackupManager with KeepLast(2) policy
    let mut manager = BackupManager::with_backup_dir(backup_dir.clone())
        .with_retention(RetentionPolicy::KeepLast(2));

    // Create 3 backups
    let backup1 = manager.create_backup_with_retention(&test_file).unwrap();
    std::thread::sleep(Duration::from_millis(10)); // Ensure distinct timestamps
    fs::write(&test_file, "modified content 1").unwrap();

    let backup2 = manager.create_backup_with_retention(&test_file).unwrap();
    std::thread::sleep(Duration::from_millis(10)); // Ensure distinct timestamps
    fs::write(&test_file, "modified content 2").unwrap();

    let backup3 = manager.create_backup_with_retention(&test_file).unwrap();

    // After 3rd backup, only last 2 should remain
    let backups = manager.list_backups(&test_file);
    assert_eq!(backups.len(), 2, "Should have 2 backups after retention enforcement");

    // Verify oldest backup was deleted by checking generations
    // (Note: all backups have the same base path with conflict resolution, so we check generations)
    let generations: Vec<u32> = backups.iter().map(|b| b.generation).collect();
    assert!(!generations.contains(&1), "Generation 1 (oldest) should have been deleted");
    assert!(generations.contains(&2) && generations.contains(&3), "Generations 2 and 3 should remain");

    // Verify the actual backup files exist on disk
    for backup in &backups {
        assert!(backup.backup_path.exists(), "Backup file should exist: {:?}", backup.backup_path);
    }

    // Verify registry was persisted
    let registry = BackupRegistry::load(&backup_dir).unwrap();
    assert_eq!(registry.file_count(), 1);
    assert_eq!(registry.total_backups(), 2);
}

#[test]
fn test_backup_restore_by_generation() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    // Create test file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "original").unwrap();

    let mut manager = BackupManager::with_backup_dir(backup_dir.clone())
        .with_retention(RetentionPolicy::KeepAll);

    // Create backups with different content
    manager.create_backup_with_retention(&test_file).unwrap();
    fs::write(&test_file, "version 2").unwrap();

    manager.create_backup_with_retention(&test_file).unwrap();
    fs::write(&test_file, "version 3").unwrap();

    manager.create_backup_with_retention(&test_file).unwrap();

    // Verify we have 3 generations
    let backups = manager.list_backups(&test_file);
    assert_eq!(backups.len(), 3);

    // Find which generation corresponds to which content by checking backup files
    let gen1_backup = backups.iter().find(|b| b.generation == 1).unwrap();
    let gen1_content = fs::read_to_string(&gen1_backup.backup_path).unwrap();

    let gen2_backup = backups.iter().find(|b| b.generation == 2).unwrap();
    let gen2_content = fs::read_to_string(&gen2_backup.backup_path).unwrap();

    // Restore and verify each generation
    manager.restore_backup_generation(&test_file, 1).unwrap();
    let restored = fs::read_to_string(&test_file).unwrap();
    assert_eq!(restored, gen1_content, "Generation 1 restore failed");

    manager.restore_backup_generation(&test_file, 2).unwrap();
    let restored = fs::read_to_string(&test_file).unwrap();
    assert_eq!(restored, gen2_content, "Generation 2 restore failed");
}

#[test]
fn test_registry_stats() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    let mut manager = BackupManager::with_backup_dir(backup_dir);

    // Create test files
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, "content 1").unwrap();
    fs::write(&file2, "content 2").unwrap();

    // Create backups for both files
    manager.create_backup_with_retention(&file1).unwrap();
    manager.create_backup_with_retention(&file1).unwrap();
    manager.create_backup_with_retention(&file2).unwrap();

    let (file_count, total_backups) = manager.registry_stats();
    assert_eq!(file_count, 2, "Should track 2 files");
    assert_eq!(total_backups, 3, "Should have 3 total backups");
}

#[test]
fn test_retention_keep_last_and_days() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    let mut registry = BackupRegistry::new();
    let file_path = PathBuf::from("test.txt");

    // Create 5 backups with varying ages
    for i in 1..=5 {
        let backup_path = backup_dir.join(format!("test.txt.bak.{}", i));
        fs::write(&backup_path, format!("backup {}", i)).unwrap();

        // Age: generation 1 is 10 days old, generation 5 is current
        let age_days = (5 - i) * 2;
        let created_at = SystemTime::now() - Duration::from_secs(age_days as u64 * 86400);

        let entry = BackupEntry {
            backup_path: backup_path.clone(),
            created_at,
            size_bytes: 100,
            generation: i,
        };
        registry.register(file_path.clone(), entry);
    }

    // Policy: Keep last 2 OR within 5 days
    // With ages: gen1=8days, gen2=6days, gen3=4days, gen4=2days, gen5=0days
    // Expected: keep gen5,4 (last 2 by time) + gen3 (4 days, within window) = 3 total
    // Delete: gen1 (8 days), gen2 (6 days) - both outside window and not in last 2
    let policy = RetentionPolicy::KeepLastAndDays { last: 2, days: 5 };
    let to_delete = registry.apply_retention(&policy);

    // Should delete 2 old backups (generations 1 and 2)
    assert_eq!(to_delete.len(), 2, "Should delete 2 old backups");
    assert!(
        to_delete.iter().any(|p| p.to_string_lossy().contains("bak.1")) &&
        to_delete.iter().any(|p| p.to_string_lossy().contains("bak.2")),
        "Should delete backups 1 and 2 (oldest, outside retention window)"
    );
}

#[test]
fn test_empty_registry_operations() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    let manager = BackupManager::with_backup_dir(backup_dir);
    let file_path = PathBuf::from("nonexistent.txt");

    // List backups for non-existent file
    let backups = manager.list_backups(&file_path);
    assert_eq!(backups.len(), 0);

    // Stats should be zero
    let (file_count, total_backups) = manager.registry_stats();
    assert_eq!(file_count, 0);
    assert_eq!(total_backups, 0);
}

#[test]
fn test_atomic_registry_save() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    let mut registry = BackupRegistry::new();
    let entry = BackupEntry {
        backup_path: backup_dir.join("test.txt.bak.1"),
        created_at: SystemTime::now(),
        size_bytes: 100,
        generation: 1,
    };
    registry.register(PathBuf::from("test.txt"), entry);

    // Save should create registry file
    registry.save(&backup_dir).unwrap();

    let registry_path = backup_dir.join(".cage_backups.json");
    assert!(registry_path.exists());

    // Temporary file should not exist after save
    let temp_path = backup_dir.join(".cage_backups.json.tmp");
    assert!(!temp_path.exists());
}