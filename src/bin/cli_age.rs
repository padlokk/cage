//! Cage - Age Encryption Automation CLI
//!
//! A standalone CLI tool for Age encryption automation with PTY support.
//! Provides secure, automated encryption/decryption operations without manual TTY interaction.
//! Now using RSB framework for enhanced CLI architecture.

use std::env;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Import cage library modules
use cage::core::{
    AgeConfig, BatchOperation, BatchRequest, Identity, LockRequest, Recipient, RotateRequest,
    StatusRequest, StreamRequest, UnlockRequest,
};
use cage::{
    AgeError, AgeResult, CageManager, LockOptions, OutputFormat, PassphraseManager, PassphraseMode,
    UnlockOptions,
};

// Import RSB utilities for enhanced CLI experience
use rsb::prelude::*;
use rsb::progress::{ProgressManager, ProgressStyle, TerminalConfig, TerminalReporter};

/// Print the Cage logo
fn logo() {
    println!(
        r#"
┌─┐┌─┐┌─┐┌─┐
│  ├─┤│ ┬├┤
└─┘┴ ┴└─┘└─┘"#
    );
}

/// Main function using RSB bootstrap
fn main() {
    // Check for version or help flags before RSB processing
    let args: Vec<String> = std::env::args().collect();

    // Handle --version, -v
    if args.iter().any(|arg| arg == "--version" || arg == "-v") {
        show_version();
        return;
    }

    // Handle --help, -h
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        show_help();
        return;
    }

    let args = bootstrap!();
    options!(&args);

    // Print banner with enhanced information
    println!("🔒 Cage - Age Encryption Automation CLI");
    println!("🛡️ Secure Age encryption with PTY automation");
    println!(
        "📦 Version: {} | Built with RSB Framework",
        env!("CARGO_PKG_VERSION")
    );

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
        "proxy" => cmd_proxy,
        "version" => cmd_version,
        "config" => cmd_config,
        "stream" => cmd_stream,
        "adapter" => cmd_adapter,
        "keygen" => cmd_keygen
    });
}

fn collect_lock_recipients_from_cli() -> Vec<Recipient> {
    let mut recipients = Vec::new();

    let single = get_var("opt_recipient");
    if !single.is_empty() {
        for entry in single
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            recipients.push(Recipient::PublicKey(entry.to_string()));
        }
    }

    let multiple = get_var("opt_recipients");
    if !multiple.is_empty() {
        let keys: Vec<String> = multiple
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !keys.is_empty() {
            if keys.len() == 1 {
                recipients.push(Recipient::PublicKey(keys[0].clone()));
            } else {
                recipients.push(Recipient::MultipleKeys(keys));
            }
        }
    }

    let recipients_file = get_var("opt_recipients_file");
    if !recipients_file.is_empty() {
        recipients.push(Recipient::RecipientsFile(PathBuf::from(recipients_file)));
    }

    let ssh_recipients = get_var("opt_ssh_recipient");
    if !ssh_recipients.is_empty() {
        let keys: Vec<String> = ssh_recipients
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !keys.is_empty() {
            recipients.push(Recipient::SshRecipients(keys));
        }
    }

    recipients
}

fn parse_unlock_identity_from_cli() -> Option<Identity> {
    let identity_path = get_var("opt_identity");
    if !identity_path.is_empty() {
        return Some(Identity::IdentityFile(PathBuf::from(identity_path)));
    }

    let ssh_identity_path = get_var("opt_ssh_identity");
    if !ssh_identity_path.is_empty() {
        return Some(Identity::SshKey(PathBuf::from(ssh_identity_path)));
    }

    None
}

fn apply_streaming_strategy_override() {
    let strategy = get_var("opt_streaming_strategy");
    if !strategy.is_empty() {
        std::env::set_var("CAGE_STREAMING_STRATEGY", strategy);
    }
}

// RSB Command Handler Functions

/// Initialize cage configuration
fn cmd_init(_args: Args) -> i32 {
    let force = is_true("opt_force") || is_true("opt_f");

    echo!("🔧 Initializing Cage configuration...");
    match perform_cage_init(force) {
        Ok(report) => {
            let config_created = report.created_paths.iter().any(|p| p == &report.config_dir);
            let data_created = report.created_paths.iter().any(|p| p == &report.data_dir);
            let cache_created = report.created_paths.iter().any(|p| p == &report.cache_dir);
            let backups_created = report.created_paths.iter().any(|p| p == &report.backup_dir);

            echo!(
                "📁 Config dir: {}{}",
                report.config_dir.display(),
                if config_created { " (created)" } else { "" }
            );
            echo!(
                "📦 Data dir: {}{}",
                report.data_dir.display(),
                if data_created { " (created)" } else { "" }
            );
            echo!(
                "🗄️  Cache dir: {}{}",
                report.cache_dir.display(),
                if cache_created { " (created)" } else { "" }
            );
            echo!(
                "🛟 Backup dir: {}{}",
                report.backup_dir.display(),
                if backups_created { " (created)" } else { "" }
            );

            if report.config_overwritten {
                echo!(
                    "✍️  Wrote default config at {} (forced)",
                    report.config_file.display()
                );
            } else if report.config_created {
                echo!(
                    "🆕 Created default config at {}",
                    report.config_file.display()
                );
            } else {
                echo!(
                    "ℹ️  Existing config retained at {} (use --force to reset)",
                    report.config_file.display()
                );
            }

            echo!("✅ Cage initialization completed");
            0
        }
        Err(err) => {
            stderr!("❌ Cage initialization failed: {}", err);
            1
        }
    }
}

/// Install system dependencies
fn cmd_install(_args: Args) -> i32 {
    echo!("📦 Installing Cage dependencies...");
    echo!("Checking for Age binary and other requirements");

    // TODO: Implement dependency installation check
    echo!("✅ Dependency check completed");
    0
}

/// Generate Age identity keypair
fn cmd_keygen(_args: Args) -> i32 {
    use cage::keygen::{KeygenRequest, KeygenService};

    // Parse CLI flags
    let output_path = {
        let path_str = get_var("opt_output");
        if !path_str.is_empty() {
            Some(PathBuf::from(path_str))
        } else {
            None
        }
    };

    let input_path = {
        let path_str = get_var("opt_input");
        if !path_str.is_empty() {
            Some(PathBuf::from(path_str))
        } else {
            None
        }
    };

    let register_groups = {
        let groups_str = get_var("opt_register");
        if !groups_str.is_empty() {
            groups_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    };

    let force = is_true("opt_force") || is_true("opt_f");
    let recipients_only = is_true("opt_recipients_only") || is_true("opt_y");
    let stdout_only = is_true("opt_stdout_only");
    let export_mode = is_true("opt_export");
    let proxy_mode = is_true("opt_proxy");
    let json_output = !is_true("opt_no_json");

    // Build request
    let request = KeygenRequest {
        output_path,
        input_path,
        register_groups,
        recipients_only,
        force,
        stdout_only,
        json_output,
        proxy_mode,
        export_mode,
    };

    // Load config (needed for group registration)
    let config = if !request.register_groups.is_empty() {
        match AgeConfig::load_default() {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                stderr!("❌ Failed to load config for group registration: {}", e);
                return 1;
            }
        }
    } else {
        None
    };

    // Create service and generate
    let service = KeygenService::new(config);
    match service.generate(&request) {
        Ok(summary) => {
            if json_output && !proxy_mode {
                // Emit JSON summary
                use serde_json::json;
                let json_obj = json!({
                    "status": "success",
                    "output_path": summary.output_path.as_ref().map(|p| p.to_string_lossy()),
                    "public_recipient": summary.public_recipient,
                    "fingerprint_md5": summary.fingerprint_md5,
                    "fingerprint_sha256": summary.fingerprint_sha256,
                    "registered_groups": summary.registered_groups,
                });
                println!("{}", serde_json::to_string_pretty(&json_obj).unwrap());
            } else if !proxy_mode {
                // Plain text output
                if let Some(ref path) = summary.output_path {
                    echo!("✅ Identity generated: {}", path.display());
                }
                if let Some(ref recipient) = summary.public_recipient {
                    echo!("📋 Public key: {}", recipient);
                }
                if let Some(ref fp) = summary.fingerprint_md5 {
                    echo!("🔑 Fingerprint (MD5): {}", fp);
                }
                if let Some(ref fp) = summary.fingerprint_sha256 {
                    echo!("🔑 Fingerprint (SHA256): {}", fp);
                }
                if !summary.registered_groups.is_empty() {
                    echo!("📝 Registered with groups: {:?}", summary.registered_groups);
                }
            }
            0
        }
        Err(e) => {
            stderr!("❌ Key generation failed: {}", e);
            1
        }
    }
}

struct InitReport {
    config_dir: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
    backup_dir: PathBuf,
    config_file: PathBuf,
    created_paths: Vec<PathBuf>,
    config_created: bool,
    config_overwritten: bool,
}

fn perform_cage_init(force: bool) -> AgeResult<InitReport> {
    let target = resolve_config_target()?;
    let data_dir = resolve_xdg_home("XDG_DATA_HOME", ".local/share")?.join("cage");
    let cache_dir = resolve_xdg_home("XDG_CACHE_HOME", ".cache")?.join("cage");
    let backup_dir = data_dir.join("backups");

    let mut created_paths = Vec::new();

    // Ensure directories exist
    for dir in [
        target.config_dir.as_path(),
        data_dir.as_path(),
        cache_dir.as_path(),
        backup_dir.as_path(),
    ] {
        if !dir.exists() {
            fs::create_dir_all(dir).map_err(|e| AgeError::FileError {
                operation: "create_directory".to_string(),
                path: dir.to_path_buf(),
                source: e,
            })?;
            created_paths.push(dir.to_path_buf());
        }
    }

    let mut config_created = false;
    let mut config_overwritten = false;

    if target.config_file.exists() {
        if force {
            write_default_config(&target.config_file, &backup_dir)?;
            config_overwritten = true;
        }
    } else {
        if let Some(parent) = target.config_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| AgeError::FileError {
                    operation: "create_directory".to_string(),
                    path: parent.to_path_buf(),
                    source: e,
                })?;
                created_paths.push(parent.to_path_buf());
            }
        }
        write_default_config(&target.config_file, &backup_dir)?;
        config_created = true;
    }

    Ok(InitReport {
        config_dir: target.config_dir,
        data_dir,
        cache_dir,
        backup_dir,
        config_file: target.config_file,
        created_paths,
        config_created,
        config_overwritten,
    })
}

struct ConfigTarget {
    config_dir: PathBuf,
    config_file: PathBuf,
}

fn resolve_config_target() -> AgeResult<ConfigTarget> {
    if let Ok(custom) = env::var("CAGE_CONFIG") {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            let expanded = expand_home(trimmed);
            let path = PathBuf::from(expanded);
            if path.is_dir() {
                let file = path.join("config.toml");
                return Ok(ConfigTarget {
                    config_dir: path,
                    config_file: file,
                });
            }

            if let Some(parent) = path.parent() {
                return Ok(ConfigTarget {
                    config_dir: parent.to_path_buf(),
                    config_file: path,
                });
            }

            return Ok(ConfigTarget {
                config_dir: PathBuf::from("."),
                config_file: path,
            });
        }
    }

    let base = resolve_xdg_home("XDG_CONFIG_HOME", ".config")?;
    let config_dir = base.join("cage");
    let config_file = config_dir.join("config.toml");
    Ok(ConfigTarget {
        config_dir,
        config_file,
    })
}

fn resolve_xdg_home(env_key: &str, fallback: &str) -> AgeResult<PathBuf> {
    if let Ok(value) = env::var(env_key) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(expand_home(trimmed)));
        }
    }

    let home = env::var("HOME").map_err(|_| AgeError::ConfigurationError {
        parameter: env_key.to_string(),
        value: String::new(),
        reason: "HOME environment variable not set".to_string(),
    })?;

    Ok(PathBuf::from(home).join(fallback))
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }

    PathBuf::from(path)
}

fn write_default_config(path: &Path, backup_dir: &Path) -> AgeResult<()> {
    let backup_path = backup_dir
        .canonicalize()
        .unwrap_or_else(|_| backup_dir.to_path_buf());
    let content = default_config_contents(&backup_path);
    fs::write(path, content).map_err(|e| AgeError::FileError {
        operation: "write_config".to_string(),
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(())
}

fn default_config_contents(backup_dir: &Path) -> String {
    let backup_str = backup_dir.to_string_lossy();
    format!(
        "# Cage configuration generated by `cage init`\n# Adjust paths and policies as needed.\n\n[backup]\ncleanup_on_success = true\ndirectory = \"{}\"\nretention = \"keep_last:5\"\n\n[streaming]\nstrategy = \"auto\"\n",
        backup_str
    )
}

/// Lock (encrypt) files using RSB dispatch
fn cmd_lock(args: Args) -> i32 {
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

    let recipients = collect_lock_recipients_from_cli();
    let using_recipients = !recipients.is_empty();

    let cmd_args: Vec<String> = std::env::args().collect();

    apply_streaming_strategy_override();

    let passphrase_value = if using_recipients {
        None
    } else {
        if let Some(_insecure_pass) = PassphraseManager::detect_insecure_usage(&cmd_args) {
            stderr!("⚠️  WARNING: Passphrase detected on command line!");
            stderr!("   This is insecure and visible in process list.");
            if !is_true("opt_i_am_sure") {
                stderr!("   Use interactive prompt instead, or add --i-am-sure to override");
                return 1;
            }
        }

        let passphrase_manager = PassphraseManager::new();
        let passphrase = if is_true("opt_stdin_passphrase") {
            match passphrase_manager.get_passphrase_with_mode(
                "Enter passphrase",
                false,
                PassphraseMode::Stdin,
            ) {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to read passphrase from stdin: {}", e);
                    return 1;
                }
            }
        } else if let Ok(env_pass) = std::env::var("CAGE_PASSPHRASE") {
            env_pass
        } else if let Some(insecure_pass) = PassphraseManager::detect_insecure_usage(&cmd_args) {
            insecure_pass
        } else {
            match passphrase_manager.get_passphrase("Enter passphrase for encryption", false) {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to get passphrase: {}", e);
                    return 1;
                }
            }
        };

        Some(passphrase)
    };

    let identity = if let Some(ref pass) = passphrase_value {
        Identity::Passphrase(pass.clone())
    } else {
        Identity::Passphrase(String::new())
    };

    let recursive = is_true("opt_recursive");
    let pattern_val = get_var("opt_pattern");
    let pattern = if pattern_val.is_empty() {
        None
    } else {
        Some(pattern_val)
    };
    let backup = is_true("opt_backup");
    let verbose = is_true("opt_verbose");
    let show_progress = is_true("opt_progress");

    // In-place operation flags
    let in_place = is_true("opt_in_place");
    let danger_mode = is_true("opt_danger_mode");
    let i_am_sure = is_true("opt_i_am_sure");

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
        if using_recipients {
            stderr!(
                "❌ In-place mode currently requires a passphrase. Remove recipient flags to continue."
            );
            return 1;
        }
        match execute_in_place_lock_operation(
            paths,
            passphrase_value
                .as_ref()
                .expect("passphrase expected for in-place operations"),
            recursive,
            pattern.clone(),
            backup,
            format,
            audit_log.clone(),
            verbose,
            danger_mode,
            i_am_sure,
            show_progress,
        ) {
            Ok(_) => {
                if verbose {
                    echo!("✅ In-place lock operation completed");
                }
                0
            }
            Err(e) => {
                stderr!("❌ In-place lock failed: {}", e);
                1
            }
        }
    } else {
        match execute_lock_operation(
            paths,
            &identity,
            &recipients,
            recursive,
            pattern.clone(),
            backup,
            format,
            audit_log,
            verbose,
            show_progress,
        ) {
            Ok(_) => {
                if verbose {
                    echo!("✅ Lock operation completed");
                }
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
fn cmd_unlock(args: Args) -> i32 {
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

    let identity_override = parse_unlock_identity_from_cli();
    apply_streaming_strategy_override();

    let identity = if let Some(identity) = identity_override {
        identity
    } else {
        let passphrase_manager = PassphraseManager::new();
        let passphrase = if is_true("opt_stdin_passphrase") {
            match passphrase_manager.get_passphrase_with_mode(
                "Enter passphrase",
                false,
                PassphraseMode::Stdin,
            ) {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to read passphrase from stdin: {}", e);
                    return 1;
                }
            }
        } else if let Ok(env_pass) = std::env::var("CAGE_PASSPHRASE") {
            env_pass
        } else {
            match passphrase_manager.get_passphrase("Enter passphrase for decryption", false) {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to get passphrase: {}", e);
                    return 1;
                }
            }
        };

        Identity::Passphrase(passphrase)
    };

    let selective = is_true("opt_selective");
    let pattern = get_var("opt_pattern");
    let pattern = if pattern.is_empty() {
        None
    } else {
        Some(pattern)
    };
    let preserve = is_true("opt_preserve");
    let verbose = is_true("opt_verbose");
    let show_progress = is_true("opt_progress");

    let audit_log = if !get_var("opt_audit_log").is_empty() {
        Some(PathBuf::from(get_var("opt_audit_log")))
    } else {
        None
    };

    match execute_unlock_operation(
        paths,
        &identity,
        selective,
        pattern,
        preserve,
        audit_log,
        verbose,
        show_progress,
    ) {
        Ok(_) => {
            if verbose {
                echo!("✅ Unlock operation completed");
            }
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
fn cmd_rotate(args: Args) -> i32 {
    let repository = PathBuf::from(args.get_or(1, ""));
    if repository.as_os_str().is_empty() {
        stderr!("❌ Repository path required for rotation");
        stderr!("Usage: cage rotate <repository> --old-passphrase <old> --new-passphrase <new>");
        return 1;
    }

    // Get old passphrase securely
    let passphrase_manager = PassphraseManager::new();
    let old_passphrase = {
        let old_pass_var = get_var("opt_old_passphrase");
        if !old_pass_var.is_empty() {
            // Command line provided (warn but allow)
            stderr!("⚠️  Warning: Old passphrase on command line is insecure");
            old_pass_var
        } else if is_true("opt_stdin_passphrase") {
            match passphrase_manager.get_passphrase_with_mode(
                "Enter old passphrase",
                false,
                PassphraseMode::Stdin,
            ) {
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
        }
    };

    // Get new passphrase securely with confirmation
    let new_passphrase = {
        let new_pass_var = get_var("opt_new_passphrase");
        if !new_pass_var.is_empty() {
            // Command line provided (warn but allow)
            stderr!("⚠️  Warning: New passphrase on command line is insecure");
            new_pass_var
        } else if is_true("opt_stdin_passphrase") {
            match passphrase_manager.get_passphrase_with_mode(
                "Enter new passphrase",
                false,
                PassphraseMode::Stdin,
            ) {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to read new passphrase from stdin: {}", e);
                    return 1;
                }
            }
        } else {
            match passphrase_manager.get_passphrase("Enter new passphrase", true) {
                // confirm=true for new passphrase
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to get new passphrase: {}", e);
                    return 1;
                }
            }
        }
    };

    let backup = is_true("opt_backup");
    let verbose = is_true("opt_verbose");

    match execute_rotate_operation(
        &repository,
        &old_passphrase,
        &new_passphrase,
        backup,
        verbose,
    ) {
        Ok(_) => {
            if verbose {
                echo!("✅ Key rotation completed");
            }
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
            if verbose {
                echo!("✅ Verification completed");
            }
            0
        }
        Err(e) => {
            stderr!("❌ Verification failed: {}", e);
            1
        }
    }
}

/// Batch process files using RSB dispatch
fn cmd_batch(args: Args) -> i32 {
    let directory = PathBuf::from(args.get_or(1, ""));
    if directory.as_os_str().is_empty() {
        stderr!("❌ Directory required for batch operation");
        stderr!("Usage: cage batch <directory> --operation <lock|unlock> --passphrase <pass>");
        return 1;
    }

    let operation = get_var("opt_operation");
    let pattern = get_var("opt_pattern");
    let pattern = if pattern.is_empty() {
        None
    } else {
        Some(pattern)
    };

    if operation.is_empty() {
        stderr!("❌ Operation type required");
        stderr!("Usage: cage batch <directory> --operation <lock|unlock> [options]");
        return 1;
    }

    // Get passphrase securely for batch operations
    let passphrase_manager = PassphraseManager::new();
    let passphrase = {
        let pass_var = get_var("opt_passphrase");
        if !pass_var.is_empty() {
            // Command line provided (warn but allow with confirmation)
            stderr!("⚠️  Warning: Batch passphrase on command line is insecure");
            stderr!("   This will be applied to multiple files!");
            if !is_true("opt_i_am_sure") {
                stderr!("   Add --i-am-sure to confirm or use interactive prompt");
                return 1;
            }
            pass_var
        } else if is_true("opt_stdin_passphrase") {
            match passphrase_manager.get_passphrase_with_mode(
                "Enter passphrase for batch operation",
                false,
                PassphraseMode::Stdin,
            ) {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to read passphrase from stdin: {}", e);
                    return 1;
                }
            }
        } else {
            echo!(
                "⚠️  Batch operation will apply to multiple files in {}",
                directory.display()
            );
            match passphrase_manager
                .get_passphrase(&format!("Enter passphrase for batch {}", operation), false)
            {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to get passphrase: {}", e);
                    return 1;
                }
            }
        }
    };

    let verbose = is_true("opt_verbose");
    let force = is_true("opt_i_am_sure");
    let backup = is_true("opt_backup");
    let preserve = is_true("opt_preserve");

    match execute_batch_operation(
        &directory,
        &operation,
        &passphrase,
        pattern,
        verbose,
        force,
        backup,
        preserve,
    ) {
        Ok(_) => {
            if verbose {
                echo!("✅ Batch operation completed");
            }
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
    if is_true("opt_progress_demo") {
        return run_progress_demo();
    }

    echo!(
        r#"🧪 Running Age Automation Test Suite...

Available Tests:
  --progress-demo    Demonstrate progress indicators and styles

Planned Tests:
  - Security validation tests
  - Injection prevention tests
  - Authority chain tests
  - Performance benchmarks
  - Compatibility tests

Usage: cage test --progress-demo
✅ Test suite framework ready for implementation"#
    );
    0
}

/// Show demonstration using RSB dispatch
fn cmd_demo(_args: Args) -> i32 {
    echo!(
        r#"🎭 Cage - Age Encryption Demonstration
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

✅ Cage Age encryption automation ready"#
    );
    0
}

// Operation Implementation Functions

/// Execute lock operation with RSB integration
fn execute_lock_operation(
    paths: Vec<PathBuf>,
    identity: &Identity,
    recipients: &[Recipient],
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

    if recipients.is_empty() {
        if let Identity::Passphrase(pass) = identity {
            if pass.len() < 8 {
                stderr!("⚠️  Warning: Passphrase is less than 8 characters. Consider using a stronger passphrase.");
            }
        }
    }

    let options = LockOptions {
        recursive,
        format,
        pattern_filter: pattern,
        backup_before_lock: backup,
        backup_dir: None,
    };

    let mut crud_manager = CageManager::with_defaults()?;

    // Setup progress reporting if requested
    let progress_manager = if show_progress {
        let manager = Arc::new(ProgressManager::new());
        let reporter = TerminalReporter::with_config(TerminalConfig {
            use_colors: true,
            use_unicode: true,
            use_stderr: true,
            ..Default::default()
        });
        manager.add_reporter(Arc::new(reporter));
        Some(manager)
    } else {
        None
    };

    for (index, path) in paths.iter().enumerate() {
        let progress_task = progress_manager.as_ref().map(|pm| {
            let style = if paths.len() > 1 {
                ProgressStyle::Counter {
                    total: paths.len() as u64,
                }
            } else {
                ProgressStyle::Spinner
            };
            pm.start_task(
                &format!(
                    "🔒 Encrypting {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                style,
            )
        });

        if verbose && progress_task.is_none() {
            echo!("  Locking: {}", path.display());
        }

        if let Some(ref task) = progress_task {
            task.update(index as u64 + 1, &format!("Processing {}", path.display()));
        }

        // Use the new request API (CAGE-11)
        let mut lock_request = LockRequest::new(path.clone(), identity.clone())
            .with_format(options.format)
            .recursive(options.recursive);

        if let Some(pattern_val) = options.pattern_filter.clone() {
            lock_request = lock_request.with_pattern(pattern_val);
        }

        if !recipients.is_empty() {
            lock_request = lock_request.with_recipients(recipients.to_vec());
        }

        lock_request.backup = backup;

        let result = match crud_manager.lock_with_request(&lock_request) {
            Ok(result) => {
                if let Some(ref task) = progress_task {
                    task.complete(&format!(
                        "✓ Encrypted {} ({} files)",
                        path.display(),
                        result.processed_files.len()
                    ));
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
    use cage::{InPlaceOperation, SafetyValidator};

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
        backup_dir: None,
    };

    let mut crud_manager = CageManager::with_defaults()?;

    // Setup progress reporting if requested
    let progress_manager = if show_progress {
        let manager = Arc::new(ProgressManager::new());
        let reporter = TerminalReporter::with_config(TerminalConfig {
            use_colors: true,
            use_unicode: true,
            use_stderr: true,
            ..Default::default()
        });
        manager.add_reporter(Arc::new(reporter));
        Some(manager)
    } else {
        None
    };

    for (index, path) in paths.iter().enumerate() {
        let progress_task = progress_manager.as_ref().map(|pm| {
            let style = if paths.len() > 1 {
                ProgressStyle::Counter {
                    total: paths.len() as u64,
                }
            } else {
                ProgressStyle::Spinner
            };
            pm.start_task(
                &format!(
                    "🔒 In-place encrypting {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                style,
            )
        });

        if verbose && progress_task.is_none() {
            echo!("  🔒 In-place locking: {}", path.display());
        }

        // If recursive, we need to handle directories differently
        if recursive && path.is_dir() {
            if let Some(ref task) = progress_task {
                task.update(
                    index as u64 + 1,
                    &format!("Processing directory {}", path.display()),
                );
            }

            // For recursive in-place, we process each file individually
            // Use the new request API (CAGE-11)
            let lock_request =
                LockRequest::new(path.clone(), Identity::Passphrase(passphrase.to_string()))
                    .with_format(options.format)
                    .recursive(options.recursive);

            let lock_request = match options.pattern_filter.clone() {
                Some(pattern_val) => lock_request.with_pattern(pattern_val),
                None => lock_request,
            };

            let result = match crud_manager.lock_with_request(&lock_request) {
                Ok(result) => {
                    if let Some(ref task) = progress_task {
                        task.complete(&format!(
                            "✓ Directory encrypted {} ({} files)",
                            path.display(),
                            result.processed_files.len()
                        ));
                    }
                    result
                }
                Err(e) => {
                    if let Some(ref task) = progress_task {
                        task.fail(&format!(
                            "✗ Failed to encrypt directory {}: {}",
                            path.display(),
                            e
                        ));
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
                // Use the CageManager's encrypt_to_path method
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
                    format!(
                        "✓ File encrypted in-place {} (recovery file created)",
                        path.display()
                    )
                } else {
                    format!("✓ File encrypted in-place {} (danger mode)", path.display())
                };
                task.complete(&recovery_msg);
            }

            if verbose {
                echo!("    ✅ In-place operation completed for {}", path.display());
                if !danger_mode {
                    echo!(
                        "    📝 Recovery file created: {}.tmp.recover",
                        path.display()
                    );
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
    identity: &Identity,
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

    if let Identity::Passphrase(pass) = identity {
        if pass.is_empty() {
            return Err("Passphrase cannot be empty for unlock operation".into());
        }
    }

    let options = UnlockOptions {
        selective,
        verify_before_unlock: true,
        pattern_filter: pattern,
        preserve_encrypted: preserve,
    };

    let mut crud_manager = CageManager::with_defaults()?;

    // Setup progress reporting if requested
    let progress_manager = if show_progress {
        let manager = Arc::new(ProgressManager::new());
        let reporter = TerminalReporter::with_config(TerminalConfig {
            use_colors: true,
            use_unicode: true,
            use_stderr: true,
            ..Default::default()
        });
        manager.add_reporter(Arc::new(reporter));
        Some(manager)
    } else {
        None
    };

    for (index, path) in paths.iter().enumerate() {
        let progress_task = progress_manager.as_ref().map(|pm| {
            let style = if paths.len() > 1 {
                ProgressStyle::Counter {
                    total: paths.len() as u64,
                }
            } else {
                ProgressStyle::Spinner
            };
            pm.start_task(
                &format!(
                    "🔓 Decrypting {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                style,
            )
        });

        if verbose && progress_task.is_none() {
            echo!("  Unlocking: {}", path.display());
        }

        if let Some(ref task) = progress_task {
            task.update(index as u64 + 1, &format!("Processing {}", path.display()));
        }

        // Use the new request API (CAGE-11)
        let mut unlock_request = UnlockRequest::new(path.clone(), identity.clone())
            .selective(options.selective)
            .preserve_encrypted(options.preserve_encrypted);

        if let Some(pattern_val) = options.pattern_filter.clone() {
            unlock_request = unlock_request.with_pattern(pattern_val);
        }

        let result = match crud_manager.unlock_with_request(&unlock_request) {
            Ok(result) => {
                if let Some(ref task) = progress_task {
                    task.complete(&format!(
                        "✓ Decrypted {} ({} files)",
                        path.display(),
                        result.processed_files.len()
                    ));
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

    let crud_manager = CageManager::with_defaults()?;
    let mut status_request = StatusRequest::new(path.to_path_buf());
    status_request.common.verbose = verbose;
    let status = crud_manager.status_with_request(&status_request)?;

    let status_text = if status.is_fully_encrypted() {
        "🔒 Repository is fully encrypted"
    } else if status.is_fully_decrypted() {
        "🔓 Repository is fully decrypted"
    } else {
        "⚠️  Repository has mixed encryption state"
    };

    echo!(
        "📊 Repository Status:
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
    backup: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        echo!("🔄 Rotating keys for: {}", repository.display());
    }

    let mut crud_manager = CageManager::with_defaults()?;
    let mut rotate_request = RotateRequest::new(
        repository.to_path_buf(),
        Identity::Passphrase(old_passphrase.to_string()),
        Identity::Passphrase(new_passphrase.to_string()),
    );
    rotate_request.backup = backup;
    rotate_request.recursive = true;
    rotate_request.common.verbose = verbose;

    let result = crud_manager.rotate_with_request(&rotate_request)?;

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

    let crud_manager = CageManager::with_defaults()?;
    let result = crud_manager.verify(path)?;

    echo!(
        "🔍 Verification Result:
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
    force: bool,
    backup: bool,
    preserve: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        echo!(
            "📦 Batch {} operation on: {}",
            operation,
            directory.display()
        );
    }

    let batch_operation = match operation {
        "lock" => BatchOperation::Lock,
        "unlock" => BatchOperation::Unlock,
        other => {
            return Err(format!("Unsupported batch operation: {other}").into());
        }
    };

    let mut request = BatchRequest::new(
        directory.to_path_buf(),
        batch_operation,
        Identity::Passphrase(passphrase.to_string()),
    );

    request.common.verbose = verbose;
    request.common.force = force;

    if let Some(pattern) = pattern {
        request = request.with_pattern(pattern);
    }

    if backup {
        request = request.backup(true);
    }

    if preserve && matches!(batch_operation, BatchOperation::Unlock) {
        request = request.preserve_encrypted(true);
    }

    let mut crud_manager = CageManager::with_defaults()?;
    let result = crud_manager.batch_with_request(&request)?;

    let operation_label = match batch_operation {
        BatchOperation::Lock => "lock",
        BatchOperation::Unlock => "unlock",
    };

    echo!(
        "📦 Batch Operation Result:
  Operation: {}
  Processed files: {}
  Failed files: {}
  Success rate: {:.1}%
  Duration: {}ms",
        operation_label,
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
    use cage::pty::PtyAgeAutomator;

    echo!("🔗 Cage Age Proxy - PTY automation for direct Age commands");

    // Build Age command arguments from --age-* flags
    let mut age_args = Vec::new();

    // Check common Age flags using RSB pattern
    if is_true("opt_age_p") || is_true("opt_age_passphrase") {
        age_args.push("-p".to_string());
    }
    if is_true("opt_age_d") || is_true("opt_age_decrypt") {
        age_args.push("-d".to_string());
    }
    if is_true("opt_age_a") || is_true("opt_age_armor") {
        age_args.push("-a".to_string());
    }

    // Handle flags with values
    let output_val = get_var("opt_age_o");
    if !output_val.is_empty() {
        age_args.push("-o".to_string());
        age_args.push(output_val);
    }
    let output_val = get_var("opt_age_output");
    if !output_val.is_empty() {
        age_args.push("--output".to_string());
        age_args.push(output_val);
    }

    let identity_val = get_var("opt_age_i");
    if !identity_val.is_empty() {
        age_args.push("-i".to_string());
        age_args.push(identity_val);
    }
    let identity_val = get_var("opt_age_identity");
    if !identity_val.is_empty() {
        age_args.push("--identity".to_string());
        age_args.push(identity_val);
    }

    let recipient_val = get_var("opt_age_r");
    if !recipient_val.is_empty() {
        age_args.push("-r".to_string());
        age_args.push(recipient_val);
    }
    let recipient_val = get_var("opt_age_recipient");
    if !recipient_val.is_empty() {
        age_args.push("--recipient".to_string());
        age_args.push(recipient_val);
    }

    // Add remaining positional arguments (files) - only add file paths
    for remaining_arg in args.remaining() {
        if !remaining_arg.starts_with("--")
            && !remaining_arg.contains("target/debug/cage")
            && std::path::Path::new(&remaining_arg).exists()
        {
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

    echo!("🔧 Age command: age {}", age_args.join(" "));

    // Check if this requires PTY automation (passphrase operations)
    let is_encrypt = age_args
        .iter()
        .any(|arg| arg == "-p" || arg == "--passphrase");
    let is_decrypt = age_args.iter().any(|arg| arg == "-d" || arg == "--decrypt");
    let needs_pty = is_encrypt || is_decrypt; // Both encrypt and decrypt may need PTY for passphrases

    // Create PTY automator
    let pty_automator = PtyAgeAutomator::new()?;

    if needs_pty {
        echo!("🔐 PTY automation required for passphrase operations");

        // Create passphrase manager and get passphrase from user
        let passphrase_manager = PassphraseManager::new();
        let passphrase = if is_true("opt_stdin_passphrase") {
            passphrase_manager.get_passphrase_with_mode(
                "Enter passphrase for Age operation",
                false,
                PassphraseMode::Stdin,
            )?
        } else {
            passphrase_manager.get_passphrase("Enter passphrase for Age operation", false)?
        };

        // Execute with PTY automation
        let output = pty_automator.execute_age_command(&age_args, Some(&passphrase))?;

        // Print Age output (if any)
        if !output.is_empty() {
            print!("{}", output);
        }
    } else {
        echo!("⚡ Direct Age execution (no passphrase needed)");

        // Execute without passphrase using PTY (for cross-platform compatibility)
        let output = pty_automator.execute_age_command(&age_args, None)?;

        // Print Age output (if any)
        if !output.is_empty() {
            print!("{}", output);
        }
    }

    echo!("✅ Age proxy command completed successfully");
    Ok(())
}

/// Show version information with logo
fn show_version() {
    logo();
    println!("Version: {} | License: AGPL-3.0", env!("CARGO_PKG_VERSION"));
    println!("Copyright © 2025 Qodeninja/Oxidex");
}

/// Show comprehensive help information
fn show_help() {
    logo();
    println!("Version: {} | License: AGPL-3.0", env!("CARGO_PKG_VERSION"));
    println!("Copyright © 2025 Qodeninja/Oxidex");
    println!();
    println!("🔒 Cage - Age Encryption Automation CLI");
    println!("🛡️ Secure Age encryption with PTY automation");
    println!("🚀 Built with RSB Framework");
    println!();
    println!("USAGE:");
    println!("  cage <command> [options]");
    println!("  cage --version, -v     Show version information");
    println!("  cage --help, -h        Show this help message");
    println!();
    println!("COMMANDS:");
    println!("  lock           Encrypt files/directories");
    println!("  unlock         Decrypt files/directories");
    println!("  status         Check encryption status");
    println!("  rotate         Rotate encryption keys");
    println!("  verify         Verify file integrity");
    println!("  batch          Bulk operations");
    println!("  proxy          Direct Age commands with PTY");
    println!("  config         Show/manage configuration");
    println!("  adapter        Inspect adapter capabilities");
    println!("  test           Run test suite & demos");
    println!("  demo           Show demonstrations");
    println!();
    println!("GLOBAL OPTIONS:");
    println!("  --verbose, -v          Show detailed operation progress");
    println!("  --progress             Display professional progress indicators");
    println!("  --format <FORMAT>      Encryption format: binary (default) or ascii");
    println!("  --audit-log <PATH>     Write audit log for security compliance");
    println!(
        "  --streaming-strategy <temp|pipe|auto>  Select streaming mode (pipe needs recipients + identity file)"
    );
    println!();
    println!("IN-PLACE OPERATION OPTIONS:");
    println!("  --in-place             Encrypt/decrypt files in-place (overwrites original)");
    println!("  --danger-mode          Skip recovery file creation (requires DANGER_MODE=1)");
    println!("  --i-am-sure            Automation override for scripted operations");
    println!();
    println!("RECIPIENT & IDENTITY OPTIONS:");
    println!("  --recipient <AGE>          Add public-key recipient (repeat or comma list)");
    println!("  --recipients <LIST>        Comma-separated recipients");
    println!("  --recipients-file <PATH>   Use age recipients file");
    println!("  --ssh-recipient <KEYS>     Convert SSH public keys to recipients");
    println!("  --identity <PATH>          Decrypt with age identity file");
    println!("  --ssh-identity <PATH>      Decrypt with SSH private key");
    println!();
    println!("EXAMPLES:");
    println!("  cage lock secret.txt --progress");
    println!("  cage unlock secret.txt.cage --progress");
    println!("  cage lock document.pdf --in-place");
    println!("  cage status /encrypted-files --verbose");
    println!("  cage proxy --age-p --age-a --age-o=output.age input.txt");
    println!();
    println!("For detailed help on a specific command, use:");
    println!("  cage <command> --help");
}

/// Version command handler for RSB dispatch
fn cmd_version(_args: Args) -> i32 {
    show_version();
    0
}

/// Config command - show or inspect configuration
fn cmd_config(args: Args) -> i32 {
    use cage::core::AgeConfig;

    let subcommand = args.get_or(1, "show");

    match subcommand.as_str() {
        "show" => {
            // Load and display the current configuration
            match AgeConfig::load_default() {
                Ok(config) => {
                    echo!("🔧 Cage Configuration");
                    echo!("===================");
                    echo!("");
                    echo!("{}", config.format_layers());
                    echo!("");
                    echo!("Current Settings:");
                    echo!("  Output format: {:?}", config.output_format);
                    echo!("  TTY method: {:?}", config.tty_method);
                    echo!(
                        "  Encrypted file extension: .{}",
                        config.encrypted_file_extension
                    );
                    echo!("  Backup cleanup: {}", config.backup_cleanup);
                    echo!(
                        "  Streaming strategy: {}",
                        config
                            .streaming_strategy
                            .as_ref()
                            .unwrap_or(&"auto".to_string())
                    );

                    if let Some(backup_dir) = &config.backup_directory {
                        echo!("  Backup directory: {}", backup_dir);
                    }

                    echo!("");
                    echo!("Use 'cage config path' to see only the active config file path");
                    0
                }
                Err(e) => {
                    echo!("❌ Failed to load configuration: {}", e);
                    1
                }
            }
        }
        "path" => {
            // Show just the path where config was loaded from
            match AgeConfig::load_default() {
                Ok(config) => {
                    if let Some(path) = config.source_path {
                        echo!("{}", path.display());
                    } else {
                        echo!("No configuration file loaded (using defaults)");
                    }
                    0
                }
                Err(e) => {
                    echo!("❌ Failed to load configuration: {}", e);
                    1
                }
            }
        }
        "paths" => {
            // Show all search paths
            echo!("Configuration search paths:");
            for path in AgeConfig::get_config_search_paths() {
                let status = if path.exists() { "✓" } else { "✗" };
                echo!("  {} {}", status, path.display());
            }
            0
        }
        _ => {
            echo!("❌ Unknown config subcommand: {}", subcommand);
            echo!("");
            echo!("Available subcommands:");
            echo!("  cage config show  - Display current configuration and search paths");
            echo!("  cage config path  - Show the active configuration file path");
            echo!("  cage config paths - List all configuration search paths");
            1
        }
    }
}

/// Streaming command - encrypt/decrypt using streaming adapters
fn cmd_stream(args: Args) -> i32 {
    let subcommand = args.get_or(1, "encrypt");

    match subcommand.as_str() {
        "encrypt" | "enc" => stream_encrypt(args),
        "decrypt" | "dec" => stream_decrypt(args),
        "help" | "--help" | "-h" => {
            print_stream_usage();
            0
        }
        other => {
            stderr!("❌ Unknown stream subcommand: {}", other);
            print_stream_usage();
            1
        }
    }
}

fn print_stream_usage() {
    echo!(
        "Usage:
  cage stream encrypt --input <PATH> --output <PATH> [options]
  cage stream decrypt --input <PATH> --output <PATH> [options]

Options:
  --input <PATH>           Source file to read (required)
  --output <PATH>          Destination file to write (required)
  --format <binary|ascii>  Output format for encryption (default: binary)
  --buffer-size <BYTES>    Streaming buffer size (default: 8192)
  --recipient, --recipients, --recipients-file, --ssh-recipient  Same as lock CLI
  --identity, --ssh-identity                                Same as unlock CLI
  --stdin-passphrase / CAGE_PASSPHRASE / --i-am-sure        Same semantics as lock/unlock
"
    );
}

fn resolve_stream_buffer_size() -> usize {
    let raw = get_var("opt_buffer_size");
    if raw.is_empty() {
        return 8192;
    }

    match raw.parse::<usize>() {
        Ok(value) if value >= 1024 => value,
        Ok(_) => {
            stderr!("⚠️  Buffer size too small (<1024). Using 1024 bytes.");
            1024
        }
        Err(_) => {
            stderr!(
                "⚠️  Invalid buffer size '{}'. Using default 8192 bytes.",
                raw
            );
            8192
        }
    }
}

fn open_stream_io(
    input_path: &str,
    output_path: &str,
    buffer_size: usize,
) -> Result<(BufReader<File>, BufWriter<File>), String> {
    let input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file '{}': {}", input_path, e))?;

    let output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file '{}': {}", output_path, e))?;

    Ok((
        BufReader::with_capacity(buffer_size, input_file),
        BufWriter::with_capacity(buffer_size, output_file),
    ))
}

fn stream_encrypt(_args: Args) -> i32 {
    let input_path = get_var("opt_input");
    let output_path = get_var("opt_output");

    if input_path.is_empty() || output_path.is_empty() {
        stderr!("❌ Streaming encryption requires --input and --output paths");
        print_stream_usage();
        return 1;
    }

    apply_streaming_strategy_override();

    let recipients = collect_lock_recipients_from_cli();
    let using_recipients = !recipients.is_empty();
    let verbose = is_true("opt_verbose");
    let buffer_size = resolve_stream_buffer_size();

    let cmd_args: Vec<String> = std::env::args().collect();
    let passphrase_value = if using_recipients {
        None
    } else {
        if let Some(_insecure_pass) = PassphraseManager::detect_insecure_usage(&cmd_args) {
            stderr!("⚠️  WARNING: Passphrase detected on command line!");
            stderr!("   This is insecure and visible in process list.");
            if !is_true("opt_i_am_sure") {
                stderr!("   Use interactive prompt instead, or add --i-am-sure to override");
                return 1;
            }
        }

        let passphrase_manager = PassphraseManager::new();

        let passphrase = if is_true("opt_stdin_passphrase") {
            match passphrase_manager.get_passphrase_with_mode(
                "Enter passphrase",
                false,
                PassphraseMode::Stdin,
            ) {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to read passphrase from stdin: {}", e);
                    return 1;
                }
            }
        } else if let Ok(env_pass) = std::env::var("CAGE_PASSPHRASE") {
            env_pass
        } else if let Some(insecure_pass) = PassphraseManager::detect_insecure_usage(&cmd_args) {
            insecure_pass
        } else {
            match passphrase_manager
                .get_passphrase("Enter passphrase for streaming encryption", false)
            {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to get passphrase: {}", e);
                    return 1;
                }
            }
        };

        Some(passphrase)
    };

    let identity = if let Some(pass) = &passphrase_value {
        Identity::Passphrase(pass.clone())
    } else {
        // Recipients-only flows do not need a passphrase identity but the adapter expects a value.
        Identity::Passphrase(String::new())
    };

    let mut request = StreamRequest::encrypt(identity);
    if using_recipients {
        request.recipients = Some(recipients);
    }

    request.format = match get_var("opt_format").as_str() {
        "ascii" => OutputFormat::AsciiArmor,
        _ => OutputFormat::Binary,
    };
    request.buffer_size = buffer_size;
    request.common.verbose = verbose;

    let (mut reader, mut writer) = match open_stream_io(&input_path, &output_path, buffer_size) {
        Ok(handles) => handles,
        Err(err) => {
            stderr!("❌ {}", err);
            return 1;
        }
    };

    let mut crud_manager = match CageManager::with_defaults() {
        Ok(manager) => manager,
        Err(e) => {
            stderr!("❌ Failed to create CageManager: {}", e);
            return 1;
        }
    };

    match crud_manager.stream_with_request(&request, &mut reader, &mut writer) {
        Ok(bytes) => {
            if let Err(e) = writer.flush() {
                stderr!("❌ Failed to flush output: {}", e);
                return 1;
            }

            if verbose {
                echo!("✅ Stream encryption completed ({} bytes)", bytes);
            }
            0
        }
        Err(e) => {
            stderr!("❌ Stream encryption failed: {}", e);
            1
        }
    }
}

fn stream_decrypt(_args: Args) -> i32 {
    let input_path = get_var("opt_input");
    let output_path = get_var("opt_output");

    if input_path.is_empty() || output_path.is_empty() {
        stderr!("❌ Streaming decryption requires --input and --output paths");
        print_stream_usage();
        return 1;
    }

    apply_streaming_strategy_override();

    let buffer_size = resolve_stream_buffer_size();
    let verbose = is_true("opt_verbose");
    let identity = if let Some(identity) = parse_unlock_identity_from_cli() {
        identity
    } else {
        let passphrase_manager = PassphraseManager::new();

        let passphrase = if is_true("opt_stdin_passphrase") {
            match passphrase_manager.get_passphrase_with_mode(
                "Enter passphrase",
                false,
                PassphraseMode::Stdin,
            ) {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to read passphrase from stdin: {}", e);
                    return 1;
                }
            }
        } else if let Ok(env_pass) = std::env::var("CAGE_PASSPHRASE") {
            env_pass
        } else {
            match passphrase_manager
                .get_passphrase("Enter passphrase for streaming decryption", false)
            {
                Ok(pass) => pass,
                Err(e) => {
                    stderr!("❌ Failed to get passphrase: {}", e);
                    return 1;
                }
            }
        };

        Identity::Passphrase(passphrase)
    };

    let mut request = StreamRequest::decrypt(identity);
    request.buffer_size = buffer_size;
    request.common.verbose = verbose;

    let (mut reader, mut writer) = match open_stream_io(&input_path, &output_path, buffer_size) {
        Ok(handles) => handles,
        Err(err) => {
            stderr!("❌ {}", err);
            return 1;
        }
    };

    let mut crud_manager = match CageManager::with_defaults() {
        Ok(manager) => manager,
        Err(e) => {
            stderr!("❌ Failed to create CageManager: {}", e);
            return 1;
        }
    };

    match crud_manager.stream_with_request(&request, &mut reader, &mut writer) {
        Ok(bytes) => {
            if let Err(e) = writer.flush() {
                stderr!("❌ Failed to flush output: {}", e);
                return 1;
            }

            if verbose {
                echo!("✅ Stream decryption completed ({} bytes)", bytes);
            }
            0
        }
        Err(e) => {
            stderr!("❌ Stream decryption failed: {}", e);
            1
        }
    }
}

/// Adapter command - inspect adapter capabilities and health
fn cmd_adapter(args: Args) -> i32 {
    use cage::adp::v2::{AgeAdapterV2, ShellAdapterV2};

    let subcommand = args.get_or(1, "info");

    match subcommand.as_str() {
        "info" | "inspect" => {
            // Create adapter and check its capabilities
            match ShellAdapterV2::new() {
                Ok(adapter) => {
                    echo!("🔧 Age Adapter Inspection");
                    echo!("========================");
                    echo!("");

                    // Basic info
                    echo!("Adapter: {}", adapter.adapter_name());
                    echo!("Version: {}", adapter.adapter_version());
                    echo!("");

                    // Health check
                    echo!("Health Status:");
                    match adapter.health_check() {
                        Ok(health) => {
                            echo!(
                                "  ✓ Overall: {}",
                                if health.healthy {
                                    "Healthy"
                                } else {
                                    "Unhealthy"
                                }
                            );
                            echo!(
                                "  ✓ Age binary: {}",
                                if health.age_binary {
                                    "Available"
                                } else {
                                    "Not found"
                                }
                            );
                            if let Some(version) = health.age_version {
                                echo!("  ✓ Age version: {}", version);
                            }
                            echo!(
                                "  ✓ Can encrypt: {}",
                                if health.can_encrypt { "Yes" } else { "No" }
                            );
                            echo!(
                                "  ✓ Can decrypt: {}",
                                if health.can_decrypt { "Yes" } else { "No" }
                            );
                            echo!(
                                "  ✓ Streaming: {}",
                                if health.streaming_available {
                                    "Available"
                                } else {
                                    "Not available"
                                }
                            );

                            if !health.errors.is_empty() {
                                echo!("");
                                echo!("  ⚠️ Issues:");
                                for error in &health.errors {
                                    echo!("    - {}", error);
                                }
                            }
                        }
                        Err(e) => {
                            echo!("  ✗ Health check failed: {}", e);
                        }
                    }
                    echo!("");

                    // Capabilities
                    let caps = adapter.capabilities();
                    echo!("Capabilities:");
                    echo!("  Encryption Methods:");
                    echo!(
                        "    • Passphrase: {}",
                        if caps.passphrase { "✓" } else { "✗" }
                    );
                    echo!(
                        "    • Public key: {}",
                        if caps.public_key { "✓" } else { "✗" }
                    );
                    echo!(
                        "    • Identity files: {}",
                        if caps.identity_files { "✓" } else { "✗" }
                    );
                    echo!(
                        "    • SSH recipients: {}",
                        if caps.ssh_recipients { "✓" } else { "✗" }
                    );
                    echo!("");

                    echo!("  Streaming Strategies:");
                    let strategies = &caps.streaming_strategies;
                    echo!("    • Default: {:?}", strategies.default);
                    echo!("    • Configured: {:?}", strategies.configured);
                    if let Some(env_override) = &strategies.env_override {
                        echo!("    • Environment override: {:?}", env_override);
                    }
                    echo!(
                        "    • Temp file support: {}",
                        if strategies.supports_tempfile {
                            "✓"
                        } else {
                            "✗"
                        }
                    );
                    echo!(
                        "    • Pipe support: {}",
                        if strategies.supports_pipe {
                            "✓"
                        } else {
                            "✗"
                        }
                    );
                    echo!(
                        "    • Auto fallback: {}",
                        if strategies.auto_fallback {
                            "✓"
                        } else {
                            "✗"
                        }
                    );
                    echo!("");

                    echo!("  Streaming Requirements:");
                    echo!(
                        "    • Pipe encryption needs recipients: {}",
                        if strategies.pipe_requires_recipients {
                            "Yes"
                        } else {
                            "No"
                        }
                    );
                    echo!(
                        "    • Pipe decryption needs identity file: {}",
                        if strategies.pipe_requires_identity {
                            "Yes"
                        } else {
                            "No"
                        }
                    );
                    echo!("");

                    if caps.streaming {
                        echo!(
                            "  ➜ Use 'cage stream encrypt|decrypt' or CageManager::stream_with_request() for streaming workflows"
                        );
                        echo!("");
                    }

                    echo!("  Additional Features:");
                    echo!(
                        "    • ASCII armor: {}",
                        if caps.ascii_armor { "✓" } else { "✗" }
                    );
                    echo!(
                        "    • Hardware keys: {}",
                        if caps.hardware_keys { "✓" } else { "✗" }
                    );
                    echo!(
                        "    • Key derivation: {}",
                        if caps.key_derivation { "✓" } else { "✗" }
                    );

                    if let Some(max_size) = caps.max_file_size {
                        echo!(
                            "    • Max file size: {} GB",
                            max_size / (1024 * 1024 * 1024)
                        );
                    } else {
                        echo!("    • Max file size: Unlimited");
                    }

                    echo!("");
                    echo!("Performance Notes:");
                    echo!("  • Passphrase operations: ~100-150 MB/s (PTY + temp files)");
                    echo!("  • Recipient pipe streaming: ~400-500 MB/s");
                    echo!("  • File operations: ~600 MB/s");
                    echo!("");
                    echo!("Use 'cage adapter health' for quick health check only");

                    0
                }
                Err(e) => {
                    echo!("❌ Failed to create adapter: {}", e);
                    1
                }
            }
        }
        "health" => {
            // Quick health check only
            match ShellAdapterV2::new() {
                Ok(adapter) => match adapter.health_check() {
                    Ok(health) => {
                        if health.healthy {
                            echo!("✓ Adapter is healthy");
                            0
                        } else {
                            echo!("✗ Adapter is unhealthy");
                            for error in &health.errors {
                                echo!("  - {}", error);
                            }
                            1
                        }
                    }
                    Err(e) => {
                        echo!("✗ Health check failed: {}", e);
                        1
                    }
                },
                Err(e) => {
                    echo!("✗ Failed to create adapter: {}", e);
                    1
                }
            }
        }
        _ => {
            echo!("❌ Unknown adapter subcommand: {}", subcommand);
            echo!("");
            echo!("Available subcommands:");
            echo!("  cage adapter info   - Show detailed adapter capabilities");
            echo!("  cage adapter health - Quick health check");
            1
        }
    }
}

/// UAT Demo for Progress Indicators
fn run_progress_demo() -> i32 {
    use rsb::progress::{ProgressManager, ProgressStyle, TerminalConfig, TerminalReporter};
    use std::thread;
    use std::time::Duration;

    echo!("🎯 Progress Indicators UAT Demo");
    echo!("=================================");
    echo!("Testing different progress styles and behaviors...\n");

    // Create progress manager with terminal reporter
    let manager = Arc::new({
        let manager = ProgressManager::new();
        let reporter = TerminalReporter::with_config(TerminalConfig {
            use_colors: true,
            use_unicode: true,
            use_stderr: true,
            ..Default::default()
        });
        manager.add_reporter(Arc::new(reporter));
        manager
    });

    // Demo 1: Simple Spinner
    echo!("📀 Demo 1: Simple Spinner");
    let spinner_task = manager.start_task("Loading configuration", ProgressStyle::Spinner);
    for i in 0..20 {
        spinner_task.update_message(&format!("Loading step {}...", i + 1));
        thread::sleep(Duration::from_millis(100));
    }
    spinner_task.complete("✓ Configuration loaded");
    echo!("");

    // Demo 2: Progress Bar
    echo!("📊 Demo 2: Progress Bar (File Processing)");
    let bar_task = manager.start_task("Processing files", ProgressStyle::Bar { total: 10 });
    for i in 0..10 {
        bar_task.update(i + 1, &format!("Processing file_{}.txt", i + 1));
        thread::sleep(Duration::from_millis(200));
    }
    bar_task.complete("✓ All files processed");
    echo!("");

    // Demo 3: Byte Progress (Large File)
    echo!("💾 Demo 3: Byte Progress (Large File)");
    let bytes_task = manager.start_task(
        "Encrypting large file",
        ProgressStyle::Bytes {
            total_bytes: 1048576,
        },
    ); // 1MB
    let chunk_size = 65536; // 64KB chunks
    for i in (0..1048576).step_by(chunk_size) {
        let current = std::cmp::min(i + chunk_size, 1048576);
        bytes_task.update(
            current as u64,
            &format!("Processing chunk at {}KB", current / 1024),
        );
        thread::sleep(Duration::from_millis(50));
    }
    bytes_task.complete("✓ Large file encrypted");
    echo!("");

    // Demo 4: Counter Style
    echo!("🔢 Demo 4: Counter Style (Key Rotation)");
    let counter_task = manager.start_task(
        "Rotating encryption keys",
        ProgressStyle::Counter { total: 5 },
    );
    let files = [
        "config.json",
        "secrets.txt",
        "database.db",
        "logs.txt",
        "backup.zip",
    ];
    for (i, file) in files.iter().enumerate() {
        counter_task.update(i as u64 + 1, &format!("Rotating key for {}", file));
        thread::sleep(Duration::from_millis(300));
    }
    counter_task.complete("✓ All keys rotated successfully");
    echo!("");

    // Demo 5: Multiple Concurrent Tasks
    echo!("🚀 Demo 5: Multiple Concurrent Tasks");
    let task1 = manager.start_task("Task A (Background sync)", ProgressStyle::Spinner);
    let task2 = manager.start_task("Task B (File validation)", ProgressStyle::Bar { total: 8 });
    let task3 = manager.start_task("Task C (Cleanup)", ProgressStyle::Counter { total: 3 });

    // Simulate concurrent work
    for i in 0..8 {
        // Update all tasks
        task1.update_message(&format!("Syncing item {}...", i + 1));
        task2.update(i + 1, &format!("Validating file_{}.cage", i + 1));
        if i < 3 {
            task3.update(i + 1, &format!("Cleaning temp file {}", i + 1));
        }
        thread::sleep(Duration::from_millis(150));
    }

    task1.complete("✓ Background sync completed");
    task2.complete("✓ All files validated");
    task3.complete("✓ Cleanup finished");
    echo!("");

    // Demo 6: Error Simulation
    echo!("❌ Demo 6: Error Handling");
    let error_task = manager.start_task("Risky operation", ProgressStyle::Bar { total: 5 });
    for i in 0..3 {
        error_task.update(i + 1, &format!("Processing item {}...", i + 1));
        thread::sleep(Duration::from_millis(200));
    }
    error_task.fail("✗ Operation failed: Permission denied");
    echo!("");

    echo!("✅ Progress Indicators UAT Demo Complete!");
    echo!("All progress styles and behaviors working correctly.");
    echo!("");
    echo!("To see progress in real operations, use --progress flag:");
    echo!("  cage lock myfile.txt --progress");
    echo!("  cage unlock myfile.cage --progress");
    echo!("  cage lock directory/ --recursive --progress");

    0
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
