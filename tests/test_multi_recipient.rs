//! Tests for multi-recipient lifecycle management (CAGE-16)

use cage::cage::{
    core::{AgeConfig, AuthorityTier, Identity, LockRequest, MultiRecipientConfig, RecipientGroup},
    manager::cage_manager::CageManager,
};
use std::path::PathBuf;
use tempfile::TempDir;

/// Skip test if age binary is not available
fn skip_if_age_unavailable() -> bool {
    if which::which("age").is_err() {
        println!("Skipping multi-recipient test - age binary not available");
        return true;
    }
    false
}

#[test]
fn test_recipient_group_creation() {
    let group = RecipientGroup::new("test_group".to_string());

    assert_eq!(group.name, "test_group");
    assert!(group.recipients.is_empty());
    assert!(group.tier.is_none());
    assert_eq!(group.len(), 0);
    assert!(group.is_empty());
}

#[test]
fn test_recipient_group_with_tier() {
    let group = RecipientGroup::with_tier("repo_group".to_string(), AuthorityTier::Repository);

    assert_eq!(group.name, "repo_group");
    assert_eq!(group.tier, Some(AuthorityTier::Repository));
}

#[test]
fn test_recipient_group_management() {
    let mut group = RecipientGroup::new("management_test".to_string());

    // Test adding recipients
    group.add_recipient("age1abc123".to_string());
    group.add_recipient("age1def456".to_string());
    assert_eq!(group.len(), 2);
    assert!(!group.is_empty());

    // Test duplicate prevention
    group.add_recipient("age1abc123".to_string());
    assert_eq!(group.len(), 2); // Should still be 2

    // Test contains check
    assert!(group.contains_recipient("age1abc123"));
    assert!(!group.contains_recipient("age1xyz789"));

    // Test removal
    assert!(group.remove_recipient("age1abc123"));
    assert_eq!(group.len(), 1);
    assert!(!group.remove_recipient("nonexistent"));

    // Test group hash for audit
    let hash = group.group_hash();
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 32); // MD5 hex string length
}

#[test]
fn test_authority_tier_enum() {
    // Test string conversion
    assert_eq!(AuthorityTier::Skull.as_str(), "X");
    assert_eq!(AuthorityTier::Master.as_str(), "M");
    assert_eq!(AuthorityTier::Repository.as_str(), "R");
    assert_eq!(AuthorityTier::Ignition.as_str(), "I");
    assert_eq!(AuthorityTier::Distro.as_str(), "D");

    // Test parsing
    assert_eq!(AuthorityTier::from_str("X"), Some(AuthorityTier::Skull));
    assert_eq!(AuthorityTier::from_str("x"), Some(AuthorityTier::Skull)); // Case insensitive
    assert_eq!(AuthorityTier::from_str("M"), Some(AuthorityTier::Master));
    assert_eq!(
        AuthorityTier::from_str("R"),
        Some(AuthorityTier::Repository)
    );
    assert_eq!(AuthorityTier::from_str("I"), Some(AuthorityTier::Ignition));
    assert_eq!(AuthorityTier::from_str("D"), Some(AuthorityTier::Distro));
    assert_eq!(AuthorityTier::from_str("Z"), None); // Invalid
}

#[test]
fn test_multi_recipient_config() {
    let mut primary_group =
        RecipientGroup::with_tier("primary".to_string(), AuthorityTier::Repository);
    primary_group.add_recipient("age1primary1".to_string());
    primary_group.add_recipient("age1primary2".to_string());

    let mut secondary_group =
        RecipientGroup::with_tier("secondary".to_string(), AuthorityTier::Ignition);
    secondary_group.add_recipient("age1secondary1".to_string());

    let config = MultiRecipientConfig::new()
        .with_primary_group(primary_group)
        .add_group(secondary_group)
        .with_authority_validation(true)
        .with_hierarchy_enforcement(true);

    // Test flattening
    let all_recipients = config.flatten_recipients();
    assert_eq!(all_recipients.len(), 3);
    assert!(all_recipients.contains(&"age1primary1".to_string()));
    assert!(all_recipients.contains(&"age1primary2".to_string()));
    assert!(all_recipients.contains(&"age1secondary1".to_string()));

    // Test total count
    assert_eq!(config.total_recipients(), 3);

    // Test groups access
    let all_groups = config.all_groups();
    assert_eq!(all_groups.len(), 2);

    // Test validation flags
    assert!(config.validate_authority);
    assert!(config.enforce_hierarchy);
}

#[test]
fn test_age_config_recipient_groups() {
    let mut config = AgeConfig::default();

    // Test initial state
    assert_eq!(config.get_recipient_group_count(), 0);
    assert_eq!(config.get_total_recipients_count(), 0);
    assert!(config.list_recipient_groups().is_empty());

    // Test adding a group
    let mut test_group = RecipientGroup::with_tier("test".to_string(), AuthorityTier::Master);
    test_group.add_recipient("age1test123".to_string());
    test_group.add_recipient("age1test456".to_string());

    config.add_recipient_group(test_group);

    // Test counts after adding
    assert_eq!(config.get_recipient_group_count(), 1);
    assert_eq!(config.get_total_recipients_count(), 2);

    // Test retrieval
    let groups = config.list_recipient_groups();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0], "test");

    let retrieved_group = config.get_recipient_group("test");
    assert!(retrieved_group.is_some());
    assert_eq!(retrieved_group.unwrap().len(), 2);

    // Test groups by tier
    let master_groups = config.get_groups_by_tier(AuthorityTier::Master);
    assert_eq!(master_groups.len(), 1);
    assert_eq!(master_groups[0].name, "test");

    let skull_groups = config.get_groups_by_tier(AuthorityTier::Skull);
    assert_eq!(skull_groups.len(), 0);

    // Test removal
    let removed = config.remove_recipient_group("test");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name, "test");
    assert_eq!(config.get_recipient_group_count(), 0);
}

#[test]
fn test_padlock_extension_support() {
    let config = AgeConfig::default();

    // Test default extensions
    assert!(config.padlock_extension_support);
    assert!(config.encrypted_extensions.contains(&"cage".to_string()));
    assert!(config.encrypted_extensions.contains(&"age".to_string()));
    assert!(config.encrypted_extensions.contains(&"padlock".to_string()));

    // Test file extension detection
    let cage_file = std::path::Path::new("test.cage");
    let age_file = std::path::Path::new("test.age");
    let padlock_file = std::path::Path::new("test.padlock");
    let txt_file = std::path::Path::new("test.txt");

    assert!(config.is_encrypted_file(cage_file));
    assert!(config.is_encrypted_file(age_file));
    assert!(config.is_encrypted_file(padlock_file));
    assert!(!config.is_encrypted_file(txt_file));
}

#[test]
fn test_lock_request_with_multi_recipient_config() {
    let mut primary_group = RecipientGroup::new("primary".to_string());
    primary_group.add_recipient("age1primary".to_string());

    let multi_config = MultiRecipientConfig::new().with_primary_group(primary_group);

    let request = LockRequest::new(
        PathBuf::from("/test/file.txt"),
        Identity::Passphrase("test123".to_string()),
    )
    .with_multi_recipient_config(multi_config);

    assert!(request.multi_recipient_config.is_some());

    let config = request.multi_recipient_config.unwrap();
    assert_eq!(config.total_recipients(), 1);
}

#[test]
fn test_cage_manager_recipient_lifecycle() {
    if let Ok(mut crud_manager) = CageManager::with_defaults() {
        // Test creating a recipient group
        let result =
            crud_manager.create_recipient_group("test_group", Some(AuthorityTier::Repository));
        assert!(result.is_ok());

        // Test listing groups
        let groups = crud_manager.list_recipient_groups();
        assert!(groups.contains(&"test_group".to_string()));

        // Test adding recipient to group
        let add_result = crud_manager.add_recipient_to_group("test_group", "age1test123");
        assert!(add_result.is_ok());

        // Test removing recipient from group
        let remove_result = crud_manager.remove_recipient_from_group("test_group", "age1test123");
        assert!(remove_result.is_ok());
        assert!(remove_result.unwrap()); // Should return true for successful removal

        // Test removing non-existent recipient
        let remove_nonexistent =
            crud_manager.remove_recipient_from_group("test_group", "nonexistent");
        assert!(remove_nonexistent.is_ok());
        assert!(!remove_nonexistent.unwrap()); // Should return false

        // Test audit functionality
        let audit_result = crud_manager.audit_recipient_groups();
        assert!(audit_result.is_ok());
        let audit_report = audit_result.unwrap();
        assert!(!audit_report.is_empty());

        // Test tier-based retrieval
        let repo_groups = crud_manager.get_groups_by_tier(AuthorityTier::Repository);
        assert!(repo_groups.contains(&"test_group".to_string()));

        // Test adapter info
        let info = crud_manager.get_adapter_info_with_groups();
        assert!(info.contains_key("total_groups"));
        assert!(info.contains_key("total_recipients"));
        assert!(info.contains_key("padlock_support"));
        assert!(info.contains_key("r_groups")); // Repository tier is "R"
    } else {
        println!("Skipping CageManager lifecycle test - Age not available");
    }
}

/// Integration test for multi-recipient encryption (requires age binary)
#[test]
fn test_multi_recipient_encryption_integration() {
    if skip_if_age_unavailable() {
        return;
    }

    // Note: This is a conceptual test structure.
    // Full implementation would require actual age recipient keys for testing.
    // For now, we test the request structure and validation logic.

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_file = temp_dir.path().join("test_file.txt");
    std::fs::write(&test_file, b"Test content for multi-recipient encryption").unwrap();

    // Create recipient groups for testing
    let mut repo_group =
        RecipientGroup::with_tier("repository".to_string(), AuthorityTier::Repository);
    // In real testing, these would be actual age public keys
    repo_group.add_recipient("age1mock_repo_key_123".to_string());

    let mut ignition_group =
        RecipientGroup::with_tier("ignition".to_string(), AuthorityTier::Ignition);
    ignition_group.add_recipient("age1mock_ignition_key_456".to_string());

    let multi_config = MultiRecipientConfig::new()
        .with_primary_group(repo_group)
        .add_group(ignition_group)
        .with_authority_validation(false) // Disabled for mock testing
        .with_hierarchy_enforcement(false); // Disabled for mock testing

    let request = LockRequest::new(
        test_file,
        Identity::Passphrase("test_passphrase".to_string()),
    )
    .with_multi_recipient_config(multi_config);

    // Validate request structure
    assert!(request.multi_recipient_config.is_some());
    let config = request.multi_recipient_config.as_ref().unwrap();
    assert_eq!(config.total_recipients(), 2);
    assert_eq!(config.all_groups().len(), 2);

    // Note: Actual encryption testing would require valid age keys
    // and would test that each recipient can successfully decrypt the file
    println!("Multi-recipient request structure validated successfully");
}

/// Test ensuring multiple recipients can decrypt the same file
/// This is a placeholder for real decryption testing with actual age keys
#[test]
fn test_multi_recipient_decryption_scenarios() {
    if skip_if_age_unavailable() {
        return;
    }

    // This test structure demonstrates the validation approach
    // In real implementation, this would:
    // 1. Generate actual age key pairs for testing
    // 2. Encrypt a file with multiple recipients
    // 3. Verify each recipient can decrypt independently
    // 4. Test rotation scenarios where recipients are added/removed

    println!("Placeholder: Multi-recipient decryption scenarios would test:");
    println!("- Each recipient can decrypt files encrypted to their group");
    println!("- Rotation scenarios work correctly");
    println!("- Authority tier validation prevents unauthorized access");
    println!("- Audit logging captures all multi-recipient operations");
}
