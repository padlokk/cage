//! Key generation module (see docs/ref/cage/KEYGEN_STRATEGY.md)
//!
//! Provides the orchestrated service API used by the `cage keygen` command.
//!
//! Layout follows MODULE_SPEC v3 guidance so the module can be promoted to a
//! standalone tool in the future without entangling the rest of the crate.

pub mod api;
pub mod error;
pub(crate) mod audit;
pub(crate) mod helpers;

pub use api::{KeygenRequest, KeygenService, KeygenSummary};
pub use error::KeygenError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keygen_request_validation_export_with_register() {
        let service = KeygenService::default();
        let request = KeygenRequest {
            export_mode: true,
            register_groups: vec!["test-group".to_string()],
            ..Default::default()
        };

        let result = service.generate(&request);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, KeygenError::InvalidRequest(_)));
        assert!(err.to_string().contains("export"));
        assert!(err.to_string().contains("register"));
    }

    #[test]
    fn keygen_request_validation_recipients_only_with_register() {
        let service = KeygenService::default();
        let request = KeygenRequest {
            recipients_only: true,
            register_groups: vec!["test-group".to_string()],
            ..Default::default()
        };

        let result = service.generate(&request);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, KeygenError::InvalidRequest(_)));
    }

    #[test]
    fn keygen_fingerprint_helpers() {
        let public_key = "age1abcdefghijklmnopqrstuvwxyz1234567890";

        let md5 = helpers::compute_fingerprint_md5(public_key);
        assert!(md5.starts_with("MD5:"));
        assert!(md5.contains(':'));

        let sha256 = helpers::compute_fingerprint_sha256(public_key);
        assert!(sha256.starts_with("SHA256:"));
        assert!(!sha256[7..].contains(':')); // Base64 portion should not have colons
    }

    #[test]
    fn keygen_path_helpers() {
        // Test default path generation
        let default_path = helpers::default_identity_path();
        assert!(default_path.is_ok());
        if let Ok(path) = default_path {
            assert!(path.to_string_lossy().contains("identities"));
            assert_eq!(path.extension().and_then(|s| s.to_str()), Some("cagekey"));
        }

        // Test export path generation
        let export_path = helpers::export_identity_path();
        assert!(export_path.is_ok());
        if let Ok(path) = export_path {
            assert!(!path.to_string_lossy().contains("identities"));
            assert_eq!(path.extension().and_then(|s| s.to_str()), Some("cagekey"));
        }
    }

    #[test]
    fn keygen_binary_check() {
        // This test just validates the error type when binary is missing
        let result = helpers::check_age_keygen_available();
        if let Err(e) = result {
            assert!(matches!(e, KeygenError::BinaryNotFound(_)));
            assert!(e.to_string().contains("age-keygen"));
        }
    }
}
