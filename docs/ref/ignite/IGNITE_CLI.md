# 🛠️ Ignite CLI Architecture & Test Harness

**Intent**: Define the admin/developer command surface for the `ignite` binary and outline the smoke-test harness used for confidence testing.

---

Refer to `docs/ref/rsb/CLI_RSB_USAGE.md` for RSB flag ordering, and `docs/ref/rsb/PRELUDE_POLICY.md` / `MODULE_SPEC.md` for module layout and prelude exports.

Implementation staging:
- RSB entry point: `src/bin/cli_ignite.rs` (bootstrap/options!/dispatch!).
- Command handlers live under `src/ignite/cli/{commands,context}.rs`.
- Shared helpers consumed via `ignite::prelude::*`.
- hub dependency excludes `cli-ext`; RSB supplies the full CLI stack.

## 1. Command Surface (RSB Framework)

| Command | Description | Notes |
|---------|-------------|-------|
| `ignite create <name> [--role=ignition|distro]` | Mint new key under repo authority | Validates hierarchy, emits proofs, updates recipients |
| `ignite rotate <name>` | Rotate existing key | Emits affected-key manifest, cascades invalidation |
| `ignite revoke <name>` | Revoke key without replacement | Creates manifest, tombstone |
| `ignite ls [--role=]` | List keys with status | Includes expiry warnings, recipient memberships |
| `ignite status` | Summarize authority health | Proof freshness, pending renewals |
| `ignite proof --verify [--all|<fingerprint>]` | Verify proof bundles | Checks canonical payload + signature + expiry |
| `ignite manifest --verify <file>` | Validate manifest digest | Confirms schema + SHA256 |
| `ignite recipients --export` | Dump current recipient set versions | For git automation integration |

---

- At startup, Ignite will verify that the `age` binary is reachable before delegating to cage, raising a friendly error if installation is missing.

## 2. Developer Hooks

- **Inspect**: RSB built-in `ignite inspect` reveals registered commands for quick debugging.
- **Stack**: `ignite stack` displays call stack and context during prototype work.
- **Test Mode**: `IGNITE_TEST_MODE=1` environment flag routes filesystem access to an in-memory temp dir for harness tests.

---

## 3. CLI Smoke-Test Harness

### 3.1 `cargo test cli_smoke`
- Spins up a temporary repo structure in `target/ignite-test/`.
- Initializes vault (`./data` override) and seeds deterministic Ed25519 keypairs for reproducibility.
- Executes sequence: create master → create repo → create ignition → rotate ignition → revoke distro.
- Asserts:
  - Proof verification commands succeed.
  - Affected-key manifest generated with expected schema and digest.
  - Recipient export matches expected set hashes.

### 3.2 Failure Paths
- Tamper manifest file → expect `ignite manifest --verify` to fail with specific error code.
- Expire proof by altering timestamp → `ignite proof --verify` should exit non-zero.
- Missing passphrase/env variable triggers safe failure and leaves no partial state.

### 3.3 Reporting
- Harness prints summarized table (command, exit code, artifacts) to simplify integration with CI logs.
- Optional `--json` flag for machine-readable output (future enhancement).

---

## 4. Integration with Padlock & Cage

- CLI intentionally mirrors legacy padlock commands (`padlock ignite ...`) so operators can migrate workflows.
- `ignite recipients --export` feeds into padlock’s git hook layer; eventual integration will map this output into `.padlock` configuration.

---

**Owner**: Ignite Core Team  
**Version**: 1.0 (2025-09-28)
