use std::collections::HashMap;

use super::language::Lang;

pub(super) struct CodePatternMatch {
    pub(super) captures: HashMap<String, String>,
    pub(super) text: String,
    pub(super) line_start: usize,
    pub(super) line_end: usize,
}

pub(super) struct CodeStructureSymbol {
    pub(super) name: String,
    pub(super) signature: String,
    pub(super) line_start: usize,
    pub(super) line_end: usize,
}

pub(super) fn extract_code_pattern_matches(
    _content: &str,
    _pattern: &str,
    _lang: Lang,
    _limit: Option<usize>,
) -> Vec<CodePatternMatch> {
    Vec::new()
}

pub(super) fn extract_code_structure_symbols(
    _content: &str,
    _lang: Lang,
) -> Vec<CodeStructureSymbol> {
    Vec::new()
}

pub(super) fn score_code_structure_query(
    search_term: Option<&str>,
    relative_path: &str,
    name: &str,
    signature: &str,
) -> Option<f64> {
    let Some(query) = search_term
        .map(str::trim)
        .filter(|query| !query.is_empty())
        .map(str::to_ascii_lowercase)
    else {
        return Some(0.5);
    };
    let name = name.to_ascii_lowercase();
    let signature = signature.to_ascii_lowercase();
    let relative_path = relative_path.to_ascii_lowercase();
    if name == query {
        Some(1.0)
    } else if name.contains(query.as_str()) {
        Some(0.88)
    } else if signature.contains(query.as_str()) {
        Some(0.82)
    } else if relative_path.contains(query.as_str()) {
        Some(0.66)
    } else {
        None
    }
}
