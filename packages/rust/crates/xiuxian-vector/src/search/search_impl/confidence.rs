use serde_json::Value;

use crate::skill;

/// Multiplier for keyword match boost.
pub(super) const KEYWORD_BOOST: f32 = 0.1;

#[derive(Debug, Clone, Copy)]
pub(super) struct ConfidenceProfile {
    high_threshold: f32,
    medium_threshold: f32,
    high_base: f32,
    high_scale: f32,
    high_cap: f32,
    medium_base: f32,
    medium_scale: f32,
    medium_cap: f32,
    low_floor: f32,
}

impl Default for ConfidenceProfile {
    fn default() -> Self {
        Self {
            high_threshold: 0.75,
            medium_threshold: 0.50,
            high_base: 0.90,
            high_scale: 0.05,
            high_cap: 0.99,
            medium_base: 0.60,
            medium_scale: 0.30,
            medium_cap: 0.89,
            low_floor: 0.10,
        }
    }
}

const CLEAR_WINNER_GAP: f32 = 0.15;
const MIN_KEYWORD_SCORE_ATTRIBUTE_HIGH: f32 = 0.2;
const MIN_VECTOR_SCORE_TOOL_DESCRIPTION_HIGH: f32 = 0.55;

fn calibrate_confidence(score: f32, profile: &ConfidenceProfile) -> (&'static str, f32) {
    if score >= profile.high_threshold {
        (
            "high",
            (profile.high_base + score * profile.high_scale).min(profile.high_cap),
        )
    } else if score >= profile.medium_threshold {
        (
            "medium",
            (profile.medium_base + score * profile.medium_scale).min(profile.medium_cap),
        )
    } else {
        ("low", score.max(profile.low_floor))
    }
}

pub(super) fn calibrate_confidence_with_attributes(
    score: f32,
    second_score: Option<f32>,
    vector_score: Option<f32>,
    keyword_score: Option<f32>,
    profile: &ConfidenceProfile,
) -> (&'static str, f32) {
    let (mut confidence, mut final_score) = calibrate_confidence(score, profile);

    if let Some(second) = second_score
        && score >= profile.medium_threshold
        && (score - second) >= CLEAR_WINNER_GAP
    {
        confidence = "high";
        final_score = (profile.high_base + score * profile.high_scale).min(profile.high_cap);
    }

    let kw = keyword_score.unwrap_or(0.0);
    let vec = vector_score.unwrap_or(0.0);

    if confidence != "high"
        && score >= profile.medium_threshold
        && vec >= MIN_VECTOR_SCORE_TOOL_DESCRIPTION_HIGH
    {
        confidence = "high";
        final_score = (profile.high_base + score * profile.high_scale).min(profile.high_cap);
    }

    if confidence != "high"
        && score >= profile.medium_threshold
        && (kw >= MIN_KEYWORD_SCORE_ATTRIBUTE_HIGH || (kw > 0.0 && vec < 0.5 && kw > vec))
    {
        confidence = "high";
        final_score = (profile.high_base + score * profile.high_scale).min(profile.high_cap);
    }

    (confidence, final_score.clamp(0.0, 1.0))
}

fn canonicalize_json_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries = map.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(key, _)| *key);
            let mut out = serde_json::Map::with_capacity(entries.len());
            for (key, child) in entries {
                out.insert(key.clone(), canonicalize_json_value(child));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_json_value).collect()),
        _ => value.clone(),
    }
}

pub(super) fn input_schema_digest(input_schema: &Value) -> String {
    let normalized = skill::normalize_input_schema_value(input_schema);
    if normalized.as_object().is_none_or(serde_json::Map::is_empty) {
        return "fnv1a64:empty".to_string();
    }
    let canonical = canonicalize_json_value(&normalized);
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for byte in canonical.to_string().bytes() {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x0100_0000_01b3);
    }
    format!("fnv1a64:{digest:016x}")
}

pub(super) fn build_ranking_reason(
    result: &crate::skill::ToolSearchResult,
    raw_score: f32,
    final_score: f32,
    confidence: &str,
) -> String {
    let mut parts = Vec::new();
    if let Some(vector_score) = result.vector_score {
        parts.push(format!("vector={vector_score:.3}"));
    }
    if let Some(keyword_score) = result.keyword_score {
        parts.push(format!("keyword={keyword_score:.3}"));
    }
    if !result.category.is_empty() {
        parts.push(format!("category={}", result.category));
    }
    if !result.intents.is_empty() {
        let top_intents = result.intents.iter().take(3).cloned().collect::<Vec<_>>();
        parts.push(format!("intents={}", top_intents.join(",")));
    }
    parts.push(format!("confidence={confidence}"));
    parts.push(format!("raw={raw_score:.3}"));
    parts.push(format!("final={final_score:.3}"));
    parts.join(" | ")
}
