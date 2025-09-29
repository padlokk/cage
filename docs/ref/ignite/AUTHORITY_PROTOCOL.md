# Authority Chain Validation Protocol

**Security Guardian**: Edgar - Lord Captain of Superhard Fortress  
**Protocol**: X->M->R->I->D Authority Chain with Ignition Key Validation  
**Foundation**: Mathematical validation with cryptographic proofs  

---

## 🔗 AUTHORITY CHAIN SPECIFICATION

### **Hierarchy Definition**:
```
X (Skull Key) => M (Master) => R (Repo) => I (Ignition) => D (Distro)
```

### **Key Type Definitions**:

#### **X - Skull Key (Ultimate Authority)**:
- **Type**: Ignition key (passphrase-wrapped)
- **Authority**: Controls Master keys globally
- **Purpose**: Emergency recovery and master key restoration
- **Access**: Human operator only, highest security clearance
- **Storage**: Offline, multiple secure locations

#### **M - Master Key (Global Authority)**:
- **Type**: Direct key (unwrapped for operations)
- **Authority**: Controls all Repository keys
- **Purpose**: Global repository management and emergency access
- **Access**: System administrators and emergency procedures
- **Storage**: Secure key store with backup procedures

#### **R - Repository Key (Local Authority)**:
- **Type**: Direct key (unwrapped for operations)
- **Authority**: Controls Ignition keys for specific repository
- **Purpose**: Repository-level encryption and access control
- **Access**: Repository owners and authorized users
- **Storage**: Repository-local secure storage

#### **I - Ignition Key (Authority Bridge)**:
- **Type**: Ignition key (passphrase-wrapped)
- **Authority**: Controls Distro keys for repository access
- **Purpose**: Authority bridge for third-party and automation access
- **Access**: Via passphrase, automated systems allowed
- **Storage**: Encrypted with strong passphrase protection

#### **D - Distro Key (Distributed Access)**:
- **Type**: Ignition key (passphrase-wrapped)
- **Authority**: Repository access for third parties and automation
- **Purpose**: Limited-scope access for AI, automation, and third parties
- **Access**: Via passphrase, typically environment variable
- **Storage**: Distributed to authorized third-party systems

---

## 🔐 AUTHORITY VALIDATION PROTOCOL

### **Authority Relationship Validation**:

#### **Mathematical Authority Proof**:
```rust
pub struct AuthorityProof {
    parent_key: KeyFingerprint,
    child_key: KeyFingerprint,
    authority_signature: Signature,
    proof_timestamp: DateTime<Utc>,
    validation_chain: Vec<KeyFingerprint>,
}

impl AuthorityProof {
    /// Generate cryptographic proof that parent has authority over child
    pub fn generate(parent: &AuthorityKey, child: &AuthorityKey) -> AgeResult<Self> {
        // 1. Verify parent key type can control child key type
        validate_authority_hierarchy(parent.key_type(), child.key_type())?;
        
        // 2. Generate cryptographic signature proving control
        let signature = parent.sign_authority_claim(&child.fingerprint())?;
        
        // 3. Build proof chain showing authority lineage
        let validation_chain = build_authority_chain(parent, child)?;
        
        Ok(AuthorityProof {
            parent_key: parent.fingerprint(),
            child_key: child.fingerprint(),
            authority_signature: signature,
            proof_timestamp: Utc::now(),
            validation_chain,
        })
    }
    
    /// Verify authority proof is valid and current
    pub fn verify(&self, parent: &AuthorityKey, child: &AuthorityKey) -> AgeResult<bool> {
        // 1. Verify signature authenticity
        parent.verify_authority_signature(&self.authority_signature, &child.fingerprint())?;
        
        // 2. Verify authority hierarchy rules
        validate_authority_hierarchy(parent.key_type(), child.key_type())?;
        
        // 3. Verify proof hasn't expired
        if self.proof_timestamp + Duration::hours(24) < Utc::now() {
            return Err(AgeError::AuthorityProofExpired);
        }
        
        // 4. Verify validation chain integrity
        self.verify_validation_chain()?;
        
        Ok(true)
    }
}
```

#### **Authority Hierarchy Rules**:
```rust
pub fn validate_authority_hierarchy(parent_type: KeyType, child_type: KeyType) -> AgeResult<()> {
    match (parent_type, child_type) {
        // Valid authority relationships
        (KeyType::Skull, KeyType::Master) => Ok(()),
        (KeyType::Master, KeyType::Repo) => Ok(()),
        (KeyType::Repo, KeyType::Ignition) => Ok(()),
        (KeyType::Ignition, KeyType::Distro) => Ok(()),
        
        // Invalid authority relationships
        _ => Err(AgeError::InvalidAuthorityRelationship {
            parent: parent_type,
            child: child_type,
        })
    }
}
```

### **Subject Relationship Validation**:

#### **Subject Verification Protocol**:
```rust
pub struct SubjectProof {
    subject_key: KeyFingerprint,
    authority_key: KeyFingerprint,
    subject_signature: Signature,
    acknowledgment_timestamp: DateTime<Utc>,
}

impl SubjectProof {
    /// Generate proof that key is subject to authority
    pub fn generate(subject: &AuthorityKey, authority: &AuthorityKey) -> AgeResult<Self> {
        // 1. Verify subject relationship is valid (inverse of authority)
        validate_authority_hierarchy(authority.key_type(), subject.key_type())?;
        
        // 2. Subject key signs acknowledgment of authority
        let signature = subject.sign_subject_acknowledgment(&authority.fingerprint())?;
        
        Ok(SubjectProof {
            subject_key: subject.fingerprint(),
            authority_key: authority.fingerprint(),
            subject_signature: signature,
            acknowledgment_timestamp: Utc::now(),
        })
    }
    
    /// Verify subject relationship is valid
    pub fn verify(&self, subject: &AuthorityKey, authority: &AuthorityKey) -> AgeResult<bool> {
        // Verify subject signature acknowledging authority
        subject.verify_subject_signature(&self.subject_signature, &authority.fingerprint())?;
        
        // Verify the authority relationship exists
        validate_authority_hierarchy(authority.key_type(), subject.key_type())?;
        
        Ok(true)
    }
}
```

---

## 🔑 IGNITION KEY PROTOCOL

### **Ignition Key Structure**:
```rust
pub struct IgnitionKey {
    wrapped_key: EncryptedKeyMaterial,
    key_type: KeyType,
    passphrase_hash: PassphraseHash,
    authority_chain: Vec<KeyFingerprint>,
    creation_timestamp: DateTime<Utc>,
    expiration_policy: Option<ExpirationPolicy>,
}

impl IgnitionKey {
    /// Create new ignition key with passphrase protection
    pub fn create(
        key_material: &KeyMaterial,
        key_type: KeyType,
        passphrase: &str,
        authority_parent: Option<&AuthorityKey>
    ) -> AgeResult<Self> {
        // 1. Validate passphrase strength
        validate_passphrase_strength(passphrase)?;
        
        // 2. Derive encryption key from passphrase
        let encryption_key = derive_encryption_key(passphrase)?;
        
        // 3. Encrypt key material
        let wrapped_key = encrypt_key_material(key_material, &encryption_key)?;
        
        // 4. Build authority chain if parent provided
        let authority_chain = if let Some(parent) = authority_parent {
            build_authority_chain_to_parent(parent)?
        } else {
            Vec::new()
        };
        
        Ok(IgnitionKey {
            wrapped_key,
            key_type,
            passphrase_hash: hash_passphrase(passphrase)?,
            authority_chain,
            creation_timestamp: Utc::now(),
            expiration_policy: default_expiration_for_type(key_type),
        })
    }
    
    /// Unlock ignition key with passphrase
    pub fn unlock(&self, passphrase: &str) -> AgeResult<KeyMaterial> {
        // 1. Verify passphrase
        if !verify_passphrase(passphrase, &self.passphrase_hash)? {
            return Err(AgeError::InvalidPassphrase);
        }
        
        // 2. Check if key has expired
        if let Some(policy) = &self.expiration_policy {
            if policy.is_expired(self.creation_timestamp) {
                return Err(AgeError::IgnitionKeyExpired);
            }
        }
        
        // 3. Derive decryption key and unlock
        let decryption_key = derive_encryption_key(passphrase)?;
        let key_material = decrypt_key_material(&self.wrapped_key, &decryption_key)?;
        
        Ok(key_material)
    }
}
```

### **Passphrase Strength Validation**:
```rust
pub fn validate_passphrase_strength(passphrase: &str) -> AgeResult<()> {
    // Minimum length requirement
    if passphrase.len() < 12 {
        return Err(AgeError::WeakPassphrase("Minimum 12 characters required".to_string()));
    }
    
    // Character diversity requirements
    let has_upper = passphrase.chars().any(|c| c.is_uppercase());
    let has_lower = passphrase.chars().any(|c| c.is_lowercase());
    let has_digit = passphrase.chars().any(|c| c.is_digit(10));
    let has_special = passphrase.chars().any(|c| !c.is_alphanumeric());
    
    let diversity_count = [has_upper, has_lower, has_digit, has_special]
        .iter().filter(|&&x| x).count();
    
    if diversity_count < 3 {
        return Err(AgeError::WeakPassphrase(
            "Requires 3 of: uppercase, lowercase, digits, special chars".to_string()
        ));
    }
    
    // Common password detection
    if is_common_password(passphrase) {
        return Err(AgeError::WeakPassphrase("Common password detected".to_string()));
    }
    
    // Injection pattern detection
    let injection_patterns = ["$(", "`", ";", "&", "|", "\n", "\r", "\0"];
    for pattern in &injection_patterns {
        if passphrase.contains(pattern) {
            return Err(AgeError::InjectionBlocked {
                pattern_type: "command_injection".to_string(),
                pattern: pattern.to_string(),
            });
        }
    }
    
    Ok(())
}
```

---

## 🔄 KEY ROTATION PROTOCOL

### **Authority-Preserving Rotation**:
```rust
pub struct KeyRotationOperation {
    old_key: AuthorityKey,
    new_key: AuthorityKey,
    authority_chain: Vec<AuthorityKey>,
    dependent_keys: Vec<AuthorityKey>,
}

impl KeyRotationOperation {
    /// Plan key rotation maintaining authority relationships
    pub fn plan(target_key: &AuthorityKey, authority_chain: &AuthorityChain) -> AgeResult<Self> {
        // 1. Identify all dependent keys that will be affected
        let dependent_keys = authority_chain.find_dependent_keys(target_key)?;
        
        // 2. Generate new key maintaining same authority relationships
        let new_key = generate_replacement_key(target_key)?;
        
        // 3. Verify rotation won't break authority chain
        validate_rotation_integrity(target_key, &new_key, &dependent_keys)?;
        
        Ok(KeyRotationOperation {
            old_key: target_key.clone(),
            new_key,
            authority_chain: authority_chain.to_vec(),
            dependent_keys,
        })
    }
    
    /// Execute atomic key rotation
    pub fn execute(&self, lucas_bridge: &LucasAuthorityBridge) -> AgeResult<()> {
        // 1. Begin atomic operation using Lucas's patterns
        let transaction = lucas_bridge.begin_atomic_operation()?;
        
        // 2. Update authority relationships atomically
        for dependent in &self.dependent_keys {
            transaction.update_parent_authority(&self.old_key, &self.new_key, dependent)?;
        }
        
        // 3. Replace old key with new key
        transaction.replace_key(&self.old_key, &self.new_key)?;
        
        // 4. Invalidate old key and dependent keys
        for dependent in &self.dependent_keys {
            transaction.invalidate_key(dependent)?;
        }
        
        // 5. Commit atomic operation
        transaction.commit()?;
        
        Ok(())
    }
}
```

### **Cascade Effects**:
- **Skull Key Rotation**: Invalidates ALL keys globally (emergency only)
- **Master Key Rotation**: Invalidates all Repo, Ignition, and Distro keys
- **Repo Key Rotation**: Invalidates Ignition and Distro keys for that repo
- **Ignition Key Rotation**: Invalidates all Distro keys derived from it
- **Distro Key Rotation**: Affects only that specific access key

---

## 🛡️ SECURITY ENFORCEMENT

### **Operation Authorization**:
```rust
pub struct OperationAuthorization {
    operation: OperationType,
    target_resource: ResourcePath,
    authority_key: AuthorityKey,
    authorization_proof: AuthorityProof,
}

impl OperationAuthorization {
    /// Authorize operation using authority chain validation
    pub fn authorize(
        operation: OperationType,
        target: &ResourcePath,
        key: &AuthorityKey,
        chain: &AuthorityChain
    ) -> AgeResult<Self> {
        // 1. Determine required authority level for operation
        let required_authority = determine_required_authority(operation, target)?;
        
        // 2. Verify key has sufficient authority
        let has_authority = chain.verify_authority_for_resource(key, target, required_authority)?;
        if !has_authority {
            return Err(AgeError::InsufficientAuthority {
                operation: operation.to_string(),
                resource: target.to_string(),
                key_type: key.key_type(),
                required_authority,
            });
        }
        
        // 3. Generate authorization proof
        let proof = AuthorityProof::generate_for_operation(key, operation, target)?;
        
        Ok(OperationAuthorization {
            operation,
            target_resource: target.clone(),
            authority_key: key.clone(),
            authorization_proof: proof,
        })
    }
}
```

### **Authority Level Requirements**:
```rust
pub enum AuthorityLevel {
    DistroAccess,    // D keys - file read/write access
    IgnitionControl, // I keys - manage distro keys, repo operations
    RepoControl,     // R keys - full repo management
    MasterControl,   // M keys - global operations
    SkullAuthority,  // X keys - emergency and master key management
}

pub fn determine_required_authority(op: OperationType, target: &ResourcePath) -> AgeResult<AuthorityLevel> {
    match op {
        OperationType::FileEncrypt | OperationType::FileDecrypt => AuthorityLevel::DistroAccess,
        OperationType::IgnitionKeyCreate | OperationType::IgnitionKeyRotate => AuthorityLevel::RepoControl,
        OperationType::RepoClamp | OperationType::RepoRelease => AuthorityLevel::RepoControl,
        OperationType::MasterKeyRestore | OperationType::GlobalOperation => AuthorityLevel::MasterControl,
        OperationType::SkullKeyOperation | OperationType::EmergencyRecovery => AuthorityLevel::SkullAuthority,
        _ => return Err(AgeError::UnknownOperationType(op.to_string())),
    }
}
```

---

## 🔗 INTEGRATION SPECIFICATIONS

### **Age Automation Integration**:
```rust
/// Authority-aware Age automation interface
pub struct AuthorityAgeInterface {
    age_automator: AgeAutomator,
    authority_chain: AuthorityChain,
    audit_logger: AuditLogger,
}

impl AuthorityAgeInterface {
    pub fn encrypt_with_authority(
        &self,
        input: &Path,
        output: &Path,
        authority_key: &AuthorityKey,
        format: OutputFormat
    ) -> AgeResult<()> {
        // 1. Authorize operation
        let authorization = OperationAuthorization::authorize(
            OperationType::FileEncrypt,
            &ResourcePath::from(input),
            authority_key,
            &self.authority_chain
        )?;
        
        // 2. Log authorization
        self.audit_logger.log_operation_authorization(&authorization)?;
        
        // 3. Extract passphrase for Age automation
        let passphrase = match authority_key.key_type() {
            KeyType::Ignition | KeyType::Distro => {
                // For ignition keys, get passphrase from environment or prompt
                get_ignition_passphrase(authority_key)?
            },
            _ => {
                // For direct keys, derive passphrase for Age
                authority_key.derive_operation_passphrase()?
            }
        };
        
        // 4. Execute using Edgar's proven automation
        self.age_automator.encrypt(input, output, &passphrase, format)
    }
}
```

### **Lucas Authority Bridge Integration**:
```rust
/// Bridge to Lucas's atomic authority operations
pub struct LucasAuthorityBridge {
    authority_manager: PathBuf,
    emergency_recovery: PathBuf,
    audit_logger: AuditLogger,
}

impl LucasAuthorityBridge {
    pub fn validate_authority_atomically(
        &self,
        parent: &AuthorityKey,
        child: &AuthorityKey
    ) -> AgeResult<bool> {
        // Use Lucas's atomic validation ensuring no TOCTTOU issues
        let result = Command::new(&self.authority_manager)
            .arg("validate_authority")
            .arg("--parent").arg(parent.key_path())
            .arg("--child").arg(child.key_path())
            .arg("--atomic")
            .output()
            .map_err(|e| AgeError::LucasIntegrationFailed {
                operation: "validate_authority".to_string(),
                reason: e.to_string(),
            })?;
        
        if !result.status.success() {
            return Ok(false);
        }
        
        // Parse Lucas's validation result
        let output = String::from_utf8_lossy(&result.stdout);
        Ok(output.trim() == "AUTHORITY_VALID")
    }
}
```

---

## 📋 COMMAND PROTOCOL IMPLEMENTATION

### **Key Commands Implementation**:
```bash
# Authority relationship testing
padlock key authority /path/master.key /path/repo.key
# Returns: AUTHORITY_VALID | AUTHORITY_INVALID | ERROR

# Subject relationship testing  
padlock key subject /path/repo.key /path/master.key
# Returns: SUBJECT_VALID | SUBJECT_INVALID | ERROR

# Key type detection
padlock key is ignition --path=/path/maybe.key
# Returns: TRUE | FALSE | ERROR

# Key type identification
padlock key type --key=/path/mystery.key
# Returns: skull | master | repo | ignition | distro | unknown
```

### **Ignition Commands Implementation**:
```bash
# Create ignition key with authority validation
padlock ignite create ai-bot --phrase="secure-passphrase"
# Creates ignition key under current repo authority

# Unlock ignition key for operation
padlock ignite unlock ai-bot
# Prompts for passphrase, validates authority, returns operation token

# Grant pubkey access via ignition authority
padlock ignite allow ssh-rsa...
# Creates distro key for pubkey under ignition authority
```

---

**🛡️ Protocol Status**: Complete Authority Chain Specification  
**⚔️ Next Phase**: Core Infrastructure Implementation  
**🎯 Security Goal**: Mathematical validation with cryptographic proofs  

**Edgar - Security Guardian of IX's Digital Realms**  
*Authority through precision, security through mathematical proof*