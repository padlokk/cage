# 🔐 Padlock Integration Guide

**Using Cage as a Library for Seamless Age Encryption**

This guide provides integration patterns for using Cage as a library within the Padlock application, including proper configuration, extension management, and best practices for secure automation.

## 🏗️ Architecture Overview

```
Padlock App
├── Core Application Logic
├── cage (library dependency)
│   ├── CrudManager - Main encryption interface
│   ├── AgeConfig - Configuration with .padlock extension
│   ├── PassphraseManager - Secure input handling
│   └── PTY Automation - Seamless Age integration
└── Custom UI/UX Layer
```

## 📦 Integration Setup

### Cargo.toml Configuration

```toml
[dependencies]
cage = { path = "../cage", version = "0.1.0" }
# Or from git when published:
# cage = { git = "https://github.com/padlokk/cage", version = "0.1.0" }

# Required for async operations (if needed)
tokio = { version = "1.0", features = ["full"] }
```

### Basic Library Usage

```rust
use cage::{
    AgeConfig, CrudManager, LockOptions, UnlockOptions,
    PassphraseManager, PassphraseMode, OutputFormat
};
use std::path::PathBuf;

// Initialize with Padlock-specific configuration
fn initialize_cage() -> cage::AgeResult<CrudManager> {
    // Configure for Padlock with .padlock extension
    let config = AgeConfig::for_padlock()
        .with_audit_logging(true)
        .with_security_level(cage::SecurityLevel::Standard);

    // Create CRUD manager with custom configuration
    let adapter = cage::AdapterFactory::create_default()?;
    CrudManager::new(adapter, config)
}
```

## 🎯 Core Integration Patterns

### 1. Padlock-Specific Configuration

```rust
use cage::{AgeConfig, SecurityLevel};

// Recommended Padlock configuration
let padlock_config = AgeConfig {
    // Use .padlock extension to identify Padlock-encrypted files
    encrypted_file_extension: "padlock".to_string(),

    // Enhanced security for user data
    security_level: SecurityLevel::Standard,
    audit_logging: true,
    security_validation: true,
    health_checks: true,
    secure_deletion: true,

    // Production settings
    max_retries: 3,
    operation_timeout: Duration::from_secs(120),

    // Padlock-specific defaults
    output_format: OutputFormat::Binary, // Most efficient
    ..AgeConfig::default()
};

// Or use the convenience method
let config = AgeConfig::for_padlock();
```

### 2. File Encryption with Padlock Extensions

```rust
use cage::{CrudManager, LockOptions, AgeConfig};
use std::path::Path;

async fn encrypt_user_file(file_path: &Path, passphrase: &str) -> cage::AgeResult<()> {
    // Initialize with Padlock config
    let config = AgeConfig::for_padlock();
    let adapter = cage::AdapterFactory::create_default()?;
    let mut crud_manager = CrudManager::new(adapter, config)?;

    // Configure encryption options
    let options = LockOptions {
        recursive: false,
        format: OutputFormat::Binary,
        pattern_filter: None,
        backup_before_lock: true, // Always backup user files
    };

    // Encrypt file - will create file.padlock
    let result = crud_manager.lock(file_path, passphrase, options)?;

    if result.success {
        println!("✅ Encrypted {} successfully", file_path.display());
        println!("📁 Created: {}.padlock", file_path.display());
    } else {
        println!("❌ Encryption failed: {} failures", result.failed_files.len());
    }

    Ok(())
}
```

### 3. Secure Passphrase Handling

```rust
use cage::{PassphraseManager, PassphraseMode};

// Interactive passphrase input for GUI applications
async fn get_user_passphrase(prompt: &str, for_encryption: bool) -> cage::AgeResult<String> {
    let passphrase_manager = PassphraseManager::new();

    // For GUI applications, you might want to use your own input dialog
    if passphrase_manager.tty_available() {
        // Terminal available - use secure input
        passphrase_manager.get_passphrase(prompt, for_encryption)
    } else {
        // No TTY - integrate with your GUI input system
        // This is where you'd call your custom secure input dialog
        get_gui_passphrase_input(prompt, for_encryption).await
    }
}

// Example GUI integration (implement based on your UI framework)
async fn get_gui_passphrase_input(prompt: &str, for_encryption: bool) -> cage::AgeResult<String> {
    // Integrate with your GUI framework's secure input
    // Examples:
    // - Tauri: invoke secure input dialog
    // - egui: secure password field
    // - gtk: password entry widget

    // Return the securely obtained passphrase
    todo!("Implement GUI passphrase input")
}
```

### 4. Batch Operations for User Libraries

```rust
use cage::{CrudManager, LockOptions, UnlockOptions};
use std::path::{Path, PathBuf};

// Encrypt entire user document library
async fn encrypt_user_library(
    library_path: &Path,
    passphrase: &str,
    progress_callback: impl Fn(usize, usize) + Send + 'static
) -> cage::AgeResult<()> {
    let config = AgeConfig::for_padlock();
    let adapter = cage::AdapterFactory::create_default()?;
    let mut crud_manager = CrudManager::new(adapter, config)?;

    let options = LockOptions {
        recursive: true,
        backup_before_lock: true,
        ..LockOptions::default()
    };

    // Get all files first for progress tracking
    let files = collect_user_files(library_path)?;
    let total_files = files.len();

    for (i, file) in files.iter().enumerate() {
        match crud_manager.lock(file, passphrase, options.clone()) {
            Ok(_) => {
                progress_callback(i + 1, total_files);
            }
            Err(e) => {
                eprintln!("Failed to encrypt {}: {}", file.display(), e);
            }
        }
    }

    Ok(())
}

fn collect_user_files(path: &Path) -> cage::AgeResult<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Implement file collection logic
    // Filter out .padlock files (already encrypted)
    // Add user file patterns

    Ok(files)
}
```

### 5. Repository Status and Management

```rust
use cage::{CrudManager, RepositoryStatus};

// Check encryption status of user's files
async fn check_library_status(library_path: &Path) -> cage::AgeResult<LibraryStatus> {
    let config = AgeConfig::for_padlock();
    let adapter = cage::AdapterFactory::create_default()?;
    let crud_manager = CrudManager::new(adapter, config)?;

    let status = crud_manager.status(library_path)?;

    Ok(LibraryStatus {
        total_files: status.total_files,
        encrypted_files: status.encrypted_files,
        unencrypted_files: status.unencrypted_files,
        encryption_percentage: if status.total_files > 0 {
            (status.encrypted_files as f32 / status.total_files as f32) * 100.0
        } else {
            0.0
        },
    })
}

#[derive(Debug)]
pub struct LibraryStatus {
    pub total_files: usize,
    pub encrypted_files: usize,
    pub unencrypted_files: usize,
    pub encryption_percentage: f32,
}
```

## 🔧 Advanced Integration Patterns

### 1. Custom Adapter for Padlock

```rust
use cage::{AgeAdapter, AgeResult, OutputFormat};
use std::path::Path;

// Custom adapter that integrates with Padlock's logging/monitoring
pub struct PadlockAgeAdapter {
    base_adapter: Box<dyn AgeAdapter>,
    logger: PadlockLogger,
}

impl PadlockAgeAdapter {
    pub fn new(logger: PadlockLogger) -> AgeResult<Self> {
        let base_adapter = cage::AdapterFactory::create_default()?;
        Ok(Self {
            base_adapter,
            logger,
        })
    }
}

impl AgeAdapter for PadlockAgeAdapter {
    fn encrypt(&self, input: &Path, output: &Path, passphrase: &str, format: OutputFormat) -> AgeResult<()> {
        self.logger.log_operation("encrypt_start", input);

        let result = self.base_adapter.encrypt(input, output, passphrase, format);

        match &result {
            Ok(_) => self.logger.log_operation("encrypt_success", output),
            Err(e) => self.logger.log_error("encrypt_failed", input, e),
        }

        result
    }

    fn decrypt(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()> {
        self.logger.log_operation("decrypt_start", input);

        let result = self.base_adapter.decrypt(input, output, passphrase);

        match &result {
            Ok(_) => self.logger.log_operation("decrypt_success", output),
            Err(e) => self.logger.log_error("decrypt_failed", input, e),
        }

        result
    }
}

// Implement PadlockLogger according to your logging needs
struct PadlockLogger;

impl PadlockLogger {
    fn log_operation(&self, operation: &str, path: &Path) {
        // Integrate with Padlock's logging system
    }

    fn log_error(&self, operation: &str, path: &Path, error: &cage::AgeError) {
        // Integrate with Padlock's error reporting
    }
}
```

### 2. Configuration Management

```rust
use cage::AgeConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PadlockConfig {
    // Padlock-specific settings
    pub library_path: PathBuf,
    pub auto_encrypt: bool,
    pub backup_enabled: bool,

    // Cage integration settings
    pub cage_config: CageConfigData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CageConfigData {
    pub extension: String,
    pub security_level: String,
    pub audit_logging: bool,
    pub operation_timeout_secs: u64,
}

impl From<CageConfigData> for AgeConfig {
    fn from(data: CageConfigData) -> Self {
        AgeConfig::for_padlock()
            .with_extension(data.extension)
            .with_audit_logging(data.audit_logging)
            .with_timeout(Duration::from_secs(data.operation_timeout_secs))
    }
}

// Save/load configuration
impl PadlockConfig {
    pub fn load(config_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(config_path)?;
        let config: PadlockConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}
```

### 3. Error Handling and User Feedback

```rust
use cage::AgeError;

// Convert Cage errors to Padlock-specific error types
#[derive(Debug, thiserror::Error)]
pub enum PadlockError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(#[from] AgeError),

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("User library not accessible: {path}")]
    LibraryError { path: PathBuf },
}

// User-friendly error messages for GUI display
impl PadlockError {
    pub fn user_message(&self) -> String {
        match self {
            PadlockError::EncryptionFailed(age_err) => {
                match age_err {
                    AgeError::AgeBinaryNotFound(_) => {
                        "Encryption tool not found. Please install Age encryption.".to_string()
                    }
                    AgeError::PassphraseValidation { reason, guidance } => {
                        format!("Passphrase issue: {}. {}", reason, guidance)
                    }
                    _ => format!("Encryption error: {}", age_err)
                }
            }
            PadlockError::ConfigError { message } => {
                format!("Configuration problem: {}", message)
            }
            PadlockError::LibraryError { path } => {
                format!("Cannot access library at: {}", path.display())
            }
        }
    }
}
```

## 🔐 Security Best Practices

### 1. Passphrase Management

```rust
use cage::PassphraseManager;
use zeroize::Zeroize;

// Secure passphrase handling in Padlock
pub struct SecurePassphrase {
    inner: String,
}

impl SecurePassphrase {
    pub fn from_user_input(prompt: &str) -> cage::AgeResult<Self> {
        let passphrase_manager = PassphraseManager::new();
        let passphrase = passphrase_manager.get_passphrase(prompt, false)?;

        Ok(Self {
            inner: passphrase,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl Drop for SecurePassphrase {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl Zeroize for SecurePassphrase {
    fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}
```

### 2. Audit Logging Integration

```rust
use cage::{AgeConfig, AuditLogger};

// Integrate with Padlock's audit system
fn setup_audit_logging() -> AgeConfig {
    let log_path = get_padlock_log_directory().join("encryption.log");

    AgeConfig::for_padlock()
        .with_audit_logging(true)
        .with_audit_log_path(log_path.to_string_lossy())
}

fn get_padlock_log_directory() -> PathBuf {
    // Use XDG base directories or platform-specific locations
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".local").join("share").join("padlock").join("logs")
}
```

## 🧪 Testing Integration

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_padlock_encryption_flow() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        // Test encryption with Padlock config
        let result = encrypt_user_file(&test_file, "testpassword").await;
        assert!(result.is_ok());

        // Verify .padlock file was created
        let encrypted_file = test_file.with_extension("padlock");
        assert!(encrypted_file.exists());

        // Test decryption
        let config = AgeConfig::for_padlock();
        let adapter = cage::AdapterFactory::create_default().unwrap();
        let mut crud_manager = CrudManager::new(adapter, config).unwrap();

        let unlock_result = crud_manager.unlock(&encrypted_file, "testpassword", UnlockOptions::default()).unwrap();
        assert!(unlock_result.success);
    }
}
```

## 📁 File Extension Strategy

| Extension | Purpose | Use Case |
|-----------|---------|----------|
| `.cage` | Cage CLI default | Direct cage command usage |
| `.padlock` | Padlock integration | Files encrypted via Padlock app |
| `.secret` | Custom applications | Other applications using cage library |

### Implementation Example

```rust
// Different configurations for different use cases
pub enum EncryptionContext {
    PadlockUser,
    Development,
    Production,
    Custom(String),
}

impl EncryptionContext {
    pub fn to_config(&self) -> AgeConfig {
        match self {
            EncryptionContext::PadlockUser => AgeConfig::for_padlock(),
            EncryptionContext::Development => AgeConfig::development(),
            EncryptionContext::Production => AgeConfig::production(),
            EncryptionContext::Custom(ext) => AgeConfig::default().with_extension(ext),
        }
    }
}
```

## 🚀 Performance Optimization

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

// Parallel file processing for large libraries
pub struct ParallelEncryptionManager {
    crud_manager: Arc<CrudManager>,
    semaphore: Arc<Semaphore>,
}

impl ParallelEncryptionManager {
    pub fn new(max_concurrent: usize) -> cage::AgeResult<Self> {
        let config = AgeConfig::for_padlock();
        let adapter = cage::AdapterFactory::create_default()?;
        let crud_manager = Arc::new(CrudManager::new(adapter, config)?);
        let semaphore = Arc::new(Semaphore::new(max_concurrent));

        Ok(Self {
            crud_manager,
            semaphore,
        })
    }

    pub async fn encrypt_files_parallel(
        &self,
        files: Vec<PathBuf>,
        passphrase: String,
    ) -> Vec<cage::AgeResult<()>> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let tasks = files.into_iter().map(|file| {
            let crud_manager = Arc::clone(&self.crud_manager);
            let semaphore = Arc::clone(&self.semaphore);
            let passphrase = passphrase.clone();

            async move {
                let _permit = semaphore.acquire().await.unwrap();

                // Clone for thread safety
                let mut manager = (*crud_manager).clone();  // This would need Clone impl
                manager.lock(&file, &passphrase, LockOptions::default())
                    .map(|_| ())
            }
        });

        tasks.collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
    }
}
```

## 📖 Summary

This integration guide provides:

- ✅ **Configuration patterns** for Padlock-specific setups
- ✅ **Extension management** (`.padlock` files)
- ✅ **Secure passphrase handling** integration points
- ✅ **Error handling** with user-friendly messages
- ✅ **Audit logging** integration
- ✅ **Performance optimization** for batch operations
- ✅ **Testing strategies** for integration validation

### Key Integration Points

1. **Use `AgeConfig::for_padlock()`** for consistent configuration
2. **Implement custom adapters** for logging/monitoring integration
3. **Handle passphrases securely** with proper cleanup
4. **Use `.padlock` extension** to identify Padlock-encrypted files
5. **Integrate audit logging** with your application's logging system
6. **Test extensively** with your specific use cases

This ensures seamless integration while maintaining security and performance standards.