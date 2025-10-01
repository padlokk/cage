//! Cage Manager Module - Central Coordination for Age Automation
//!
//! This module provides comprehensive coordination for Age encryption operations,
//! consolidating all create/read/update/delete workflows through the `CageManager`.
//! The manager integrates with TTY automation patterns and authority management systems
//! documented in `docs/ref/cage/KEYGEN_STRATEGY.md` and related process guides.
//!
//! Security Guardian: Edgar - Production management framework

pub mod cage_manager;

// Re-export core manager types
pub use cage_manager::{CageManager, LockOptions, UnlockOptions, VerificationResult};
