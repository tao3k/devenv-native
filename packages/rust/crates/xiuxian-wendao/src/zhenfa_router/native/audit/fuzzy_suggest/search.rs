//! `zhenfa_router::native::audit::fuzzy_suggest::search` owns Wendao audit fuzzy suggest search behavior.

use super::cache::{
    CONFIDENCE_THRESHOLD, hash_content, lookup_cached_candidates, store_cached_candidates,
};
use super::pattern::PatternSkeleton;
use super::similarity::{calculate_skeleton_similarity, string_similarity};
use super::types::{CandidateMatch, FuzzySuggestion, SourceFile};
use xiuxian_code_intelligence::{
    CodeLanguageId, code_pattern_signature_line_for_language_id,
    extract_code_structure_symbols_for_language_id,
};

/// Search for similar patterns when validation fails.
///
/// # Arguments
///
/// * `original_pattern` - The pattern that failed validation
/// * `language_id` - Target code-intelligence language id for the pattern
/// * `source_files` - Source files to search for candidates
/// * `threshold` - Optional confidence threshold (uses `CONFIDENCE_THRESHOLD` if None)
///
/// # Returns
///
/// `Some(FuzzySuggestion)` if a good match is found, `None` otherwise.
#[must_use]
pub fn suggest_pattern_fix(
    original_pattern: &str,
    language_id: &CodeLanguageId,
    source_files: &[SourceFile],
) -> Option<FuzzySuggestion> {
    suggest_pattern_fix_with_threshold(original_pattern, language_id, source_files, None)
}

/// Search for similar patterns with a custom confidence threshold.
///
/// # Arguments
///
/// * `original_pattern` - The pattern that failed validation
/// * `language_id` - Target code-intelligence language id for the pattern
/// * `source_files` - Source files to search for candidates
/// * `threshold` - Optional confidence threshold (uses `CONFIDENCE_THRESHOLD` if None)
///
/// # Returns
///
/// `Some(FuzzySuggestion)` if a good match is found, `None` otherwise.
#[must_use]
pub fn suggest_pattern_fix_with_threshold(
    original_pattern: &str,
    language_id: &CodeLanguageId,
    source_files: &[SourceFile],
    threshold: Option<f32>,
) -> Option<FuzzySuggestion> {
    let effective_threshold = threshold.unwrap_or(CONFIDENCE_THRESHOLD);
    if source_files.is_empty() {
        return None;
    }
    let original_skeleton = PatternSkeleton::extract(original_pattern);
    let candidates = scan_source_files_for_candidates(source_files, language_id);
    let (confidence, best_match) =
        best_scored_candidate(&candidates, &original_skeleton, effective_threshold)?;
    Some(fuzzy_suggestion_from_match(
        original_pattern,
        language_id,
        best_match,
        confidence,
    ))
}

fn scan_source_files_for_candidates(
    source_files: &[SourceFile],
    language_id: &CodeLanguageId,
) -> Vec<CandidateMatch> {
    source_files
        .iter()
        .flat_map(|source_file| {
            scan_for_candidates(&source_file.content, language_id, &source_file.path)
        })
        .collect()
}

fn best_scored_candidate<'a>(
    candidates: &'a [CandidateMatch],
    original_skeleton: &PatternSkeleton,
    threshold: f32,
) -> Option<(f32, &'a CandidateMatch)> {
    candidates
        .iter()
        .filter_map(|candidate| scored_candidate(candidate, original_skeleton, threshold))
        .max_by(|left, right| {
            left.0
                .partial_cmp(&right.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn scored_candidate<'a>(
    candidate: &'a CandidateMatch,
    original_skeleton: &PatternSkeleton,
    threshold: f32,
) -> Option<(f32, &'a CandidateMatch)> {
    let total_score = calculate_skeleton_similarity(original_skeleton, &candidate.skeleton)
        + identifier_similarity_bonus(original_skeleton, candidate);
    (total_score >= threshold).then_some((total_score, candidate))
}

fn identifier_similarity_bonus(
    original_skeleton: &PatternSkeleton,
    candidate: &CandidateMatch,
) -> f32 {
    let (Some(original_identifier), Some(candidate_identifier)) = (
        original_skeleton.identifiers.first(),
        candidate.identifier.as_ref(),
    ) else {
        return 0.0;
    };
    let similarity = string_similarity(original_identifier, candidate_identifier);
    if similarity > 0.2 {
        return similarity * 0.4;
    }
    0.0
}

fn fuzzy_suggestion_from_match(
    original_pattern: &str,
    language_id: &CodeLanguageId,
    best_match: &CandidateMatch,
    confidence: f32,
) -> FuzzySuggestion {
    let suggested_pattern = generate_suggested_pattern(original_pattern, best_match, language_id);
    let replacement_drawer = format!(
        ":OBSERVE: lang:{} \"{}\"",
        language_id.as_str(),
        suggested_pattern.replace('"', "\\\"")
    );
    let source_location = Some(format!(
        "{}:{}",
        best_match.file_path, best_match.line_number
    ));
    FuzzySuggestion {
        suggested_pattern,
        confidence,
        source_location,
        replacement_drawer,
    }
}

/// Scan source content for candidate code patterns.
/// Uses caching to avoid re-scanning unchanged files.
fn scan_for_candidates(
    content: &str,
    language_id: &CodeLanguageId,
    file_path: &str,
) -> Vec<CandidateMatch> {
    let content_hash = hash_content(content);

    // Check cache first
    if let Some(candidates) = lookup_cached_candidates(file_path, content_hash) {
        return candidates;
    }

    let candidates = extract_code_structure_symbols_for_language_id(content, language_id)
        .into_iter()
        .map(|symbol| {
            let signature =
                code_pattern_signature_line_for_language_id(symbol.signature.as_str(), language_id);
            let identifier = symbol
                .captures
                .get("NAME")
                .cloned()
                .or_else(|| symbol.captures.values().next().cloned());
            let skeleton = PatternSkeleton::extract(&signature);

            CandidateMatch {
                matched_text: signature,
                file_path: file_path.to_string(),
                line_number: symbol.line_start,
                identifier,
                skeleton,
            }
        })
        .collect::<Vec<_>>();

    // Store in cache for future lookups
    store_cached_candidates(file_path, content_hash, candidates.clone());

    candidates
}

/// Generate a suggested pattern from the original pattern and best match.
fn generate_suggested_pattern(
    _original_pattern: &str,
    best_match: &CandidateMatch,
    language_id: &CodeLanguageId,
) -> String {
    // Extract the signature line from the match
    let signature =
        code_pattern_signature_line_for_language_id(&best_match.matched_text, language_id);

    // Convert to a pattern by replacing identifiers with metavariables
    patternize_signature(&signature, best_match.identifier.as_ref())
}

/// Convert a signature to a pattern by adding metavariables.
fn patternize_signature(signature: &str, identifier: Option<&String>) -> String {
    let mut pattern = signature.to_string();

    // If we have an identifier, keep it concrete but add wildcards for arguments
    if let Some(id) = identifier {
        // For function patterns, replace argument lists with wildcards
        if signature.contains(id) {
            // Pattern already has identifier - just ensure wildcards
            if signature.contains('(') && signature.contains(')') {
                // Keep the structure, add $$$ for body if not present
                if !pattern.contains("$$$") {
                    pattern = format!("{} $$$", pattern.trim());
                }
            }
        }
    }

    // Clean up multiple spaces
    while pattern.contains("  ") {
        pattern = pattern.replace("  ", " ");
    }

    pattern.trim().to_string()
}
