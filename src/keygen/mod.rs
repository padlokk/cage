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
    fn keygen_service_defaults_return_not_implemented() {
        let service = KeygenService::default();
        let request = KeygenRequest::default();
        let result = service.generate(&request);
        assert!(matches!(result, Err(KeygenError::NotImplemented)));
    }
}
