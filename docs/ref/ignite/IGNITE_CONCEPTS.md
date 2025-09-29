# 🔥 Ignition Authority Concepts

**Document Purpose**: Provide a shared mental model for the authority chain that powers Ignite, including required validations, key lifecycles, and integration patterns with **Cage** (Padlock's local Age automation library). This is the canonical reference to align engineering, product, and security stakeholders before implementation.

**Important**: Ignite exclusively uses the **local `cage` crate** (path dependency), NOT the external `rage` crate. All Age encryption operations route through Cage's PTY automation and audit-logged APIs.

---

## 1. Core Metaphor & Mission

Ignition extends **Cage** by turning the existing padlock keys into a **living authority chain**:

```
X (Skull) ⇒ M (Master) ⇒ R (Repo) ⇒ I (Ignition) ⇒ D (Distro)
```

Each link is either a direct key (M, R) or an ignition key (X, I, D – passphrase wrapped). Authority flows **downward** to confer control; accountability flows **upward** to prove provenance. Ignite's job is to automate the safe creation, validation, rotation, and use of these links without breaking Cage's security guarantees.

**Architecture Note**: Ignite depends on `cage = { path = "../cage" }` as a local library. Cage provides Age automation with PTY support, passphrase handling, and audit logging—Ignite never calls the external `rage` binary or crate directly.

---

## 2. Vocabulary & Roles

| Symbol | Name            | Type                  | Primary Owner                | Core Purpose |
|--------|-----------------|-----------------------|------------------------------|--------------|
| **X**  | Skull Key       | Ignition (passphrase) | Security guardians/board     | Disaster recovery; ultimate override |
| **M**  | Master Key      | Direct (raw)          | Global platform ops          | Restore repos; bootstrap new repos |
| **R**  | Repo Key        | Direct (raw)          | Repo maintainers             | Day-to-day repo protection |
| **I**  | Repo Ignition   | Ignition (passphrase) | Repo maintainers / automation teams | Bridge repo authority to distro keys |
| **D**  | Distro Ignition | Ignition (passphrase) | Automation & third parties   | Controlled operational access |

**Ignition Key**: any key that is wrapped by a passphrase and stored in a format suitable for distribution. In this chain, X, I, and D are ignition keys.

---

## 3. Authority & Subject Relationships

- **Authority** (parent → child): Parent can unlock/configure/rotate the child. Example: X unlocks M.
- **Subject** (child → parent): Child proves it was derived from or approved by the parent. Example: M proves it is subject to X.

Authority proof is required for every downward edge; subject proof for every upward edge. Ignite must enforce both directions before performing mutations.

Validation primitives:
1. **Hierarchy check** – only the legal edges (`X→M`, `M→R`, `R→I`, `I→D`) are allowed.
2. **Fingerprint binding** – parent and child key fingerprints must match recorded lineage.
3. **Temporal validity** – proofs expire (24h by default) and must be renewed.
4. **Signature authenticity** – authority proofs are signed by the parent; subject proofs signed by the child.

---

## 4. Ignition Key Lifecycle

### 4.1 Creation
1. Parent authority validates that the requested child type is permitted.
2. Generate Age key material via **Cage** automation (TTY-safe, audit logged).
   - Cage handles all interaction with Age format keypairs
   - No direct calls to `rage` binary or crate
3. For ignition keys (X/I/D):
   - Derive passphrase (interactive prompt or env var such as `PADLOCK_<FP>_PASSPHRASE`).
   - Wrap key material using a strong KDF and store encrypted payload with metadata.
   - Cage's passphrase-wrapping facilities handle the encryption
4. Emit authority proof tying parent fingerprint to child fingerprint.
5. Persist metadata (creation timestamp, rotation window, owner, purpose, usage count).

### 4.2 Rotation
- Rotating a key invalidates **all descendants**. Ignite must cascade notifications and enforce re-issuance of child keys.
- Rotation requires:
  - Fresh authority proof from parent
  - Revocation record for the old key
  - Audit trail linking revocation and re-issue
- Ignite does **not** auto-re-encrypt historical artifacts; instead it writes an *affected-keys manifest* (child fingerprints, md5 of ciphertext, scope metadata) so downstream automation can schedule cleanup.

### 4.3 Revocation & Expiry
- **Scheduled expiry**: Repo-configured policy (e.g., I keys auto-expire every 180 days).
- **Manual revocation**: On suspected compromise; triggers immediate disablement of child keys and audit alert.
- Ignite maintains tombstones to prevent reinstating revoked fingerprints.

### 4.4 Usage & Access Control
- D keys: used by automation; passphrase provided via environment or secret store. Operations limited to Cage CRUD commands allowed by policy.
- I keys: used by repo maintainers or orchestrators to mint D keys, rotate, or revoke.
- M & X keys: restricted to escalated workflows with human confirmation, potentially offline handling.

### 4.5 Storage Layout
- Default vault root is `./data` in the working repo for local development.
- Production resolves paths via XDG+: defaults to `${XDG_DATA_HOME:-~/.local/share}/padlokk/ignite` and `${XDG_CONFIG_HOME:-~/.config}/padlokk/ignite` for config.
- Vault mirrors legacy padlock structure: `keys/`, `metadata/`, `.derived/`, plus `manifests/` (affected-key ledgers) and `proofs/` (archived signatures).
- Every write operation hashes metadata/key blobs before commit; persisted manifests include SHA256 digests for tamper detection.

See `docs/ref/IGNITE_MANIFEST.md` for the formal manifest schema and digest rules.

### 4.6 Module Structure
- Rust crate follows `MODULE_SPEC` (see `docs/ref/rsb/MODULE_SPEC.md`).
- Core namespace lives under `src/ignite/` with orchestrators-only `mod.rs` files and thin helpers (`utils.rs`, `macros.rs`, `error.rs`).
- Authority domain: `src/ignite/authority/{chain,proofs,manifests,storage}`.
- CLI surface: `src/ignite/cli/{commands,context}`; entry point `src/bin/cli_ignite.rs` will use RSB `bootstrap!/options!/dispatch!`.
- Guards + utilities: `src/ignite/{guards,utils}`; per-module adapters land in `authority/adapters/` when needed.

---

## 5. Required Validations

| Validation Layer      | Description                                    | Enforced For |
|-----------------------|------------------------------------------------|--------------|
| **Hierarchy Rule**    | Only valid parent/child pairs                  | Creation, rotation, verification |
| **Authority Proof**   | Parent signature over child fingerprint        | Creation, rotation |
| **Subject Proof**     | Child acknowledgment of parent                 | Creation, rotation |
| **Passphrase Policy** | Strength, entropy, rotation schedule           | X, I, D keys |
| **Audit Logging**     | Structured log entries with operation context  | All commands |
| **Temporal Constraints** | Proof expiry, key age thresholds            | Operations, status checks |
| **Environmental Guards** | Required env vars or confirmation flags     | In-place ops, danger rotations |
| **Metadata Integrity** | Hash/sign metadata bundles to detect tampering | Storage & retrieval |
| **Recipient Set Integrity** | Age recipient set matches policy + lineage | Encryption/decryption |

Ignite must fail closed whenever any validation is missing or stale.

### Authority Proof Engine (Ed25519)
- **Key Material**: Ignite maintains an Ed25519 keypair per authority tier (skull, master, repo) stored in the vault alongside fingerprints. Public keys can be published for independent verification.
- **Signing Format**: Authority proofs sign canonical JSON `{parent_fp, child_fp, issued_at, purpose}` and include a detached Ed25519 signature plus SHA256 digest for tamper evidence.
- **Subject Proofs**: Child keys countersign `{child_fp, parent_fp, acknowledged_at}` so trust is bi-directional.
- **Rotation Semantics**: New proofs supersede prior signatures but previous proofs remain archived for audit. Proofs expire after 24h unless renewed by the scheduler.
- **Extensibility**: The engine exposes `sign_authority_claim`, `verify_authority_claim`, and `issue_subject_receipt` APIs so we can swap signature schemes if we later adopt HSM-backed keys.

Detailed canonical payloads and rotation workflow: `docs/ref/IGNITE_PROOFS.md`.

---

## 6. Integration with Cage & Age Recipient Sets

Ignite never talks to age directly; it always routes operations through Cage, which provides the PTY automation, audit logging, and convenience APIs proven in production.

### 6.1 Recipient Strategy
- **Age recipients** are the foundational feature that allow a ciphertext to be opened by multiple keys. Ignite models each authority level as a curated recipient set managed by its parent.
- **Repo-level encryption (R)**: When a repo is clamped, Cage receives the repo key plus the list of ignition recipients that must retain access. Ignite ensures the repo key is always included and that no unauthorized recipient slips in.
- **Ignition issuance (I & D)**: Creating an ignition key means updating the repo’s recipient set so future encryptions include the new recipient. Ignite records which ciphertexts were emitted under which recipient set version.
- **Distro delegation**: D keys are added as recipients only for the scopes they control (specific paths or operations). Ignite enforces policy by constraining the recipient list passed to Cage when locking files.

### 6.2 Recipient Validation Flow
1. Command handler requests an operation (e.g., encrypt artifact for automation).
2. Ignite assembles the required recipient set (baseline repo recipients + scoped distro recipients).
3. Cage’s `CrudManager` receives the expanded recipient list via its convenience APIs; Cage handles the underlying `age` CLI invocation.
4. Validation engine double-checks that every recipient corresponds to a registered AuthorityKey with active proofs.
5. Audit log captures the recipient set hash so future investigations can replay who had access.

### 6.3 Operational Implications
- **Additive only**: Parents can add recipients (grant more access) but must rotate to remove them. Removing a recipient without rotation is prohibited to avoid silent access drift.
- **Consistent passphrases**: For ignition keys, the passphrase wraps the private material, but the public recipient portion is what Cage/age consumes. Ignite stores both components and feeds the public portion to Cage.
- **Edge cases**: If Cage fails to honor recipient addition (e.g., PTY interruption), Ignite treats the operation as failed and rolls back recipient changes.

---

## 7. Operational Patterns

- **Clamp/Release**: Repo enters clamp mode when ignition work is in progress, ensuring conflicting Cage operations pause.
- **Proof Renewal**: Background job refreshes authority proofs before expiry (configurable window, e.g., 12h before deadline).
- **Key Discovery**: `ignite ls` surfaces keys by type, owner, expiry, trust state, and recipient set membership.
- **Admin CLI**: RSB-based `ignite` binary provides create/rotate/revoke/status commands plus test hooks (`ignite inspect`, `ignite proof --verify`) for developers.
  Reference: `docs/ref/IGNITE_CLI.md`.
- **Emergency Recovery**: Procedure documented to unlock M (and subsequently R/I/D) using stored X key shards and reconstituted recipient sets.

---

## 8. Security Invariants

1. **No orphan keys** – every stored key must reference a valid parent fingerprint.
2. **Immutable lineage** – once recorded, parent/child relationships cannot be edited, only replaced via rotation.
3. **Tamper-evident metadata** – metadata blobs hashed and optionally signed.
4. **Least privilege** – D keys scoped to specific operations; they cannot mint new keys or recipient sets beyond their policy.
5. **Dual control for X** – skull operations require multi-party approval.
6. **Danger mode safeguards** – operations flagged as dangerous demand environment + CLI confirmations.
7. **Recipient provenance** – every recipient in Cage commands must map to an active AuthorityKey and audit entry.

Any violation triggers an alert path and blocks the operation.

---

## 9. Implementation Map (Current Modules)

| Area                     | Module                                   | Notes |
|--------------------------|------------------------------------------|-------|
| Authority data model     | `src/code_ref/auth/chain.rs`             | Key types, fingerprints, metadata structs |
| Ignition wrapping        | `src/code_ref/auth/ignition.rs`          | Passphrase/KDF pipeline (to implement fully) |
| Validation engine        | `src/code_ref/auth/validation.rs`        | Authority & subject proofs, hierarchy + recipient checks |
| Operations & automation  | `src/code_ref/auth/operations/`          | Key generation and Age encryption integration |
| Cage bridge              | `src/code_ref/auth/bridge/age_integration.rs` | Authority-aware CRUD interface feeding recipient sets |

Use this map to align roadmap milestones and tickets.

---

## 10. Open Questions & Assumptions

1. **Signing material**: Current signatures are mock hashes; must decide on real crypto (Ed25519 vs age identities).
2. **Storage backend**: Need canonical location for ignition key vault (filesystem vs hub secrets).
3. **Proof expiry window**: 24h default—validate against operational needs.
4. **Dual control flows**: Determine exact interface (CLI confirmation, out-of-band approval).
5. **Audit transport**: Confirm whether logs ship to central SIEM or remain local.
6. **Recipient scoping rules**: Decide how to express per-recipient scope (path filters, command filters) and how Cage APIs consume them.

Document answers here as decisions are made.

---

**Owner**: Ignition Core Team  
**Revision Date**: 2025-09-28  
**Status**: Conceptual blueprint – ready to drive roadmap and task planning.
