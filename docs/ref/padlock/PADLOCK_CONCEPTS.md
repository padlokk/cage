# 🔑 Padlock Concepts – Authority, Automation, and Recovery

This document distills the core mental models underpinning Padlock. It is written for engineers building or operating the platform, and should be read alongside `PADLOCK_PRD.md` and `PADLOAD_ROADMAP.md`.

---

## 1. Authority Chain at a Glance

```
X (Skull) → M (Master) → R (Repo) → I (Ignition) → D (Distro)
```

| Tier | Type | Storage Form | Primary Owner | Responsibilities |
|------|------|--------------|---------------|------------------|
| **X** | Ignition (passphrase) | Offline / Split custody | Security Guardians | Disaster recovery, M key bootstrap |
| **M** | Raw keypair | Platform Security | Global operations, new repo onboarding |
| **R** | Raw keypair | Repo Maintainers | Day-to-day encryption + rotations |
| **I** | Ignition (passphrase) | Repo Ops | Bridge between R and D, rotation orchestrator |
| **D** | Ignition (passphrase) | Automation / AI Agents | Scoped access for CI, tooling, AI |

### Directionality
- **Authority** flows downward: each parent can create, rotate, or revoke its child tier.
- **Subject (lineage)** flows upward: each child proves provenance to its parent via Ed25519 receipts.

### Ignition Keys
“Ignition” refers to any key material wrapped by a passphrase. X, I, and D are ignition keys. Cage provides the passphrase UX; Ignite records the lineage and manifests.

---

## 2. Core Components

### 2.1 Cage (Automation Layer)
- PTY-safe Age automation (lock/unlock/status) with zero TTY prompts.
- Passphrase manager abstractions (environment variables, delegated prompts).
- Audit logger capturing operation, scope, and recipient hashes.
- Exposes `CrudManager`, `LockOptions`, `UnlockOptions` for higher-level workflows.

### 2.2 Ignite (Authority Layer)
- Data models for keys, fingerprints, manifests, proofs.
- Storage adapters (XDG-aware vault: `keys/`, `proofs/`, `manifests/`, `metadata/`).
- CLI surface for key lifecycle, status, manifest verification.
- Proof engine (Ed25519) signing authority claims + subject receipts.
- Manifest generation for rotations/revocations with digest verification.

### 2.3 Padlock CLI (Unification Layer)
- Orchestrates Cage + Ignite commands into end-to-end workflows.
- Provides guardrails (dependency checks, policy enforcement, dual control prompts).
- Hosts high-level commands: `padlock init`, `padlock rotate`, `padlock ignite`, `padlock status`.

---

## 3. Data & Storage Model

```
~/.local/share/padlokk/ignite
├── keys/
│   ├── skull/
│   ├── master/
│   ├── repo/
│   ├── ignition/
│   └── distro/
├── proofs/
│   └── <fingerprint>/timestamp.json
├── manifests/
│   └── <parent-short>/timestamp_event.json
└── metadata/
    ├── policy.toml
    └── tombstones/
```

- **Keys** – Serialized `AuthorityKey` records, including material (encrypted where applicable) and metadata.
- **Proofs** – Canonical JSON payloads + signature bundles, one directory per parent fingerprint.
- **Manifests** – Affected-key manifests capturing cascaded revocations/rotations (immutable).
- **Metadata** – Policy definitions, rotation schedules, tombstones for revoked fingerprints.

Atomic write path: serialize → temp file (`.tmp`) → fsync → rename. All reads validate JSON + digest.

---

## 4. Cryptography & Proofs

- **Fingerprints**: SHA256 of key material (`algorithm:fingerprint` format).
- **Proof Bundle**: `{ payload_json, digest, signature, public_key, expires_at }`.
- **Canonical JSON**: Keys sorted lexicographically, RFC3339 timestamps, ASCII only.
- **Nonces**: 128-bit cryptographically random strings via `hub::random_ext`.
- **Expiry**: Default 24h; renewal scheduler ensures fresh proofs before expiry window.

Validation flow:
1. Check payload digest matches stored `digest`.
2. Verify Ed25519 signature against `public_key`.
3. Ensure `expires_at` future timestamp.
4. Confirm parent/child fingerprints align with registered keys.

---

## 5. Operational Lifecycles

### 5.1 Key Creation
1. Ensure parent tier exists and has authority.
2. Generate key material (Ed25519 or Age) using Cage helper.
3. Wrap private material if ignition tier.
4. Persist key + metadata (`AuthorityKey::save`).
5. Emit proof bundle + manifest entry if the command affects existing hierarchy.

### 5.2 Rotation
1. Validate parent authority and child state.
2. Create new key material; mark old key as archived.
3. Generate manifests enumerating all affected descendants.
4. Cascade updates to D keys (either revoke or reissue).
5. Trigger Cage workflows to re-lock artifacts with new recipients.
6. Store proofs + manifests; schedule follow-up tasks (e.g., notifications).

### 5.3 Revocation
1. Identify target key + dependents.
2. Update manifests with revocation event.
3. Move key to tombstones; block re-registration.
4. Notify consuming systems (webhook / CLI output).
5. Optionally enforce clamp mode (Cage) until replacements issued.

---

## 6. Policy & Governance

- **Policy Engine** – Modular `PolicyEngine` composes policies that participate in `apply_key_defaults`, `validate_key`, and `validate_passphrase` phases (default bundle: expiration + passphrase strength).
- **Expiration Policies** – Defaults applied automatically (Ignition ≈30d, Distro ≈7d) with warning windows; additional tiers can opt-in via policy configuration.
- **Passphrase Enforcement** – Ignition-tier keys are wrapped only when passphrases satisfy length/diversity/ban-list rules; violations bubble up as CLI errors.
- **Dual Control** – Skull and Master actions require secondary confirmation (e.g., signed token, out-of-band approval).
- **Recipient Scoping** – D keys are tagged with repo + path scopes, enforced by Cage when constructing recipient lists.
- **Audit Requirements** – Every mutation must produce manifest + proof; logs contain fingerprint, actor, reason codes.

---

## 7. Interfaces & Extensibility

### CLI Highlights
- `ignite create --key-type <tier>`
- `ignite list --key-type <tier>`
- `ignite status`
- `ignite verify <file>`

Planned Padlock CLI wrappers will expose higher-level workflows (`padlock rotate`, `padlock ignite allow`).

### Library Surface
- `ignite::authority::{AuthorityKey, KeyType, ManifestEvent, ProofBundle}`
- `ignite::authority::storage::{save_key, load_key, save_manifest, save_proof}`
- `ignite::guards::ensure_age_available()` for dependency checks.

### Integration Hooks (Planned)
- Webhook dispatch on manifest creation.
- JSON output (`--output json`) for CLI commands.
- Policy config file watchers to auto-apply rotation cadences.
- Until Cage exposes first-class multi-recipient APIs, Ignite can lean on Cage’s proxy mode to call `age` directly with multiple recipients; this is a temporary bridge tracked by CAGE-08/09.

---

## 8. Recovery & Incident Patterns

1. **Emergency Access (Skull)** – Offline passphrase fragments reconstituted; Skull unlocks Master; operations logged in manifest `emergency` event.
2. **Repo Compromise** – `padlock ignite revoke --scope=repo/<id>` triggers manifest + clamps Cage operations to read-only until new D keys deployed.
3. **Lost D Key** – operator issues `padlock ignite rotate distro <name>` to replace passphrase; manifest marks old key revoked and new key issued.

---

## 9. Related Documents
- Product strategy: `PADLOCK_PRD.md`
- Roadmap & milestones: `PADLOAD_ROADMAP.md`
- Security analysis: `docs/ref/padlock/security_analysis.md`
- Runbooks & test plans: `docs/ref/padlock/TEST_PLAN_001.md`

---

## 10. Future Considerations
- Multi-repo policy coordination (e.g., global rotations on schedule).
- Federation across organizations (delegated trust domains).
- Metrics/telemetry pipeline respecting privacy while surfacing health signals.
- UX for AI agents to request temporary elevation with human approval.
- Alternate authority topologies: abstract the AuthorityChain rules behind a pluggable topology interface so different domains (e.g., software licensing, supply-chain signing) can reuse Ignite with custom parent/child relationships while sharing storage, policy, and CLI infrastructure.
