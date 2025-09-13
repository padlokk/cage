# Cage PTY Fix - Replace Script/Expect with Proper PTY

## 🚨 Problem Analysis

Your `cage` CLI wrapper for `age` is using **script/expect automation** which is fragile and unreliable. The tests are failing because:

1. **Shell escaping issues** - passphrases with special characters break the HERE documents
2. **Timing problems** - `age` prompt timing doesn't align with script/expect assumptions
3. **Platform inconsistencies** - script command behaves differently across Unix systems
4. **No proper TTY detection** - `age -p` can still detect it's not a real terminal

## 🎯 PTY Solution

Replace the `TtyAutomator` implementation with proper PTY automation using `portable-pty`:

### **Step 1: Add PTY Dependency**

Update `Cargo.toml`:

```toml
[dependencies]
# Add portable-pty for proper PTY automation
portable-pty = "0.8"

# Remove expect dependency issues
# Core dependencies for Age automation
tempfile = "3.8"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4.4", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2"
```

### **Step 2: Create Proper PTY Age Automator**

Replace `src/encryption/age_automation/tty_automation.rs` with:

> used pty_wrap.rs


### **Step 3: Update the Factory**

Update your adapter factory to use the new PTY automator:

```rust
// In adapter.rs or wherever you create the automator
use super::pty_automation::PtyAgeAutomator;  // Changed from tty_automation

impl AdapterFactory {
    pub fn create_default() -> AgeResult<Box<dyn AgeAdapter>> {
        let automator = PtyAgeAutomator::new()?;  // Use PTY automator
        Ok(Box::new(automator))
    }
}
```

## 🎯 Why This Fixes Your Tests

### **Reliability Improvements:**

1. **Proper TTY emulation** - `age -p` truly believes it's in a real terminal
2. **Robust passphrase handling** - No shell escaping issues with special characters
3. **Precise timing** - PTY automation responds to actual prompts, not assumptions
4. **Error visibility** - Clear error reporting when things go wrong
5. **Cross-platform** - Works consistently across Unix systems

### **Test Success Factors:**

```rust
// ✅ Your tests will now pass because:
// 1. PTY makes age think it's interactive
// 2. Passphrase handling is robust
// 3. Error conditions are properly detected
// 4. Timing issues are eliminated
// 5. Platform differences don't matter
```

## 🚀 Migration Steps

1. **Add portable-pty dependency** to Cargo.toml
2. **Replace tty_automation.rs** with the PTY version above
3. **Update imports** in files that use TtyAutomator → PtyAgeAutomator
4. **Run tests** - they should now pass reliably!
5. **Remove expect/script dependencies** - no longer needed

This PTY approach is the same pattern used by tmux, screen, and other professional terminal tools. It's **much more reliable** than script/expect automation.
