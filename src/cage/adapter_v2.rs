//! Enhanced Age Adapter Interface with Streaming Support (CAGE-12)
//!
//! This module extends the adapter pattern to support both file and streaming operations,
//! providing a unified trait for all encryption backends with enhanced capabilities.

use super::config::OutputFormat;
use super::error::{AgeError, AgeResult};
use super::pty_wrap::PtyAgeAutomator;
use super::requests::{Identity, Recipient};
use super::strings;
use age::ssh::Recipient as AgeSshRecipient;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use tempfile::tempdir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamingStrategy {
    TempFile,
    Pipe,
    Auto,
}

fn streaming_strategy_from_env() -> StreamingStrategy {
    match env::var("CAGE_STREAMING_STRATEGY")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "pipe" | "pipes" => StreamingStrategy::Pipe,
        "auto" => StreamingStrategy::Auto,
        "temp" | "tempfile" => StreamingStrategy::TempFile,
        _ => StreamingStrategy::TempFile,
    }
}

// ============================================================================
// CORE ADAPTER TRAITS
// ============================================================================

/// Enhanced Age operations interface with streaming support
pub trait AgeAdapterV2: Send + Sync {
    // ========================================================================
    // FILE-BASED OPERATIONS (Original Interface)
    // ========================================================================

    /// Encrypt a file with the given identity
    fn encrypt_file(
        &self,
        input: &Path,
        output: &Path,
        identity: &Identity,
        recipients: Option<&[Recipient]>,
        format: OutputFormat,
    ) -> AgeResult<()>;

    /// Decrypt a file with the given identity
    fn decrypt_file(&self, input: &Path, output: &Path, identity: &Identity) -> AgeResult<()>;

    // ========================================================================
    // STREAMING OPERATIONS (New Interface)
    // ========================================================================

    /// Encrypt from reader to writer
    fn encrypt_stream(
        &self,
        input: &mut (dyn Read + Send),
        output: &mut (dyn Write + Send),
        _identity: &Identity,
        recipients: Option<&[Recipient]>,
        format: OutputFormat,
    ) -> AgeResult<u64>; // Returns bytes processed

    /// Decrypt from reader to writer
    fn decrypt_stream(
        &self,
        input: &mut (dyn Read + Send),
        output: &mut (dyn Write + Send),
        _identity: &Identity,
    ) -> AgeResult<u64>; // Returns bytes processed

    // ========================================================================
    // IDENTITY & RECIPIENT OPERATIONS
    // ========================================================================

    /// Validate an identity (passphrase, key file, etc.)
    fn validate_identity(&self, identity: &Identity) -> AgeResult<()>;

    /// Validate recipients
    fn validate_recipients(&self, recipients: &[Recipient]) -> AgeResult<()>;

    /// Generate a new identity
    fn generate_identity(&self) -> AgeResult<(String, String)>; // (private, public)

    /// Convert SSH key to Age recipient
    fn ssh_to_recipient(&self, ssh_pubkey: &str) -> AgeResult<String>;

    // ========================================================================
    // VERIFICATION & INSPECTION
    // ========================================================================

    /// Verify encrypted file integrity (without decrypting fully)
    fn verify_file(
        &self,
        file: &Path,
        identity: Option<&Identity>,
    ) -> AgeResult<VerificationResult>;

    /// Get metadata from encrypted file
    fn inspect_file(&self, file: &Path) -> AgeResult<FileMetadata>;

    /// Check if a file is encrypted with Age
    fn is_encrypted(&self, file: &Path) -> bool;

    // ========================================================================
    // HEALTH & DIAGNOSTICS
    // ========================================================================

    /// Validate adapter is functional
    fn health_check(&self) -> AgeResult<HealthStatus>;

    /// Get adapter capabilities
    fn capabilities(&self) -> AdapterCapabilities;

    /// Get adapter name
    fn adapter_name(&self) -> &'static str;

    /// Get adapter version
    fn adapter_version(&self) -> String;

    /// Clone this adapter into a boxed trait object
    fn clone_box(&self) -> Box<dyn AgeAdapterV2>;
}

// ============================================================================
// SUPPORT STRUCTURES
// ============================================================================

/// Verification result for encrypted files
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// File is properly formatted
    pub format_valid: bool,

    /// Header is intact
    pub header_valid: bool,

    /// Can be decrypted with provided identity
    pub decryptable: Option<bool>,

    /// File size
    pub size_bytes: u64,

    /// Detected format
    pub format: DetectedFormat,
}

/// Detected encryption format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetectedFormat {
    /// Age binary format
    AgeBinary,
    /// Age ASCII armor format
    AgeArmor,
    /// Unknown format
    Unknown,
}

/// Metadata from encrypted file
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// Number of recipients (if detectable)
    pub recipient_count: Option<usize>,

    /// File format
    pub format: DetectedFormat,

    /// Encrypted size
    pub encrypted_size: u64,

    /// Creation timestamp (if available)
    pub created: Option<std::time::SystemTime>,
}

/// Health check status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Overall health
    pub healthy: bool,

    /// Age binary available
    pub age_binary: bool,

    /// Version of Age binary
    pub age_version: Option<String>,

    /// Can perform encryption
    pub can_encrypt: bool,

    /// Can perform decryption
    pub can_decrypt: bool,

    /// Streaming support available
    pub streaming_available: bool,

    /// Error messages if unhealthy
    pub errors: Vec<String>,
}

/// Adapter capabilities
#[derive(Debug, Clone)]
pub struct AdapterCapabilities {
    /// Supports passphrase encryption
    pub passphrase: bool,

    /// Supports public key encryption
    pub public_key: bool,

    /// Supports identity files
    pub identity_files: bool,

    /// Supports SSH recipients
    pub ssh_recipients: bool,

    /// Supports streaming operations
    pub streaming: bool,

    /// Streaming strategy metadata
    pub streaming_strategies: StreamingStrategyInfo,

    /// Supports ASCII armor format
    pub ascii_armor: bool,

    /// Supports hardware keys (e.g., YubiKey)
    pub hardware_keys: bool,

    /// Supports key derivation
    pub key_derivation: bool,

    /// Maximum file size (None for unlimited)
    pub max_file_size: Option<u64>,
}

/// Describes available streaming strategies and constraints
#[derive(Debug, Clone)]
pub struct StreamingStrategyInfo {
    /// Strategy selected when no configuration overrides are present
    pub default: StreamingStrategyKind,

    /// Strategy currently configured (env or config overrides applied)
    pub configured: StreamingStrategyKind,

    /// Environment override when present
    pub env_override: Option<StreamingStrategyKind>,

    /// Temp-file staging support available
    pub supports_tempfile: bool,

    /// Direct pipe streaming support available
    pub supports_pipe: bool,

    /// Auto mode negotiates between strategies
    pub auto_fallback: bool,

    /// Pipe encryption requires recipients (public-key mode)
    pub pipe_requires_recipients: bool,

    /// Pipe decryption requires identity files/SSH keys
    pub pipe_requires_identity: bool,
}

/// Supported streaming strategy kinds for capability reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamingStrategyKind {
    TempFile,
    Pipe,
    Auto,
}

// ============================================================================
// BACKWARD COMPATIBILITY WRAPPER
// ============================================================================

/// Wrapper to adapt V2 interface to original AgeAdapter trait
pub struct AdapterV1Compat {
    inner: Arc<dyn AgeAdapterV2>,
}

impl AdapterV1Compat {
    /// Create a new compatibility wrapper
    pub fn new(adapter: impl AgeAdapterV2 + 'static) -> Self {
        Self {
            inner: Arc::new(adapter),
        }
    }
}

impl super::adapter::AgeAdapter for AdapterV1Compat {
    fn encrypt(
        &self,
        input: &Path,
        output: &Path,
        passphrase: &str,
        format: OutputFormat,
    ) -> AgeResult<()> {
        let identity = Identity::Passphrase(passphrase.to_string());
        self.inner
            .encrypt_file(input, output, &identity, None, format)
    }

    fn decrypt(&self, input: &Path, output: &Path, passphrase: &str) -> AgeResult<()> {
        let identity = Identity::Passphrase(passphrase.to_string());
        self.inner.decrypt_file(input, output, &identity)
    }

    fn health_check(&self) -> AgeResult<()> {
        let status = self.inner.health_check()?;
        if status.healthy {
            Ok(())
        } else {
            Err(AgeError::HealthCheckFailed(status.errors.join(", ")))
        }
    }

    fn adapter_name(&self) -> &'static str {
        self.inner.adapter_name()
    }

    fn adapter_version(&self) -> String {
        self.inner.adapter_version()
    }

    fn clone_box(&self) -> Box<dyn super::adapter::AgeAdapter> {
        Box::new(AdapterV1Compat {
            inner: Arc::clone(&self.inner),
        })
    }
}

// ============================================================================
// SHELL ADAPTER V2 IMPLEMENTATION
// ============================================================================

#[derive(Clone, Default)]
pub struct ShellAdapterV2;

impl ShellAdapterV2 {
    fn join_stream_thread<T>(
        handle: thread::ScopedJoinHandle<'_, AgeResult<T>>,
        context: &'static str,
    ) -> AgeResult<T> {
        handle.join().map_err(|_| AgeError::InvalidOperation {
            operation: context.into(),
            reason: "Streaming worker thread panicked".into(),
        })?
    }

    pub fn new() -> AgeResult<Self> {
        let automator = PtyAgeAutomator::new()?;
        automator.check_age_binary()?;
        Ok(Self)
    }

    fn encrypt_with_passphrase(
        &self,
        input: &Path,
        output: &Path,
        passphrase: &str,
        format: OutputFormat,
    ) -> AgeResult<()> {
        let automator = PtyAgeAutomator::new()?;
        automator.encrypt(input, output, passphrase, format)
    }

    fn encrypt_with_recipients(
        &self,
        input: &Path,
        output: &Path,
        recipients: &[Recipient],
        format: OutputFormat,
    ) -> AgeResult<()> {
        let args = collect_recipient_args(recipients)?;
        if args.is_empty() {
            return Err(AgeError::InvalidOperation {
                operation: "encrypt_stream_pipe".into(),
                reason: "Recipient list cannot be empty".into(),
            });
        }
        if args.is_empty() {
            return Err(AgeError::InvalidOperation {
                operation: "encrypt".into(),
                reason: "Recipient list cannot be empty".into(),
            });
        }

        let mut cmd = Command::new("age");
        if matches!(format, OutputFormat::AsciiArmor) {
            cmd.arg("-a");
        }
        cmd.args(&args);
        cmd.arg("-o");
        cmd.arg(output);
        cmd.arg(input);

        let status = cmd.status().map_err(|e| AgeError::ProcessExecutionFailed {
            command: "age".into(),
            exit_code: None,
            stderr: e.to_string(),
        })?;

        if status.success() {
            Ok(())
        } else {
            Err(AgeError::ProcessExecutionFailed {
                command: "age".into(),
                exit_code: status.code(),
                stderr: format!("Age command failed with status {:?}", status),
            })
        }
    }

    fn decrypt_with_identity_file(
        &self,
        input: &Path,
        output: &Path,
        identity_path: &Path,
    ) -> AgeResult<()> {
        let mut cmd = Command::new("age");
        cmd.arg("-d");
        cmd.arg("-i");
        cmd.arg(identity_path);
        cmd.arg("-o");
        cmd.arg(output);
        cmd.arg(input);

        let status = cmd.status().map_err(|e| AgeError::ProcessExecutionFailed {
            command: "age".into(),
            exit_code: None,
            stderr: e.to_string(),
        })?;

        if status.success() {
            Ok(())
        } else {
            Err(AgeError::ProcessExecutionFailed {
                command: "age".into(),
                exit_code: status.code(),
                stderr: format!("Age command failed with status {:?}", status),
            })
        }
    }
}

impl AgeAdapterV2 for ShellAdapterV2 {
    fn encrypt_file(
        &self,
        input: &Path,
        output: &Path,
        identity: &Identity,
        recipients: Option<&[Recipient]>,
        format: OutputFormat,
    ) -> AgeResult<()> {
        if let Some(recips) = recipients {
            if !recips.is_empty() {
                return self.encrypt_with_recipients(input, output, recips, format);
            }
        }

        let pass = match identity {
            Identity::Passphrase(p) => p.clone(),
            Identity::PromptPassphrase => {
                return Err(AgeError::AdapterNotImplemented(
                    "PromptPassphrase not supported in ShellAdapterV2".into(),
                ));
            }
            Identity::IdentityFile(_) | Identity::SshKey(_) => {
                return Err(AgeError::AdapterNotImplemented(
                    "Identity-based encryption not yet implemented".into(),
                ));
            }
        };

        self.encrypt_with_passphrase(input, output, &pass, format)
    }

    fn decrypt_file(&self, input: &Path, output: &Path, identity: &Identity) -> AgeResult<()> {
        match identity {
            Identity::Passphrase(pass) => {
                let automator = PtyAgeAutomator::new()?;
                automator.decrypt(input, output, pass)
            }
            Identity::IdentityFile(path) | Identity::SshKey(path) => {
                self.decrypt_with_identity_file(input, output, path)
            }
            Identity::PromptPassphrase => Err(AgeError::AdapterNotImplemented(
                "PromptPassphrase not supported in ShellAdapterV2".into(),
            )),
        }
    }

    fn encrypt_stream(
        &self,
        input: &mut (dyn Read + Send),
        output: &mut (dyn Write + Send),
        identity: &Identity,
        recipients: Option<&[Recipient]>,
        format: OutputFormat,
    ) -> AgeResult<u64> {
        let strategy = streaming_strategy_from_env();
        let recipients_list = recipients.unwrap_or(&[]);
        let can_use_pipe = !recipients_list.is_empty();

        if !can_use_pipe && matches!(strategy, StreamingStrategy::Pipe) {
            return Err(AgeError::InvalidOperation {
                operation: "encrypt_stream".into(),
                reason: strings::ERR_STREAM_PIPE_REQUIRES_RECIPIENTS.into(),
            });
        }

        if can_use_pipe && matches!(strategy, StreamingStrategy::Pipe | StreamingStrategy::Auto) {
            match self.encrypt_stream_pipe(input, output, recipients_list, format) {
                Ok(bytes) => return Ok(bytes),
                Err(err) => {
                    if strategy == StreamingStrategy::Pipe {
                        return Err(err);
                    } else {
                        eprintln!("[cage] {} ({err})", strings::WARN_STREAM_PIPE_FALLBACK);
                    }
                }
            }
        }

        self.encrypt_stream_temp(input, output, identity, recipients, format)
    }

    fn decrypt_stream(
        &self,
        input: &mut (dyn Read + Send),
        output: &mut (dyn Write + Send),
        identity: &Identity,
    ) -> AgeResult<u64> {
        let strategy = streaming_strategy_from_env();
        let can_use_pipe = matches!(identity, Identity::IdentityFile(_) | Identity::SshKey(_));

        if !can_use_pipe && matches!(strategy, StreamingStrategy::Pipe) {
            return Err(AgeError::InvalidOperation {
                operation: "decrypt_stream".into(),
                reason: strings::ERR_STREAM_PIPE_REQUIRES_IDENTITY.into(),
            });
        }

        if can_use_pipe && matches!(strategy, StreamingStrategy::Pipe | StreamingStrategy::Auto) {
            match self.decrypt_stream_pipe(input, output, identity) {
                Ok(bytes) => return Ok(bytes),
                Err(err) => {
                    if strategy == StreamingStrategy::Pipe {
                        return Err(err);
                    } else {
                        eprintln!("[cage] {} ({err})", strings::WARN_STREAM_PIPE_FALLBACK);
                    }
                }
            }
        }

        self.decrypt_stream_temp(input, output, identity)
    }

    fn validate_identity(&self, identity: &Identity) -> AgeResult<()> {
        match identity {
            Identity::Passphrase(pass) => {
                if pass.is_empty() {
                    Err(AgeError::InvalidOperation {
                        operation: "validate_identity".into(),
                        reason: "Empty passphrase".into(),
                    })
                } else {
                    Ok(())
                }
            }
            Identity::IdentityFile(path) => {
                if path.exists() {
                    Ok(())
                } else {
                    Err(AgeError::InvalidOperation {
                        operation: "validate_identity".into(),
                        reason: format!("Identity file not found: {}", path.display()),
                    })
                }
            }
            Identity::SshKey(path) => {
                if path.exists() {
                    Ok(())
                } else {
                    Err(AgeError::InvalidOperation {
                        operation: "validate_identity".into(),
                        reason: format!("SSH key not found: {}", path.display()),
                    })
                }
            }
            Identity::PromptPassphrase => Err(AgeError::AdapterNotImplemented(
                "PromptPassphrase not supported in ShellAdapterV2".into(),
            )),
        }
    }

    fn validate_recipients(&self, recipients: &[Recipient]) -> AgeResult<()> {
        collect_recipient_args(recipients).map(|_| ())
    }

    fn generate_identity(&self) -> AgeResult<(String, String)> {
        Err(AgeError::AdapterNotImplemented(
            "Identity generation not implemented".into(),
        ))
    }

    fn ssh_to_recipient(&self, ssh_pubkey: &str) -> AgeResult<String> {
        AgeSshRecipient::from_str(ssh_pubkey)
            .map(|r| r.to_string())
            .map_err(|e| AgeError::InvalidOperation {
                operation: "ssh_to_recipient".into(),
                reason: format!("{e:?}"),
            })
    }

    fn verify_file(
        &self,
        _file: &Path,
        _identity: Option<&Identity>,
    ) -> AgeResult<VerificationResult> {
        Err(AgeError::AdapterNotImplemented(
            "verify_file not implemented".into(),
        ))
    }

    fn inspect_file(&self, _file: &Path) -> AgeResult<FileMetadata> {
        Err(AgeError::AdapterNotImplemented(
            "inspect_file not implemented".into(),
        ))
    }

    fn is_encrypted(&self, file: &Path) -> bool {
        file.extension().map_or(false, |e| e == "cage")
    }

    fn health_check(&self) -> AgeResult<HealthStatus> {
        let age_available = PtyAgeAutomator::new()?.check_age_binary().is_ok();
        Ok(HealthStatus {
            healthy: age_available,
            age_binary: age_available,
            age_version: None,
            can_encrypt: age_available,
            can_decrypt: age_available,
            streaming_available: age_available,
            errors: if age_available {
                vec![]
            } else {
                vec!["Age binary not available".into()]
            },
        })
    }

    fn capabilities(&self) -> AdapterCapabilities {
        let env_override = match env::var("CAGE_STREAMING_STRATEGY")
            .ok()
            .map(|v| v.to_lowercase())
            .as_deref()
        {
            Some("pipe") | Some("pipes") => Some(StreamingStrategyKind::Pipe),
            Some("auto") => Some(StreamingStrategyKind::Auto),
            Some("temp") | Some("tempfile") => Some(StreamingStrategyKind::TempFile),
            _ => None,
        };

        let configured = env_override.unwrap_or(StreamingStrategyKind::TempFile);

        AdapterCapabilities {
            passphrase: true,
            public_key: true,
            identity_files: true,
            ssh_recipients: false,
            streaming: true,
            streaming_strategies: StreamingStrategyInfo {
                default: StreamingStrategyKind::TempFile,
                configured,
                env_override,
                supports_tempfile: true,
                supports_pipe: true,
                auto_fallback: true,
                pipe_requires_recipients: true,
                pipe_requires_identity: true,
            },
            ascii_armor: true,
            hardware_keys: false,
            key_derivation: false,
            max_file_size: None,
        }
    }

    fn adapter_name(&self) -> &'static str {
        "ShellAdapterV2"
    }

    fn adapter_version(&self) -> String {
        format!("shell-v2-{}", super::VERSION)
    }

    fn clone_box(&self) -> Box<dyn AgeAdapterV2> {
        Box::new(Self)
    }
}

impl ShellAdapterV2 {
    fn encrypt_stream_temp(
        &self,
        input: &mut (dyn Read + Send),
        output: &mut (dyn Write + Send),
        identity: &Identity,
        recipients: Option<&[Recipient]>,
        format: OutputFormat,
    ) -> AgeResult<u64> {
        let temp_dir = tempdir().map_err(|e| AgeError::TemporaryResourceError {
            resource_type: "dir".into(),
            operation: "create".into(),
            reason: format!("{e:?}"),
        })?;

        let input_path = temp_dir.path().join("stream_input");
        let mut temp_in = File::create(&input_path)
            .map_err(|e| AgeError::file_error("create", input_path.clone(), e))?;
        let bytes_copied = std::io::copy(input, &mut temp_in).map_err(|e| AgeError::IoError {
            operation: "stream_copy".into(),
            context: "encrypt_stream".into(),
            source: e,
        })?;
        temp_in.flush().map_err(|e| AgeError::IoError {
            operation: "flush".into(),
            context: "encrypt_stream".into(),
            source: e,
        })?;

        let output_path = temp_dir.path().join("stream_output");

        if let Some(recips) = recipients {
            if !recips.is_empty() {
                self.encrypt_with_recipients(&input_path, &output_path, recips, format)?;
            } else {
                let pass = match identity {
                    Identity::Passphrase(p) => p.clone(),
                    _ => {
                        return Err(AgeError::AdapterNotImplemented(
                            "Streaming requires passphrase or recipients".into(),
                        ))
                    }
                };
                self.encrypt_with_passphrase(&input_path, &output_path, &pass, format)?;
            }
        } else {
            let pass = match identity {
                Identity::Passphrase(p) => p.clone(),
                _ => {
                    return Err(AgeError::AdapterNotImplemented(
                        "Streaming requires passphrase or recipients".into(),
                    ))
                }
            };
            self.encrypt_with_passphrase(&input_path, &output_path, &pass, format)?;
        }

        let mut encrypted = File::open(&output_path)
            .map_err(|e| AgeError::file_error("open", output_path.clone(), e))?;
        std::io::copy(&mut encrypted, output).map_err(|e| AgeError::IoError {
            operation: "stream_copy".into(),
            context: "encrypt_stream".into(),
            source: e,
        })?;

        Ok(bytes_copied)
    }

    fn encrypt_stream_pipe(
        &self,
        input: &mut (dyn Read + Send),
        output: &mut (dyn Write + Send),
        recipients: &[Recipient],
        format: OutputFormat,
    ) -> AgeResult<u64> {
        let args = collect_recipient_args(recipients)?;

        let mut cmd = Command::new("age");
        if matches!(format, OutputFormat::AsciiArmor) {
            cmd.arg("-a");
        }
        cmd.args(&args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| AgeError::ProcessExecutionFailed {
            command: "age".into(),
            exit_code: None,
            stderr: e.to_string(),
        })?;

        let mut child_stdin =
            child
                .stdin
                .take()
                .ok_or_else(|| AgeError::ProcessExecutionFailed {
                    command: "age".into(),
                    exit_code: None,
                    stderr: "Failed to open stdin for age".into(),
                })?;
        let child_stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgeError::ProcessExecutionFailed {
                command: "age".into(),
                exit_code: None,
                stderr: "Failed to open stdout for age".into(),
            })?;
        let mut stderr_pipe = child.stderr.take();
        let mut stderr_buffer = Vec::new();

        let bytes_copied = thread::scope(|scope| -> AgeResult<u64> {
            let output_writer: &mut (dyn Write + Send) = output;
            let stdout_handle = scope.spawn(move || -> AgeResult<u64> {
                let mut stdout_pipe = child_stdout;
                std::io::copy(&mut stdout_pipe, output_writer).map_err(|e| AgeError::IoError {
                    operation: "stream_copy".into(),
                    context: "encrypt_stream_pipe:stdout".into(),
                    source: e,
                })
            });

            let stderr_handle = stderr_pipe.take().map(|mut pipe| {
                let stderr_ref: &mut Vec<u8> = &mut stderr_buffer;
                scope.spawn(move || -> AgeResult<()> {
                    pipe.read_to_end(stderr_ref)
                        .map_err(|e| AgeError::IoError {
                            operation: "read".into(),
                            context: "encrypt_stream_pipe:stderr".into(),
                            source: e,
                        })?;
                    Ok(())
                })
            });

            let bytes_written =
                std::io::copy(input, &mut child_stdin).map_err(|e| AgeError::IoError {
                    operation: "stream_copy".into(),
                    context: "encrypt_stream_pipe:stdin".into(),
                    source: e,
                })?;

            child_stdin.flush().map_err(|e| AgeError::IoError {
                operation: "flush".into(),
                context: "encrypt_stream_pipe:stdin".into(),
                source: e,
            })?;
            drop(child_stdin);

            let _ = Self::join_stream_thread(stdout_handle, "encrypt_stream_pipe:stdout")?;
            if let Some(handle) = stderr_handle {
                let _ = Self::join_stream_thread(handle, "encrypt_stream_pipe:stderr")?;
            }

            Ok(bytes_written)
        })?;

        let status = child.wait().map_err(|e| AgeError::ProcessExecutionFailed {
            command: "age".into(),
            exit_code: None,
            stderr: e.to_string(),
        })?;

        if !status.success() {
            let stderr_msg = if !stderr_buffer.is_empty() {
                String::from_utf8_lossy(&stderr_buffer).to_string()
            } else {
                "age command failed without stderr output".to_string()
            };
            return Err(AgeError::ProcessExecutionFailed {
                command: "age".into(),
                exit_code: status.code(),
                stderr: stderr_msg,
            });
        }

        Ok(bytes_copied)
    }

    fn decrypt_stream_temp(
        &self,
        input: &mut (dyn Read + Send),
        output: &mut (dyn Write + Send),
        identity: &Identity,
    ) -> AgeResult<u64> {
        let temp_dir = tempdir().map_err(|e| AgeError::TemporaryResourceError {
            resource_type: "dir".into(),
            operation: "create".into(),
            reason: format!("{e:?}"),
        })?;

        let input_path = temp_dir.path().join("stream_input.cage");
        let mut temp_in = File::create(&input_path)
            .map_err(|e| AgeError::file_error("create", input_path.clone(), e))?;
        let bytes_copied = std::io::copy(input, &mut temp_in).map_err(|e| AgeError::IoError {
            operation: "stream_copy".into(),
            context: "decrypt_stream".into(),
            source: e,
        })?;
        temp_in.flush().map_err(|e| AgeError::IoError {
            operation: "flush".into(),
            context: "decrypt_stream".into(),
            source: e,
        })?;

        let output_path = temp_dir.path().join("stream_output");

        match identity {
            Identity::Passphrase(pass) => {
                let automator = PtyAgeAutomator::new()?;
                automator.decrypt(&input_path, &output_path, pass)?;
            }
            Identity::IdentityFile(path) | Identity::SshKey(path) => {
                self.decrypt_with_identity_file(&input_path, &output_path, path)?;
            }
            Identity::PromptPassphrase => {
                return Err(AgeError::AdapterNotImplemented(
                    "PromptPassphrase not supported in ShellAdapterV2".into(),
                ));
            }
        }

        let mut decrypted = File::open(&output_path)
            .map_err(|e| AgeError::file_error("open", output_path.clone(), e))?;
        std::io::copy(&mut decrypted, output).map_err(|e| AgeError::IoError {
            operation: "stream_copy".into(),
            context: "decrypt_stream".into(),
            source: e,
        })?;

        Ok(bytes_copied)
    }

    fn decrypt_stream_pipe(
        &self,
        input: &mut (dyn Read + Send),
        output: &mut (dyn Write + Send),
        identity: &Identity,
    ) -> AgeResult<u64> {
        let identity_path = match identity {
            Identity::IdentityFile(path) | Identity::SshKey(path) => path,
            Identity::PromptPassphrase => {
                return Err(AgeError::AdapterNotImplemented(
                    "PromptPassphrase not supported in ShellAdapterV2".into(),
                ))
            }
            Identity::Passphrase(_) => {
                return Err(AgeError::AdapterNotImplemented(
                    "Passphrase-based streaming requires PTY; pipe strategy unavailable".into(),
                ))
            }
        };

        let mut cmd = Command::new("age");
        cmd.arg("-d");
        cmd.arg("-i");
        cmd.arg(identity_path);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| AgeError::ProcessExecutionFailed {
            command: "age".into(),
            exit_code: None,
            stderr: e.to_string(),
        })?;

        let mut child_stdin =
            child
                .stdin
                .take()
                .ok_or_else(|| AgeError::ProcessExecutionFailed {
                    command: "age".into(),
                    exit_code: None,
                    stderr: "Failed to open stdin for age".into(),
                })?;
        let child_stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgeError::ProcessExecutionFailed {
                command: "age".into(),
                exit_code: None,
                stderr: "Failed to open stdout for age".into(),
            })?;
        let mut stderr_pipe = child.stderr.take();
        let mut stderr_buffer = Vec::new();

        let bytes_copied = thread::scope(|scope| -> AgeResult<u64> {
            let output_writer: &mut (dyn Write + Send) = output;
            let stdout_handle = scope.spawn(move || -> AgeResult<u64> {
                let mut stdout_pipe = child_stdout;
                std::io::copy(&mut stdout_pipe, output_writer).map_err(|e| AgeError::IoError {
                    operation: "stream_copy".into(),
                    context: "decrypt_stream_pipe:stdout".into(),
                    source: e,
                })
            });

            let stderr_handle = stderr_pipe.take().map(|mut pipe| {
                let stderr_ref: &mut Vec<u8> = &mut stderr_buffer;
                scope.spawn(move || -> AgeResult<()> {
                    pipe.read_to_end(stderr_ref)
                        .map_err(|e| AgeError::IoError {
                            operation: "read".into(),
                            context: "decrypt_stream_pipe:stderr".into(),
                            source: e,
                        })?;
                    Ok(())
                })
            });

            let bytes_written =
                std::io::copy(input, &mut child_stdin).map_err(|e| AgeError::IoError {
                    operation: "stream_copy".into(),
                    context: "decrypt_stream_pipe:stdin".into(),
                    source: e,
                })?;

            child_stdin.flush().map_err(|e| AgeError::IoError {
                operation: "flush".into(),
                context: "decrypt_stream_pipe:stdin".into(),
                source: e,
            })?;
            drop(child_stdin);

            let _ = Self::join_stream_thread(stdout_handle, "decrypt_stream_pipe:stdout")?;
            if let Some(handle) = stderr_handle {
                let _ = Self::join_stream_thread(handle, "decrypt_stream_pipe:stderr")?;
            }

            Ok(bytes_written)
        })?;

        let status = child.wait().map_err(|e| AgeError::ProcessExecutionFailed {
            command: "age".into(),
            exit_code: None,
            stderr: e.to_string(),
        })?;

        if !status.success() {
            let stderr_msg = if !stderr_buffer.is_empty() {
                String::from_utf8_lossy(&stderr_buffer).to_string()
            } else {
                "age command failed without stderr output".to_string()
            };
            return Err(AgeError::ProcessExecutionFailed {
                command: "age".into(),
                exit_code: status.code(),
                stderr: stderr_msg,
            });
        }

        Ok(bytes_copied)
    }
}

// ============================================================================
// STREAMING UTILITIES
// ============================================================================

/// Buffer size for streaming operations
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

/// Helper for buffered streaming
pub struct StreamBuffer {
    buffer: Vec<u8>,
    position: usize,
    capacity: usize,
}

fn collect_recipient_args(recipients: &[Recipient]) -> AgeResult<Vec<String>> {
    let mut args = Vec::new();
    for recipient in recipients {
        match recipient {
            Recipient::PublicKey(pk) => {
                args.push("-r".to_string());
                args.push(pk.clone());
            }
            Recipient::MultipleKeys(list) => {
                for pk in list {
                    args.push("-r".to_string());
                    args.push(pk.clone());
                }
            }
            Recipient::RecipientsFile(path) => {
                if !path.exists() {
                    return Err(AgeError::InvalidOperation {
                        operation: "encrypt".into(),
                        reason: format!("Recipients file not found: {}", path.display()),
                    });
                }
                args.push("-R".to_string());
                args.push(path.display().to_string());
            }
            Recipient::SshRecipients(keys) => {
                for key in keys {
                    let converted =
                        AgeSshRecipient::from_str(key).map_err(|e| AgeError::InvalidOperation {
                            operation: "ssh_to_recipient".into(),
                            reason: format!("{e:?}"),
                        })?;
                    args.push("-r".to_string());
                    args.push(converted.to_string());
                }
            }
            Recipient::SelfRecipient => {
                return Err(AgeError::AdapterNotImplemented(
                    "Self recipient flow not yet implemented".into(),
                ));
            }
        }
    }
    Ok(args)
}

impl StreamBuffer {
    /// Create a new stream buffer
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            position: 0,
            capacity,
        }
    }

    /// Create with default size
    pub fn default() -> Self {
        Self::new(DEFAULT_BUFFER_SIZE)
    }

    /// Get mutable buffer slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buffer[..self.capacity]
    }

    /// Get buffer slice up to position
    pub fn filled(&self) -> &[u8] {
        &self.buffer[..self.position]
    }

    /// Update position after write
    pub fn advance(&mut self, count: usize) {
        self.position = (self.position + count).min(self.capacity);
    }

    /// Reset buffer
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::ExposeSecret;
    use age::x25519::Identity as X25519Identity;
    use std::env;
    use std::io::Write as IoWrite;
    use tempfile::NamedTempFile;

    struct EnvVarGuard {
        key: String,
        prev: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &str, value: &str) -> Self {
            let prev = env::var(key).ok();
            env::set_var(key, value);
            Self {
                key: key.to_string(),
                prev,
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(prev) = &self.prev {
                env::set_var(&self.key, prev);
            } else {
                env::remove_var(&self.key);
            }
        }
    }

    #[test]
    fn test_stream_buffer() {
        let mut buffer = StreamBuffer::new(1024);
        assert_eq!(buffer.capacity, 1024);
        assert_eq!(buffer.position, 0);

        buffer.advance(512);
        assert_eq!(buffer.position, 512);

        buffer.reset();
        assert_eq!(buffer.position, 0);
    }

    #[test]
    fn test_adapter_capabilities() {
        let caps = AdapterCapabilities {
            passphrase: true,
            public_key: true,
            identity_files: true,
            ssh_recipients: false,
            streaming: true,
            streaming_strategies: StreamingStrategyInfo {
                default: StreamingStrategyKind::Auto,
                configured: StreamingStrategyKind::Auto,
                env_override: None,
                supports_tempfile: true,
                supports_pipe: true,
                auto_fallback: true,
                pipe_requires_recipients: true,
                pipe_requires_identity: true,
            },
            ascii_armor: true,
            hardware_keys: false,
            key_derivation: false,
            max_file_size: Some(1024 * 1024 * 1024), // 1GB
        };

        assert!(caps.passphrase);
        assert!(caps.streaming);
        assert!(caps.streaming_strategies.supports_pipe);
        assert_eq!(
            caps.streaming_strategies.default,
            StreamingStrategyKind::Auto
        );
        assert_eq!(caps.max_file_size, Some(1_073_741_824));
    }

    #[test]
    fn test_shell_adapter_v2_stream_round_trip() {
        if which::which("age").is_err() {
            println!("Streaming test skipped: age binary not available");
            return;
        }

        let adapter = ShellAdapterV2::new().expect("Failed to create ShellAdapterV2");

        let mut plaintext = std::io::Cursor::new(b"streaming round trip".to_vec());
        let mut encrypted = Vec::new();

        adapter
            .encrypt_stream(
                &mut plaintext,
                &mut encrypted,
                &Identity::Passphrase("passphrase123".to_string()),
                None,
                OutputFormat::Binary,
            )
            .expect("Streaming encrypt failed");

        let mut encrypted_cursor = std::io::Cursor::new(encrypted);
        let mut decrypted = Vec::new();

        adapter
            .decrypt_stream(
                &mut encrypted_cursor,
                &mut decrypted,
                &Identity::Passphrase("passphrase123".to_string()),
            )
            .expect("Streaming decrypt failed");

        assert_eq!(decrypted, b"streaming round trip");
    }

    #[test]
    fn test_shell_adapter_v2_pipe_stream_round_trip() {
        if which::which("age").is_err() {
            println!("Pipe streaming test skipped: age binary not available");
            return;
        }

        let adapter = ShellAdapterV2::new().expect("Failed to create ShellAdapterV2");

        let identity = X25519Identity::generate();
        let recipient = identity.to_public().to_string();

        let mut identity_file = NamedTempFile::new().expect("create identity file");
        let identity_string = identity.to_string();
        identity_file
            .write_all(identity_string.expose_secret().as_bytes())
            .expect("write identity");
        identity_file.flush().expect("flush identity file");

        let mut data = vec![0_u8; 512 * 1024];
        for (idx, byte) in data.iter_mut().enumerate() {
            *byte = (idx % 251) as u8;
        }

        let _guard = EnvVarGuard::set("CAGE_STREAMING_STRATEGY", "pipe");

        let recipients = vec![Recipient::PublicKey(recipient)];

        let mut plaintext = std::io::Cursor::new(data.clone());
        let mut encrypted = Vec::new();

        adapter
            .encrypt_stream(
                &mut plaintext,
                &mut encrypted,
                &Identity::Passphrase("placeholder".to_string()),
                Some(&recipients),
                OutputFormat::Binary,
            )
            .expect("Pipe streaming encrypt failed");

        assert!(!encrypted.is_empty());

        let mut encrypted_cursor = std::io::Cursor::new(encrypted);
        let mut decrypted = Vec::new();

        adapter
            .decrypt_stream(
                &mut encrypted_cursor,
                &mut decrypted,
                &Identity::IdentityFile(identity_file.path().to_path_buf()),
            )
            .expect("Pipe streaming decrypt failed");

        assert_eq!(decrypted, data);
    }

    #[test]
    #[ignore = "SSH recipient conversion not fully implemented (CAGE-09/CAGE-14)"]
    fn test_ssh_recipient_conversion() {
        let adapter = ShellAdapterV2::new().expect("Failed to create adapter");
        let ssh_key =
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICowKIiMzZLpy0X58F3RrgPf63HgFUsVTN4egkwh28yk";
        let converted = adapter
            .ssh_to_recipient(ssh_key)
            .expect("Conversion failed");
        assert!(converted.starts_with("age1"));
    }
}
