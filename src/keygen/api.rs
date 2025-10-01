//! Key generation service API.
//!
//! This stub captures the contract that the CLI and future library callers will
//! rely on. Implementation work is tracked under task CAGE-21 (CLI workflow) and
//! CAGE-22 (adapter-native identities). See `docs/ref/cage/KEYGEN_STRATEGY.md`
//! for the authoritative specification.

use crate::core::AgeConfig;
use crate::keygen::error::KeygenError;
use std::path::PathBuf;

/// Request payload accepted by the key generation service.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct KeygenRequest {
    /// Optional explicit output path supplied by the caller.
    pub output_path: Option<PathBuf>,
    /// Optional input identity when operating in recipients-only mode.
    pub input_path: Option<PathBuf>,
    /// Recipient groups to register the generated public key with.
    pub register_groups: Vec<String>,
    /// When true, convert an existing identity to recipients-only output.
    pub recipients_only: bool,
    /// Permit overwriting an existing output file.
    pub force: bool,
    /// Skip writing to disk and emit results on stdout only.
    pub stdout_only: bool,
    /// Emit structured JSON summary.
    pub json_output: bool,
    /// Force proxy mode (direct passthrough to `age-keygen`).
    pub proxy_mode: bool,
    /// Export mode: generate keypair to current directory without registry entry.
    pub export_mode: bool,
}

/// Result summary returned by the key generation workflow.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct KeygenSummary {
    /// Path where the identity (or recipients) was written, when applicable.
    pub output_path: Option<PathBuf>,
    /// Derived public recipient (if available).
    pub public_recipient: Option<String>,
    /// Collected MD5 fingerprint of the private key (if applicable).
    pub fingerprint_md5: Option<String>,
    /// Collected SHA256 fingerprint of the private key (if applicable).
    pub fingerprint_sha256: Option<String>,
    /// Recipient groups that were updated as part of the request.
    pub registered_groups: Vec<String>,
}

/// Primary key generation service entry point.
#[derive(Debug, Clone, Default)]
pub struct KeygenService {
    config: Option<AgeConfig>,
}

impl KeygenService {
    /// Create a new service with the provided configuration context.
    pub fn new(config: Option<AgeConfig>) -> Self {
        Self { config }
    }

    /// Generate an identity according to `request`.
    pub fn generate(&self, request: &KeygenRequest) -> Result<KeygenSummary, KeygenError> {
        use crate::keygen::{audit, helpers};
        use std::fs;
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Log operation start
        audit::log_keygen_start(request);

        // Validate request
        self.validate_request(request)?;

        // Handle recipients-only mode
        if request.recipients_only {
            return self.handle_recipients_only(request);
        }

        // Handle proxy mode (direct passthrough to age-keygen)
        if request.proxy_mode {
            return self.handle_proxy_mode(request);
        }

        // Check age-keygen availability
        helpers::check_age_keygen_available()?;

        // Determine output path
        let output_path = if let Some(ref path) = request.output_path {
            path.clone()
        } else if request.export_mode {
            helpers::export_identity_path()?
        } else {
            helpers::default_identity_path()?
        };

        // Check overwrite protection
        if output_path.exists() && !request.force {
            return Err(KeygenError::FileExists(
                output_path.to_string_lossy().to_string(),
            ));
        }

        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| KeygenError::Io(format!("failed to create directory: {}", e)))?;
            }
        }

        // Generate identity by invoking age-keygen
        let output = Command::new("age-keygen")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| KeygenError::Subprocess(format!("failed to execute age-keygen: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(KeygenError::Subprocess(format!(
                "age-keygen failed: {}",
                stderr
            )));
        }

        // Parse identity output
        let identity_content = String::from_utf8_lossy(&output.stdout);

        // Write identity to file
        let mut file = fs::File::create(&output_path)
            .map_err(|e| KeygenError::Io(format!("failed to create identity file: {}", e)))?;
        file.write_all(identity_content.as_bytes())
            .map_err(|e| KeygenError::Io(format!("failed to write identity: {}", e)))?;

        // Set secure permissions
        helpers::set_identity_permissions(&output_path)?;

        // Extract public key using age-keygen -y
        let pub_output = Command::new("age-keygen")
            .arg("-y")
            .arg(&output_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                KeygenError::Subprocess(format!("failed to extract public key: {}", e))
            })?;

        if !pub_output.status.success() {
            let stderr = String::from_utf8_lossy(&pub_output.stderr);
            return Err(KeygenError::Subprocess(format!(
                "age-keygen -y failed: {}",
                stderr
            )));
        }

        let public_recipient = String::from_utf8_lossy(&pub_output.stdout)
            .trim()
            .to_string();

        // Compute fingerprints
        let fingerprint_md5 = helpers::compute_fingerprint_md5(&public_recipient);
        let fingerprint_sha256 = helpers::compute_fingerprint_sha256(&public_recipient);

        // Handle group registration (skip if export mode)
        let registered_groups = if !request.export_mode && !request.register_groups.is_empty() {
            self.register_with_groups(&public_recipient, &request.register_groups)?
        } else {
            Vec::new()
        };

        // Build summary
        let summary = KeygenSummary {
            output_path: Some(output_path),
            public_recipient: Some(public_recipient),
            fingerprint_md5: Some(fingerprint_md5),
            fingerprint_sha256: Some(fingerprint_sha256),
            registered_groups,
        };

        // Log completion
        audit::log_keygen_complete(&summary);

        Ok(summary)
    }

    /// Validate the request for conflicting options.
    fn validate_request(&self, request: &KeygenRequest) -> Result<(), KeygenError> {
        // Export mode conflicts with register
        if request.export_mode && !request.register_groups.is_empty() {
            return Err(KeygenError::InvalidRequest(
                "--export cannot be used with --register".to_string(),
            ));
        }

        // Recipients-only conflicts with register
        if request.recipients_only && !request.register_groups.is_empty() {
            return Err(KeygenError::InvalidRequest(
                "--recipients-only cannot be used with --register".to_string(),
            ));
        }

        Ok(())
    }

    /// Handle recipients-only mode (convert identity to public key).
    fn handle_recipients_only(&self, request: &KeygenRequest) -> Result<KeygenSummary, KeygenError> {
        use crate::keygen::helpers;
        use std::process::{Command, Stdio};

        helpers::check_age_keygen_available()?;

        let output = if let Some(ref input_path) = request.input_path {
            // Use input file
            Command::new("age-keygen")
                .arg("-y")
                .arg(input_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| {
                    KeygenError::Subprocess(format!("failed to extract public key: {}", e))
                })?
        } else {
            // Read from stdin (not implemented yet - would need piping)
            return Err(KeygenError::NotImplemented);
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(KeygenError::Subprocess(format!(
                "age-keygen -y failed: {}",
                stderr
            )));
        }

        let public_recipient = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        let fingerprint_md5 = helpers::compute_fingerprint_md5(&public_recipient);
        let fingerprint_sha256 = helpers::compute_fingerprint_sha256(&public_recipient);

        Ok(KeygenSummary {
            output_path: request.output_path.clone(),
            public_recipient: Some(public_recipient),
            fingerprint_md5: Some(fingerprint_md5),
            fingerprint_sha256: Some(fingerprint_sha256),
            registered_groups: Vec::new(),
        })
    }

    /// Handle proxy mode (direct passthrough to age-keygen).
    fn handle_proxy_mode(&self, _request: &KeygenRequest) -> Result<KeygenSummary, KeygenError> {
        use crate::keygen::helpers;
        use std::process::{Command, Stdio};

        helpers::check_age_keygen_available()?;

        // Direct execution - output goes to stdout/stderr as-is
        let status = Command::new("age-keygen")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| KeygenError::Subprocess(format!("failed to execute age-keygen: {}", e)))?;

        if !status.success() {
            return Err(KeygenError::Subprocess(
                "age-keygen proxy mode failed".to_string(),
            ));
        }

        // Proxy mode doesn't capture details
        Ok(KeygenSummary::default())
    }

    /// Register the public recipient with the specified groups.
    fn register_with_groups(
        &self,
        _public_recipient: &str,
        groups: &[String],
    ) -> Result<Vec<String>, KeygenError> {
        // Get mutable config reference
        let config = self.config.as_ref().ok_or_else(|| {
            KeygenError::InvalidRequest("config required for group registration".to_string())
        })?;

        let mut registered = Vec::new();

        for group_name in groups {
            // Validate group exists
            if config.get_recipient_group(group_name).is_none() {
                return Err(KeygenError::InvalidGroup(group_name.clone()));
            }

            // Note: We need mutable access to actually append the recipient
            // This will be handled by the CLI layer which can get a mutable config
            // For now, we just validate and track what would be registered
            registered.push(group_name.clone());
        }

        Ok(registered)
    }

    /// Access the underlying configuration (when available).
    pub fn config(&self) -> Option<&AgeConfig> {
        self.config.as_ref()
    }
}
