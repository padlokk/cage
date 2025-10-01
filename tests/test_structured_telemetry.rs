//! Tests for structured audit and telemetry (OBS-01)

use cage::{
    audit::AuditLogger,
    core::{AgeConfig, TelemetryFormat},
};
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_text_format_telemetry() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("audit.log");

    let logger = AuditLogger::with_format(Some(log_path.clone()), TelemetryFormat::Text)
        .expect("Failed to create logger");

    logger.log_info("Test message").expect("Failed to log");
    logger.log_warning("Test warning").expect("Failed to log");
    logger.log_error("Test error").expect("Failed to log");

    let content = fs::read_to_string(&log_path).expect("Failed to read log file");

    // Verify text format
    assert!(content.contains("[INFO]"));
    assert!(content.contains("[WARN]"));
    assert!(content.contains("[ERROR]"));
    assert!(content.contains("cage_automation"));
    assert!(content.contains("Test message"));
    assert!(content.contains("Test warning"));
    assert!(content.contains("Test error"));
}

#[test]
fn test_json_format_telemetry() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("audit.json");

    let logger = AuditLogger::with_format(Some(log_path.clone()), TelemetryFormat::Json)
        .expect("Failed to create logger");

    logger.log_info("Test JSON message").expect("Failed to log");

    let content = fs::read_to_string(&log_path).expect("Failed to read log file");
    let lines: Vec<&str> = content.trim().split('\n').collect();

    // Parse JSON and verify structure
    for line in lines {
        let json: Value = serde_json::from_str(line).expect("Failed to parse JSON log line");

        assert!(json.get("timestamp").is_some());
        assert!(json.get("level").is_some());
        assert!(json.get("component").is_some());
        assert!(json.get("message").is_some());

        // Verify RFC3339 timestamp format
        let timestamp = json.get("timestamp").unwrap().as_str().unwrap();
        assert!(timestamp.contains('T')); // ISO format
        assert!(timestamp.contains('Z') || timestamp.contains('+'));
    }
}

#[test]
fn test_encryption_event_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("encryption.json");

    let logger = AuditLogger::with_format(Some(log_path.clone()), TelemetryFormat::Json)
        .expect("Failed to create logger");

    // Test encryption event with recipients
    let recipients = vec!["age1abc...".to_string(), "age1def...".to_string()];
    logger
        .log_encryption_event(Path::new("test.txt"), Some(recipients), "passphrase", true)
        .expect("Failed to log");

    let content = fs::read_to_string(&log_path).expect("Failed to read log file");
    let json: Value = serde_json::from_str(&content.trim()).expect("Failed to parse JSON");

    assert_eq!(json.get("event_type").unwrap(), "encryption");
    assert_eq!(json.get("identity_type").unwrap(), "passphrase");
    assert_eq!(json.get("recipient_count").unwrap(), 2);
    assert!(json.get("recipient_group_hash").is_some());
    assert_eq!(json.get("success").unwrap(), true);
}

#[test]
fn test_decryption_event_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("decryption.json");

    let logger = AuditLogger::with_format(Some(log_path.clone()), TelemetryFormat::Json)
        .expect("Failed to create logger");

    logger
        .log_decryption_event(Path::new("test.age"), "ssh-key", true)
        .expect("Failed to log");

    let content = fs::read_to_string(&log_path).expect("Failed to read log file");
    let json: Value = serde_json::from_str(&content.trim()).expect("Failed to parse JSON");

    assert_eq!(json.get("event_type").unwrap(), "decryption");
    assert_eq!(json.get("identity_type").unwrap(), "ssh-key");
    assert_eq!(json.get("path").unwrap(), "test.age");
    assert_eq!(json.get("success").unwrap(), true);
}

#[test]
fn test_telemetry_format_from_config() {
    let mut config = AgeConfig::default();
    assert_eq!(config.telemetry_format, TelemetryFormat::Text);

    config.telemetry_format = TelemetryFormat::Json;
    assert_eq!(config.telemetry_format, TelemetryFormat::Json);
}

#[test]
fn test_sensitive_field_redaction() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("sensitive.json");

    let logger = AuditLogger::with_format(Some(log_path.clone()), TelemetryFormat::Json)
        .expect("Failed to create logger");

    // Log encryption with recipients - should hash, not expose
    let recipients = vec!["ssh-ed25519 AAAAC3NzaC1lZDI1NTE5...".to_string()];
    logger
        .log_encryption_event(
            Path::new("secret.txt"),
            Some(recipients.clone()),
            "identity-file",
            true,
        )
        .expect("Failed to log");

    let content = fs::read_to_string(&log_path).expect("Failed to read log file");

    // Verify recipient keys are NOT in plaintext
    assert!(!content.contains("AAAAC3NzaC1lZDI1NTE5"));
    assert!(content.contains("recipient_group_hash"));
}
