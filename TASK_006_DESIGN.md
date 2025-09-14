# TASK-006: Safe In-Place Operations Design

## 🎯 Goal
Enable `cage lock file.txt` to replace `file.txt` with encrypted version safely, with multiple layers of protection against data loss.

## 🛡️ Safety Architecture

### Layer 1: Explicit Opt-in Required
```bash
cage lock file.txt --in-place    # Required flag for in-place operations
```

### Layer 2: Recovery File Creation (Default)
```bash
# Creates file.txt.tmp.recover with:
# - Original filename
# - Passphrase used for encryption
# - Recovery instructions
# - Timestamp of operation
# - Command to reverse the operation
```

### Layer 3: Danger Mode (Skip Recovery)
```bash
cage lock file.txt --in-place --danger-mode    # Prompts for confirmation
```

### Layer 4: Environment Confirmation
```bash
DANGER_MODE=1 cage lock file.txt --in-place --danger-mode    # Still prompts
```

### Layer 5: Automation Override (Maximum Flags Required)
```bash
DANGER_MODE=1 cage lock file.txt --in-place --danger-mode --i-am-sure
```

## 🔧 Implementation Strategy

### Phase 1: Recovery File System
```rust
pub struct RecoveryManager {
    create_recovery: bool,
    danger_mode: bool,
}

impl RecoveryManager {
    pub fn create_recovery_file(&self, original: &Path, passphrase: &str, operation: &str) -> Result<PathBuf> {
        let recovery_path = original.with_extension("tmp.recover");
        let content = format!(r#"# CAGE RECOVERY INFORMATION
# Generated: {}
# Original: {}
# Operation: {}
# Passphrase: {}
#
# TO RECOVER YOUR FILE:
# cage unlock {} {}
#
# DELETE THIS FILE ONCE YOU'VE VERIFIED YOUR ENCRYPTION!
# This file contains your passphrase and is a security risk if left around.
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            original.display(),
            operation,
            passphrase,
            original.display(),
            passphrase
        );

        std::fs::write(&recovery_path, content)?;
        Ok(recovery_path)
    }
}
```

### Phase 2: Atomic In-Place Operations
```rust
pub struct InPlaceOperation {
    original: PathBuf,
    temp_encrypted: PathBuf,
    recovery_file: Option<PathBuf>,
    completed: bool,
}

impl InPlaceOperation {
    pub fn new(file: &Path) -> Self {
        Self {
            original: file.to_path_buf(),
            temp_encrypted: file.with_extension("tmp.cage"),
            recovery_file: None,
            completed: false,
        }
    }

    pub fn execute_lock(&mut self, passphrase: &str, options: &InPlaceOptions) -> AgeResult<()> {
        // 1. Create recovery file if enabled
        if !options.danger_mode {
            let recovery_manager = RecoveryManager::new();
            self.recovery_file = Some(recovery_manager.create_recovery_file(
                &self.original,
                passphrase,
                "encrypt"
            )?);
        }

        // 2. Encrypt original -> temp
        encrypt_file(&self.original, &self.temp_encrypted, passphrase)?;

        // 3. Verify temp file
        verify_encrypted_file(&self.temp_encrypted)?;

        // 4. Preserve metadata
        copy_metadata(&self.original, &self.temp_encrypted)?;

        // 5. Atomic replace (this is the dangerous moment)
        std::fs::rename(&self.temp_encrypted, &self.original)?;

        self.completed = true;
        Ok(())
    }
}

impl Drop for InPlaceOperation {
    fn drop(&mut self) {
        if !self.completed {
            // Rollback: remove temp file if operation failed
            if self.temp_encrypted.exists() {
                let _ = std::fs::remove_file(&self.temp_encrypted);
            }

            // Remove recovery file if operation failed
            if let Some(ref recovery) = self.recovery_file {
                if recovery.exists() {
                    let _ = std::fs::remove_file(recovery);
                }
            }
        }
    }
}
```

### Phase 3: Safety Prompts and Validation
```rust
pub struct SafetyValidator {
    danger_mode: bool,
    i_am_sure: bool,
    env_danger: bool,
}

impl SafetyValidator {
    pub fn new(args: &Args) -> Self {
        Self {
            danger_mode: args.has("--danger-mode"),
            i_am_sure: args.has("--i-am-sure"),
            env_danger: std::env::var("DANGER_MODE").map(|v| v == "1").unwrap_or(false),
        }
    }

    pub fn validate_in_place_operation(&self, file: &Path) -> AgeResult<()> {
        // Check if file exists and will be replaced
        if !file.exists() {
            return Err(AgeError::FileError {
                operation: "in-place".to_string(),
                path: file.to_path_buf(),
                source: io::Error::new(io::ErrorKind::NotFound, "File not found"),
            });
        }

        // Danger mode validation
        if self.danger_mode {
            if !self.env_danger {
                return Err(AgeError::InvalidOperation {
                    operation: "in-place-danger".to_string(),
                    reason: "DANGER_MODE=1 environment variable required with --danger-mode".to_string(),
                });
            }

            if !self.i_am_sure {
                // Prompt for confirmation
                eprintln!("⚠️  DANGER MODE: This action is UNRECOVERABLE!");
                eprintln!("   File: {}", file.display());
                eprintln!("   No recovery file will be created.");
                eprintln!("   If encryption fails or you forget the passphrase, your file is LOST FOREVER.");
                eprintln!();
                eprint!("Type 'DELETE MY FILE' to confirm this unrecoverable action: ");

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim() != "DELETE MY FILE" {
                    return Err(AgeError::InvalidOperation {
                        operation: "in-place-danger".to_string(),
                        reason: "User cancelled dangerous operation".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}
```

## 🎛️ Command Line Interface Changes

### New Flags
- `--in-place` - Enable in-place file replacement (required)
- `--danger-mode` - Skip recovery file creation (dangerous)
- `--i-am-sure` - Skip confirmation prompts (automation)

### Usage Examples
```bash
# Safe in-place (creates recovery file)
cage lock file.txt --in-place

# Dangerous (requires env var + confirmation)
DANGER_MODE=1 cage lock file.txt --in-place --danger-mode

# Automation (requires all flags)
DANGER_MODE=1 cage lock file.txt --in-place --danger-mode --i-am-sure
```

## 🧪 Testing Strategy

### Test Cases
1. **Normal in-place** - creates recovery file, encrypts correctly
2. **Failed encryption** - rollback preserves original file
3. **Danger mode prompts** - requires proper confirmation
4. **Recovery file contents** - contains correct passphrase and instructions
5. **Metadata preservation** - permissions, timestamps maintained
6. **Atomic operations** - no partial states visible to other processes

### Error Scenarios
1. **Insufficient permissions** - graceful failure with clear error
2. **Disk full** - rollback removes temp files
3. **Process interruption** - Drop trait cleans up temp files
4. **Invalid passphrase** - detected before file replacement

## 🚨 Security Considerations

### Recovery File Security
- Contains plaintext passphrase (necessary for recovery)
- Created with restrictive permissions (600)
- User warned to delete after verification
- Contains clear instructions for cleanup

### Attack Vectors Mitigated
- **Accidental data loss** - Recovery file provides fallback
- **Process interruption** - Drop trait ensures cleanup
- **Partial operations** - Atomic rename prevents corruption
- **Metadata loss** - Explicit preservation of file attributes

## 📊 User Experience Flow

```
User: cage lock file.txt --in-place
System: 🔐 Enter passphrase for encryption: [hidden input]
System: 📝 Creating recovery file: file.txt.tmp.recover
System: 🔄 Encrypting file.txt in-place...
System: ✅ Encryption complete. Recovery info saved to file.txt.tmp.recover
System: ⚠️  Delete recovery file once you've verified encryption worked!
```

This design provides maximum safety while still enabling the requested in-place functionality. The multiple layers ensure users can't accidentally destroy their data, while still allowing automation for those who really need it.