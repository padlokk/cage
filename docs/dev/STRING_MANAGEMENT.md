# String Management Guidelines (SEC-01)

**Purpose**: Centralize user-facing strings to improve security, maintainability, and consistency.
**Created**: 2025-09-29
**Status**: Partially Implemented

## Overview

The Cage project uses centralized string management to:
1. Reduce binary snooping risks by consolidating strings
2. Maintain consistent messaging across the application
3. Enable future internationalization (i18n)
4. Provide a single source of truth for all user-facing text

## Current State

- ✅ Central string module exists: `src/cage/strings.rs`
- ✅ Common messages, errors, and prompts are defined
- 🟡 Many inline strings still exist in critical modules
- 🟡 Emojis/glyphs retained for UX (intentional design choice)

## Guidelines

### What to Centralize

✅ **SHOULD** be in `strings.rs`:
- User-facing error messages
- Status messages and prompts
- Help text and descriptions
- Warning messages
- Operation names
- File operation feedback

### What Can Stay Inline

✅ **MAY** remain inline:
- Debug/trace output (not user-facing)
- Test assertions and test messages
- Binary/command names (e.g., "age", "ssh-keygen")
- File extensions (e.g., ".age", ".bak")
- Path separators and special characters
- Format strings with complex interpolation
- Emojis/glyphs for UX enhancement (project choice)

### When Adding New Strings

1. **Check if it exists**: Look in `src/cage/strings.rs` first
2. **Consider the audience**: Is this user-facing or internal?
3. **Add to strings.rs if**:
   - It's shown to users
   - It might need translation later
   - It's used in multiple places
4. **Keep inline if**:
   - It's debug/development only
   - It's a technical identifier
   - It's test-specific

## Usage Examples

### Good: Using Centralized Strings

```rust
use crate::cage::strings;

// User-facing error
return Err(AgeError::FileNotFound {
    path: path.to_path_buf(),
    message: strings::ERR_FILE_NOT_FOUND.to_string(),
});

// Status message
println!("{}: {}", strings::STATUS_PROCESSING, filename);
```

### Acceptable: Inline Technical Strings

```rust
// Binary names - technical requirement
let mut cmd = Command::new("age");

// Debug output - not user-facing
trace!("Executing age with args: {:?}", args);

// Test assertions - test-specific
assert!(result.is_ok(), "Encryption should succeed");
```

## Audit Tools

### Check for Inline Strings

```bash
# Run the audit script
./scripts/check_inline_strings.sh

# See detailed results
./scripts/check_inline_strings.sh --verbose
```

### Current Statistics

As of 2025-09-29:
- Total string literals: ~1079
- Potential candidates for centralization: ~705
- Critical modules with most inline strings:
  - `src/bin/cli_age.rs` (304 candidates)
  - `src/cage/lifecycle/crud_manager.rs` (182 candidates)

## ASCII-Safe Mode (Future Enhancement)

While emojis and glyphs are currently retained for UX, we may add an optional ASCII-safe mode:

```rust
// Potential future feature
if config.ascii_safe_mode {
    println!("[OK] {}", message);
} else {
    println!("✅ {}", message);
}
```

## Migration Strategy

1. **Phase 1** (Current): Core strings centralized
2. **Phase 2**: Audit and migrate high-value user messages
3. **Phase 3**: Add lint rules to prevent regression
4. **Phase 4**: Consider i18n framework if needed

## Enforcement

### CI/CD Checks (Recommended)

```yaml
# Example GitHub Actions check
- name: Check inline strings
  run: |
    ./scripts/check_inline_strings.sh
    if [ $? -ne 0 ]; then
      echo "Warning: High number of inline strings detected"
    fi
```

### Pre-commit Hook (Optional)

```bash
#!/bin/bash
# .git/hooks/pre-commit
./scripts/check_inline_strings.sh
```

## Exceptions

The following are explicitly exempt from centralization:
- RSB framework glyphs (part of the framework's identity)
- Terminal color codes (handled by RSB)
- Regex patterns (technical, not user-facing)
- SQL queries (if any)
- JSON field names (structural)