# 🔐 Padlock Platform – Product Requirements Document (2025 Refresh)

## 1. Executive Summary

Padlock secures source code and operational assets for high-trust engineering teams. With Cage (non-interactive Age automation) and Ignite (authority chain management) nearing completion, Padlock is evolving into a cohesive platform that blends airtight encryption with automation-friendly workflows. This PRD re-baselines the product vision, articulates user needs, and charts the roadmap required to graduate Padlock from individual components to a production-ready platform.

### Product Goals
- **Security without friction** – deliver end-to-end encryption that survives automation, CI, and AI-assisted workflows.
- **Deterministic authority** – enforce the X→M→R→I→D key hierarchy with auditable provenance and revocation.
- **Operational leverage** – allow platform teams to manage hundreds of encrypted repos with policy-driven tooling.
- **Extensibility** – expose stable APIs/CLIs so partner teams can integrate approvals, monitoring, and analytics.

### Success Metrics
- ✅ 95% of encrypted-repo operations executed through Padlock CLI/SDK succeed without manual intervention.
- ✅ <10 minutes to onboard a new repo (from bootstrap to automated unlock in CI).
- ✅ Rotation + manifest pipeline completes in <5 minutes with zero undetected errors.
- ✅ All key operations emit signed manifests and proofs, traceable in the central audit log.
- ✅ NPS ≥ +40 from target operators (security engineers, platform SREs) during pilot.

---

## 2. Users & Needs

### Primary Personas
| Persona | Responsibilities | Pain Without Padlock | Needs |
|---------|------------------|-----------------------|-------|
| **Security Engineer** | Define crypto policy, approve access | Manual Age key juggling, no visibility into access lineage | Enforced hierarchy, auditable manifests, rotation automation |
| **Platform SRE** | Run CI/CD, secrets distribution | CI breaks on passphrase prompts, difficult rollbacks | Non-interactive unlock, key discovery, failure-safe workflows |
| **Repo Maintainer** | Daily workflows inside encrypted repos | Slow key issuance, unclear revocation paths | Self-service ignition commands, status diagnostics |
| **Automation / AI Agent** | Build/test/triage inside locked repos | Cannot satisfy Age prompts; missing context | Stable passphrase interface, minimal secrets exposure |

### Secondary Stakeholders
- **Compliance & Audit** – require tamper-evident manifest history and proof bundles.
- **Incident Response** – need emergency shutoff (Skull key workflows) and forensic lineage.
- **Partner Integrations** – expect documented APIs/webhooks for approvals, notifications, analytics.

---

## 3. Problems & Opportunities

1. **Disjoint Tooling** – Cage and Ignite each solve part of the equation; operators lack a single guided workflow that spans bootstrap → daily use → recovery.
2. **Manual Authority Tracking** – Without automated manifests/proofs, revocations and rotations are error-prone and hard to audit.
3. **AI/Automation Access** – Demand for AI assistants and bots is rising; passphrase distribution must be controlled yet ergonomic.
4. **Incident Readiness** – Skull/Master recovery plans remain ad-hoc, risking downtime during critical incidents.
5. **Operator Onboarding** – Documentation is fragmented; new teams struggle to understand the authority stack.

---

## 4. Scope & Feature Requirements

### 4.1 Foundation (Complete / In-flight)
- **Cage** – PTY-safe Age automation, atomic lock/unlock, audit logging (shipping now).
- **Ignite** – Authority chain data model, Ed25519 proof engine, manifest storage, CLI (near completion).

### 4.2 Platform Capabilities (This PRD)

| Capability | Description | Must / Should / Could | Owner |
|------------|-------------|-----------------------|-------|
| Authority Chain Persistence | Vault-backed storage for keys, manifests, proofs, tombstones | **MUST** | Ignite |
| Key Lifecycle Automation | Create/list/rotate/revoke for X/M/R/I/D tiers with guardrails | **MUST** | Ignite CLI |
| Nonce & Proof Hardening | Cryptographically strong nonces; proof rotation scheduler | **MUST** | Ignite Security |
| Rotation Pipeline | Auto-manifest generation, dependent key discovery, staged roll-out | **MUST** | Ignite + Cage |
| Audit Surfacing | Uniform log format; CLI/API to query manifests/proofs | **SHOULD** | Padlock Core |
| Policy Engine | Configuration of expiry, rotation cadence, dual control | **SHOULD** | Security |
| Integration Hooks | Webhooks / task runners for approvals and notifications | **COULD** | Platform Integrations |
| UX Polish | Rich CLI help, guided flows, failure diagnostics, docs refresh | **SHOULD** | DevRel |

### 4.3 Out of Scope (for this release)
- Cloud-hosted key escrow or KMS integration (tracked separately).
- Mobile/GUI clients – CLI + docs remain the primary interface.
- Full-blown secrets manager; focus stays on git-centric repos and artifacts.

---

## 5. User Journeys

1. **Bootstrap a new repo**
   - `padlock init` → Scaffolds repo metadata, verifies dependencies.
   - `padlock master mint` → Issues M key.
   - `padlock repo enroll` → Creates R key, registers repo.
   - `padlock ignite create` → Generates I key, writes manifest.
   - `padlock ignite create --type=distro` → Issues D key for CI.
   - CI pipeline exports D key passphrase via secret manager.

2. **Rotation & Manifest Workflow**
   - Operator triggers `padlock rotate ignition <repo>`.
   - Ignite validates proofs, cascades dependent D keys, writes manifest.
   - Cage re-encrypts recipients with updated public keys.
   - Automation receives manifest webhook, schedules re-lock jobs.

3. **Incident Containment**
   - `padlock skull unlock` with dual control to access master vault.
   - `padlock ignite revoke --scope=repo/<id>` to freeze access.
   - Audit log aggregated for IR review; manifests attached to ticket.

---

## 6. Functional Requirements

- **Command Surface** – Complete CLI parity for every key tier (create/list/show/rotate/revoke/import/export/status).
- **Validation** – Pre-flight checks for binary dependencies (age), vault permissions, configuration.
- **Storage Layout** – XDG-aware directories (`keys/`, `proofs/`, `manifests/`, `metadata/`) with atomic writes.
- **Proof Engine** – Ed25519 signatures, canonical JSON, 24h expiry defaults, renewal scheduler (configurable).
- **Manifest Integrity** – Canonicalization, SHA256 digests, CLI verify command returning non-zero on failure.
- **Observability** – Structured logs, per-command correlation IDs, optional JSON output.
- **Extensibility** – Public crate exports enable workflows in SDK or automation tasks.

---

## 7. Non-Functional Requirements

| Category | Requirement |
|----------|-------------|
| Security | Zero plaintext key storage; private material on disk encrypted or passphrase-wrapped.
| Reliability | Commands idempotent where possible; safe retries; explicit tombstones for revoked keys.
| Performance | CLI operations <2s under nominal load; rotation pipeline <5 minutes for repos ≤1k recipients.
| Portability | Linux/macOS support; respect XDG variables; works in containerized CI.
| Observability | Logs to stdout + optional file; manifests/proofs accessible via CLI.
| Documentation | Up-to-date reference manual, runbooks, and onboarding guides at GA.

---

## 8. Release Plan & Milestones

| Milestone | Target | Description | Exit Criteria |
|-----------|--------|-------------|---------------|
| **M1 – Authority Core GA** | Apr 2025 | Ignite storage, CLI, proof verification | `cargo test` green; `ignite create/list/status/verify` production ready |
| **M2 – Rotation Readiness** | May 2025 | Cascade rotation, manifest pipeline, nonce hardening | Automated rotation demo; manifest verify passes; scheduler jobs |
| **M3 – Padlock CLI Integration** | Jun 2025 | Wrap Cage + Ignite under unified `padlock` CLI | End-to-end repo bootstrap & CI unlock works |
| **M4 – Audit & Policy** | Jul 2025 | Logging, policy config, dual-control enforcement | Policy file format stable; audit log ingestion MVP |
| **M5 – Pilot & Feedback** | Aug 2025 | Rollout to 3 pilot teams, capture metrics | Onboarding <10 min; NPS ≥40; incident drills passed |

---

## 9. Open Questions & Risks

- **Scheduler Ownership** – Should proof/manifest renewal run inside Ignite or an external orchestrator?
- **Secrets Distribution** – Preferred mechanism for delivering passphrases to CI? (HashiCorp Vault, SSM, internal service?)
- **Dual Control** – Implementation detail for Skull/Master unlock approvals (cli prompt vs. delegated service).
- **Telemetry** – What metrics can be safely collected without leaking sensitive metadata?
- **AI Access Controls** – Policy guardrails to prevent AI agents from writing secrets back into repos.

---

## 10. Appendices

- **Glossary** – see `PADLOCK_CONCEPTS.md`.
- **Roadmap** – see `PADLOAD_ROADMAP.md` for granular tasks.
- **Change Log** – tracked in repo releases; initial 2025 refresh authored by Padlock Core Team.
