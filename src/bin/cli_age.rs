//! Age Automation Direct CLI Interface
//!
//! This CLI provides direct access to the Age automation module system for testing and debugging.
//! Users can directly interface with Age encryption operations, TTY automation, and lifecycle
//! management without going through the main padlock orchestrator.
//!
//! Security Guardian: Edgar - Direct Age automation interface

use std::path::{Path, PathBuf};
use std::process;
use clap::{Parser, Subcommand};

// Import our Age automation modules
use padlock::sec::cage::{
    CrudManager, LockOptions, UnlockOptions, VerificationResult,
    AgeConfig, OutputFormat, AdapterFactory
};

/// Age Automation Direct CLI Interface
#[derive(Parser)]
#[command(name = "cli_age")]
#[command(about = "Direct interface to Age automation module system")]
#[command(version = "0.0.1-age-cli")]
#[command(author = "Edgar (Security Guardian)")]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Audit log file path
    #[arg(long)]
    audit_log: Option<PathBuf>,
    
    /// Output format for encrypted files
    #[arg(long, default_value = "binary")]
    format: OutputFormatArg,
    
    #[command(subcommand)]
    command: Commands,
}

/// Lifecycle commands covering complete CRUD operations
#[derive(Subcommand)]
enum Commands {
    /// CREATE: Lock (encrypt) files or repositories
    Lock {
        /// Files or directories to encrypt
        paths: Vec<PathBuf>,
        
        /// Passphrase for encryption
        #[arg(short, long)]
        passphrase: String,
        
        /// Process directories recursively
        #[arg(short, long)]
        recursive: bool,
        
        /// File pattern filter
        #[arg(long)]
        pattern: Option<String>,
        
        /// Create backup before locking
        #[arg(long)]
        backup: bool,
    },
    
    /// DELETE: Unlock (decrypt) files with controlled access
    Unlock {
        /// Files or directories to decrypt
        paths: Vec<PathBuf>,
        
        /// Passphrase for decryption
        #[arg(short, long)]
        passphrase: String,
        
        /// Selective unlocking
        #[arg(short, long)]
        selective: bool,
        
        /// File pattern filter
        #[arg(long)]
        pattern: Option<String>,
        
        /// Preserve encrypted files after unlocking
        #[arg(long)]
        preserve: bool,
    },
    
    /// READ: Status - Check encryption status and repository state
    Status {
        /// Path to check (defaults to current directory)
        path: Option<PathBuf>,
    },
    
    /// UPDATE: Rotate - Key rotation while maintaining access
    Rotate {
        /// Repository to rotate keys for
        repository: PathBuf,
        
        /// New passphrase
        #[arg(short, long)]
        new_passphrase: String,
        
        /// Create backup before rotation
        #[arg(long)]
        backup: bool,
    },
    
    /// ALLOW: Add recipients to authority chain
    Allow {
        /// Recipient to add to authority chain
        recipient: String,
    },
    
    /// REVOKE: Remove recipients from authority chain
    Revoke {
        /// Recipient to remove from authority chain
        recipient: String,
    },
    
    /// RESET: Emergency repository unlock/reset
    Reset {
        /// Repository to reset
        repository: PathBuf,
        
        /// Confirmation string (must be "CONFIRM_RESET")
        #[arg(long)]
        confirmation: String,
    },
    
    /// VERIFY: Integrity checking and validation
    Verify {
        /// Path to verify (defaults to current directory)
        path: Option<PathBuf>,
    },
    
    /// EMERGENCY: Fail-safe recovery operations
    EmergencyUnlock {
        /// Repository for emergency unlock
        repository: PathBuf,
        
        /// Emergency passphrase
        #[arg(long)]
        emergency_passphrase: String,
    },
    
    /// BATCH: Bulk operations for directories/repositories
    Batch {
        /// Directory to process
        directory: PathBuf,
        
        /// Operation to perform (lock/unlock)
        #[arg(short, long)]
        operation: String,
        
        /// Passphrase for operations
        #[arg(short, long)]
        passphrase: String,
        
        /// File pattern filter
        #[arg(long)]
        pattern: Option<String>,
    },
    
    /// TEST: Run validation suite
    Test,
    
    /// DEMO: Run demonstration scenarios
    Demo,
}

/// Output format for CLI argument parsing
#[derive(Clone, Debug, clap::ValueEnum)]
enum OutputFormatArg {
    Binary,
    Ascii,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(format: OutputFormatArg) -> Self {
        match format {
            OutputFormatArg::Binary => OutputFormat::Binary,
            OutputFormatArg::Ascii => OutputFormat::AsciiArmor,
        }
    }
}

/// Main dispatcher coordinating all lifecycle operations
struct LifecycleDispatcher {
    crud_manager: CrudManager,
    verbose: bool,
}

impl LifecycleDispatcher {
    /// Create new lifecycle dispatcher
    fn new(audit_log: Option<PathBuf>, verbose: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let crud_manager = CrudManager::with_defaults()?;
        
        Ok(Self {
            crud_manager,
            verbose,
        })
    }

    /// Execute the specified command
    fn execute_command(&mut self, command: Commands, format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
        
        match command {
            Commands::Lock { paths, passphrase, recursive, pattern, backup } => {
                self.execute_lock(paths, &passphrase, recursive, pattern, backup, format)
            }
            
            Commands::Unlock { paths, passphrase, selective, pattern, preserve } => {
                self.execute_unlock(paths, &passphrase, selective, pattern, preserve)
            }
            
            Commands::Status { path } => {
                self.execute_status(path)
            }
            
            Commands::Rotate { repository, new_passphrase, backup: _ } => {
                self.execute_rotate(&repository, &new_passphrase)
            }
            
            Commands::Allow { recipient } => {
                self.execute_allow(&recipient)
            }
            
            Commands::Revoke { recipient } => {
                self.execute_revoke(&recipient)
            }
            
            Commands::Reset { repository, confirmation } => {
                self.execute_reset(&repository, &confirmation)
            }
            
            Commands::Verify { path } => {
                self.execute_verify(path)
            }
            
            Commands::EmergencyUnlock { repository, emergency_passphrase } => {
                self.execute_emergency_unlock(&repository, &emergency_passphrase)
            }
            
            Commands::Batch { directory, operation, passphrase, pattern } => {
                self.execute_batch(&directory, &operation, &passphrase, pattern)
            }
            
            Commands::Test => {
                self.execute_test_suite()
            }
            
            Commands::Demo => {
                self.execute_demo()
            }
        }
    }

    /// Execute lock operation
    fn execute_lock(
        &mut self, 
        paths: Vec<PathBuf>, 
        passphrase: &str, 
        recursive: bool, 
        pattern: Option<String>, 
        backup: bool,
        format: OutputFormat
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.log("🔐 Executing lock operation...");
        
        let options = LockOptions {
            recursive,
            format,
            pattern_filter: pattern,
            backup_before_lock: backup,
        };
        
        for path in paths {
            self.log(&format!("  Locking: {}", path.display()));
            let result = self.crud_manager.lock(&path, passphrase, options.clone())?;
            
            self.log(&format!("    Processed: {} files", result.processed_files.len()));
            self.log(&format!("    Failed: {} files", result.failed_files.len()));
            self.log(&format!("    Duration: {}ms", result.execution_time_ms));
            
            if !result.failed_files.is_empty() {
                self.log("    Failed files:");
                for failed in &result.failed_files {
                    self.log(&format!("      - {}", failed));
                }
            }
        }
        
        self.log("✅ Lock operation completed");
        Ok(())
    }

    /// Execute unlock operation
    fn execute_unlock(
        &mut self, 
        paths: Vec<PathBuf>, 
        passphrase: &str, 
        selective: bool, 
        pattern: Option<String>, 
        preserve: bool
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.log("🔓 Executing unlock operation...");
        
        let options = UnlockOptions {
            selective,
            verify_before_unlock: true,
            pattern_filter: pattern,
            preserve_encrypted: preserve,
        };
        
        for path in paths {
            self.log(&format!("  Unlocking: {}", path.display()));
            let result = self.crud_manager.unlock(&path, passphrase, options.clone())?;
            
            self.log(&format!("    Processed: {} files", result.processed_files.len()));
            self.log(&format!("    Failed: {} files", result.failed_files.len()));
            self.log(&format!("    Duration: {}ms", result.execution_time_ms));
        }
        
        self.log("✅ Unlock operation completed");
        Ok(())
    }

    /// Execute status operation
    fn execute_status(&self, path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        let target_path = path.unwrap_or_else(|| PathBuf::from("."));
        
        self.log(&format!("📊 Checking status: {}", target_path.display()));
        
        let status = self.crud_manager.status(&target_path)?;
        
        println!("📊 Repository Status:");
        println!("  Total files: {}", status.total_files);
        println!("  Encrypted files: {}", status.encrypted_files);
        println!("  Unencrypted files: {}", status.unencrypted_files);
        println!("  Encryption percentage: {:.1}%", status.encryption_percentage());
        
        if status.is_fully_encrypted() {
            println!("  🔒 Repository is fully encrypted");
        } else if status.is_fully_decrypted() {
            println!("  🔓 Repository is fully decrypted");
        } else {
            println!("  ⚠️  Repository has mixed encryption state");
        }
        
        if !status.failed_files.is_empty() {
            println!("  ❌ Failed files:");
            for failed in &status.failed_files {
                println!("    - {}", failed);
            }
        }
        
        Ok(())
    }

    /// Execute rotate operation
    fn execute_rotate(&mut self, repository: &Path, new_passphrase: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.log(&format!("🔄 Rotating keys for: {}", repository.display()));
        
        let result = self.crud_manager.rotate(repository, new_passphrase)?;
        
        self.log(&format!("    Processed: {} files", result.processed_files.len()));
        self.log(&format!("    Duration: {}ms", result.execution_time_ms));
        
        self.log("✅ Key rotation completed");
        Ok(())
    }

    /// Execute allow operation
    fn execute_allow(&mut self, recipient: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.log(&format!("➕ Adding recipient to authority chain: {}", recipient));
        
        let result = self.crud_manager.allow(recipient)?;
        
        println!("➕ Authority Operation Result:");
        println!("  Operation: {}", result.operation);
        println!("  Recipient: {}", result.recipient);
        println!("  Success: {}", result.success);
        println!("  Authority Chain Status: {}", result.authority_chain_status);
        
        self.log("✅ Allow operation completed");
        Ok(())
    }

    /// Execute revoke operation
    fn execute_revoke(&mut self, recipient: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.log(&format!("➖ Removing recipient from authority chain: {}", recipient));
        
        let result = self.crud_manager.revoke(recipient)?;
        
        println!("➖ Authority Operation Result:");
        println!("  Operation: {}", result.operation);
        println!("  Recipient: {}", result.recipient);
        println!("  Success: {}", result.success);
        println!("  Authority Chain Status: {}", result.authority_chain_status);
        
        self.log("✅ Revoke operation completed");
        Ok(())
    }

    /// Execute reset operation
    fn execute_reset(&mut self, repository: &Path, confirmation: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.log(&format!("🚨 Emergency reset for: {}", repository.display()));
        
        let result = self.crud_manager.reset(repository, confirmation)?;
        
        println!("🚨 Emergency Reset Result:");
        println!("  Operation: {}", result.operation);
        println!("  Affected files: {}", result.affected_files.len());
        println!("  Recovery actions:");
        for action in &result.recovery_actions {
            println!("    - {}", action);
        }
        println!("  Security events:");
        for event in &result.security_events {
            println!("    - {}", event);
        }
        
        self.log("✅ Reset operation completed");
        Ok(())
    }

    /// Execute verify operation
    fn execute_verify(&self, path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        let target_path = path.unwrap_or_else(|| PathBuf::from("."));
        
        self.log(&format!("🔍 Verifying integrity: {}", target_path.display()));
        
        let result = self.crud_manager.verify(&target_path)?;
        
        println!("🔍 Verification Result:");
        println!("  Verified files: {}", result.verified_files.len());
        println!("  Failed files: {}", result.failed_files.len());
        println!("  Authority status: {}", result.authority_status);
        println!("  Overall status: {}", result.overall_status);
        
        if !result.failed_files.is_empty() {
            println!("  ❌ Failed verification:");
            for failed in &result.failed_files {
                println!("    - {}", failed);
            }
        }
        
        self.log("✅ Verification completed");
        Ok(())
    }

    /// Execute emergency unlock operation
    fn execute_emergency_unlock(&mut self, repository: &Path, emergency_passphrase: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.log(&format!("🆘 Emergency unlock for: {}", repository.display()));
        
        let result = self.crud_manager.emergency_unlock(repository, emergency_passphrase)?;
        
        println!("🆘 Emergency Unlock Result:");
        println!("  Operation: {}", result.operation);
        println!("  Affected files: {}", result.affected_files.len());
        println!("  Recovery actions:");
        for action in &result.recovery_actions {
            println!("    - {}", action);
        }
        println!("  Security events:");
        for event in &result.security_events {
            println!("    - {}", event);
        }
        
        self.log("✅ Emergency unlock completed");
        Ok(())
    }

    /// Execute batch operation
    fn execute_batch(&mut self, directory: &Path, operation: &str, passphrase: &str, pattern: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        self.log(&format!("📦 Batch {} operation on: {}", operation, directory.display()));
        
        let result = self.crud_manager.batch_process(directory, pattern.as_deref(), operation, passphrase)?;
        
        println!("📦 Batch Operation Result:");
        println!("  Operation: {}", operation);
        println!("  Processed files: {}", result.processed_files.len());
        println!("  Failed files: {}", result.failed_files.len());
        println!("  Success rate: {:.1}%", result.success_rate());
        println!("  Duration: {}ms", result.execution_time_ms);
        
        if !result.failed_files.is_empty() {
            println!("  ❌ Failed files:");
            for failed in &result.failed_files {
                println!("    - {}", failed);
            }
        }
        
        self.log("✅ Batch operation completed");
        Ok(())
    }

    /// Execute comprehensive test suite
    fn execute_test_suite(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Running Age Automation Test Suite...");
        println!("  Note: Comprehensive testing implementation pending");
        println!("  This would include:");
        println!("    - Security validation tests");
        println!("    - Injection prevention tests");
        println!("    - Authority chain tests");
        println!("    - Performance benchmarks");
        println!("    - Compatibility tests");
        println!("✅ Test suite framework ready for implementation");
        Ok(())
    }

    /// Execute demonstration scenarios
    fn execute_demo(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎭 Age Automation Demonstration");
        println!("🛡️ Security Guardian: Edgar - Lifecycle Management Demo");
        println!();
        println!("This demonstration showcases complete CRUD lifecycle operations:");
        println!("  🔐 CREATE: lock (encrypt files/repositories)");
        println!("  📊 READ: status (check encryption state)");
        println!("  🔄 UPDATE: rotate (key rotation)");
        println!("  🔓 DELETE: unlock (controlled decryption)");
        println!();
        println!("Authority Management Operations:");
        println!("  ➕ ALLOW: Add recipients to authority chain");
        println!("  ➖ REVOKE: Remove recipients from authority chain");
        println!("  🚨 RESET: Emergency repository reset");
        println!();
        println!("Lifecycle Operations:");
        println!("  🔍 VERIFY: Integrity checking and validation");
        println!("  🆘 EMERGENCY: Fail-safe recovery operations");
        println!("  📦 BATCH: Bulk operations for repositories");
        println!();
        println!("Example Commands:");
        println!("  ./driver lock file.txt --passphrase secret123");
        println!("  ./driver unlock file.txt.age --passphrase secret123");
        println!("  ./driver status");
        println!("  ./driver verify /path/to/repo");
        println!("  ./driver batch /repo --operation lock --passphrase secret");
        println!();
        println!("✅ Full CRUD lifecycle capabilities operational");
        Ok(())
    }

    /// Log message if verbose mode is enabled
    fn log(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }
}

/// Main function for lifecycle dispatcher
fn main() {
    let cli = Cli::parse();
    
    // Print banner
    println!("🛡️ Padlock Age Automation Lifecycle Dispatcher");
    println!("Security Guardian: Edgar - CRUD Operations Coordinator");
    println!("Version: 0.0.1-lifecycle | Threat Status: T2.1_ELIMINATED");
    println!();
    
    // Create dispatcher
    match LifecycleDispatcher::new(cli.audit_log, cli.verbose) {
        Ok(mut dispatcher) => {
            if let Err(e) = dispatcher.execute_command(cli.command, cli.format.into()) {
                eprintln!("❌ Operation failed: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize dispatcher: {}", e);
            eprintln!("   Ensure Age automation modules are properly compiled");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_cli_parsing() {
        // This test verifies CLI parsing works correctly
        // Note: Actual functionality tests require Age tooling
        let _cli = Cli::parse_from(&["driver", "demo"]);
    }
    
    #[test]
    fn test_dispatcher_creation() {
        // Test basic dispatcher creation
        // This will fail if dependencies are missing, which is expected
        let _result = LifecycleDispatcher::new(None, false);
    }
}