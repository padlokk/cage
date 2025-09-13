//! Lifecycle Management Module - CRUD Coordination for Age Automation
//!
//! This module provides comprehensive lifecycle management for Age encryption operations,
//! coordinating all CRUD operations through a central CrudManager that integrates with
//! the proven TTY automation patterns and authority management systems.
//!
//! Security Guardian: Edgar - Production lifecycle management framework

pub mod crud_manager;

// Re-export core lifecycle types
pub use crud_manager::{CrudManager, LockOptions, UnlockOptions, VerificationResult};