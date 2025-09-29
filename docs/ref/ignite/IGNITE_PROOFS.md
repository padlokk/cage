# 🔐 Ignite Authority Proof Specification

**Intent**: Describe how Ignite canonicalizes payloads, signs authority/subject proofs with Ed25519, and manages key rotation lifecycle.

---

## 1. Key Material Lifecycle

1. **Generation**: Each authority tier (X, M, R) maintains an Ed25519 keypair stored under `proofs/<tier>/` inside the vault. Keypairs are generated with `ed25519_dalek` and wrapped using the same passphrase/KDF policy as ignition keys when at rest.
2. **Distribution**: Public keys are exported to `proofs/<tier>/public.pem` for downstream verifiers. Private keys never leave the vault.
3. **Rotation**: Rotating an authority key writes the old private key to an archival bundle (`proofs/archive/<timestamp>_<tier>.age`) signed by the parent tier before marking it inactive.

---

## 2. Canonical Payloads

### 2.1 Authority Claim Payload

```json
{
  "schema_version": "1.0",
  "parent_fp": "SHA256:a1b2c3d4",
  "child_fp": "SHA256:deadbeef",
  "issued_at": "2025-09-28T12:30:00Z",
  "purpose": "create-ignition",
  "nonce": "2b7e151628aed2a6abf7158809cf4f3c"
}
```

- `nonce` is 128-bit random to prevent replay collisions.
- JSON keys are sorted alphabetically before signing.

### 2.2 Subject Receipt Payload

```json
{
  "schema_version": "1.0",
  "child_fp": "SHA256:deadbeef",
  "parent_fp": "SHA256:a1b2c3d4",
  "acknowledged_at": "2025-09-28T12:30:05Z",
  "nonce": "a0fafe1788542cb123a339392a6c7605"
}
```

---

## 3. Signing & Verification

1. Payloads are serialized to canonical JSON (sorted keys, UTF-8, LF) and hashed with SHA256 to create a digest.
2. The parent (or child for receipts) signs the digest using Ed25519.
3. Proof bundle structure:
   ```json
   {
     "payload": { ... },
     "digest": "ecf21f2...",
     "signature": "base64(ed25519_sig)",
     "public_key": "base64(ed25519_pub)",
     "expires_at": "2025-09-29T12:30:00Z"
   }
   ```
4. Verification recomputes the digest, checks signature, ensures `expires_at` in future, and confirms fingerprints align with registered keys.

---

## 4. Rotation Procedure

1. Parent validates child fingerprint and initiates rotation.
2. Ignite emits new authority claim payload with purpose `rotate-<role>`.
3. Child issues receipt acknowledging new parent or new key material.
4. Old proofs move to `proofs/archive/` alongside a manifest entry referencing the rotation event.
5. Scheduler queues proof renewal 12h before `expires_at` based on policy (configurable).

---

## 5. Confidence & Testing

- **Unit Tests**: Validate canonicalization, signature verification, expiry handling, and nonce uniqueness.
- **Property Tests**: Fuzz payload orderings to prove canonical sorting is enforced.
- **Integration Tests**: Simulate chain creation X→M→R→I, verifying proofs at each step.
- **CLI Tests**: `ignite proof --verify` to check on-disk bundles and failure modes (tampered signatures, expired proofs).

---

**Owner**: Ignite Core Team  
**Version**: 1.0 (2025-09-28)
