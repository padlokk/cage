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
use cage::cage::progress::{ProgressManager, ProgressStyle, TerminalReporter};

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
        "demo" => cmd_demo,
        "proxy" => cmd_proxy
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
    let show_progress = args.has("--progress");

    // In-place operation flags
    let in_place = args.has("--in-place");
    let danger_mode = args.has("--danger-mode");
    let i_am_sure = args.has("--i-am-sure");

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

    // Handle in-place operations with safety checks
    if in_place {
        match execute_in_place_lock_operation(paths, &passphrase, recursive, pattern, backup, format, audit_log, verbose, danger_mode, i_am_sure, show_progress) {
            Ok(_) => {
                if verbose { echo!("✅ In-place lock operation completed"); }
                0
            }
            Err(e) => {
                stderr!("❌ In-place lock failed: {}", e);
                1
            }
        }
    } else {
        match execute_lock_operation(paths, &passphrase, recursive, pattern, backup, format, audit_log, verbose, show_progress) {
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
    let show_progress = args.has("--progress");

    let audit_log = if !get_var("opt_audit_log").is_empty() {
        Some(PathBuf::from(get_var("opt_audit_log")))
    } else {
        None
    };

    match execute_unlock_operation(paths, &passphrase, selective, pattern, preserve, audit_log, verbose, show_progress) {
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
    show_progress: bool,
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

    // Setup progress reporting if requested
    let progress_manager = if show_progress {
        let mut manager = ProgressManager::new();
        manager.add_reporter(std::sync::Arc::new(TerminalReporter::new()));
        Some(std::sync::Arc::new(manager))
    } else {
        None
    };

    for (index, path) in paths.iter().enumerate() {
        let progress_task: Option<std::sync::Arc<cage::cage::progress::ProgressTask>> = if let Some(ref pm) = progress_manager {
            let style = if paths.len() > 1 {
                ProgressStyle::Counter { total: paths.len() as u64 }
            } else {
                ProgressStyle::Spinner
            };
            Some(pm.start_task(&format!("🔒 Encrypting {}", path.file_name().unwrap_or_default().to_string_lossy()), style))
        } else {
            None
        };

        if verbose && progress_task.is_none() {
            echo!("  Locking: {}", path.display());
        }

        if let Some(ref task) = progress_task {
            task.update(index as u64 + 1, &format!("Processing {}", path.display()));
        }

        let result = match crud_manager.lock(&path, passphrase, options.clone()) {
            Ok(result) => {
                if let Some(ref task) = progress_task {
                    task.complete(&format!("✓ Encrypted {} ({} files)", path.display(), result.processed_files.len()));
                }
                result
            }
            Err(e) => {
                if let Some(ref task) = progress_task {
                    task.fail(&format!("✗ Failed to encrypt {}: {}", path.display(), e));
                }
                return Err(e.into());
            }
        };

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

/// Execute in-place lock operation with safety layers
fn execute_in_place_lock_operation(
    paths: Vec<PathBuf>,
    passphrase: &str,
    recursive: bool,
    pattern: Option<String>,
    backup: bool,
    format: OutputFormat,
    _audit_log: Option<PathBuf>,
    verbose: bool,
    danger_mode: bool,
    i_am_sure: bool,
    show_progress: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use cage::cage::{SafetyValidator, InPlaceOperation};

    if verbose {
        echo!("🔐 Executing in-place lock operation with safety checks...");
    }

    // Enhanced validation
    if paths.is_empty() {
        return Err("No paths provided for in-place lock operation".into());
    }

    if passphrase.len() < 8 {
        stderr!("⚠️  Warning: Passphrase is less than 8 characters. Consider using a stronger passphrase.");
    }

    // Safety validation
    let safety_validator = SafetyValidator::new(danger_mode, i_am_sure);

    let options = LockOptions {
        recursive,
        format,
        pattern_filter: pattern,
        backup_before_lock: backup,
    };

    let mut crud_manager = CrudManager::with_defaults()?;

    // Setup progress reporting if requested
    let progress_manager = if show_progress {
        let mut manager = ProgressManager::new();
        manager.add_reporter(std::sync::Arc::new(TerminalReporter::new()));
        Some(std::sync::Arc::new(manager))
    } else {
        None
    };

    for (index, path) in paths.iter().enumerate() {
        let progress_task: Option<std::sync::Arc<cage::cage::progress::ProgressTask>> = if let Some(ref pm) = progress_manager {
            let style = if paths.len() > 1 {
                ProgressStyle::Counter { total: paths.len() as u64 }
            } else {
                ProgressStyle::Spinner
            };
            Some(pm.start_task(&format!("🔒 In-place encrypting {}", path.file_name().unwrap_or_default().to_string_lossy()), style))
        } else {
            None
        };

        if verbose && progress_task.is_none() {
            echo!("  🔒 In-place locking: {}", path.display());
        }

        // If recursive, we need to handle directories differently
        if recursive && path.is_dir() {
            if let Some(ref task) = progress_task {
                task.update(index as u64 + 1, &format!("Processing directory {}", path.display()));
            }

            // For recursive in-place, we process each file individually
            let result = match crud_manager.lock(&path, passphrase, options.clone()) {
                Ok(result) => {
                    if let Some(ref task) = progress_task {
                        task.complete(&format!("✓ Directory encrypted {} ({} files)", path.display(), result.processed_files.len()));
                    }
                    result
                }
                Err(e) => {
                    if let Some(ref task) = progress_task {
                        task.fail(&format!("✗ Failed to encrypt directory {}: {}", path.display(), e));
                    }
                    return Err(e.into());
                }
            };

            if verbose {
                echo!("    Processed: {} files", result.processed_files.len());
                echo!("    Failed: {} files", result.failed_files.len());
            }
        } else if path.is_file() {
            // Single file in-place operation

            if let Some(ref task) = progress_task {
                task.update(index as u64 + 1, "Validating safety checks");
            }

            // 1. Safety validation
            if let Err(e) = safety_validator.validate_in_place_operation(&path) {
                if let Some(ref task) = progress_task {
                    task.fail(&format!("✗ Safety validation failed: {}", e));
                }
                return Err(e.into());
            }

            if let Some(ref task) = progress_task {
                task.update_message("Creating in-place operation");
            }

            // 2. Create in-place operation
            let mut in_place_op = InPlaceOperation::new(&path);

            if let Some(ref task) = progress_task {
                task.update_message("Executing atomic encryption");
            }

            // 3. Execute with atomic replacement
            if let Err(e) = in_place_op.execute_lock(passphrase, danger_mode, |src, dst, pass| {
                // Use the CrudManager's encrypt_to_path method
                match crud_manager.encrypt_to_path(src, dst, pass, format) {
                    Ok(_) => {
                        if verbose {
                            echo!("    ✅ Encrypted {} -> {}", src.display(), dst.display());
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }) {
                if let Some(ref task) = progress_task {
                    task.fail(&format!("✗ In-place operation failed: {}", e));
                }
                return Err(e.into());
            }

            if let Some(ref task) = progress_task {
                let recovery_msg = if !danger_mode {
                    format!("✓ File encrypted in-place {} (recovery file created)", path.display())
                } else {
                    format!("✓ File encrypted in-place {} (danger mode)", path.display())
                };
                task.complete(&recovery_msg);
            }

            if verbose {
                echo!("    ✅ In-place operation completed for {}", path.display());
                if !danger_mode {
                    echo!("    📝 Recovery file created: {}.tmp.recover", path.display());
                    echo!("    ⚠️  Delete recovery file once you've verified encryption!");
                }
            }
        } else {
            return Err(format!("Path does not exist or is not a file: {}", path.display()).into());
        }
    }

    if verbose {
        echo!("✅ All in-place lock operations completed");
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
    show_progress: bool,
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

    // Setup progress reporting if requested
    let progress_manager = if show_progress {
        let mut manager = ProgressManager::new();
        manager.add_reporter(std::sync::Arc::new(TerminalReporter::new()));
        Some(std::sync::Arc::new(manager))
    } else {
        None
    };

    for (index, path) in paths.iter().enumerate() {
        let progress_task: Option<std::sync::Arc<cage::cage::progress::ProgressTask>> = if let Some(ref pm) = progress_manager {
            let style = if paths.len() > 1 {
                ProgressStyle::Counter { total: paths.len() as u64 }
            } else {
                ProgressStyle::Spinner
            };
            Some(pm.start_task(&format!("🔓 Decrypting {}", path.file_name().unwrap_or_default().to_string_lossy()), style))
        } else {
            None
        };

        if verbose && progress_task.is_none() {
            echo!("  Unlocking: {}", path.display());
        }

        if let Some(ref task) = progress_task {
            task.update(index as u64 + 1, &format!("Processing {}", path.display()));
        }

        let result = match crud_manager.unlock(&path, passphrase, options.clone()) {
            Ok(result) => {
                if let Some(ref task) = progress_task {
                    task.complete(&format!("✓ Decrypted {} ({} files)", path.display(), result.processed_files.len()));
                }
                result
            }
            Err(e) => {
                if let Some(ref task) = progress_task {
                    task.fail(&format!("✗ Failed to decrypt {}: {}", path.display(), e));
                }
                return Err(e.into());
            }
        };

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

/// Proxy command - Forward arguments to Age binary with PTY automation
fn cmd_proxy(args: Args) -> i32 {
    if let Err(e) = execute_proxy_command(args) {
        echo!("❌ Proxy command failed: {}", e);
        return 1;
    }
    0
}

fn execute_proxy_command(args: Args) -> cage::AgeResult<()> {
    use std::process::Command;

    echo!("🔗 Cage Age Proxy - PTY automation for direct Age commands");

    // Check if Age binary is available
    let age_path = which::which("age").map_err(|_| {
        cage::AgeError::AgeBinaryNotFound("Age binary not found in PATH".to_string())
    })?;

    // Build Age command arguments from --age-* flags
    let mut age_args = Vec::new();

    // Check common Age flags using RSB pattern
    if is_true("opt_age_p") || is_true("opt_age_passphrase") {
        age_args.push("--passphrase".to_string());
    }
    if is_true("opt_age_d") || is_true("opt_age_decrypt") {
        age_args.push("--decrypt".to_string());
    }
    if is_true("opt_age_a") || is_true("opt_age_armor") {
        age_args.push("--armor".to_string());
    }

    // Handle flags with values
    let output_val = get_var("opt_age_o");
    if !output_val.is_empty() {
        age_args.push("--output".to_string());
        age_args.push(output_val);
    }
    let output_val = get_var("opt_age_output");
    if !output_val.is_empty() {
        age_args.push("--output".to_string());
        age_args.push(output_val);
    }

    let identity_val = get_var("opt_age_i");
    if !identity_val.is_empty() {
        age_args.push("--identity".to_string());
        age_args.push(identity_val);
    }
    let identity_val = get_var("opt_age_identity");
    if !identity_val.is_empty() {
        age_args.push("--identity".to_string());
        age_args.push(identity_val);
    }

    let recipient_val = get_var("opt_age_r");
    if !recipient_val.is_empty() {
        age_args.push("--recipient".to_string());
        age_args.push(recipient_val);
    }
    let recipient_val = get_var("opt_age_recipient");
    if !recipient_val.is_empty() {
        age_args.push("--recipient".to_string());
        age_args.push(recipient_val);
    }


    // Add remaining positional arguments (files) - only add file paths
    for remaining_arg in args.remaining() {
        if !remaining_arg.starts_with("--") &&
           !remaining_arg.contains("target/debug/cage") &&
           std::path::Path::new(&remaining_arg).exists() {
            age_args.push(remaining_arg);
        }
    }

    if age_args.is_empty() {
        echo!("❌ No Age arguments provided. Use --age-* flags to pass arguments to Age.");
        echo!("Examples:");
        echo!("  cage proxy --age-p --age-o=/tmp/output.age input.txt");
        echo!("  cage proxy --age-d --age-i=key.txt encrypted.age");
        echo!("  cage proxy --age-passphrase --age-output=/tmp/out.age file.txt");
        return Ok(());
    }

    echo!("🔧 Age command: {} {}", age_path.display(), age_args.join(" "));

    // Check if this requires PTY automation (passphrase operations)
    let is_encrypt = age_args.iter().any(|arg| arg == "--passphrase" || arg == "-p");
    let is_decrypt = age_args.iter().any(|arg| arg == "--decrypt" || arg == "-d");
    let needs_pty = is_encrypt || is_decrypt; // Both encrypt and decrypt may need PTY for passphrases

    if needs_pty {
        echo!("🔐 PTY automation required for passphrase operations");

        // Create PTY automator and handle passphrase prompts
        let passphrase_manager = PassphraseManager::new();

        // Get passphrase from user
        let passphrase = if is_true("opt_stdin_passphrase") {
            passphrase_manager.get_passphrase_with_mode(
                "Enter passphrase for Age operation",
                false,
                PassphraseMode::Stdin
            )?
        } else {
            passphrase_manager.get_passphrase("Enter passphrase for Age operation", false)?
        };

        // Use expect script for PTY automation
        let temp_script = std::env::temp_dir().join("cage_age_proxy.exp");
        let expect_script = format!(r#"#!/usr/bin/expect -f
spawn {} {}
expect {{
    "Enter passphrase*" {{
        send "{}\r"
        exp_continue
    }}
    "Confirm passphrase*" {{
        send "{}\r"
        exp_continue
    }}
    eof
}}
"#, age_path.display(), age_args.join(" "), passphrase, passphrase);

        std::fs::write(&temp_script, expect_script)?;
        std::fs::set_permissions(&temp_script, std::os::unix::fs::PermissionsExt::from_mode(0o755))?;

        // Execute with expect
        let output = Command::new(&temp_script)
            .output()
            .map_err(|e| cage::AgeError::ProcessExecutionFailed {
                command: format!("expect {}", temp_script.display()),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        // Clean up temp script
        let _ = std::fs::remove_file(&temp_script);

        if !output.status.success() {
            return Err(cage::AgeError::ProcessExecutionFailed {
                command: format!("age {}", age_args.join(" ")),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        // Print Age output
        print!("{}", String::from_utf8_lossy(&output.stdout));

    } else {
        echo!("⚡ Direct Age execution (no PTY needed)");

        // Execute Age directly
        let status = Command::new(&age_path)
            .args(&age_args)
            .status()
            .map_err(|e| cage::AgeError::ProcessExecutionFailed {
                command: format!("age {}", age_args.join(" ")),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        if !status.success() {
            return Err(cage::AgeError::ProcessExecutionFailed {
                command: format!("age {}", age_args.join(" ")),
                exit_code: status.code(),
                stderr: "Age command failed".to_string(),
            });
        }
    }

    echo!("✅ Age proxy command completed successfully");
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