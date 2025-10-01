# MODULE_SPEC Reorganization Plan
**Date:** 2025-10-01
**Project:** Cage v0.5.0
**Objective:** Align src/cage structure with RSB MODULE_SPEC v3

---

## Current State Analysis

### Organized Modules (✅ Compliant)
```
src/cage/manager/        - CageManager (121KB)
  ├── mod.rs
  └── cage_manager.rs

src/cage/operations/     - File/Repo operations
  ├── mod.rs
  ├── file_operations.rs
  └── repository_operations.rs

src/cage/keygen/         - Key generation (NEW)
  ├── mod.rs
  ├── api.rs
  ├── error.rs
  └── helpers.rs

src/cage/chunker/        - Progress-aware processing
  └── mod.rs (12KB, single file module)
```

### Scattered Files (❌ Needs Reorganization)

**Adapter Files (3 files, 2.8KB total):**
- `adapter.rs` (273 lines) - v1 adapter
- `adapter_v2.rs` (1653 lines) - v2 adapter with streaming
- `adapter_v2_pipe_passphrase.rs` (88 lines) - pipe streaming experiment

**Security/PTY Files (4 files, 2.3KB total):**
- `security.rs` (677 lines) - audit logging
- `passphrase.rs` (341 lines) - passphrase management
- `pty_wrap.rs` (892 lines) - PTY automation
- `tty_automation.rs` (433 lines) - TTY automation

**Core/Config Files (4 files, 1.9KB total):**
- `config.rs` (859 lines) - AgeConfig
- `requests.rs` (868 lines) - request structs
- `age_engine.rs` (207 lines) - age engine
- `in_place.rs` (346 lines) - recovery manager

**Support Files (2 files):**
- `error.rs` (567 lines) - error types
- `strings.rs` (173 lines) - string constants

---

## Approved Reorganization (Full RSB Alignment)

**Decision:** Combine Option B + C with custom naming conventions

### Final Target Structure

```
src/
├── lang.rs            [NEW] String constants (from cage/strings.rs)
├── prelude.rs         [KEEP] Prelude
├── deps.rs            [KEEP] Deps
├── lib.rs             [KEEP] Lib entry
│
├── bin/               [KEEP] CLI binaries
│
└── cage/              Main package namespace
    ├── mod.rs
    ├── error.rs       [KEEP] Top-level errors
    │
    ├── adp/           [NEW] Adapters (adapter pattern)
    │   ├── mod.rs
    │   ├── v1.rs      (from adapter.rs)
    │   ├── v2.rs      (from adapter_v2.rs)
    │   └── pipe.rs    (from adapter_v2_pipe_passphrase.rs)
    │
    ├── pty/           [NEW] PTY/TTY automation
    │   ├── mod.rs
    │   ├── wrap.rs    (from pty_wrap.rs)
    │   ├── tty.rs     (from tty_automation.rs)
    │   └── pass.rs    (from passphrase.rs)
    │
    ├── audit/         [NEW] Audit & security
    │   └── mod.rs     (from security.rs)
    │
    ├── core/          [NEW] Core primitives
    │   ├── mod.rs
    │   ├── config.rs  (from config.rs)
    │   ├── requests.rs (from requests.rs)
    │   ├── engine.rs  (from age_engine.rs)
    │   └── recovery.rs (from in_place.rs)
    │
    ├── mgr/           [RENAME] manager/ → mgr/
    ├── forge/         [RENAME] operations/ → forge/
    ├── keygen/        [KEEP] Key generation
    └── buff/          [RENAME] chunker/ → buff/
```

**Story Points:** 15 pts total (5 phases)

---

## Naming Conventions Applied

| Old Name | New Name | Rationale |
|----------|----------|-----------|
| `adapter*` | `adp/` | Standard adapter abbreviation pattern |
| `operations/` | `forge/` | Aligns with future operational patterns |
| `chunker/` | `buff/` | Buffer processing concept |
| `manager/` | `mgr/` | Standard manager abbreviation |
| `security.rs` | `audit/` | Single-file module for audit logging |
| `strings.rs` | `lang.rs` | MODULE_SPEC standard location |

---

## Migration Strategy

**5-Phase Sequential Execution:**

### Phase 1: Adapter Consolidation (MOD4-01) - 3 pts
- Create `src/cage/adp/` directory structure
- Move and rename adapter files
- Update all import paths
- Verify tests pass

### Phase 2: PTY Automation (MOD4-02) - 3 pts
- Create `src/cage/pty/` directory structure
- Move and rename PTY/TTY/passphrase files
- Update all import paths
- Verify tests pass

### Phase 3: Audit Module (MOD4-03) - 2 pts
- Create `src/cage/audit/` directory
- Move security.rs → audit/mod.rs
- Update all import paths
- Verify tests pass

### Phase 4: Core Primitives (MOD4-04) - 3 pts
- Create `src/cage/core/` directory structure
- Move config, requests, engine, recovery files
- Update all import paths
- Verify tests pass

### Phase 5: Directory Renames (MOD4-05) - 2 pts
- Rename `manager/` → `mgr/`
- Rename `operations/` → `forge/`
- Rename `chunker/` → `buff/`
- Update all import paths
- Verify tests pass

### Phase 6: Lang Module (MOD4-06) - 2 pts
- Move `src/cage/strings.rs` → `src/lang.rs`
- Update all imports from `cage::strings` → `crate::lang`
- Update prelude re-exports
- Verify tests pass

**Total:** 15 pts over 6 phases

---

## Key Decisions Needed

### 1. Module Names
- `adapt` vs `adapters`?
- `pty` vs `automation`?
- `audit` vs `security`?
- `core` vs `base` vs `types`?

### 2. What to Keep Scattered
- `error.rs` - top-level errors (often scattered per MODULE_SPEC)
- `strings.rs` - should become `src/lang.rs` per spec

### 3. Standard Files (helpers.rs, utils.rs, macros.rs)
- **Only create if functions exist to populate them**
- Currently: No obvious helpers/utils scattered in need of consolidation
- Defer until actual need emerges

### 4. Renames
- Keep `operations` or shorten to `ops`?
- Keep `chunker` or shorten to `chunk`?
- Keep `manager` or shorten to `mgr`?

---

## Import Impact Analysis

**Files importing adapters:**
```bash
grep -r "use.*cage::adapter" src/ tests/ | wc -l
# ~15-20 import sites
```

**Files importing security/pty:**
```bash
grep -r "use.*cage::(security|passphrase|pty_wrap|tty)" src/ tests/ | wc -l
# ~10-15 import sites
```

**Mitigation:**
- Update `src/cage/mod.rs` re-exports to maintain backward compat
- Use `pub use` chains to preserve old paths temporarily
- Update imports incrementally, test after each module

---

## Testing Strategy

After each module reorganization:
1. `cargo build` - verify compilation
2. `cargo test --lib` - run unit tests
3. `cargo test --test '*'` - run integration tests
4. `./bin/test.sh run smoke` - CLI smoke tests

---

## Next Steps

1. **Stakeholder approval** on Option A/B/C and module names
2. **Create detailed MOD4-NN task breakdowns** with file moves
3. **Execute Phase 1** (adapters) as proof of concept
4. **Review impact** before proceeding to Phase 2+

---

## Questions for Review

1. Preferred option: A (minimal), B (moderate), or C (full)?
2. Module naming: Short (`adapt`, `pty`, `core`) or descriptive?
3. Keep `operations`, `chunker`, `manager` names or abbreviate?
4. Move `strings.rs` → `src/lang.rs` now or defer?
5. Any modules that should stay scattered?
