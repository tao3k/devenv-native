//! Core token counting, truncation, and chunking operations.

use std::sync::OnceLock;

use thiserror::Error;

/// A helper struct for counting tokens.
#[derive(Debug, Clone)]
pub struct TokenCounter;

impl TokenCounter {
    /// Create a new `TokenCounter` instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Count the number of tokens in a text string.
    #[must_use]
    pub fn count_tokens(text: &str) -> usize {
        count_tokens(text)
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors for tokenization operations.
#[derive(Error, Debug)]
pub enum TokenizerError {
    /// Failed to initialize the tokenization model.
    #[error("Tokenization model initialization failed: {0}")]
    ModelInit(String),
    /// Failed to encode text to tokens.
    #[error("Token encoding failed: {0}")]
    Encoding(String),
    /// Failed to decode tokens back to text.
    #[error("Token decoding failed: {0}")]
    Decoding(String),
}

static CL100K_BASE: OnceLock<tiktoken_rs::CoreBPE> = OnceLock::new();

fn get_cl100k_base() -> &'static tiktoken_rs::CoreBPE {
    CL100K_BASE.get_or_init(|| {
        tiktoken_rs::cl100k_base()
            .unwrap_or_else(|error| panic!("Failed to initialize cl100k_base: {error}"))
    })
}

/// Count tokens in text using `cl100k_base`.
#[must_use]
pub fn count_tokens(text: &str) -> usize {
    get_cl100k_base().encode_with_special_tokens(text).len()
}

/// Count tokens using a specific model.
///
/// # Errors
///
/// Returns [`TokenizerError`] when model initialization or encoding fails.
pub fn count_tokens_with_model(text: &str, model: &str) -> Result<usize, TokenizerError> {
    let bpe = match model {
        "cl100k_base" => tiktoken_rs::cl100k_base(),
        "p50k_base" => tiktoken_rs::p50k_base(),
        "r50k_base" => tiktoken_rs::r50k_base(),
        _ => return Err(TokenizerError::ModelInit(model.to_string())),
    };

    bpe.map(|bpe| bpe.encode_with_special_tokens(text).len())
        .map_err(|error| TokenizerError::Encoding(error.to_string()))
}

/// Truncate text to fit within a maximum token count.
#[must_use]
pub fn truncate(text: &str, max_tokens: usize) -> String {
    let bpe = get_cl100k_base();

    let tokens = bpe.encode_with_special_tokens(text);
    let token_count = tokens.len();

    if token_count <= max_tokens {
        return text.to_string();
    }

    let truncated = tokens.into_iter().take(max_tokens).collect();
    bpe.decode(truncated)
        .unwrap_or_else(|_| estimate_truncate(text, max_tokens))
}

/// Truncate using a specific model.
///
/// # Errors
///
/// Returns [`TokenizerError`] when model initialization or decoding fails.
pub fn truncate_with_model(
    text: &str,
    max_tokens: usize,
    model: &str,
) -> Result<String, TokenizerError> {
    let bpe = match model {
        "cl100k_base" => tiktoken_rs::cl100k_base(),
        "p50k_base" => tiktoken_rs::p50k_base(),
        "r50k_base" => tiktoken_rs::r50k_base(),
        _ => return Err(TokenizerError::ModelInit(model.to_string())),
    };

    let bpe = bpe.map_err(|error| TokenizerError::ModelInit(error.to_string()))?;
    let tokens = bpe.encode_with_special_tokens(text);

    if tokens.len() <= max_tokens {
        return Ok(text.to_string());
    }

    let truncated = tokens.into_iter().take(max_tokens).collect();
    bpe.decode(truncated)
        .map_err(|error| TokenizerError::Decoding(error.to_string()))
}

fn estimate_truncate(text: &str, max_tokens: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let target_words = std::cmp::min(max_tokens * 2, words.len());
    words[..target_words].join(" ")
}

/// Get the encoding name for a model.
#[must_use]
pub fn get_encoding_name(model: &str) -> &'static str {
    match model {
        "gpt-3" | "code-davinci-002" | "p50k_base" => "p50k_base",
        "gpt-2" | "r50k_base" => "r50k_base",
        _ => "cl100k_base",
    }
}

/// Chunk text by token boundaries with overlap.
#[must_use]
pub fn chunk_text(
    text: &str,
    chunk_size_tokens: usize,
    overlap_tokens: usize,
) -> Vec<(String, u32)> {
    if text.is_empty() {
        return vec![];
    }
    let chunk_size = chunk_size_tokens.max(1);
    let overlap = overlap_tokens.min(chunk_size.saturating_sub(1));

    let bpe = get_cl100k_base();
    let tokens = bpe.encode_with_special_tokens(text);
    let n = tokens.len();

    if n <= chunk_size {
        return vec![(text.to_string(), 0)];
    }

    let step = chunk_size.saturating_sub(overlap).max(1);
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut chunk_index = 0u32;

    while start < n {
        let end = (start + chunk_size).min(n);
        let slice = tokens[start..end].to_vec();
        let decoded = bpe
            .decode(slice)
            .unwrap_or_else(|_| text.get(..).unwrap_or("").to_string());
        out.push((decoded, chunk_index));
        chunk_index += 1;
        if end >= n {
            break;
        }
        start += step;
    }

    out
}
