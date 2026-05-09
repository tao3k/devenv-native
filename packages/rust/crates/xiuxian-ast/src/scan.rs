//! Pattern utilities for AST matching.
//!
//! Provides high-level functions for creating patterns and scanning content.

use anyhow::{Context, Result};

use crate::item::Match;
use crate::lang::Lang;
use crate::re_exports::{
    Doc, LanguageExt, MatcherExt, MetaVariable, NodeMatch, Pattern, SupportLang,
};

/// Create a search pattern for a language
///
/// # Errors
/// Returns an error when the language or pattern cannot be parsed.
pub fn pattern(pattern: &str, lang: Lang) -> Result<Pattern> {
    let lang_str = lang.as_str();
    let support_lang: SupportLang = lang_str
        .parse()
        .with_context(|| format!("Failed to parse language: {lang_str}"))?;
    Pattern::try_new(pattern, support_lang)
        .with_context(|| format!("Failed to parse pattern: {pattern}"))
}

/// Scan content and find all matches for a pattern
///
/// # Errors
/// Returns an error when the language or pattern cannot be parsed.
pub fn scan(content: &str, pat: &str, lang: Lang) -> Result<Vec<Match>> {
    let lang_str = lang.as_str();
    let support_lang: SupportLang = lang_str
        .parse()
        .with_context(|| format!("Failed to parse language: {lang_str}"))?;
    scan_with_lang(content, pat, support_lang)
}

/// Extract a single capture value from pattern matches
#[must_use]
pub fn extract(content: &str, pattern: &str, var: &str, lang: Lang) -> Option<String> {
    scan(content, pattern, lang)
        .ok()?
        .into_iter()
        .flat_map(|matched| matched.captures)
        .find_map(|(name, value)| (name == var).then_some(value))
}

/// Scan with `SupportLang` directly.
///
/// # Errors
/// Returns an error when the pattern cannot be parsed.
pub fn scan_with_lang(content: &str, pat: &str, support_lang: SupportLang) -> Result<Vec<Match>> {
    let search_pattern = Pattern::try_new(pat, support_lang)
        .with_context(|| format!("Failed to parse pattern: {pat}"))?;
    Ok(scan_with_pattern(content, support_lang, &search_pattern))
}

fn scan_with_pattern(
    content: &str,
    support_lang: SupportLang,
    search_pattern: &Pattern,
) -> Vec<Match> {
    let grep_result = support_lang.ast_grep(content);
    let root_node = grep_result.root();

    root_node
        .dfs()
        .filter_map(|node| search_pattern.match_node(node.clone()))
        .map(|matched| match_from_node(&matched))
        .collect()
}

fn match_from_node(matched: &NodeMatch<impl Doc>) -> Match {
    Match {
        text: matched.text().to_string(),
        start: matched.range().start,
        end: matched.range().end,
        captures: captures_from_node(matched),
    }
}

fn captures_from_node(matched: &NodeMatch<impl Doc>) -> Vec<(String, String)> {
    let env = matched.get_env();
    env.get_matched_variables()
        .filter_map(capture_name)
        .filter_map(|name| {
            env.get_match(&name)
                .map(|captured| (name.clone(), captured.text().to_string()))
        })
        .collect()
}

fn capture_name(meta_variable: MetaVariable) -> Option<String> {
    match meta_variable {
        MetaVariable::Capture(name, _) | MetaVariable::MultiCapture(name) => Some(name),
        _ => None,
    }
}
