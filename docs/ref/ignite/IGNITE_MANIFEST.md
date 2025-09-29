# 🔧 Ignite Affected-Key Manifest Specification

**Purpose**: Document the JSON schema used to record descendants invalidated by a rotation or revocation event. Manifests enable downstream automation to repair or re-encrypt artifacts without Ignite touching historical data.

---

## 1. File Location

- Stored under `manifests/` inside the Ignite vault (`./data/manifests/` for local, XDG data path for production).
- Filename pattern: `<parent_fingerprint>/<timestamp>_<event>.json` (e.g., `a1b2c3d4/2025-09-28T12-30-00Z_rotate.json`).
- Parent fingerprint directories keep manifest history grouped per authority key.

---

## 2. Canonical JSON Structure

```json
{
  "schema_version": "1.0",
  "event": {
    "type": "rotation",
    "parent_fingerprint": "SHA256:a1b2c3d4",
    "initiated_at": "2025-09-28T12:30:00Z",
    "initiated_by": "ignite-cli",
    "reason": "scheduled-rotation"
  },
  "digest": {
    "algorithm": "SHA256",
    "value": "ecf21f2...",
    "manifest_body": "canonical"
  },
  "children": [
    {
      "fingerprint": "SHA256:deadbeef",
      "role": "ignition",
      "status": "revoked",
      "ciphertext_md5": "8f14e45fceea167a5a36dedd4bea2543",
      "scope": {
        "paths": ["locker/docs_sec"],
        "env": "production"
      },
      "issued_at": "2025-03-01T09:00:00Z",
      "revoked_at": "2025-09-28T12:30:00Z"
    }
  ]
}
```

### Fields
- `schema_version`: Allows future migrations.
- `event.type`: `rotation` or `revocation`.
- `event.parent_fingerprint`: SHA256 identifier of the parent key performing the action.
- `digest`: Hash of the manifest body computed after sorting for canonical JSON (see §4).
- `children[]`: List of affected descendant keys with role, status, and context metadata.
- `ciphertext_md5`: Existing checksum of affected artifact for correlation; optional if not applicable.
- `scope`: Free-form object for automation hints (paths, environment, repo id, etc.).

---

## 3. Generation Rules

1. Manifests are produced after a rotation/revocation completes and before the new lineage is committed.
2. Each manifest is immutable; Ignite never edits in place. Follow-up corrections produce a new manifest referencing the same children.
3. If no descendants exist, Ignite still emits an empty `children` array to record the event.

---

## 4. Canonicalization & Digests

- Ignite serializes the manifest using canonical JSON (sorted keys, UTF-8, LF line endings) prior to hashing.
- `digest.value` is computed over the canonical body excluding the `digest` object.
- Verification tools recompute the digest to assert no tampering occurred.

---

## 5. Validation & Confidence Tests

- Unit tests must ensure manifest files validate against this schema and reject missing required fields.
- Sanity check for hub re-exports (serde/uuid/rand) lives in `tests/unit/deps.rs`.
- Integration tests (IG-0207) verify manifests emit during rotations and can be reloaded to reconstruct affected keys.
- CLI harness (`ignite proof --verify`) will include `ignite manifest --verify <file>` to confirm hashes.

---

**Owner**: Ignite Core Team  
**Version**: 1.0 (2025-09-28)
