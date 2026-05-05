//! Vision OCR sensitive-text scrubbing.

use super::anchor::TextAnchor;

/// Visibility scrubbing policy for OCR/text anchors.
#[derive(Debug, Clone, PartialEq)]
pub struct VisibilityScrubPolicy {
    /// Minimum confidence required for a kept anchor.
    pub min_confidence: f32,
    /// Minimum normalized text length.
    pub min_text_len: usize,
    /// Maximum normalized text length.
    pub max_text_len: usize,
}

impl Default for VisibilityScrubPolicy {
    fn default() -> Self {
        Self {
            min_confidence: 0.30,
            min_text_len: 2,
            max_text_len: 256,
        }
    }
}

/// Scrubs OCR anchors by dropping low-quality and noisy entries.
#[must_use]
pub fn scrub_text_anchors(
    anchors: impl IntoIterator<Item = TextAnchor>,
    policy: &VisibilityScrubPolicy,
) -> Vec<TextAnchor> {
    anchors
        .into_iter()
        .filter(|anchor| anchor.confidence > policy.min_confidence)
        .map(normalize_anchor_text)
        .filter(|anchor| {
            let text = anchor.text.as_ref();
            let text_len = text.chars().count();
            text_len >= policy.min_text_len
                && text_len <= policy.max_text_len
                && !is_noise_text(text)
                && contains_visible_alnum(text)
        })
        .collect()
}

fn normalize_anchor_text(mut anchor: TextAnchor) -> TextAnchor {
    let normalized = anchor.text.trim().replace('\n', " ");
    anchor.text = normalized.into();
    anchor
}

fn is_noise_text(text: &str) -> bool {
    const NOISE_TOKENS: &[&str] = &[
        "...", "…", "---", "___", "||", "•", "·", ".", ",", "|", "-", "_",
    ];
    let normalized = text.trim();
    if NOISE_TOKENS.contains(&normalized) {
        return true;
    }
    normalized
        .chars()
        .all(|ch| ch.is_ascii_punctuation() || ch.is_whitespace())
}

fn contains_visible_alnum(text: &str) -> bool {
    text.chars().any(char::is_alphanumeric)
}
