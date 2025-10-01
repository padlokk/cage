//! Audit logging for key generation operations.

use crate::keygen::api::{KeygenRequest, KeygenSummary};
use chrono::Utc;

/// Log the start of a key generation operation.
pub(crate) fn log_keygen_start(request: &KeygenRequest) {
    let timestamp = Utc::now().to_rfc3339();
    let mode = if request.recipients_only {
        "recipients-only"
    } else if request.export_mode {
        "export"
    } else if request.proxy_mode {
        "proxy"
    } else {
        "generate"
    };

    eprintln!(
        "[AUDIT] {} KEYGEN_START mode={} output={:?} export={} register={:?}",
        timestamp,
        mode,
        request.output_path,
        request.export_mode,
        request.register_groups
    );
}

/// Log the completion of a key generation operation.
/// Ensures NO secrets are logged (§6 requirement).
pub(crate) fn log_keygen_complete(summary: &KeygenSummary) {
    let timestamp = Utc::now().to_rfc3339();

    // Redact the full public key, only log a hash for audit trail
    let recipient_hash = summary
        .public_recipient
        .as_ref()
        .map(|r| format!("{:x}", md5::compute(r.as_bytes())))
        .unwrap_or_else(|| "none".to_string());

    eprintln!(
        "[AUDIT] {} KEYGEN_COMPLETE path={:?} recipient_hash={} md5={:?} sha256={:?} groups={:?}",
        timestamp,
        summary.output_path,
        recipient_hash,
        summary.fingerprint_md5,
        summary.fingerprint_sha256,
        summary.registered_groups
    );
}
