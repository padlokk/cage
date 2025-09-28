//! Centralized String Constants Module (SEC-01)
//!
//! This module centralizes all user-facing strings and messages to:
//! 1. Mitigate binary snooping risks
//! 2. Provide consistent messaging across the application
//! 3. Enable easy internationalization in the future
//! 4. Maintain a single source of truth for all text output

use rsb::visual::glyphs::glyph;

// ============================================================================
// OPERATION NAMES
// ============================================================================

pub const OP_LOCK: &str = "lock";
pub const OP_UNLOCK: &str = "unlock";
pub const OP_STATUS: &str = "status";
pub const OP_VERIFY: &str = "verify";
pub const OP_ROTATE: &str = "rotate";
pub const OP_BACKUP: &str = "backup";
pub const OP_RESTORE: &str = "restore";
pub const OP_PROXY: &str = "proxy";

// ============================================================================
// FILE OPERATION MESSAGES
// ============================================================================

pub const MSG_FILE_LOCKED: &str = "File successfully encrypted";
pub const MSG_FILE_UNLOCKED: &str = "File successfully decrypted";
pub const MSG_FILE_VERIFIED: &str = "File integrity verified";
pub const MSG_FILE_CORRUPTED: &str = "File appears to be corrupted";
pub const MSG_FILE_SKIPPED: &str = "File skipped";
pub const MSG_FILE_BACKED_UP: &str = "File backed up";
pub const MSG_FILE_RESTORED: &str = "File restored from backup";

// ============================================================================
// ERROR MESSAGES
// ============================================================================

pub const ERR_FILE_NOT_FOUND: &str = "File not found";
pub const ERR_PERMISSION_DENIED: &str = "Permission denied";
pub const ERR_INVALID_PASSPHRASE: &str = "Invalid passphrase";
pub const ERR_ENCRYPTION_FAILED: &str = "Encryption failed";
pub const ERR_DECRYPTION_FAILED: &str = "Decryption failed";
pub const ERR_BACKUP_FAILED: &str = "Backup creation failed";
pub const ERR_RESTORE_FAILED: &str = "Restore operation failed";
pub const ERR_ADAPTER_NOT_FOUND: &str = "Age adapter not found";
pub const ERR_ADAPTER_NOT_IMPLEMENTED: &str = "Adapter not implemented";
pub const ERR_AGE_NOT_INSTALLED: &str = "Age binary not installed or not in PATH";

// ============================================================================
// WARNING MESSAGES
// ============================================================================

pub const WARN_FILE_EXISTS: &str = "File already exists";
pub const WARN_OVERWRITE: &str = "File will be overwritten";
pub const WARN_NO_BACKUP: &str = "No backup available";
pub const WARN_WEAK_PASSPHRASE: &str = "Passphrase is weak";
pub const WARN_NON_UTF8: &str = "File has non-UTF8 name";
pub const WARN_WRONG_EXTENSION: &str = "File has unexpected extension";
pub const WARN_DELETE_FAILED: &str = "Failed to delete encrypted file";

// ============================================================================
// PROMPT MESSAGES
// ============================================================================

pub const PROMPT_PASSPHRASE: &str = "Enter passphrase";
pub const PROMPT_CONFIRM_PASSPHRASE: &str = "Confirm passphrase";
pub const PROMPT_CONTINUE: &str = "Continue?";
pub const PROMPT_OVERWRITE: &str = "Overwrite existing file?";
pub const PROMPT_SELECT_ACTION: &str = "Select action";

// ============================================================================
// STATUS MESSAGES
// ============================================================================

pub const STATUS_PROCESSING: &str = "Processing";
pub const STATUS_COMPLETED: &str = "Completed";
pub const STATUS_FAILED: &str = "Failed";
pub const STATUS_ENCRYPTED: &str = "Encrypted";
pub const STATUS_DECRYPTED: &str = "Decrypted";
pub const STATUS_SCANNING: &str = "Scanning directory";
pub const STATUS_VERIFYING: &str = "Verifying integrity";

// ============================================================================
// CLI HELP TEXT
// ============================================================================

pub const HELP_DESCRIPTION: &str = "Age encryption automation tool with TTY bypass";
pub const HELP_LOCK: &str = "Encrypt files or directories";
pub const HELP_UNLOCK: &str = "Decrypt encrypted files";
pub const HELP_STATUS: &str = "Check encryption status";
pub const HELP_VERIFY: &str = "Verify encrypted file integrity";
pub const HELP_PROXY: &str = "Proxy Age commands with automated TTY";
pub const HELP_ROTATE: &str = "Rotate encryption passphrases";

// ============================================================================
// FORMAT STRINGS (with glyph placeholders)
// ============================================================================

/// Format a success message with appropriate glyph
pub fn fmt_success(msg: &str) -> String {
    format!("{} {}", glyph("pass"), msg)
}

/// Format an error message with appropriate glyph
pub fn fmt_error(msg: &str) -> String {
    format!("{} {}", glyph("cross"), msg)
}

/// Format a warning message with appropriate glyph
pub fn fmt_warning(msg: &str) -> String {
    format!("{} {}", glyph("warn"), msg)
}

/// Format an info message with appropriate glyph
pub fn fmt_info(msg: &str) -> String {
    format!("{} {}", glyph("info"), msg)
}

/// Format a file deletion message
pub fn fmt_deleted(file: &str) -> String {
    format!("{} Deleted encrypted file: {}", glyph("trash"), file)
}

/// Format a file preservation message
pub fn fmt_preserved(file: &str) -> String {
    format!("{} Preserved encrypted file: {}", glyph("folder"), file)
}

/// Format a progress message
pub fn fmt_progress(action: &str, file: &str) -> String {
    format!("{} {} {}", glyph("gear"), action, file)
}

// ============================================================================
// AUDIT LOG MESSAGES
// ============================================================================

pub const AUDIT_LOCK_START: &str = "Lock operation started";
pub const AUDIT_LOCK_SUCCESS: &str = "Lock operation completed successfully";
pub const AUDIT_LOCK_FAILED: &str = "Lock operation failed";
pub const AUDIT_UNLOCK_START: &str = "Unlock operation started";
pub const AUDIT_UNLOCK_SUCCESS: &str = "Unlock operation completed successfully";
pub const AUDIT_UNLOCK_FAILED: &str = "Unlock operation failed";
pub const AUDIT_VERIFY_START: &str = "Verification started";
pub const AUDIT_VERIFY_SUCCESS: &str = "Verification successful";
pub const AUDIT_VERIFY_FAILED: &str = "Verification failed";
pub const AUDIT_BACKUP_CREATED: &str = "Backup created";
pub const AUDIT_BACKUP_RESTORED: &str = "Backup restored";

// ============================================================================
// VALIDATION MESSAGES
// ============================================================================

pub const VAL_PASSPHRASE_TOO_SHORT: &str = "Passphrase must be at least 8 characters";
pub const VAL_PASSPHRASE_MISMATCH: &str = "Passphrases do not match";
pub const VAL_FILE_NOT_ENCRYPTED: &str = "File is not encrypted";
pub const VAL_FILE_ALREADY_ENCRYPTED: &str = "File is already encrypted";
pub const VAL_INVALID_PATTERN: &str = "Invalid pattern filter";

// ============================================================================
// TEST SUPPORT MESSAGES
// ============================================================================

pub const TEST_SKIP_NO_AGE: &str = "SKIPPED: Age binary not found in PATH";
pub const TEST_PASS: &str = "[PASS]";
pub const TEST_FAIL: &str = "[FAIL]";
