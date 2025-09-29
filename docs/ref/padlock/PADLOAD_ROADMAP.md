# 🗺️ Padlock Roadmap (Cage + Ignite Consolidation)

> Updated: 2025-01 — Maintained by Padlock Core. This roadmap captures major milestones and supporting tasks required to ship the unified Padlock experience.

---

## 1. Roadmap Snapshot

| Timeline | Milestone | Status | Owner | Notes |
|----------|-----------|--------|-------|-------|
| **Mar 2025** | Cage 1.0 hardening | ✅ Done | Cage Team | Non-interactive unlock GA, audit log stabilization |
| **Apr 2025** | Ignite Authority GA | 🟡 In Progress | Ignite Team | Storage + CLI shipped; rotation + proof scheduler outstanding |
| **May 2025** | Unified Padlock CLI beta | ⏳ Planned | Padlock Core | Wrap Cage + Ignite workflows; deliver `padlock init`, `padlock rotate`, `padlock status` |
| **Jun 2025** | Rotation & Revocation automation | ⏳ Planned | Ignite + Cage | Cascade manifests, dependent key handling, notifications |
| **Jul 2025** | Policy & audit surfacing | ⏳ Planned | Security Engineering | Dual control enforcement, policy files, centralized audit queries |
| **Aug 2025** | Pilot rollout (3 teams) | ⏳ Planned | Program Ops | Measure onboarding time, gather NPS, run incident simulations |

Legend: ✅ Done · 🟡 In Progress · ⏳ Planned · 🔴 Blocked

---

## 2. Detailed Workstreams

### 2.1 Authority Chain Completion (Ignite)

**Objectives**
- Finalize rotation/revocation flows for I/D keys.
- Harden nonce generation & proof renewal scheduler.
- Expand CLI coverage (rotate, revoke, import, export).

**Key Tasks**
1. Implement crypto-secure nonce generation (hub::random_ext) [✅]
2. AuthorityChain graph validation + dependent key discovery [🟡]
3. Rotation command with manifest generation & tombstones [⏳]
4. Revocation workflow with CLI prompts + policy hooks [⏳]
5. Proof renewal background job (cron or external trigger) [⏳]

**Risks & Mitigations**
- *Risk*: Proof scheduler timing drift → Mitigation: allow manual `--force-renew` CLI.
- *Risk*: Manifest tampering → Mitigation: digest verification enforced in CLI + audits.

### 2.2 Cage Enhancements

**Objectives**
- Ensure Cage exposes recipient management APIs consumed by Ignite rotation.
- Improve observability (structured logs, exit codes) for automation.

**Key Tasks**
1. CRUD manager integration tests with Ignite-generated keysets [🟡]
2. Recipient scoping tags (path-based) [⏳]
3. Clamp mode improvements & status reporting [⏳]

### 2.3 Unified Padlock CLI

**Objectives**
- Provide a single binary (`padlock`) that orchestrates ignite + cage flows.
- Deliver guided onboarding (bootstrap) and rotation commands with guardrails.

**Key Tasks**
1. Re-export common prelude (env checks, logging) [🟡]
2. Command scaffolding: `padlock init`, `padlock ignite`, `padlock rotate`, `padlock status` [⏳]
3. Add JSON output mode for automation [⏳]
4. Integration tests spanning bootstrap → CI unlock → rotation [⏳]

### 2.4 Policy & Audit Layer

**Objectives**
- Express rotation cadence, expiry, dual-control requirements in a central config.
- Provide CLI/API to inspect audit history across repos.

**Key Tasks**
1. Policy schema (`metadata/policy.toml`) + loader [⏳]
2. Dual-control enforcement for Skull/Master actions [⏳]
3. Audit log aggregation + search CLI (`padlock audit list`) [⏳]
4. Manifest/proof webhook integration (Slack/Jira) [⏳]

### 2.5 Documentation & Enablement

**Objectives**
- Deliver cohesive documentation for onboarding, operations, and incident response.

**Key Tasks**
1. Update README + architecture diagrams (this doc) [✅]
2. Runbook for common operations (bootstrap, rotation, recovery) [🟡]
3. Incident drill playbook (Skull unlock, mass revocation) [⏳]
4. Video walkthrough / workshop for pilot teams [⏳]

---

## 3. Dependency Map

```
AuthorityChain graph → Rotation CLI → Manifest pipeline → Cage recipient updates → Policy enforcement → Audit surfacing
```

- Rotation CLI cannot ship until AuthorityChain validation + dependent key discovery are stable.
- Padlock CLI beta requires Ignite rotation APIs and Cage recipient tagging.
- Policy enforcement depends on manifest/rotation signals and CLI exit code reliability.

---

## 4. Metrics & Checkpoints

| Metric | Target | Checkpoint |
|--------|--------|------------|
| Repo bootstrap time | ≤ 10 min | M2 exit |
| Rotation success rate | ≥ 99% automated | M3 exit |
| Mean manifest generation time | ≤ 1 min | M3 exit |
| Proof renewal freshness | 100% renewed ≥12h before expiry | M4 exit |
| Pilot satisfaction (NPS) | ≥ +40 | M5 exit |

---

## 5. Open Issues & Follow-ups

- Define storage encryption strategy for private key blobs (Cage integration vs. OS keyring).
- Decide on scheduler runtime (cron job, systemd timer, Kubernetes job).
- Establish incident escalation contacts and on-call rotations for Padlock services.
- Confirm compliance requirements for manifest retention and secure deletion.

---

## 6. Change Log

- **2025-01-12** – Initial roadmap drafted post Ignite CLI MVP (Padlock Core Team).
- Subsequent changes tracked via Git history and release notes.
