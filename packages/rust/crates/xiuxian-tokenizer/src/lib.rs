//! xiuxian-tokenizer - High-performance token counting and truncation.
//!
//! Uses `tiktoken-rs` for BPE tokenization compatible with common `OpenAI`
//! tokenizer families.
//!
//! # Example
//!
//! ```rust,ignore
//! use xiuxian_tokenizer::{count_tokens, truncate};
//!
//! let text = "Hello, world!";
//! let count = count_tokens(text);
//! let truncated = truncate(text, 5);
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

mod core;
/// Token pruning utilities for context window management.
pub mod pruner;

pub use core::{
    TokenCounter, TokenizerError, chunk_text, count_tokens, count_tokens_with_model,
    get_encoding_name, truncate, truncate_with_model,
};
pub use pruner::{ContextPruner, Message};
