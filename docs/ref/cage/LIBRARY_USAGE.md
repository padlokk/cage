# 📚 Cage Library Documentation

**Version**: 0.5.0
**Updated**: 2025-09-28
**API Stability**: Production Ready (request API preferred)

Cage provides a comprehensive Rust library for Age encryption automation with advanced features including in-place operations, progress tracking, and PTY automation.

---

## 🎯 Quick Start

### Adding Cage to Your Project

```toml
[dependencies]
cage = { path = "path/to/cage" }
# Or when published to crates.io:
# cage = "0.3.1"
```

### Basic Encryption Example (Request API)

```rust
use cage::cage::{CrudManager, OutputFormat};
use cage::cage::requests::{LockRequest, Identity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut crud_manager = CrudManager::with_defaults()?;

    let request = LockRequest::new(
        std::path::PathBuf::from("secret.txt"),
        Identity::Passphrase("mypassword".to_string())
    )
    .with_format(OutputFormat::Binary)
    .backup(true);

    let result = crud_manager.lock_with_request(&request)?;
    println!("Encrypted {} files", result.processed_files.len());
    Ok(())
}
```

> Legacy code that calls `lock(&path, passphrase, LockOptions)` still works, but new
> development should prefer the typed request API for parity with the CLI.

---

## 🏗️ Core Modules

### 1. CrudManager + Request API

The primary interface now accepts typed request structs that mirror CLI features.

```rust
use cage::cage::{CrudManager, OutputFormat};
use cage::cage::requests::{LockRequest, UnlockRequest, Identity};

let mut crud_manager = CrudManager::with_defaults()?;

let lock_request = LockRequest::new(
    std::path::PathBuf::from("docs/"),
    Identity::Passphrase("secret123".to_string())
)
.recursive(true)
.with_format(OutputFormat::AsciiArmor)
.backup(true);

let lock_result = crud_manager.lock_with_request(&lock_request)?;

let unlock_request = UnlockRequest::new(
    std::path::PathBuf::from("docs/"),
    Identity::Passphrase("secret123".to_string())
)
.selective(true)
.preserve_encrypted(true);

let unlock_result = crud_manager.unlock_with_request(&unlock_request)?;
```

#### Legacy API (Still Available)

For existing integrations you can continue using `lock(&path, passphrase, LockOptions)` and
`unlock(&path, passphrase, UnlockOptions)`. Those internally delegate to the same logic used by request structs.

```rust
use cage::cage::{CrudManager, LockOptions, OutputFormat};

let mut crud_manager = CrudManager::with_defaults()?;

let legacy_options = LockOptions {
    recursive: false,
    pattern_filter: None,
    backup_before_lock: true,
    format: OutputFormat::Binary,
};

let result = crud_manager.lock(&std::path::Path::new("legacy.txt"), "pass", legacy_options)?;
```

### 2. Streaming Encryption/Decryption

`ShellAdapterV2` now supports both staged (temp-file) and pipe-based streaming. The adapter selects a strategy at runtime using `CAGE_STREAMING_STRATEGY`, the CLI `--streaming-strategy` flag, or the `[streaming]` section in `cage.toml`.

#### Strategy Selection

- **Passphrase-based operations** always use temp-file staging due to age's TTY requirements
  - Performance: ~100-150 MB/s for large files
  - This is a fundamental limitation of the age binary's security model
- **Recipient-based encryption** (`Recipient::PublicKey`, `Recipient::MultipleKeys`, `Recipient::RecipientsFile`, `Recipient::SshRecipients`) supports true pipe streaming
  - Performance: ~400-500 MB/s when using `pipe` or `auto` strategy
- **Identity-driven decryption** (`Identity::IdentityFile` or `Identity::SshKey`) also supports pipe streaming

#### Performance Guidelines

| Operation | Strategy | Throughput | Memory Usage | Recommendation |
|-----------|----------|------------|--------------|----------------|
| Passphrase encrypt/decrypt | temp | ~100-150 MB/s | Temp file on disk | Use for files < 100MB |
| Recipient encrypt | pipe | ~400-500 MB/s | Constant memory | Preferred for all sizes |
| File operations | N/A | ~600 MB/s | Minimal | Best for files > 100MB with passphrases |

`ShellAdapterV2::capabilities()` exposes detailed metadata through `AdapterCapabilities::streaming_strategies`, allowing callers to confirm the default strategy, pipe availability, and prerequisites before opting in.

```rust
use cage::cage::adapter_v2::{AgeAdapterV2, ShellAdapterV2};
use cage::cage::config::OutputFormat;
use cage::cage::requests::{Identity, Recipient};

let adapter = ShellAdapterV2::new()?;

// Opt into pipe streaming for recipient flows
std::env::set_var("CAGE_STREAMING_STRATEGY", "pipe");

let recipients = vec![Recipient::PublicKey("age1exampleKey".into())];
let mut plaintext = std::io::Cursor::new(b"stream me".to_vec());
let mut ciphertext = Vec::new();
adapter.encrypt_stream(
    &mut plaintext,
    &mut ciphertext,
    &Identity::Passphrase("topsecret".into()),
    Some(&recipients),
    OutputFormat::Binary,
)?;

let identity_path = std::path::PathBuf::from("/tmp/identity.txt"); // update with your identity file
let mut cipher_cursor = std::io::Cursor::new(ciphertext);
let mut recovered = Vec::new();
adapter.decrypt_stream(
    &mut cipher_cursor,
    &mut recovered,
    &Identity::IdentityFile(identity_path),
)?;
println!("Recovered: {}", String::from_utf8_lossy(&recovered));
```

> **Fallback behaviour:** Even in `auto`, the adapter falls back to temp files when prerequisites are missing (no recipients or identity) or when the Age CLI returns an error.

> **Configuration shortcut:** Set `[streaming]
strategy = "pipe"` in `~/.config/cage/config.toml` (or the path referenced by `CAGE_CONFIG`) to make pipe streaming the default.

> **Requirements:** Streaming helpers expect the `age` binary to be installed and visible on `PATH`. Test suites skip automatically when the binary is absent.

### 2. Progress Framework

Professional progress indicators with multiple styles and terminal features.

```rust
use cage::cage::progress::{
    ProgressManager, ProgressStyle, TerminalReporter
};
use std::sync::Arc;

// Create progress manager
let manager = ProgressManager::new();
manager.add_reporter(Arc::new(TerminalReporter::new()));

// Different progress styles
let spinner = manager.start_task("Loading", ProgressStyle::Spinner);
let bar = manager.start_task("Processing", ProgressStyle::Bar { total: 100 });
let counter = manager.start_task("Steps", ProgressStyle::Counter { total: 5 });
let bytes = manager.start_task("Transfer", ProgressStyle::Bytes { total_bytes: 1048576 });

// Update progress
bar.update(50, "Halfway done");
counter.increment("Step completed");
bytes.update(524288, "50% transferred");

// Complete tasks
spinner.complete("Loading finished");
bar.complete("Processing complete");
```

#### Progress Styles

- **Spinner**: Animated Unicode spinner for indeterminate operations
- **Bar**: Progress bar with percentage, ETA, and custom messages
- **Counter**: Step-by-step counter for discrete operations
- **Bytes**: Byte transfer progress with rate calculation

#### Terminal Features

```rust
use cage::cage::progress::{TerminalReporter, TerminalConfig};

let config = TerminalConfig {
    use_colors: true,         // Enable ANSI colors
    use_unicode: true,        // Unicode spinners vs ASCII fallback
    use_stderr: false,        // Output to stdout vs stderr
    update_interval_ms: 50,   // Update throttling
    clear_on_complete: true,  // Clear progress on completion
};

let reporter = TerminalReporter::with_config(config);
```

### 3. In-Place Operations with Safety

Multi-layered safety architecture for in-place file operations.

```rust
use cage::cage::{SafetyValidator, InPlaceOperation, RecoveryManager};

// Safety validation
let safety_validator = SafetyValidator::new(
    false,  // danger_mode
    false   // i_am_sure
)?;

safety_validator.validate_in_place_operation(&path)?;

// Recovery manager for backup creation
let recovery_manager = RecoveryManager::new(true, false)?; // create_recovery, danger_mode

// In-place operation
let mut in_place_op = InPlaceOperation::new(&path);

// Execute with safety and recovery
let result = in_place_op.execute(|| {
    crud_manager.lock(&path, passphrase, options)
})?;

println!("In-place operation completed: {:?}", result);
```

#### Safety Layers

1. **Explicit Opt-in**: Must explicitly enable in-place operations
2. **Recovery Files**: Automatic `.tmp.recover` file creation
3. **Danger Mode**: Additional validation for destructive operations
4. **Environment Checks**: `DANGER_MODE=1` environment variable requirement
5. **Automation Override**: `--i-am-sure` flag for scripted operations

### 4. PTY Automation

Direct Age binary automation with pseudo-terminal support.

```rust
use cage::cage::pty_wrap::PtyAgeAutomator;

let automator = PtyAgeAutomator::new()?;

// Execute Age command with automated passphrase input
let result = automator.execute_age_command(
    &["--encrypt", "--passphrase", "input.txt"],
    Some("mypassword"),  // Passphrase for automation
    30000               // Timeout in milliseconds
)?;

match result.exit_code {
    0 => println!("Success: {}", result.output),
    _ => eprintln!("Failed: {}", result.error),
}
```

#### Advanced PTY Features

```rust
// Custom Age binary path
let automator = PtyAgeAutomator::with_age_path("/custom/path/to/age")?;

// Interactive mode (no passphrase)
let result = automator.execute_age_command(
    &["--decrypt", "file.age"],
    None,    // No passphrase - user will be prompted
    60000    // Longer timeout for user input
)?;
```

### 5. Passphrase Management

Secure passphrase handling with multiple input modes.

```rust
use cage::cage::{PassphraseManager, PassphraseMode};

let passphrase_manager = PassphraseManager::new();

// Interactive terminal input (secure, hidden)
let passphrase = passphrase_manager.get_passphrase_with_mode(
    "Enter passphrase: ",
    false,  // confirm (ask twice)
    PassphraseMode::Interactive
)?;

// Stdin input for automation
let passphrase = passphrase_manager.get_passphrase_with_mode(
    "Passphrase: ",
    false,
    PassphraseMode::Stdin
)?;

// Environment variable
let passphrase = passphrase_manager.get_passphrase_with_mode(
    "",
    false,
    PassphraseMode::Environment("CAGE_PASSPHRASE".to_string())
)?;
```

---

## 🔧 Advanced Usage Patterns

### 1. Integrated Progress with Operations

```rust
use cage::cage::progress::{ProgressManager, ProgressStyle, TerminalReporter};
use std::sync::Arc;

fn encrypt_with_progress(files: Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let manager = ProgressManager::new();
    manager.add_reporter(Arc::new(TerminalReporter::new()));

    let task = manager.start_task(
        "Encrypting files",
        ProgressStyle::Bar { total: files.len() as u64 }
    );

    let mut crud_manager = CrudManager::with_defaults()?;
    let options = LockOptions::default();

    for (i, file) in files.iter().enumerate() {
        task.update(i as u64 + 1, &format!("Processing {}", file.display()));

        let result = crud_manager.lock(file, "passphrase", options.clone())?;

        // Handle result...
    }

    task.complete("All files encrypted successfully");
    Ok(())
}
```

### 2. Batch Operations with Error Handling

```rust
use cage::cage::{CrudManager, LockOptions};

fn batch_encrypt_with_recovery(
    files: Vec<PathBuf>,
    passphrase: &str
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut crud_manager = CrudManager::with_defaults()?;
    let options = LockOptions::default();
    let mut errors = Vec::new();

    for file in files {
        match crud_manager.lock(&file, passphrase, options.clone()) {
            Ok(result) => {
                println!("✓ Encrypted: {}", file.display());
            }
            Err(e) => {
                let error_msg = format!("✗ Failed {}: {}", file.display(), e);
                eprintln!("{}", error_msg);
                errors.push(error_msg);
            }
        }
    }

    if errors.is_empty() {
        Ok(vec!["All files processed successfully".to_string()])
    } else {
        Ok(errors)
    }
}
```

### 3. Custom Progress Reporter

```rust
use cage::cage::progress::{ProgressReporter, ProgressEvent};
use std::sync::{Arc, Mutex};

struct CustomReporter {
    log_file: Arc<Mutex<std::fs::File>>,
}

impl ProgressReporter for CustomReporter {
    fn report(&self, event: &ProgressEvent) {
        let mut file = self.log_file.lock().unwrap();
        writeln!(file, "[{}] Task {}: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            event.task_id,
            event.message
        ).ok();
    }
}

// Usage
let manager = ProgressManager::new();
let custom_reporter = Arc::new(CustomReporter::new("/var/log/progress.log")?);
manager.add_reporter(custom_reporter);
```

### 4. Configuration Management

#### Loading and Inspecting Configuration

Cage provides a comprehensive configuration system through `AgeConfig`:

```rust
use cage::cage::config::AgeConfig;

// Load configuration from default paths
let config = AgeConfig::load_default()?;

// Access configuration values
println!("Output format: {:?}", config.output_format);
println!("Streaming strategy: {:?}", config.streaming_strategy);
println!("Backup directory: {:?}", config.backup_directory);

// Get configuration source path (if loaded from file)
if let Some(path) = &config.source_path {
    println!("Loaded from: {}", path.display());
}

// Get all configuration search paths
let search_paths = AgeConfig::get_config_search_paths();
for path in &search_paths {
    println!("Search path: {}", path.display());
}

// Format configuration layers for display
println!("{}", config.format_layers());
```

The configuration system checks these paths in order:
1. `CAGE_CONFIG` environment variable (if set)
2. `$XDG_CONFIG_HOME/cage/config.toml`
3. `$HOME/.config/cage/config.toml`
4. `./cage.toml`

#### Telemetry & Audit Logging Configuration

Cage provides structured telemetry for machine-readable audit trails, critical for Padlock/Ignite integration:

```rust
use cage::cage::config::{AgeConfig, TelemetryFormat};

// Create config with JSON telemetry
let mut config = AgeConfig::default();
config.telemetry_format = TelemetryFormat::Json;
config.audit_log_path = Some("/var/log/cage/audit.json".to_string());

let mut crud_manager = CrudManager::with_config(config)?;
```

**Telemetry Formats:**
- `TelemetryFormat::Text` - Human-readable log format (default)
- `TelemetryFormat::Json` - Machine-readable JSON lines for log ingestion

**JSON Event Types:**
- `operation_start` - Operation initiated
- `operation_complete` - Operation finished (includes processed file list, timing)
- `encryption` - File encrypted (includes recipient count, group hash, streaming strategy, authority tier)
- `decryption` - File decrypted (includes identity type, streaming strategy)
- `status_check` - Repository status checked
- `authority_operation` - Authority chain operation

**Example JSON Output:**
```json
{
  "timestamp": "2025-09-29T21:50:48Z",
  "level": "INFO",
  "component": "cage_automation",
  "event_type": "encryption",
  "path": "/secret.txt",
  "identity_type": "age_identity",
  "recipient_count": 2,
  "recipient_group_hash": "24a003292f7c8b35f07122d4ff0e1fd9",
  "streaming_strategy": "pipe",
  "authority_tier": "M",
  "success": true
}
```

**Security Notes:**
- Recipient public keys are hashed (MD5) for audit trails, not logged in plaintext
- Passphrases are never logged
- All sensitive fields are automatically redacted

**Configuration via TOML:**
```toml
[telemetry]
format = "json"  # or "text"
audit_log_path = "/var/log/cage/audit.json"
```

#### Recipient Groups (Experimental Roadmap)

Recipient group persistence is planned as an optional convenience feature (see task `CONFIG-01`). The current release exposes runtime helpers via `RecipientGroup`/`MultiRecipientConfig`, but durable storage will be formalized later through the JSON backup registry (CAGE-03). For now, treat any TOML examples as experimental and subject to change.

### 5. Configuration-Driven Operations

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct EncryptionConfig {
    passphrase: String,
    recursive: bool,
    backup: bool,
    format: String,
    in_place: bool,
    danger_mode: bool,
}

fn encrypt_from_config(config_path: &str, target: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_str = std::fs::read_to_string(config_path)?;
    let config: EncryptionConfig = serde_json::from_str(&config_str)?;

    let mut crud_manager = CrudManager::with_defaults()?;
    let options = LockOptions {
        recursive: config.recursive,
        backup: config.backup,
        format: match config.format.as_str() {
            "ascii" => OutputFormat::Ascii,
            _ => OutputFormat::Binary,
        },
        pattern: None,
        preserve_encrypted: false,
        audit_log: None,
    };

    if config.in_place {
        // Use in-place operations
        let safety_validator = SafetyValidator::new(config.danger_mode, false)?;
        safety_validator.validate_in_place_operation(&PathBuf::from(target))?;

        let mut in_place_op = InPlaceOperation::new(&PathBuf::from(target));
        in_place_op.execute(|| {
            crud_manager.lock(&PathBuf::from(target), &config.passphrase, options)
        })?;
    } else {
        // Regular operations
        crud_manager.lock(&PathBuf::from(target), &config.passphrase, options)?;
    }

    Ok(())
}
```

---

## 🔍 Error Handling

Cage provides comprehensive error types with detailed context.

```rust
use cage::cage::error::CageError;

match crud_manager.lock(&path, passphrase, options) {
    Ok(result) => {
        println!("Success: {} files processed", result.processed_files.len());
    }
    Err(CageError::FileNotFound(path)) => {
        eprintln!("File not found: {}", path.display());
    }
    Err(CageError::PermissionDenied(msg)) => {
        eprintln!("Permission denied: {}", msg);
    }
    Err(CageError::AgeError(msg)) => {
        eprintln!("Age encryption error: {}", msg);
    }
    Err(CageError::SecurityViolation(msg)) => {
        eprintln!("Security violation: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

---

## 🧪 Testing Your Integration

### Unit Testing with Cage

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_basic_encryption() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "secret content")?;

        let mut crud_manager = CrudManager::with_defaults()?;
        let options = LockOptions::default();

        let result = crud_manager.lock(&test_file, "testpassword", options)?;
        assert_eq!(result.processed_files.len(), 1);

        // Verify encrypted file exists
        assert!(test_file.with_extension("txt.cage").exists());

        Ok(())
    }

    #[test]
    fn test_progress_integration() {
        let manager = ProgressManager::new();
        let task = manager.start_task("Test", ProgressStyle::Bar { total: 3 });

        task.update(1, "Step 1");
        task.update(2, "Step 2");
        task.complete("Done");

        assert_eq!(task.state(), ProgressState::Complete);
    }
}
```

---

## 📈 Performance Considerations

### 1. Progress Update Throttling

Progress updates are automatically throttled to prevent performance impact:

```rust
// Progress updates limited to 50ms intervals by default
let config = TerminalConfig {
    update_interval_ms: 100,  // Reduce update frequency for better performance
    ..Default::default()
};
```

### 2. Memory Management

Cage handles large files efficiently with streaming operations:

```rust
// Large file operations use streaming to minimize memory usage
let options = LockOptions {
    // Cage automatically handles chunked processing for large files
    ..Default::default()
};
```

### 3. Concurrent Operations

Use multiple progress tasks for concurrent operations:

```rust
let manager = Arc::new(ProgressManager::new());

// Spawn multiple concurrent encryption tasks
let handles: Vec<_> = files.into_iter().enumerate().map(|(i, file)| {
    let manager = manager.clone();
    std::thread::spawn(move || {
        let task = manager.start_task(
            &format!("Worker {}", i),
            ProgressStyle::Spinner
        );
        // Perform encryption work
        task.complete("Worker finished");
    })
}).collect();

// Wait for all workers
for handle in handles {
    handle.join().unwrap();
}
```

---

## 🔗 Integration Examples

### Web Service Integration

```rust
use warp::Filter;

async fn encrypt_endpoint(
    file_data: bytes::Bytes,
    passphrase: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|_| warp::reject::custom(ServerError))?;

    std::fs::write(&temp_file, &file_data)
        .map_err(|_| warp::reject::custom(ServerError))?;

    let mut crud_manager = CrudManager::with_defaults()
        .map_err(|_| warp::reject::custom(ServerError))?;

    let options = LockOptions::default();
    let result = crud_manager.lock(temp_file.path(), &passphrase, options)
        .map_err(|_| warp::reject::custom(ServerError))?;

    // Return encrypted data
    Ok(warp::reply::json(&result))
}
```

### CLI Tool Integration

```rust
use clap::{App, Arg};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("my-encryption-tool")
        .arg(Arg::with_name("file").required(true))
        .arg(Arg::with_name("progress").long("progress"))
        .get_matches();

    let file = matches.value_of("file").unwrap();
    let show_progress = matches.is_present("progress");

    if show_progress {
        let manager = ProgressManager::new();
        manager.add_reporter(Arc::new(TerminalReporter::new()));

        let task = manager.start_task("Encrypting", ProgressStyle::Spinner);

        // Perform encryption with progress
        let mut crud_manager = CrudManager::with_defaults()?;
        crud_manager.lock(&PathBuf::from(file), "password", LockOptions::default())?;

        task.complete("Encryption finished");
    } else {
        // Perform encryption without progress
        let mut crud_manager = CrudManager::with_defaults()?;
        crud_manager.lock(&PathBuf::from(file), "password", LockOptions::default())?;
    }

    Ok(())
}
```

---

## 📚 API Reference

For complete API documentation, run:

```bash
cargo doc --open
```

This will generate and open the full API documentation with all available methods, types, and examples.

---

## 📋 Feature Status

### ✅ Completed Features

- **SSH Identity Support**: Full SSH key encryption/decryption support via CLI pass-through
- **Streaming Strategies**: Configurable streaming (Pipe/TempFile) with environment overrides
- **Request API**: Typed request builders for all operations (Lock/Unlock/Rotate)
- **PTY Automation**: Hardened PTY wrapper with configurable timeouts
- **Config Discovery**: Helper methods and CLI inspection commands
- **Adapter Inspection**: `cage adapter info` command for capability reporting

### 🚧 Roadmap Features

- **Deterministic Key Derivation** (CAGE-15): Not yet implemented - will support `--derive` flag
- **Multi-Recipient Lifecycle** (CAGE-16): Not yet implemented - will support multiple recipients per file
- **Backup Retention Lifecycle** (CAGE-03): Partially implemented - registry and cleanup pending

### ⚠️ Known Limitations

- **Passphrase Streaming**: Due to age's TTY requirements, passphrase operations always use temp files regardless of streaming strategy
- **SSH Key Conversion**: SSH keys are passed directly to age CLI (no rust-native conversion)

---

## 🔍 Audit & Telemetry

### Structured Logging (OBS-01)

Cage supports both human-readable and machine-readable audit trails for Padlock/Ignite integration:

#### Configuration

```toml
# cage.toml
[telemetry]
format = "json"  # Options: "text" (default) or "json"

[audit]
logging = true
log_path = "/var/log/cage/audit.json"
```

#### JSON Event Format

```json
{
  "timestamp": "2025-09-29T12:34:56Z",
  "level": "INFO",
  "component": "cage_automation",
  "event_type": "encryption",
  "path": "/data/secret.txt",
  "identity_type": "passphrase",
  "recipient_count": 2,
  "recipient_group_hash": "a1b2c3d4e5f6",
  "success": true
}
```

#### Library Usage

```rust
use cage::cage::{
    config::TelemetryFormat,
    security::AuditLogger,
};

// Create logger with JSON format
let logger = AuditLogger::with_format(
    Some(log_path),
    TelemetryFormat::Json
)?;

// Log structured events
logger.log_encryption_event(
    path,
    Some(recipients),
    "passphrase",
    true
)?;
```

#### Field Redaction

Sensitive data is automatically redacted:
- Recipient keys are hashed (not exposed in plaintext)
- Passphrases are never logged
- File paths can be anonymized via configuration

---

## 🚀 Migration Guide

### From 0.1.x to 0.3.x

Key changes in the 0.3.x series:

1. **Progress Framework Added**: New progress reporting capabilities
2. **In-Place Operations**: Multi-layered safety architecture added
3. **RSB Framework**: Pure RSB CLI implementation (breaking change for CLI integration)
4. **Enhanced Error Types**: More specific error variants

```rust
// OLD (0.1.x)
let result = crud_manager.lock(&path, "pass", basic_options)?;

// NEW (0.3.x) - same interface, enhanced features
let result = crud_manager.lock(&path, "pass", enhanced_options)?;

// NEW features
let progress_manager = ProgressManager::new();  // Progress tracking
let safety_validator = SafetyValidator::new(false, false)?;  // In-place safety
```

---

**Last Updated**: 2025-09-29
**Cage Version**: 0.5.0
**Documentation Version**: 1.1.0
