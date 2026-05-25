//! Intent encoding utilities for self-evolving memory.
//!
//! Provides simple intent embedding encoding for episode similarity search.
//! Uses token-aware feature hashing for quick lexical-semantic recall without
//! external embedding dependencies.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Encode intent text into a fixed-size embedding vector.
///
/// Uses a simple hash-based encoding that maps similar intents to similar vectors.
/// For production, this would be replaced with actual embedding models.
#[derive(Clone)]
pub struct IntentEncoder {
    /// Dimension of the embedding vector
    dimension: usize,
}

impl IntentEncoder {
    /// Create a new encoder with specified dimension.
    #[must_use]
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// Encode intent text into an embedding vector.
    ///
    /// Uses token-aware feature hashing:
    /// 1. Extract normalized ASCII/CJK tokens from the intent.
    /// 2. Hash each token into stable vector buckets.
    /// 3. Add a secondary bucket per token to reduce collision damage.
    /// 4. Normalize the resulting vector.
    #[must_use]
    pub fn encode(&self, intent: &str) -> Vec<f32> {
        let mut embedding = vec![0.0; self.dimension];
        if self.dimension == 0 {
            return embedding;
        }

        let tokens = normalized_intent_tokens(intent);
        if tokens.is_empty() {
            return embedding;
        }

        for token in tokens {
            let primary = token_bucket(token.as_str(), self.dimension, 0);
            let secondary = token_bucket(token.as_str(), self.dimension, 1);
            embedding[primary] += 1.0;
            if secondary != primary {
                embedding[secondary] += 0.5;
            }
        }

        Self::normalize(&embedding)
    }

    /// Normalize vector to unit length.
    fn normalize(v: &[f32]) -> Vec<f32> {
        let sum: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if sum == 0.0 {
            return v.to_vec();
        }
        v.iter().map(|x| x / sum).collect()
    }

    /// Calculate cosine similarity between two embeddings.
    #[must_use]
    pub fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if self.dimension == 0 || a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a * norm_b)
    }

    /// Get the dimension of embeddings.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

impl Default for IntentEncoder {
    fn default() -> Self {
        Self::new(384) // Common embedding dimension
    }
}

fn token_bucket(token: &str, dimension: usize, salt: u64) -> usize {
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    salt.hash(&mut hasher);
    hasher
        .finish()
        .to_le_bytes()
        .into_iter()
        .fold(0usize, |bucket, byte| {
            bucket.wrapping_mul(257).wrapping_add(usize::from(byte))
        })
        % dimension
}

fn normalized_intent_tokens(intent: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut ascii_run = String::new();
    let mut cjk_run = Vec::new();
    for character in intent.chars() {
        if character.is_ascii_alphanumeric() {
            flush_cjk_run(&mut tokens, &mut cjk_run);
            ascii_run.push(character);
        } else if is_cjk_character(character) {
            flush_ascii_run(&mut tokens, &mut ascii_run);
            cjk_run.push(character);
        } else {
            flush_ascii_run(&mut tokens, &mut ascii_run);
            flush_cjk_run(&mut tokens, &mut cjk_run);
        }
    }
    flush_ascii_run(&mut tokens, &mut ascii_run);
    flush_cjk_run(&mut tokens, &mut cjk_run);
    tokens
}

fn flush_ascii_run(tokens: &mut Vec<String>, ascii_run: &mut String) {
    if ascii_run.is_empty() {
        return;
    }
    let run = std::mem::take(ascii_run);
    push_unique_token(tokens, run.to_ascii_lowercase());
    for segment in ascii_semantic_segments(run.as_str()) {
        push_unique_token(tokens, segment);
    }
}

fn ascii_semantic_segments(value: &str) -> Vec<String> {
    let characters = value.chars().collect::<Vec<_>>();
    let mut segments = Vec::new();
    let mut current = String::new();

    for (index, character) in characters.iter().copied().enumerate() {
        if !current.is_empty() && ascii_segment_boundary(&characters, index) {
            push_unique_token(&mut segments, current.to_ascii_lowercase());
            current.clear();
        }
        current.push(character);
    }
    if !current.is_empty() {
        push_unique_token(&mut segments, current.to_ascii_lowercase());
    }
    segments
}

fn ascii_segment_boundary(characters: &[char], index: usize) -> bool {
    if index == 0 {
        return false;
    }
    let previous = characters[index - 1];
    let current = characters[index];
    let next = characters.get(index + 1).copied();
    (previous.is_ascii_lowercase() && current.is_ascii_uppercase())
        || (previous.is_ascii_alphabetic() && current.is_ascii_digit())
        || (previous.is_ascii_digit() && current.is_ascii_alphabetic())
        || (previous.is_ascii_uppercase()
            && current.is_ascii_uppercase()
            && next.is_some_and(|next| next.is_ascii_lowercase()))
}

fn flush_cjk_run(tokens: &mut Vec<String>, cjk_run: &mut Vec<char>) {
    if cjk_run.is_empty() {
        return;
    }
    for character in cjk_run.iter() {
        push_unique_token(tokens, character.to_string());
    }
    for window in cjk_run.windows(2) {
        push_unique_token(tokens, window.iter().collect::<String>());
    }
    cjk_run.clear();
}

fn push_unique_token(tokens: &mut Vec<String>, token: String) {
    if token.chars().count() > 1 && !tokens.iter().any(|existing| existing == &token) {
        tokens.push(token);
    }
}

fn is_cjk_character(character: char) -> bool {
    matches!(
        character,
        '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{20000}'..='\u{2A6DF}'
            | '\u{2A700}'..='\u{2B73F}'
            | '\u{2B740}'..='\u{2B81F}'
            | '\u{2B820}'..='\u{2CEAF}'
    )
}
