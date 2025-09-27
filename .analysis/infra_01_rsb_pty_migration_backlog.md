# INFRA-01 BACKLOG: RSB dev-pty Migration

**Date**: 2025-09-27
**Priority**: Low (Infrastructure Improvement)
**Blocked By**: RSB API Export Issues

---

## 🎯 Task Overview

**Objective**: Migrate from direct `portable-pty` dependency to RSB's `dev-pty` module when API exports are corrected.

**Current State**: RSB 0.6.2 includes `dev-pty` feature but doesn't export the full `portable-pty` API that Cage requires.

---

## 🔍 Investigation Summary

### RSB dev-pty Feature Status
- **Feature Exists**: ✅ `dev-pty` feature flag works
- **Module Available**: ✅ `rsb::dev::pty` module compiles
- **API Complete**: ❌ Missing key exports we need

### Missing APIs in RSB dev-pty
```rust
// Currently not exported by RSB dev-pty module:
native_pty_system()   // Core PTY system access
PtySize              // PTY size configuration
CommandBuilder       // Command building for PTY
```

### Current Working Implementation
```rust
// Current (working):
use portable_pty::*;

// Future target (when RSB exports are fixed):
use rsb::dev::pty::*;
```

---

## 📋 Migration Checklist

### Prerequisites (RSB Framework Changes)
- [ ] RSB exports `native_pty_system` function
- [ ] RSB exports `PtySize` struct
- [ ] RSB exports `CommandBuilder` struct
- [ ] RSB dev-pty module provides full `portable-pty` API compatibility

### Migration Tasks
- [ ] Test RSB dev-pty API compatibility
- [ ] Update imports in `src/cage/pty_wrap.rs`
- [ ] Update imports in `src/driver.rs`
- [ ] Remove direct `portable-pty` dependency from `Cargo.toml`
- [ ] Verify all PTY automation functionality works
- [ ] Update documentation

---

## 🧪 Validation Plan

### Test Cases
1. **Basic PTY Operations**: Age command execution
2. **Progress Integration**: PTY with progress reporting
3. **Error Handling**: PTY timeout and failure scenarios
4. **Cross-Platform**: Unix and Windows (if supported)

### Validation Commands
```bash
# Test core functionality
CAGE_PASSPHRASE=test ./target/debug/cage lock test.txt --progress
CAGE_PASSPHRASE=test ./target/debug/cage unlock test.txt.cage --progress

# Test in-place operations
CAGE_PASSPHRASE=test ./target/debug/cage lock test.txt --in-place --progress

# Test Age proxy mode
echo "test" | ./target/debug/cage proxy --age-p --stdin-passphrase input.txt
```

---

## 📈 Benefits of Migration

### Dependency Reduction
- **Before**: `portable-pty = "0.9"` + `rsb = { git = "..." }`
- **After**: `rsb = { git = "...", features = ["dev-pty"] }`

### Ecosystem Integration
- **Consistency**: All CLI and PTY functionality from RSB
- **Maintenance**: Single dependency for core infrastructure
- **Updates**: Coordinated RSB ecosystem updates

### Code Simplification
```rust
// Simplified imports (future state)
use rsb::prelude::*;        // CLI framework
use rsb::dev::pty::*;       // PTY automation
```

---

## 🚧 Current Workaround

**Status**: Using direct `portable-pty` dependency until RSB exports are corrected.

**Dependencies**:
```toml
[dependencies]
portable-pty = "0.9"                               # ✅ Required for now
rsb = { git = "https://github.com/oodx/rsb", branch = "main" }  # ✅ CLI framework
```

**Implementation**:
```rust
// src/cage/pty_wrap.rs
use portable_pty::*;  // Direct dependency until RSB migration
```

---

## 📞 Action Items

### For RSB Framework Team
- Export full `portable-pty` API in `dev-pty` module
- Ensure API compatibility with existing `portable-pty` usage patterns
- Document dev-pty module API surface

### For Cage Team (Future)
- Monitor RSB releases for dev-pty API completion
- Test migration path when RSB exports are ready
- Update documentation after successful migration

---

## 🔗 References

- **RSB Repository**: https://github.com/oodx/rsb
- **portable-pty**: https://crates.io/crates/portable-pty
- **Investigation Date**: 2025-09-27
- **Current RSB Version**: 0.6.2
- **Current portable-pty Version**: 0.9.0

---

**Status**: ✅ **COMPLETE** - Migrated to Hub terminal-ext
**Actual Effort**: 0.5 story points (simpler than expected)
**Impact**: Medium (Cleaner ecosystem integration, one less direct dependency)

### UPDATE: Migration Complete via Hub

Instead of waiting for RSB dev-pty exports, we discovered Hub's `terminal-ext` feature provides `portable-pty` integration:

```toml
# Before
portable-pty = "0.9"

# After
hub = { git = "https://github.com/oodx/hub.git", features = ["terminal-ext"] }
```

```rust
// Implementation
use hub::terminal_ext::portable_pty::*;  // RSB ecosystem approach
```

This follows the official RSB ecosystem pattern documented in HOWTO_HUB.