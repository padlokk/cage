//! Internal helpers for the key generation module.
//!
//! These are intentionally minimal placeholders; real implementations will
//! surface in CAGE-21 as the CLI workflow is delivered.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Compute the default identity path for a newly generated key.
pub(crate) fn default_identity_path() -> PathBuf {
    // Timestamp-based naming placeholder. CAGE-21 will align this with the
    // strategy document and configuration helpers.
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    PathBuf::from(format!("identity-{}.agekey", timestamp))
}

#[cfg(test)]
mod tests {
    use super::default_identity_path;

    #[test]
    fn default_path_has_expected_extension() {
        let path = default_identity_path();
        assert_eq!(
            path.extension().map(|ext| ext.to_string_lossy()),
            Some("agekey".into())
        );
    }
}
