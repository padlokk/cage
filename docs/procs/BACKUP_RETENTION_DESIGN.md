# Backup & Recovery Retention Policy Design (CAGE-03 Follow-up)

## Current State

**Existing Implementation** (`src/cage/lifecycle/crud_manager.rs:83-233`):
- `BackupManager` with basic backup/restore/cleanup operations
- Conflict handling via timestamped `.conflict.{timestamp}` files
- Single-file backup on lock operations when `backup_before_lock: true`
- Auto-cleanup on success (controlled by `cleanup_on_success`)

**Gaps:**
1. No retention policy for old backups
2. No automatic cleanup of conflict files
3. Backup directory not wired to config/CLI
4. No multi-run conflict strategy
5. No backup discovery/listing functionality

---

## Design Goals

1. **Retention Policy**: Automatically manage backup lifecycle
2. **Conflict Resolution**: Handle multiple backup generations gracefully
3. **Configuration**: User-configurable via `AgeConfig` and CLI
4. **Discovery**: List/query existing backups
5. **Cleanup**: Periodic purging of old backups based on policy

---

## Proposed Architecture

### 1. Retention Policy Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionPolicy {
    /// Keep all backups indefinitely (manual cleanup only)
    KeepAll,

    /// Keep backups for N days, then auto-delete
    KeepDays(u32),

    /// Keep only the last N backups per file
    KeepLast(usize),

    /// Combined: keep last N backups AND those within M days
    KeepLastAndDays { last: usize, days: u32 },
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        // Conservative default: keep last 3 backups
        RetentionPolicy::KeepLast(3)
    }
}
```

### 2. Backup Metadata Registry

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRegistry {
    /// Map of original file path to list of backup entries
    backups: HashMap<PathBuf, Vec<BackupEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub backup_path: PathBuf,
    pub created_at: SystemTime,
    pub size_bytes: u64,
    pub generation: u32,  // 1 = most recent, 2 = previous, etc.
}

impl BackupRegistry {
    /// Load registry from disk (stored as .cage_backups.json)
    pub fn load(backup_dir: &Path) -> AgeResult<Self>;

    /// Save registry to disk
    pub fn save(&self, backup_dir: &Path) -> AgeResult<()>;

    /// Register a new backup
    pub fn register(&mut self, original: PathBuf, backup: BackupEntry);

    /// Apply retention policy and return backups to delete
    pub fn apply_retention(&mut self, policy: &RetentionPolicy) -> Vec<PathBuf>;

    /// List all backups for a file
    pub fn list_for_file(&self, file: &Path) -> Vec<&BackupEntry>;
}
```

### 3. Enhanced BackupManager

```rust
pub struct BackupManager {
    backup_dir: Option<PathBuf>,
    backup_extension: String,
    cleanup_on_success: bool,
    retention_policy: RetentionPolicy,    // NEW
    registry: BackupRegistry,              // NEW
}

impl BackupManager {
    /// Create backup with retention enforcement
    pub fn create_backup_with_retention(&mut self, file_path: &Path) -> AgeResult<BackupInfo> {
        // 1. Create backup
        let backup_info = self.create_backup(file_path)?;

        // 2. Register in registry
        self.registry.register(
            file_path.to_path_buf(),
            BackupEntry {
                backup_path: backup_info.backup_path.clone(),
                created_at: backup_info.created_at,
                size_bytes: backup_info.size_bytes,
                generation: self.registry.next_generation(file_path),
            },
        );

        // 3. Apply retention policy
        let to_delete = self.registry.apply_retention(&self.retention_policy);
        for old_backup in to_delete {
            self.cleanup_old_backup(&old_backup)?;
        }

        // 4. Save registry
        if let Some(ref dir) = self.backup_dir {
            self.registry.save(dir)?;
        }

        Ok(backup_info)
    }

    /// Cleanup old backup by path
    fn cleanup_old_backup(&self, path: &Path) -> AgeResult<()>;

    /// List all backups for a file
    pub fn list_backups(&self, file: &Path) -> Vec<BackupEntry>;

    /// Restore from specific backup generation (1 = latest, 2 = previous, etc.)
    pub fn restore_backup_generation(&self, file: &Path, generation: u32) -> AgeResult<()>;
}
```

### 4. Configuration Integration

**Update `AgeConfig`** (`src/cage/config.rs`):
```rust
pub struct AgeConfig {
    // ... existing fields ...

    // NEW backup configuration
    pub backup_dir: Option<PathBuf>,
    pub backup_retention: RetentionPolicy,
    pub backup_extension: String,
}

impl Default for AgeConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            backup_dir: None,
            backup_retention: RetentionPolicy::default(),
            backup_extension: ".bak".to_string(),
        }
    }
}
```

### 5. CLI Integration

**New CLI commands** (future `src/bin/cli_age.rs`):
```bash
# List backups for a file
cage backup list <file>

# Restore from specific generation
cage backup restore <file> --generation 2

# Manually cleanup old backups
cage backup cleanup [--dry-run]

# Configure retention policy
cage config set backup.retention keep-last=5
cage config set backup.retention keep-days=30
```

---

## Implementation Plan

### Phase 1: Core Retention Logic (2-3 hours)
- [ ] Add `RetentionPolicy` enum to `crud_manager.rs`
- [ ] Implement retention calculation logic
- [ ] Add unit tests for retention policies

### Phase 2: Backup Registry (3-4 hours)
- [ ] Create `BackupRegistry` struct with JSON serialization
- [ ] Implement load/save from `.cage_backups.json`
- [ ] Add registry management methods
- [ ] Add integration tests

### Phase 3: BackupManager Enhancement (2-3 hours)
- [ ] Add `retention_policy` and `registry` fields
- [ ] Implement `create_backup_with_retention()`
- [ ] Add generation-based restore
- [ ] Update existing usages in `lock_single_file`

### Phase 4: Configuration Wiring (1-2 hours)
- [ ] Add backup config fields to `AgeConfig`
- [ ] Wire config to `BackupManager` construction
- [ ] Add config validation

### Phase 5: CLI Commands (3-4 hours) - DEFERRED
- [ ] Implement `cage backup list`
- [ ] Implement `cage backup restore`
- [ ] Implement `cage backup cleanup`

**Total Estimated Effort**: 11-16 hours (excluding CLI commands)

---

## Retention Policy Examples

### Example 1: Keep Last 3
```rust
RetentionPolicy::KeepLast(3)
```
- Keeps: backup.txt.bak.1, backup.txt.bak.2, backup.txt.bak.3
- Deletes: backup.txt.bak.4 and older

### Example 2: Keep 7 Days
```rust
RetentionPolicy::KeepDays(7)
```
- Keeps: All backups < 7 days old
- Deletes: All backups >= 7 days old

### Example 3: Combined Strategy
```rust
RetentionPolicy::KeepLastAndDays { last: 3, days: 30 }
```
- Always keeps last 3 backups (even if > 30 days)
- Plus any additional backups within 30 days
- Deletes: Backups beyond 3 generations AND older than 30 days

---

## Conflict Resolution Strategy

### Current Behavior
When backup file exists, rename to `.conflict.{timestamp}`

### Enhanced Behavior
1. Check registry for existing backups
2. Increment generation number
3. Name backups with generation: `file.txt.bak.1`, `file.txt.bak.2`, etc.
4. Apply retention policy immediately
5. Move expired backups to archive dir (optional) before deletion

---

## Security & Safety Considerations

1. **Atomic Operations**: Registry updates must be atomic (write to temp, then rename)
2. **Permissions**: Preserve file permissions on backups
3. **Disk Space**: Add optional disk space limits
4. **Audit Trail**: Log all backup creations and deletions
5. **Recovery**: Keep registry backups (`.cage_backups.json.backup`)

---

## Testing Strategy

1. **Unit Tests**:
   - RetentionPolicy calculation logic
   - BackupRegistry operations
   - Generation numbering

2. **Integration Tests**:
   - Create multiple backups, verify retention
   - Restore from specific generations
   - Registry persistence across sessions

3. **Edge Cases**:
   - Corrupted registry file
   - Missing backup files referenced in registry
   - Concurrent backup operations

---

## Migration Path

**Backward Compatibility**:
- Existing `.bak` files continue to work
- Registry is optional (creates on first enhanced backup)
- Old conflict files (`.conflict.{timestamp}`) detected and migrated

**Migration Process**:
1. Detect old backup files without registry
2. Create initial registry from filesystem scan
3. Assign generation numbers based on mtime
4. Save registry

---

## Open Questions

1. Should we support compression for old backups?
2. Should we add backup verification (checksum)?
3. Should registry be global or per-directory?
4. Should we support remote backup locations?

---

**Status**: Design phase - awaiting implementation
**Priority**: P1 (High) - needed before CAGE-04 safety layers
**Dependencies**: None (self-contained enhancement)
**Related**: CAGE-04 (in-place safety), CAGE-06 (config file support)