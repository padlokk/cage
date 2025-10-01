//! Adapter implementations for age CLI and library backends
//!
//! This module provides adapter patterns for age encryption operations,
//! supporting both CLI-based (shell) adapters and future library-based adapters.
//!
//! # Adapter Versions
//!
//! - **v1**: Original adapter implementation with basic CLI wrapping
//! - **v2**: Enhanced adapter with streaming support and improved error handling
//! - **pipe**: Experimental pipe streaming for passphrase-based encryption
//!
//! # Examples
//!
//! ```rust
//! use cage::adp::{AdapterFactory, AgeAdapter};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let adapter = AdapterFactory::create_default()?;
//! // Use adapter for encryption operations
//! # Ok(())
//! # }
//! ```

pub mod v1;
pub mod v2;
pub mod pipe;

// Re-export primary adapter types
pub use v1::{AgeAdapter, AdapterFactory};
pub use v2::{AgeAdapterV2, ShellAdapterV2, AdapterV1Compat, StreamingStrategy};
