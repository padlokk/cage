use padlock::authority::chain::{KeyType, AuthorityKey, AuthorityChain, KeyMaterial, KeyFormat, KeyMetadata};
use padlock::authority::ignition::{IgnitionKey, validate_passphrase_strength};
use padlock::authority::validation::{AuthorityValidationEngine, validate_authority_hierarchy, AuthorityLevel};
use padlock::encryption::age_automation::{AgeConfig, OutputFormat, TtyMethod, AgeResult};
use std::path::Path;
use chrono::Utc;

/// Test Suite: Authority Chain Data Structures API
/// Validates complete Story 1.1 implementation
#[cfg(test)]
mod authority_chain_tests {
    use super::*;

    #[test]
    fn test_key_type_hierarchy_api() {
        // Test complete X->M->R->I->D hierarchy
        assert!(KeyType::Skull.can_control(KeyType::Master));
        assert!(KeyType::Master.can_control(KeyType::Repo));
        assert!(KeyType::Repo.can_control(KeyType::Ignition));
        assert!(KeyType::Ignition.can_control(KeyType::Distro));
        
        // Test invalid relationships
        assert!(!KeyType::Distro.can_control(KeyType::Ignition));
        assert!(!KeyType::Master.can_control(KeyType::Ignition)); // Must go through Repo
        assert!(!KeyType::Repo.can_control(KeyType::Master)); // Reverse not allowed
    }

    #[test] 
    fn test_ignition_key_detection_api() {
        // Test which keys can be ignition keys (passphrase-wrapped)
        assert!(KeyType::Skull.is_ignition_key()); // X keys
        assert!(!KeyType::Master.is_ignition_key()); // M keys (direct)
        assert!(!KeyType::Repo.is_ignition_key()); // R keys (direct)
        assert!(KeyType::Ignition.is_ignition_key()); // I keys
        assert!(KeyType::Distro.is_ignition_key()); // D keys
    }

    #[test]
    fn test_key_type_parent_child_api() {
        // Test parent relationships
        assert_eq!(KeyType::Master.parent_type(), Some(KeyType::Skull));
        assert_eq!(KeyType::Repo.parent_type(), Some(KeyType::Master));
        assert_eq!(KeyType::Ignition.parent_type(), Some(KeyType::Repo));
        assert_eq!(KeyType::Distro.parent_type(), Some(KeyType::Ignition));
        assert_eq!(KeyType::Skull.parent_type(), None);

        // Test child relationships
        assert_eq!(KeyType::Skull.child_types(), vec![KeyType::Master]);
        assert_eq!(KeyType::Master.child_types(), vec![KeyType::Repo]);
        assert_eq!(KeyType::Repo.child_types(), vec![KeyType::Ignition]);
        assert_eq!(KeyType::Ignition.child_types(), vec![KeyType::Distro]);
        assert_eq!(KeyType::Distro.child_types(), vec![]);
    }

    #[test]
    fn test_authority_key_creation_api() {
        let key_material = KeyMaterial::new(
            b"test_public_key".to_vec(),
            Some(b"test_private_key".to_vec()),
            KeyFormat::Age,
        );

        let metadata = KeyMetadata {
            creation_time: Utc::now(),
            creator: "test_creator".to_string(),
            description: "Test authority key".to_string(),
            expiration: None,
            last_used: None,
            usage_count: 0,
        };

        let authority_key = AuthorityKey::new(
            key_material,
            KeyType::Master,
            None,
            Some(metadata.clone()),
        ).expect("Authority key creation should succeed");

        assert_eq!(authority_key.key_type(), KeyType::Master);
        assert_eq!(authority_key.metadata().creator, "test_creator");
        assert_eq!(authority_key.metadata().description, "Test authority key");
        assert!(!authority_key.is_expired());
    }

    #[test]
    fn test_authority_chain_management_api() {
        let mut chain = AuthorityChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);

        // Create test keys
        let master_key_material = KeyMaterial::new(
            b"master_public".to_vec(),
            Some(b"master_private".to_vec()),
            KeyFormat::Age,
        );
        let repo_key_material = KeyMaterial::new(
            b"repo_public".to_vec(),
            Some(b"repo_private".to_vec()),
            KeyFormat::Age,
        );

        let master_key = AuthorityKey::new(master_key_material, KeyType::Master, None, None)
            .expect("Master key creation should succeed");
        let repo_key = AuthorityKey::new(repo_key_material, KeyType::Repo, None, None)
            .expect("Repo key creation should succeed");

        let master_fp = master_key.fingerprint().clone();
        let repo_fp = repo_key.fingerprint().clone();

        // Add keys to chain
        chain.add_key(master_key).expect("Adding master key should succeed");
        chain.add_key(repo_key).expect("Adding repo key should succeed");

        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());

        // Add authority relationship
        chain.add_authority_relationship(&master_fp, &repo_fp)
            .expect("Adding authority relationship should succeed");

        // Test relationship queries
        assert!(chain.has_authority(&master_fp, &repo_fp));
        assert!(chain.is_subject_to(&repo_fp, &master_fp));
        assert!(!chain.has_authority(&repo_fp, &master_fp)); // Reverse not true

        // Test parent/child retrieval
        let children = chain.get_children(&master_fp);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].key_type(), KeyType::Repo);

        let parent = chain.get_parent(&repo_fp);
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().key_type(), KeyType::Master);
    }

    #[test]
    fn test_authority_chain_integrity_validation_api() {
        let mut chain = AuthorityChain::new();

        // Create valid hierarchy: Skull -> Master -> Repo
        let skull_key = AuthorityKey::new(
            KeyMaterial::new(b"skull_pub".to_vec(), Some(b"skull_priv".to_vec()), KeyFormat::Age),
            KeyType::Skull, None, None
        ).unwrap();
        let master_key = AuthorityKey::new(
            KeyMaterial::new(b"master_pub".to_vec(), Some(b"master_priv".to_vec()), KeyFormat::Age),
            KeyType::Master, None, None
        ).unwrap();
        let repo_key = AuthorityKey::new(
            KeyMaterial::new(b"repo_pub".to_vec(), Some(b"repo_priv".to_vec()), KeyFormat::Age),
            KeyType::Repo, None, None
        ).unwrap();

        let skull_fp = skull_key.fingerprint().clone();
        let master_fp = master_key.fingerprint().clone();
        let repo_fp = repo_key.fingerprint().clone();

        chain.add_key(skull_key).unwrap();
        chain.add_key(master_key).unwrap();
        chain.add_key(repo_key).unwrap();

        // Add valid relationships
        chain.add_authority_relationship(&skull_fp, &master_fp).unwrap();
        chain.add_authority_relationship(&master_fp, &repo_fp).unwrap();

        // Validate chain integrity
        assert!(chain.validate_integrity().is_ok());

        // Test key retrieval by type
        let skull_keys = chain.get_keys_by_type(KeyType::Skull);
        assert_eq!(skull_keys.len(), 1);
        let master_keys = chain.get_keys_by_type(KeyType::Master);
        assert_eq!(master_keys.len(), 1);
        let repo_keys = chain.get_keys_by_type(KeyType::Repo);
        assert_eq!(repo_keys.len(), 1);
    }
}

/// Test Suite: Ignition Key Management API
/// Validates complete Story 1.2 implementation
#[cfg(test)]
mod ignition_key_tests {
    use super::*;

    #[test]
    fn test_passphrase_validation_api() {
        // Test valid passphrase
        let valid_passphrase = "MySecure123!Pass";
        assert!(validate_passphrase_strength(valid_passphrase).is_ok());

        // Test minimum length requirement
        let too_short = "short";
        assert!(validate_passphrase_strength(too_short).is_err());

        // Test maximum length prevention (DoS protection)
        let too_long = "a".repeat(300);
        assert!(validate_passphrase_strength(&too_long).is_err());

        // Test character diversity requirements
        let no_upper = "lowercase123!";
        assert!(validate_passphrase_strength(no_upper).is_err());
        
        let no_lower = "UPPERCASE123!";
        assert!(validate_passphrase_strength(no_lower).is_err());
        
        let no_digit = "UpperLower!@#";
        assert!(validate_passphrase_strength(no_digit).is_err());
        
        let no_special = "UpperLower123";
        assert!(validate_passphrase_strength(no_special).is_err());

        // Test common password detection
        let common_password = "password123";
        assert!(validate_passphrase_strength(common_password).is_err());

        // Test injection pattern detection
        let injection_attempt = "test$(rm -rf /)";
        assert!(validate_passphrase_strength(injection_attempt).is_err());
    }

    #[test]
    fn test_ignition_key_creation_api() {
        let key_material = KeyMaterial::new(
            b"ignition_public_key".to_vec(),
            Some(b"ignition_private_key".to_vec()),
            KeyFormat::Age,
        );

        // Test successful ignition key creation
        let ignition_key = IgnitionKey::create(
            &key_material,
            KeyType::Ignition,
            "SecureTestPass123!",
            None,
            Some("test-ignition-key".to_string()),
        );

        assert!(ignition_key.is_ok(), "Ignition key creation should succeed");
        let key = ignition_key.unwrap();
        
        assert_eq!(key.key_type(), KeyType::Ignition);
        assert_eq!(key.metadata().name, "test-ignition-key");
        assert!(!key.is_expired());
        assert!(!key.is_warning());
    }

    #[test]
    fn test_ignition_key_unlock_api() {
        let key_material = KeyMaterial::new(
            b"unlock_test_public".to_vec(),
            Some(b"unlock_test_private".to_vec()),
            KeyFormat::Age,
        );

        let passphrase = "UnlockTestPass123!";
        let mut ignition_key = IgnitionKey::create(
            &key_material,
            KeyType::Ignition,
            passphrase,
            None,
            Some("unlock-test".to_string()),
        ).expect("Ignition key creation should succeed");

        // Test successful unlock
        let unlocked_material = ignition_key.unlock(passphrase);
        assert!(unlocked_material.is_ok(), "Unlock with correct passphrase should succeed");

        // Verify metadata updated
        assert_eq!(ignition_key.metadata().unlock_count, 1);
        assert!(ignition_key.metadata().last_unlock.is_some());

        // Test failed unlock
        let failed_unlock = ignition_key.unlock("WrongPassphrase123!");
        assert!(failed_unlock.is_err(), "Unlock with wrong passphrase should fail");
        assert_eq!(ignition_key.metadata().failed_unlock_attempts, 1);
    }

    #[test]
    fn test_ignition_key_passphrase_change_api() {
        let key_material = KeyMaterial::new(
            b"change_test_public".to_vec(),
            Some(b"change_test_private".to_vec()),
            KeyFormat::Age,
        );

        let old_passphrase = "OldTestPass123!";
        let new_passphrase = "NewSecurePass456@";
        
        let mut ignition_key = IgnitionKey::create(
            &key_material,
            KeyType::Ignition,
            old_passphrase,
            None,
            Some("passphrase-change-test".to_string()),
        ).expect("Ignition key creation should succeed");

        // Test successful passphrase change
        let change_result = ignition_key.change_passphrase(old_passphrase, new_passphrase);
        assert!(change_result.is_ok(), "Passphrase change should succeed");

        // Test old passphrase no longer works
        let old_unlock = ignition_key.unlock(old_passphrase);
        assert!(old_unlock.is_err(), "Old passphrase should no longer work");

        // Test new passphrase works
        let new_unlock = ignition_key.unlock(new_passphrase);
        assert!(new_unlock.is_ok(), "New passphrase should work");
    }

    #[test]
    fn test_ignition_key_types_validation_api() {
        let key_material = KeyMaterial::new(
            b"type_test_public".to_vec(),
            Some(b"type_test_private".to_vec()),
            KeyFormat::Age,
        );

        let passphrase = "TypeTestPass123!";

        // Test valid ignition key types
        let skull_key = IgnitionKey::create(&key_material, KeyType::Skull, passphrase, None, None);
        assert!(skull_key.is_ok(), "Skull keys can be ignition keys");

        let ignition_key = IgnitionKey::create(&key_material, KeyType::Ignition, passphrase, None, None);
        assert!(ignition_key.is_ok(), "Ignition keys can be ignition keys");

        let distro_key = IgnitionKey::create(&key_material, KeyType::Distro, passphrase, None, None);
        assert!(distro_key.is_ok(), "Distro keys can be ignition keys");

        // Test invalid ignition key types
        let master_key = IgnitionKey::create(&key_material, KeyType::Master, passphrase, None, None);
        assert!(master_key.is_err(), "Master keys cannot be ignition keys");

        let repo_key = IgnitionKey::create(&key_material, KeyType::Repo, passphrase, None, None);
        assert!(repo_key.is_err(), "Repo keys cannot be ignition keys");
    }
}

/// Test Suite: Authority Validation Engine API
/// Validates complete Story 1.3 implementation
#[cfg(test)]
mod validation_engine_tests {
    use super::*;

    #[test]
    fn test_authority_hierarchy_validation_api() {
        // Test valid authority relationships
        assert!(validate_authority_hierarchy(KeyType::Skull, KeyType::Master).is_ok());
        assert!(validate_authority_hierarchy(KeyType::Master, KeyType::Repo).is_ok());
        assert!(validate_authority_hierarchy(KeyType::Repo, KeyType::Ignition).is_ok());
        assert!(validate_authority_hierarchy(KeyType::Ignition, KeyType::Distro).is_ok());

        // Test invalid authority relationships
        assert!(validate_authority_hierarchy(KeyType::Distro, KeyType::Ignition).is_err());
        assert!(validate_authority_hierarchy(KeyType::Master, KeyType::Ignition).is_err());
        assert!(validate_authority_hierarchy(KeyType::Repo, KeyType::Master).is_err());
    }

    #[test]
    fn test_authority_levels_api() {
        // Test authority level conversion
        assert_eq!(AuthorityLevel::from_key_type(KeyType::Skull), AuthorityLevel::SkullAuthority);
        assert_eq!(AuthorityLevel::from_key_type(KeyType::Master), AuthorityLevel::MasterControl);
        assert_eq!(AuthorityLevel::from_key_type(KeyType::Repo), AuthorityLevel::RepoControl);
        assert_eq!(AuthorityLevel::from_key_type(KeyType::Ignition), AuthorityLevel::IgnitionControl);
        assert_eq!(AuthorityLevel::from_key_type(KeyType::Distro), AuthorityLevel::DistroAccess);

        // Test authority level comparisons
        assert!(AuthorityLevel::SkullAuthority > AuthorityLevel::MasterControl);
        assert!(AuthorityLevel::MasterControl > AuthorityLevel::RepoControl);
        assert!(AuthorityLevel::RepoControl > AuthorityLevel::IgnitionControl);
        assert!(AuthorityLevel::IgnitionControl > AuthorityLevel::DistroAccess);

        // Test authority level capabilities
        assert!(AuthorityLevel::SkullAuthority.can_perform(AuthorityLevel::DistroAccess));
        assert!(AuthorityLevel::MasterControl.can_perform(AuthorityLevel::RepoControl));
        assert!(AuthorityLevel::RepoControl.can_perform(AuthorityLevel::IgnitionControl));
        assert!(!AuthorityLevel::DistroAccess.can_perform(AuthorityLevel::MasterControl));
    }

    #[test]
    fn test_validation_engine_authority_testing_api() {
        // Create authority chain with test keys
        let mut chain = AuthorityChain::new();
        
        let master_key = AuthorityKey::new(
            KeyMaterial::new(b"master_test".to_vec(), Some(b"master_private".to_vec()), KeyFormat::Age),
            KeyType::Master, None, None
        ).unwrap();
        let repo_key = AuthorityKey::new(
            KeyMaterial::new(b"repo_test".to_vec(), Some(b"repo_private".to_vec()), KeyFormat::Age),
            KeyType::Repo, None, None
        ).unwrap();

        let master_fp = master_key.fingerprint().clone();
        let repo_fp = repo_key.fingerprint().clone();

        chain.add_key(master_key).unwrap();
        chain.add_key(repo_key).unwrap();
        chain.add_authority_relationship(&master_fp, &repo_fp).unwrap();

        // Create validation engine
        let mut engine = AuthorityValidationEngine::new(chain);

        // Test authority relationship
        let authority_result = engine.test_authority(&master_fp, &repo_fp);
        assert!(authority_result.is_ok(), "Authority test should succeed");
        assert!(authority_result.unwrap(), "Master should have authority over Repo");

        // Test subject relationship
        let subject_result = engine.test_subject(&repo_fp, &master_fp);
        assert!(subject_result.is_ok(), "Subject test should succeed");
        assert!(subject_result.unwrap(), "Repo should be subject to Master");

        // Test invalid authority
        let invalid_authority = engine.test_authority(&repo_fp, &master_fp);
        assert!(invalid_authority.is_err(), "Invalid authority should fail");
    }

    #[test]
    fn test_validation_engine_operation_authorization_api() {
        let mut chain = AuthorityChain::new();
        
        let master_key = AuthorityKey::new(
            KeyMaterial::new(b"auth_master".to_vec(), Some(b"auth_master_priv".to_vec()), KeyFormat::Age),
            KeyType::Master, None, None
        ).unwrap();
        let distro_key = AuthorityKey::new(
            KeyMaterial::new(b"auth_distro".to_vec(), Some(b"auth_distro_priv".to_vec()), KeyFormat::Age),
            KeyType::Distro, None, None
        ).unwrap();

        let master_fp = master_key.fingerprint().clone();
        let distro_fp = distro_key.fingerprint().clone();

        chain.add_key(master_key).unwrap();
        chain.add_key(distro_key).unwrap();

        let engine = AuthorityValidationEngine::new(chain);

        // Test master key can perform high-level operations
        let master_auth = engine.validate_operation_authorization(
            "global_config",
            &master_fp,
            AuthorityLevel::MasterControl,
        );
        assert!(master_auth.is_ok(), "Master key should authorize master operations");

        // Test master key can perform low-level operations
        let master_low_auth = engine.validate_operation_authorization(
            "file_access",
            &master_fp,
            AuthorityLevel::DistroAccess,
        );
        assert!(master_low_auth.is_ok(), "Master key should authorize distro operations");

        // Test distro key cannot perform high-level operations
        let distro_high_auth = engine.validate_operation_authorization(
            "global_config",
            &distro_fp,
            AuthorityLevel::MasterControl,
        );
        assert!(distro_high_auth.is_err(), "Distro key should not authorize master operations");

        // Test distro key can perform appropriate operations
        let distro_auth = engine.validate_operation_authorization(
            "file_access",
            &distro_fp,
            AuthorityLevel::DistroAccess,
        );
        assert!(distro_auth.is_ok(), "Distro key should authorize distro operations");
    }
}

/// Test Suite: Age Automation Configuration API
/// Validates TTY automation configuration and setup
#[cfg(test)]
mod age_automation_tests {
    use super::*;

    #[test]
    fn test_age_config_api() {
        // Test default configuration
        let default_config = AgeConfig::default();
        assert_eq!(default_config.output_format(), OutputFormat::Binary);
        assert_eq!(default_config.tty_method(), TtyMethod::Auto);

        // Test ASCII armor configuration
        let ascii_config = AgeConfig::new(OutputFormat::AsciiArmor, TtyMethod::Script);
        assert_eq!(ascii_config.output_format(), OutputFormat::AsciiArmor);
        assert_eq!(ascii_config.tty_method(), TtyMethod::Script);

        // Test configuration validation
        let validation_result = ascii_config.validate();
        assert!(validation_result.is_ok(), "Valid configuration should pass validation");
    }

    #[test]
    fn test_output_format_api() {
        // Test binary format
        let binary_format = OutputFormat::Binary;
        assert!(!binary_format.is_ascii_armor());

        // Test ASCII armor format
        let ascii_format = OutputFormat::AsciiArmor;
        assert!(ascii_format.is_ascii_armor());

        // Test format descriptions
        assert_eq!(binary_format.description(), "Binary");
        assert_eq!(ascii_format.description(), "ASCII Armor");
    }

    #[test]
    fn test_tty_method_api() {
        // Test TTY method preferences
        assert_eq!(TtyMethod::Auto.description(), "Automatic (script -> expect)");
        assert_eq!(TtyMethod::Script.description(), "Script command");
        assert_eq!(TtyMethod::Expect.description(), "Expect automation");

        // Test method priority
        let auto_methods = TtyMethod::Auto.preferred_order();
        assert_eq!(auto_methods, vec![TtyMethod::Script, TtyMethod::Expect]);
    }
}

/// Integration Test Suite: End-to-End API Functionality  
/// Tests complete integration across all components
#[cfg(test)]
mod integration_api_tests {
    use super::*;

    #[test]
    fn test_complete_authority_chain_workflow_api() {
        // Create complete authority chain: Skull -> Master -> Repo -> Ignition -> Distro
        let mut chain = AuthorityChain::new();
        
        // Create all key types
        let skull_key = AuthorityKey::new(
            KeyMaterial::new(b"skull_integration".to_vec(), Some(b"skull_priv".to_vec()), KeyFormat::Age),
            KeyType::Skull, None, None
        ).unwrap();
        let master_key = AuthorityKey::new(
            KeyMaterial::new(b"master_integration".to_vec(), Some(b"master_priv".to_vec()), KeyFormat::Age),
            KeyType::Master, None, None
        ).unwrap();
        let repo_key = AuthorityKey::new(
            KeyMaterial::new(b"repo_integration".to_vec(), Some(b"repo_priv".to_vec()), KeyFormat::Age),
            KeyType::Repo, None, None
        ).unwrap();
        let ignition_key_material = KeyMaterial::new(
            b"ignition_integration".to_vec(),
            Some(b"ignition_priv".to_vec()),
            KeyFormat::Age
        );
        let distro_key_material = KeyMaterial::new(
            b"distro_integration".to_vec(),
            Some(b"distro_priv".to_vec()),
            KeyFormat::Age
        );

        let skull_fp = skull_key.fingerprint().clone();
        let master_fp = master_key.fingerprint().clone();
        let repo_fp = repo_key.fingerprint().clone();

        // Add direct keys to chain
        chain.add_key(skull_key).unwrap();
        chain.add_key(master_key).unwrap();
        chain.add_key(repo_key).unwrap();

        // Create ignition keys for passphrase-wrapped types
        let ignition_key = IgnitionKey::create(
            &ignition_key_material,
            KeyType::Ignition,
            "IgnitionTestPass123!",
            Some(chain.get_key(&repo_fp).unwrap()),
            Some("integration-ignition".to_string()),
        ).unwrap();

        let distro_key = IgnitionKey::create(
            &distro_key_material,
            KeyType::Distro,
            "DistroTestPass456@",
            None,
            Some("integration-distro".to_string()),
        ).unwrap();

        // Establish authority relationships
        chain.add_authority_relationship(&skull_fp, &master_fp).unwrap();
        chain.add_authority_relationship(&master_fp, &repo_fp).unwrap();

        // Validate complete chain integrity
        assert!(chain.validate_integrity().is_ok());

        // Test validation engine with complete chain
        let mut engine = AuthorityValidationEngine::new(chain);
        
        // Test complete authority flow
        assert!(engine.test_authority(&skull_fp, &master_fp).unwrap());
        assert!(engine.test_authority(&master_fp, &repo_fp).unwrap());
        
        // Test authorization across all levels
        assert!(engine.validate_operation_authorization(
            "emergency_recovery", &skull_fp, AuthorityLevel::SkullAuthority
        ).is_ok());
        
        assert!(engine.validate_operation_authorization(
            "global_config", &master_fp, AuthorityLevel::MasterControl
        ).is_ok());
        
        assert!(engine.validate_operation_authorization(
            "repo_management", &repo_fp, AuthorityLevel::RepoControl
        ).is_ok());

        // Verify ignition keys maintain their properties
        assert_eq!(ignition_key.key_type(), KeyType::Ignition);
        assert_eq!(ignition_key.metadata().name, "integration-ignition");
        assert!(ignition_key.authority_chain().len() > 0);

        assert_eq!(distro_key.key_type(), KeyType::Distro);
        assert_eq!(distro_key.metadata().name, "integration-distro");
    }

    #[test]
    fn test_configuration_integration_api() {
        // Test complete configuration workflow
        let config = AgeConfig::new(OutputFormat::AsciiArmor, TtyMethod::Script);
        
        // Validate configuration
        assert!(config.validate().is_ok());
        assert!(config.output_format().is_ascii_armor());
        assert_eq!(config.tty_method(), TtyMethod::Script);

        // Test configuration with authority requirements
        let passphrase_requirement = "ConfigTestPass123!";
        assert!(validate_passphrase_strength(passphrase_requirement).is_ok());

        // Test that configuration integrates with ignition key requirements
        let key_material = KeyMaterial::new(
            b"config_integration_pub".to_vec(),
            Some(b"config_integration_priv".to_vec()),
            KeyFormat::Age,
        );

        let ignition_key = IgnitionKey::create(
            &key_material,
            KeyType::Ignition,
            passphrase_requirement,
            None,
            Some("config-integration-test".to_string()),
        );

        assert!(ignition_key.is_ok(), "Configuration should integrate with ignition keys");
    }
}

/// Performance and Edge Case Test Suite
/// Tests system behavior under stress and edge conditions
#[cfg(test)]
mod performance_api_tests {
    use super::*;

    #[test]
    fn test_large_authority_chain_performance_api() {
        let mut chain = AuthorityChain::new();
        
        // Create multiple keys of each type to test scalability
        for i in 0..10 {
            let skull_key = AuthorityKey::new(
                KeyMaterial::new(
                    format!("skull_perf_{}", i).as_bytes().to_vec(),
                    Some(format!("skull_priv_{}", i).as_bytes().to_vec()),
                    KeyFormat::Age,
                ),
                KeyType::Skull, None, None
            ).unwrap();
            
            chain.add_key(skull_key).unwrap();
        }

        // Verify scalability doesn't break basic operations
        assert_eq!(chain.get_keys_by_type(KeyType::Skull).len(), 10);
        assert!(chain.validate_integrity().is_ok());
    }

    #[test]
    fn test_validation_engine_caching_api() {
        let mut chain = AuthorityChain::new();
        
        let master_key = AuthorityKey::new(
            KeyMaterial::new(b"cache_master".to_vec(), Some(b"cache_master_priv".to_vec()), KeyFormat::Age),
            KeyType::Master, None, None
        ).unwrap();
        let repo_key = AuthorityKey::new(
            KeyMaterial::new(b"cache_repo".to_vec(), Some(b"cache_repo_priv".to_vec()), KeyFormat::Age),
            KeyType::Repo, None, None
        ).unwrap();

        let master_fp = master_key.fingerprint().clone();
        let repo_fp = repo_key.fingerprint().clone();

        chain.add_key(master_key).unwrap();
        chain.add_key(repo_key).unwrap();
        chain.add_authority_relationship(&master_fp, &repo_fp).unwrap();

        let mut engine = AuthorityValidationEngine::new(chain);

        // First authority test (should generate proof)
        let result1 = engine.test_authority(&master_fp, &repo_fp);
        assert!(result1.is_ok());

        // Second authority test (should use cached proof)
        let result2 = engine.test_authority(&master_fp, &repo_fp);
        assert!(result2.is_ok());

        // Verify cache statistics
        let (total, expired) = engine.cache_stats();
        assert!(total >= 1, "Cache should contain at least one proof");
        assert!(expired == 0, "No proofs should be expired immediately");
    }

    #[test]
    fn test_maximum_passphrase_edge_cases_api() {
        // Test exactly at limit
        let max_length_passphrase = format!("A1a!{}", "x".repeat(256 - 4));
        assert!(validate_passphrase_strength(&max_length_passphrase).is_err(), "Exactly max length should be rejected");

        // Test just under limit with valid diversity
        let valid_long_passphrase = format!("ValidPass123!{}", "x".repeat(200));
        assert!(validate_passphrase_strength(&valid_long_passphrase).is_ok(), "Long valid passphrase should be accepted");

        // Test minimum valid passphrase
        let min_valid = "ValidPass1!";
        assert!(validate_passphrase_strength(min_valid).is_ok(), "Minimum valid passphrase should be accepted");
    }
}

/// Error Handling and Security Test Suite
/// Tests comprehensive error conditions and security validation
#[cfg(test)]
mod security_api_tests {
    use super::*;

    #[test]
    fn test_authority_hierarchy_security_enforcement_api() {
        // Test that invalid authority relationships are rejected
        let master_key = AuthorityKey::new(
            KeyMaterial::new(b"security_master".to_vec(), Some(b"security_master_priv".to_vec()), KeyFormat::Age),
            KeyType::Master, None, None
        ).unwrap();
        let ignition_key = AuthorityKey::new(
            KeyMaterial::new(b"security_ignition".to_vec(), Some(b"security_ignition_priv".to_vec()), KeyFormat::Age),
            KeyType::Ignition, None, None
        ).unwrap();

        let master_fp = master_key.fingerprint().clone();
        let ignition_fp = ignition_key.fingerprint().clone();

        let mut chain = AuthorityChain::new();
        chain.add_key(master_key).unwrap();
        chain.add_key(ignition_key).unwrap();

        // Should fail: Master cannot directly control Ignition (must go through Repo)
        let invalid_relationship = chain.add_authority_relationship(&master_fp, &ignition_fp);
        assert!(invalid_relationship.is_err(), "Invalid authority relationship should be rejected");
    }

    #[test]
    fn test_injection_prevention_comprehensive_api() {
        let injection_patterns = vec![
            "test$(rm -rf /)",
            "test`whoami`",
            "test;ls",
            "test&whoami",
            "test|ls",
            "test\nrm -rf /",
            "test\rrm -rf /",
            "test\0whoami",
        ];

        for pattern in injection_patterns {
            let result = validate_passphrase_strength(pattern);
            assert!(result.is_err(), "Injection pattern '{}' should be rejected", pattern);
        }
    }

    #[test]
    fn test_key_fingerprint_uniqueness_api() {
        // Test that different keys generate different fingerprints
        let key1_material = KeyMaterial::new(
            b"unique_key_1".to_vec(),
            Some(b"unique_private_1".to_vec()),
            KeyFormat::Age,
        );
        let key2_material = KeyMaterial::new(
            b"unique_key_2".to_vec(),
            Some(b"unique_private_2".to_vec()),
            KeyFormat::Age,
        );

        let key1 = AuthorityKey::new(key1_material, KeyType::Master, None, None).unwrap();
        let key2 = AuthorityKey::new(key2_material, KeyType::Master, None, None).unwrap();

        assert_ne!(key1.fingerprint().hex(), key2.fingerprint().hex(), "Different keys should have different fingerprints");
        assert_ne!(key1.fingerprint().short(), key2.fingerprint().short(), "Different keys should have different short fingerprints");
    }

    #[test]
    fn test_ignition_key_security_properties_api() {
        let key_material = KeyMaterial::new(
            b"security_test_key".to_vec(),
            Some(b"security_test_private".to_vec()),
            KeyFormat::Age,
        );

        let passphrase = "SecurityTestPass123!";
        let mut ignition_key = IgnitionKey::create(
            &key_material,
            KeyType::Ignition,
            passphrase,
            None,
            Some("security-test".to_string()),
        ).unwrap();

        // Test that failed unlock attempts are tracked
        let initial_failed_attempts = ignition_key.metadata().failed_unlock_attempts;
        
        let _failed_unlock = ignition_key.unlock("WrongPassword123!");
        assert_eq!(
            ignition_key.metadata().failed_unlock_attempts,
            initial_failed_attempts + 1,
            "Failed unlock attempts should be tracked"
        );

        // Test that successful unlock resets failure tracking mindset
        let successful_unlock = ignition_key.unlock(passphrase);
        assert!(successful_unlock.is_ok(), "Correct passphrase should unlock key");
        assert_eq!(ignition_key.metadata().unlock_count, 1, "Successful unlocks should be counted");
        assert!(ignition_key.metadata().last_unlock.is_some(), "Last unlock time should be recorded");
    }
}