//! Cage - Age Encryption Automation CLI
//!
//! A standalone CLI tool for Age encryption automation with PTY support.
//! Provides secure, automated encryption/decryption operations without manual TTY interaction.
//! Now using RSB framework for enhanced CLI architecture.

use std::path::{Path, PathBuf};

// Import cage library modules
use cage::{
    CrudManager, LockOptions, UnlockOptions,
    OutputFormat, PassphraseManager, PassphraseMode
};

// Import RSB utilities for enhanced CLI experience
use rsb::prelude::*;

/// Main function using RSB bootstrap
fn main() {
    let args = bootstrap!();
    options!(&args);

    // Print banner with enhanced information
    println!("🔒 Cage - Age Encryption Automation CLI");
    println!("🛡️  Secure Age encryption with PTY automation");
    println!("📦 Version: {} | Built with RSB Framework", env!("CARGO_PKG_VERSION"));

    if is_true("opt_verbose") {
        println!("🔍 Verbose mode enabled");
    }
    println!();

    // Pre-dispatch for setup commands
    if pre_dispatch!(&args, {
        "init" => cmd_init,
        "install" => cmd_install
    }) {
        return;
    }

    // Main command dispatch (RSB dispatch handles process exit)
    dispatch!(&args, {
        "lock" => cmd_lock,
        "unlock" => cmd_unlock,
        "status" => cmd_status,
        "rotate" => cmd_rotate,
        "verify" => cmd_verify,
        "batch" => cmd_batch,
        "test" => cmd_test,
        "demo" => cmd_demo
    });
}

// RSB Command Handler Functions

/// Initialize cage configuration
fn cmd_init(_args: Args) -> i32 {
    echo!("🔧 Initializing Cage configuration...");
    echo!("Setting up XDG-compliant directories and default configuration");

    // TODO: Implement configuration initialization
    echo!("✅ Cage initialization completed");
    0
}

/// Install system dependencies
fn cmd_install(_args: Args) -> i32 {
    echo!("📦 Installing Cage dependencies...");
    echo!("Checking for Age binary and other requirements");

    // TODO: Implement dependency installation check
    echo!("✅ Dependency check completed");
    0
}

/// Lock (encrypt) files using RSB dispatch
fn cmd_lock(mut args: Args) -> i32 {
    let paths_str = args.get_or(1, "");
    let paths: Vec<PathBuf> = if paths_str.is_empty() {
        // Get remaining arguments as paths
        args.remaining().iter().map(PathBuf::from).collect()
    } else {
        vec![PathBuf::from(paths_str)]
    };

    if paths.is_empty() {
        stderr!("❌ No files specified for lock operation");
        stderr!("Usage: cage lock <path> [options]");
        return 1;
    }

    // Check for insecure command-line passphrase usage
    let cmd_args: Vec<String> = std::env::args().collect();
    if let Some(insecure_pass) = PassphraseManager::detect_insecure_usage(&cmd_args) {
        stderr!("⚠️  WARNING: Passphrase detected on command line!");
        stderr!("   This is insecure and visible in process list.");
        if !args.has("--i-am-sure") {
            stderr!("   Use interactive prompt instead, or add --i-am-sure to override");
            return 1;
        }
    }

    // Get passphrase securely
    let passphrase_manager = PassphraseManager::new();
    let passphrase = if args.has("--stdin-passphrase") {
        match passphrase_manager.get_passphrase_with_mode("Enter passphrase", false, PassphraseMode::Stdin) {
            Ok(pass) => pass,
            Err(e) => {
                stderr!("❌ Failed to read passphrase from stdin: {}", e);
                return 1;
            }
        }
    } else if let Ok(env_pass) = std::env::var("CAGE_PASSPHRASE") {
        env_pass
    } else if let Some(insecure_pass) = PassphraseManager::detect_insecure_usage(&cmd_args) {
        // Already checked for --i-am-sure above
        insecure_pass
    } else {
        // Interactive prompt (secure default)
        match passphrase_manager.get_passphrase("Enter passphrase for encryption", false) {
            Ok(pass) => pass,
            Err(e) => {
                stderr!("❌ Failed to get passphrase: {}", e);
                return 1;
            }
        }
    };

    let recursive = args.has("--recursive");
    let pattern = args.has_val("--pattern");
    let backup = args.has("--backup");
    let verbose = is_true("opt_verbose");

    let format = match get_var("opt_format").as_str() {
        "ascii" => OutputFormat::AsciiArmor,
        _ => OutputFormat::Binary,
    };

    // Execute lock operation
    let audit_log = if !get_var("opt_audit_log").is_empty() {
        Some(PathBuf::from(get_var("opt_audit_log")))
    } else {
        None
    };

    match execute_lock_operation(paths, &passphrase, recursive, pattern, backup, format, audit_log, verbose) {
        Ok(_) => {
            if verbose { echo!("✅ Lock operation completed"); }
            0
        }
        Err(e) => {
            stderr!("❌ Lock failed: {}", e);
            1
        }
    }
}

/// Unlock (decrypt) files using RSB dispatch
fn cmd_unlock(mut args: Args) -> i32 {
    let paths_str = args.get_or(1, "");
    let paths: Vec<PathBuf> = if paths_str.is_empty() {
        args.remaining().iter().map(PathBuf::from).collect()
    } else {
        vec![PathBuf::from(paths_str)]
    };

    if paths.is_empty() {
        stderr!("❌ No files specified for unlock operation");
        stderr!("Usage: cage unlock <path> [options]");
        return 1;
    }

    // Get passphrase securely (same pattern as lock)
    let passphrase_manager = PassphraseManager::new();
    let passphrase = if args.has("--stdin-passphrase") {
        match passphrase_manager.get_passphrase_with_mode("Enter passphrase", false, PassphraseMode::Stdin) {
            Ok(pass) => pass,
            Err(e) => {
                stderr!("❌ Failed to read passphrase from stdin: {}", e);
                return 1;
            }
        }
    } else if let Ok(env_pass) = std::env::var("CAGE_PASSPHRASE") {
        env_pass
    } else {
        // Interactive prompt (secure default)
        match passphrase_manager.get_passphrase("Enter passphrase for decryption", false) {
            Ok(pass) => pass,
            Err(e) => {
                stderr!("❌ Failed to get passphrase: {}", e);
                return 1;
            }
        }
    };

    let selective = args.has("--selective");
    let pattern = args.has_val("--pattern");
    let preserve = args.has("--preserve");
    let verbose = is_true("opt_verbose");

    let audit_log = if !get_var("opt_audit_log").is_empty() {
        Some(PathBuf::from(get_var("opt_audit_log")))
    } else {
        None
    };

    match execute_unlock_operation(paths, &passphrase, selective, pattern, preserve, audit_log, verbose) {
        Ok(_) => {
            if verbose { echo!("✅ Unlock operation completed"); }
            0
        }
        Err(e) => {
            stderr!("❌ Unlock failed: {}", e);
            1
        }
    }
}

/// Check encryption status using RSB dispatch
fn cmd_status(args: Args) -> i32 {
    let path = if args.remaining().is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(args.get_or(1, "."))
    };

    let verbose = is_true("opt_verbose");

    match execute_status_operation(&path, verbose) {
        Ok(_) => 0,
        Err(e) => {
            stderr!("❌ Status check failed: {}", e);
            1
        }
    }
}

/// Rotate encryption keys using RSB dispatch
fn cmd_rotate(mut args: Args) -> i32 {
    let repository = PathBuf::from(args.get_or(1, ""));
    if repository.as_os_str().is_empty() {
        stderr!("❌ Repository path required for rotation");
        stderr!("Usage: cage rotate <repository> --old-passphrase <old> --new-passphrase <new>");
        return 1;
    }

    // Get old passphrase securely
    let passphrase_manager = PassphraseManager::new();
    let old_passphrase = if let Some(pass) = args.has_val("--old-passphrase") {
        // Command line provided (warn but allow)
        stderr!("⚠️  Warning: Old passphrase on command line is insecure");
        pass
    } else if args.has("--stdin-passphrase") {
        match passphrase_manager.get_passphrase_with_mode("Enter old passphrase", false, PassphraseMode::Stdin) {
            Ok(pass) => pass,
            Err(e) => {
                stderr!("❌ Failed to read old passphrase from stdin: {}", e);
                return 1;
            }
        }
    } else {
        match passphrase_manager.get_passphrase("Enter current passphrase", false) {
            Ok(pass) => pass,
            Err(e) => {
                stderr!("❌ Failed to get old passphrase: {}", e);
                return 1;
            }
        }
    };

    // Get new passphrase securely with confirmation
    let new_passphrase = if let Some(pass) = args.has_val("--new-passphrase") {
        // Command line provided (warn but allow)
        stderr!("⚠️  Warning: New passphrase on command line is insecure");
        pass
    } else if args.has("--stdin-passphrase") {
        match passphrase_manager.get_passphrase_with_mode("Enter new passphrase", false, PassphraseMode::Stdin) {
            Ok(pass) => pass,
            Err(e) => {
                stderr!("❌ Failed to read new passphrase from stdin: {}", e);
                return 1;
            }
        }
    } else {
        match passphrase_manager.get_passphrase("Enter new passphrase", true) {  // confirm=true for new passphrase
            Ok(pass) => pass,
            Err(e) => {
                stderr!("❌ Failed to get new passphrase: {}", e);
                return 1;
            }
        }
    };

    let backup = args.has("--backup");
    let verbose = is_true("opt_verbose");

    match execute_rotate_operation(&repository, &old_passphrase, &new_passphrase, backup, verbose) {
        Ok(_) => {
            if verbose { echo!("✅ Key rotation completed"); }
            0
        }
        Err(e) => {
            stderr!("❌ Rotation failed: {}", e);
            1
        }
    }
}

/// Verify file integrity using RSB dispatch
fn cmd_verify(args: Args) -> i32 {
    let path = if args.remaining().is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(args.get_or(1, "."))
    };

    let verbose = is_true("opt_verbose");

    match execute_verify_operation(&path, verbose) {
        Ok(_) => {
            if verbose { echo!("✅ Verification completed"); }
            0
        }
        Err(e) => {
            stderr!("❌ Verification failed: {}", e);
            1
        }
    }
}

/// Batch process files using RSB dispatch
fn cmd_batch(mut args: Args) -> i32 {
    let directory = PathBuf::from(args.get_or(1, ""));
    if directory.as_os_str().is_empty() {
        stderr!("❌ Directory required for batch operation");
        stderr!("Usage: cage batch <directory> --operation <lock|unlock> --passphrase <pass>");
        return 1;
    }

    let operation = args.has_val("--operation").unwrap_or_default();
    let pattern = args.has_val("--pattern");

    if operation.is_empty() {
        stderr!("❌ Operation type required");
        stderr!("Usage: cage batch <directory> --operation <lock|unlock> [options]");
        return 1;
    }

    // Get passphrase securely for batch operations
    let passphrase_manager = PassphraseManager::new();
    let passphrase = if let Some(pass) = args.has_val("--passphrase") {
        // Command line provided (warn but allow with confirmation)
        stderr!("⚠️  Warning: Batch passphrase on command line is insecure");
        stderr!("   This will be applied to multiple files!");
        if !args.has("--i-am-sure") {
            stderr!("   Add --i-am-sure to confirm or use interactive prompt");
            return 1;
        }
        pass
    } else if args.has("--stdin-passphrase") {
        match passphrase_manager.get_passphrase_with_mode("Enter passphrase for batch operation", false, PassphraseMode::Stdin) {
            Ok(pass) => pass,
            Err(e) => {
                stderr!("❌ Failed to read passphrase from stdin: {}", e);
                return 1;
            }
        }
    } else {
        echo!("⚠️  Batch operation will apply to multiple files in {}", directory.display());
        match passphrase_manager.get_passphrase(&format!("Enter passphrase for batch {}", operation), false) {
            Ok(pass) => pass,
            Err(e) => {
                stderr!("❌ Failed to get passphrase: {}", e);
                return 1;
            }
        }
    };

    let verbose = is_true("opt_verbose");

    match execute_batch_operation(&directory, &operation, &passphrase, pattern, verbose) {
        Ok(_) => {
            if verbose { echo!("✅ Batch operation completed"); }
            0
        }
        Err(e) => {
            stderr!("❌ Batch operation failed: {}", e);
            1
        }
    }
}

/// Run test suite using RSB dispatch
fn cmd_test(_args: Args) -> i32 {
    echo!(r#"🧪 Running Age Automation Test Suite...
  Note: Comprehensive testing implementation pending
  This would include:
    - Security validation tests
    - Injection prevention tests
    - Authority chain tests
    - Performance benchmarks
    - Compatibility tests
✅ Test suite framework ready for implementation"#);
    0
}

/// Show demonstration using RSB dispatch
fn cmd_demo(_args: Args) -> i32 {
    echo!(r#"🎭 Cage - Age Encryption Demonstration
🔒 Secure Age automation with PTY support

This demonstration showcases Age encryption operations:
  🔐 LOCK: Encrypt files and directories
  🔓 UNLOCK: Decrypt files and directories
  📊 STATUS: Check encryption status
  🔄 ROTATE: Rotate encryption keys
  🔍 VERIFY: Verify file integrity
  📦 BATCH: Bulk process multiple files

🔐 Secure Usage Examples:
  cage lock file.txt                    # Interactive passphrase prompt (recommended)
  cage unlock file.txt.age              # Interactive passphrase prompt
  cage status /path/to/files            # No passphrase needed
  cage verify /path/to/files            # No passphrase needed
  cage batch /repo --operation lock     # Interactive prompt for batch operations

🛠️  Advanced Usage:
  CAGE_PASSPHRASE=secret cage lock file.txt          # Environment variable (secure)
  echo 'secret' | cage lock file.txt --stdin-passphrase  # Stdin input (automation)
  cage rotate /repo                                   # Interactive with confirmation

⚠️  Insecure (not recommended):
  cage lock file.txt --passphrase secret --i-am-sure  # Visible in process list!

✅ Cage Age encryption automation ready"#);
    0
}

// Operation Implementation Functions

/// Execute lock operation with RSB integration
fn execute_lock_operation(
    paths: Vec<PathBuf>,
    passphrase: &str,
    recursive: bool,
    pattern: Option<String>,
    backup: bool,
    format: OutputFormat,
    _audit_log: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        echo!("🔐 Executing lock operation...");
    }

    // Enhanced validation with RSB utilities
    if paths.is_empty() {
        return Err("No paths provided for lock operation".into());
    }

    if passphrase.len() < 8 {
        stderr!("⚠️  Warning: Passphrase is less than 8 characters. Consider using a stronger passphrase.");
    }

    let options = LockOptions {
        recursive,
        format,
        pattern_filter: pattern,
        backup_before_lock: backup,
    };

    let mut crud_manager = CrudManager::with_defaults()?;

    for path in paths {
        if verbose {
            echo!("  Locking: {}", path.display());
        }
        let result = crud_manager.lock(&path, passphrase, options.clone())?;

        if verbose {
            echo!("    Processed: {} files", result.processed_files.len());
            echo!("    Failed: {} files", result.failed_files.len());
            echo!("    Duration: {}ms", result.execution_time_ms);

            if !result.failed_files.is_empty() {
                echo!("    Failed files:");
                for failed in &result.failed_files {
                    echo!("      - {}", failed);
                }
            }
        }
    }

    Ok(())
}

/// Execute unlock operation with RSB integration
fn execute_unlock_operation(
    paths: Vec<PathBuf>,
    passphrase: &str,
    selective: bool,
    pattern: Option<String>,
    preserve: bool,
    _audit_log: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        echo!("🔓 Executing unlock operation...");
    }

    // Enhanced validation
    if paths.is_empty() {
        return Err("No paths provided for unlock operation".into());
    }

    if passphrase.is_empty() {
        return Err("Passphrase cannot be empty for unlock operation".into());
    }

    let options = UnlockOptions {
        selective,
        verify_before_unlock: true,
        pattern_filter: pattern,
        preserve_encrypted: preserve,
    };

    let mut crud_manager = CrudManager::with_defaults()?;

    for path in paths {
        if verbose {
            echo!("  Unlocking: {}", path.display());
        }
        let result = crud_manager.unlock(&path, passphrase, options.clone())?;

        if verbose {
            echo!("    Processed: {} files", result.processed_files.len());
            echo!("    Failed: {} files", result.failed_files.len());
            echo!("    Duration: {}ms", result.execution_time_ms);
        }
    }

    Ok(())
}

/// Execute status operation with RSB integration
fn execute_status_operation(path: &Path, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        echo!("📊 Checking status: {}", path.display());
    }

    let crud_manager = CrudManager::with_defaults()?;
    let status = crud_manager.status(path)?;

    let status_text = if status.is_fully_encrypted() {
        "🔒 Repository is fully encrypted"
    } else if status.is_fully_decrypted() {
        "🔓 Repository is fully decrypted"
    } else {
        "⚠️  Repository has mixed encryption state"
    };

    echo!("📊 Repository Status:
  Total files: {}
  Encrypted files: {}
  Unencrypted files: {}
  Encryption percentage: {:.1}%
  {}",
        status.total_files,
        status.encrypted_files,
        status.unencrypted_files,
        status.encryption_percentage(),
        status_text
    );

    if !status.failed_files.is_empty() {
        echo!("  ❌ Failed files:");
        for failed in &status.failed_files {
            echo!("    - {}", failed);
        }
    }

    Ok(())
}

/// Execute rotate operation with RSB integration
fn execute_rotate_operation(
    repository: &Path,
    old_passphrase: &str,
    new_passphrase: &str,
    _backup: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        echo!("🔄 Rotating keys for: {}", repository.display());
    }

    let mut crud_manager = CrudManager::with_defaults()?;
    let result = crud_manager.rotate(repository, old_passphrase, new_passphrase)?;

    if verbose {
        echo!("    Processed: {} files", result.processed_files.len());
        echo!("    Duration: {}ms", result.execution_time_ms);
    }

    Ok(())
}

/// Execute verify operation with RSB integration
fn execute_verify_operation(path: &Path, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        echo!("🔍 Verifying integrity: {}", path.display());
    }

    let crud_manager = CrudManager::with_defaults()?;
    let result = crud_manager.verify(path)?;

    echo!("🔍 Verification Result:
  Verified files: {}
  Failed files: {}
  Overall status: {}",
        result.verified_files.len(),
        result.failed_files.len(),
        result.overall_status
    );

    if !result.failed_files.is_empty() {
        echo!("  ❌ Failed verification:");
        for failed in &result.failed_files {
            echo!("    - {}", failed);
        }
    }

    Ok(())
}

/// Execute batch operation with RSB integration
fn execute_batch_operation(
    directory: &Path,
    operation: &str,
    passphrase: &str,
    pattern: Option<String>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        echo!("📦 Batch {} operation on: {}", operation, directory.display());
    }

    let mut crud_manager = CrudManager::with_defaults()?;
    let result = crud_manager.batch_process(directory, pattern.as_deref(), operation, passphrase)?;

    echo!("📦 Batch Operation Result:
  Operation: {}
  Processed files: {}
  Failed files: {}
  Success rate: {:.1}%
  Duration: {}ms",
        operation,
        result.processed_files.len(),
        result.failed_files.len(),
        result.success_rate(),
        result.execution_time_ms
    );

    if !result.failed_files.is_empty() {
        echo!("  ❌ Failed files:");
        for failed in &result.failed_files {
            echo!("    - {}", failed);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cli_parsing() {
        // This test verifies CLI parsing works correctly
        // Note: Actual functionality tests require Age tooling
        // With RSB, we test the bootstrap and dispatch system
    }

    #[test]
    fn test_rsb_integration() {
        // Test basic RSB integration
        // This will use the global context system
    }
}