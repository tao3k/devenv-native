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

mod core;
/// Token pruning utilities for context window management.
pub mod pruner;

pub use core::{
    TokenCounter, TokenizerError, chunk_text, count_tokens, count_tokens_with_model,
    get_encoding_name, truncate, truncate_with_model,
};
pub use pruner::{ContextPruner, Message};
