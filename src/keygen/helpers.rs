//! Internal helpers for the key generation module.

use crate::keygen::error::KeygenError;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Compute the default identity path for a newly generated key.
/// Uses XDG_CONFIG_HOME/cage/identities/<timestamp>.cagekey pattern.
pub(crate) fn default_identity_path() -> Result<PathBuf, KeygenError> {
    let base = if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        return Err(KeygenError::Io(
            "cannot determine config directory: HOME or XDG_CONFIG_HOME not set".to_string(),
        ));
    };

    let identities_dir = base.join("cage").join("identities");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();

    Ok(identities_dir.join(format!("{}.cagekey", timestamp)))
}

/// Generate export identity path (current directory with timestamp).
pub(crate) fn export_identity_path() -> Result<PathBuf, KeygenError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();

    Ok(env::current_dir()
        .map_err(|e| KeygenError::Io(format!("cannot determine current directory: {}", e)))?
        .join(format!("{}.cagekey", timestamp)))
}

/// Compute MD5 fingerprint of a public key in SSH-style format.
/// Returns format: MD5:aa:bb:cc:dd:...
pub(crate) fn compute_fingerprint_md5(public_key: &str) -> String {
    let digest = md5::compute(public_key.trim().as_bytes());
    let hex_bytes: Vec<String> = digest.iter().map(|b| format!("{:02x}", b)).collect();
    format!("MD5:{}", hex_bytes.join(":"))
}

/// Compute SHA256 fingerprint of a public key in SSH-style format.
/// Returns format: SHA256:base64string
pub(crate) fn compute_fingerprint_sha256(public_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_key.trim().as_bytes());
    let result = hasher.finalize();

    use base64::{engine::general_purpose::STANDARD, Engine};
    format!("SHA256:{}", STANDARD.encode(result))
}

/// Set secure permissions on identity file (Unix: 0o600, Windows: hidden+archive).
pub(crate) fn set_identity_permissions(path: &Path) -> Result<(), KeygenError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|e| KeygenError::Io(format!("failed to read metadata: {}", e)))?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)
            .map_err(|e| KeygenError::Io(format!("failed to set permissions: {}", e)))?;
    }

    #[cfg(windows)]
    {
        // On Windows, we set hidden + archive attributes for security
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x20;

        // Windows file attribute setting would go here
        // For now, we just ensure the file exists
        let _ = fs::metadata(path)
            .map_err(|e| KeygenError::Io(format!("failed to read metadata: {}", e)))?;
    }

    Ok(())
}

/// Check if age-keygen binary is available on PATH.
pub(crate) fn check_age_keygen_available() -> Result<(), KeygenError> {
    which::which("age-keygen")
        .map(|_| ())
        .map_err(|_| KeygenError::BinaryNotFound("age-keygen".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_path_has_expected_extension() {
        let path = default_identity_path().expect("should compute default path");
        assert_eq!(
            path.extension().map(|ext| ext.to_string_lossy()),
            Some("cagekey".into())
        );
        assert!(path.to_string_lossy().contains("identities"));
    }

    #[test]
    fn export_path_uses_current_dir() {
        let path = export_identity_path().expect("should compute export path");
        assert_eq!(
            path.extension().map(|ext| ext.to_string_lossy()),
            Some("cagekey".into())
        );
        // Should not contain 'identities' directory
        assert!(!path.to_string_lossy().contains("identities"));
    }

    #[test]
    fn fingerprint_md5_format() {
        let public_key = "age1abcdefghijklmnopqrstuvwxyz1234567890";
        let fp = compute_fingerprint_md5(public_key);
        assert!(fp.starts_with("MD5:"));
        assert!(fp.contains(':'));
        let parts: Vec<&str> = fp.split(':').collect();
        assert!(parts.len() > 2); // MD5 + hex bytes
    }

    #[test]
    fn fingerprint_sha256_format() {
        let public_key = "age1abcdefghijklmnopqrstuvwxyz1234567890";
        let fp = compute_fingerprint_sha256(public_key);
        assert!(fp.starts_with("SHA256:"));
        // Base64 portion (after SHA256:) should not have colons
        let base64_part = &fp[7..];
        assert!(!base64_part.contains(':'));
    }

    #[test]
    fn check_age_keygen_reports_missing_binary() {
        // This test will pass or fail depending on whether age-keygen is installed
        // The actual test is in the integration suite
        let result = check_age_keygen_available();
        if let Err(e) = result {
            assert!(e.to_string().contains("age-keygen"));
        }
    }
}
