#![allow(clippy::doc_markdown)]

//! omni-io - Safe and efficient file I/O for Omni DevEnv
//!
//! A high-performance file reading library with safety features.
//!
//! # Features
//!
//! - **Dual API**: Sync (std::fs) for Python bindings, Async (tokio::fs) for Rust core
//! - **Safety**: Binary detection & Size limits
//! - **Lightweight**: Zero-dependency UTF-8 fallback
//!
//! # Architecture (ODF-REP Compliant)
//!
//! ```text
//! omni-io/src/
//! ├── lib.rs      # Re-exports (this file)
//! ├── error.rs    # IoError enum
//! ├── detect.rs   # Binary detection & decoding
//! ├── sync.rs     # Synchronous API
//! └── async_io.rs # Asynchronous API (Tokio)
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use omni_io::{read_text_safe, read_text_safe_async, IoError};
//!
//! // Synchronous (for Python bindings)
//! let content = read_text_safe("file.txt", 1024 * 1024)?;
//!
//! // Asynchronous (for Rust core)
//! let content = read_text_safe_async("file.txt", 1024 * 1024).await?;
//! ```

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for cargo-test verification"),
        )
    }
);

// ============================================================================
// Module Declarations (ODF-REP: Atomic Structure)
// ============================================================================

#[cfg(feature = "assembler")]
mod assembler;
mod async_io;
mod detect;
mod dirs;
mod discover;
mod error;
mod sync;
#[cfg(feature = "notify")]
mod watcher;

// ============================================================================
// Public Re-exports
// ============================================================================

#[cfg(feature = "assembler")]
pub use assembler::{AssemblyResult, ContextAssembler};
pub use async_io::read_text_safe_async;
pub use dirs::{PrjDirs, get_cache_home, get_config_home, get_data_home};
pub use discover::{
    DiscoverOptions, count_files_in_dir, discover_files, discover_files_in_dir, should_skip_path,
};
pub use error::{IoError, Result};
pub use sync::read_text_safe;
#[cfg(feature = "notify")]
pub use watcher::{
    FileEvent, FileWatcherHandle, WatcherConfig, WatcherResult, start_file_watcher, watch_path,
};

// Re-export detection utilities for advanced use
pub use detect::{decode_buffer, is_binary};
