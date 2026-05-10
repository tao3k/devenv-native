//! Generic source-code structure analysis helpers.

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use xiuxian_ast::{Lang, extract_items, extract_items_for_patterns, get_skeleton_patterns};

use crate::error::CodeSearchError;
use crate::parser_evidence::normalize_code_language_identifier;
use crate::types::{
    CodeDependencySymbol, CodeLanguageId, CodePatternMatch, CodeSourceFile, CodeStructureSymbol,
    SymbolKind, first_code_signature_line,
};

/// Extract structural pattern matches from source content.
#[must_use]
pub fn extract_code_pattern_matches(
    content: &str,
    pattern: &str,
    lang: Lang,
    capture_names: Option<Vec<&str>>,
) -> Vec<CodePatternMatch> {
    extract_items(content, pattern, lang, capture_names)
        .into_iter()
        .map(|result| CodePatternMatch {
            text: result.text,
            start: result.start,
            end: result.end,
            line_start: result.line_start,
            line_end: result.line_end,
            captures: result.captures.into_iter().collect::<HashMap<_, _>>(),
        })
        .collect()
}

/// Count ast-grep pattern matches in source content.
///
/// # Errors
///
/// Returns a pattern error when the requested language or ast-grep pattern
/// cannot be parsed.
pub fn count_code_pattern_matches(
    content: &str,
    pattern: &str,
    lang: Lang,
) -> Result<usize, CodeSearchError> {
    xiuxian_ast::scan(content, pattern, lang)
        .map(|matches| matches.len())
        .map_err(|error| CodeSearchError::Pattern(error.to_string()))
}

/// Count ast-grep pattern matches using a code-intelligence language id.
///
/// # Errors
///
/// Returns a pattern error when the language id or ast-grep pattern cannot be
/// parsed.
pub fn count_code_pattern_matches_for_language_id(
    content: &str,
    pattern: &str,
    language_id: &CodeLanguageId,
) -> Result<usize, CodeSearchError> {
    let lang = Lang::try_from(language_id.as_str())
        .map_err(|error| CodeSearchError::Pattern(error.to_string()))?;
    count_code_pattern_matches(content, pattern, lang)
}

/// Extract skeleton symbols from source content for a supported AST language.
#[must_use]
pub fn extract_code_structure_symbols(content: &str, lang: Lang) -> Vec<CodeStructureSymbol> {
    extract_items_for_patterns(
        content,
        get_skeleton_patterns(lang),
        lang,
        Some(vec!["NAME"]),
    )
    .into_iter()
    .filter_map(|result| {
        let signature = first_code_signature_line(result.text.as_str()).to_string();
        if signature.is_empty() {
            return None;
        }
        let name = result
            .captures
            .get("NAME")
            .cloned()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| signature.clone());
        Some(CodeStructureSymbol {
            name,
            signature,
            line_start: result.line_start,
            line_end: result.line_end,
            captures: result.captures.into_iter().collect::<HashMap<_, _>>(),
        })
    })
    .collect()
}

/// Extract skeleton symbols using a code-intelligence language id.
#[must_use]
pub fn extract_code_structure_symbols_for_language_id(
    content: &str,
    language_id: &CodeLanguageId,
) -> Vec<CodeStructureSymbol> {
    Lang::try_from(language_id.as_str()).map_or_else(
        |_| Vec::new(),
        |lang| extract_code_structure_symbols(content, lang),
    )
}

/// Extract source symbols for dependency indexing.
///
/// This intentionally keeps package/crate ownership out of the code-intelligence
/// layer. Owner crates can map these neutral symbols into their own persisted
/// indexes.
#[must_use]
pub fn extract_code_dependency_symbols(
    content: &str,
    language_id: &CodeLanguageId,
) -> Vec<CodeDependencySymbol> {
    match language_id.as_str() {
        "rust" => extract_rust_dependency_symbols(content),
        "python" => extract_python_dependency_symbols(content),
        _ => Vec::new(),
    }
}

/// Resolve shallow source files for a specific AST language.
#[must_use]
pub fn resolve_code_source_files(paths: &[&Path], lang: Lang) -> Vec<CodeSourceFile> {
    let extensions = lang.extensions();
    paths
        .iter()
        .flat_map(|path| resolve_code_source_path(path, extensions.as_slice()))
        .collect()
}

/// Resolve shallow source files for a code-intelligence language id.
#[must_use]
pub fn resolve_code_source_files_for_language_id(
    paths: &[&Path],
    language_id: &CodeLanguageId,
) -> Vec<CodeSourceFile> {
    Lang::try_from(language_id.as_str()).map_or_else(
        |_| Vec::new(),
        |lang| resolve_code_source_files(paths, lang),
    )
}

/// Resolve a generic semantic-fingerprint language id from a source path.
#[must_use]
pub fn code_semantic_fingerprint_language_id_from_path(path: &Path) -> Option<CodeLanguageId> {
    let lang = Lang::from_path(path)?;
    xiuxian_ast::supports_semantic_fingerprint(lang).then(|| CodeLanguageId::from(lang.as_str()))
}

/// Resolve a generic semantic-fingerprint language id from a language or parser id.
#[must_use]
pub fn code_semantic_fingerprint_language_id_from_identifier(
    identifier: &str,
) -> Option<CodeLanguageId> {
    let normalized = normalize_code_language_identifier(identifier);
    let lang = Lang::try_from(normalized.as_str()).ok()?;
    xiuxian_ast::supports_semantic_fingerprint(lang).then(|| CodeLanguageId::from(lang.as_str()))
}

/// Return whether the language has a generic structural semantic fingerprint.
#[must_use]
pub fn supports_code_semantic_fingerprint(language_id: &CodeLanguageId) -> bool {
    Lang::try_from(language_id.as_str())
        .ok()
        .is_some_and(xiuxian_ast::supports_semantic_fingerprint)
}

/// Build a generic structural semantic fingerprint for source code.
#[must_use]
pub fn code_semantic_fingerprint(content: &str, language_id: &CodeLanguageId) -> Option<String> {
    let lang = Lang::try_from(language_id.as_str()).ok()?;
    xiuxian_ast::semantic_fingerprint(content, lang)
}

fn resolve_code_source_path(path: &Path, extensions: &[&str]) -> Vec<CodeSourceFile> {
    if path.is_file() {
        return read_matching_source_file(path, extensions)
            .into_iter()
            .collect();
    }
    if path.is_dir() {
        return std::fs::read_dir(path)
            .into_iter()
            .flat_map(std::iter::Iterator::flatten)
            .filter_map(|entry| read_matching_source_file(entry.path().as_path(), extensions))
            .collect();
    }
    Vec::new()
}

fn read_matching_source_file(path: &Path, extensions: &[&str]) -> Option<CodeSourceFile> {
    if !path.is_file() {
        return None;
    }
    let extension = path.extension().and_then(|extension| extension.to_str())?;
    if !extensions.contains(&extension) {
        return None;
    }
    std::fs::read_to_string(path)
        .ok()
        .map(|content| CodeSourceFile {
            path: path.display().to_string(),
            content,
        })
}

fn compile_regex(pattern: &str) -> regex::Regex {
    match regex::Regex::new(pattern) {
        Ok(regex) => regex,
        Err(_pattern_err) => match regex::Regex::new(r"$^") {
            Ok(fallback) => fallback,
            Err(fallback_err) => panic!("hardcoded fallback regex must compile: {fallback_err}"),
        },
    }
}

static RUST_DEPENDENCY_PATTERNS: LazyLock<Vec<(SymbolKind, regex::Regex)>> = LazyLock::new(|| {
    vec![
        (
            SymbolKind::Struct,
            compile_regex(r"(?:pub\s+)?struct\s+(\w+)"),
        ),
        (SymbolKind::Enum, compile_regex(r"(?:pub\s+)?enum\s+(\w+)")),
        (
            SymbolKind::Trait,
            compile_regex(r"(?:pub\s+)?trait\s+(\w+)"),
        ),
        (
            SymbolKind::Function,
            compile_regex(r"(?:pub\s+)?fn\s+(\w+)"),
        ),
        (SymbolKind::Impl, compile_regex(r"impl\s+(\w+)")),
        (SymbolKind::Module, compile_regex(r"(?:pub\s+)?mod\s+(\w+)")),
        (
            SymbolKind::TypeAlias,
            compile_regex(r"(?:pub\s+)?type\s+(\w+)"),
        ),
        (
            SymbolKind::Const,
            compile_regex(r"(?:pub\s+)?const\s+(\w+)"),
        ),
        (
            SymbolKind::Static,
            compile_regex(r"(?:pub\s+)?static\s+(\w+)"),
        ),
    ]
});

static PYTHON_DEPENDENCY_PATTERNS: LazyLock<Vec<(SymbolKind, regex::Regex)>> =
    LazyLock::new(|| {
        vec![
            (SymbolKind::Struct, compile_regex(r"class\s+(\w+)")),
            (SymbolKind::Function, compile_regex(r"async\s+def\s+(\w+)")),
            (SymbolKind::Function, compile_regex(r"def\s+(\w+)")),
        ]
    });

fn extract_rust_dependency_symbols(content: &str) -> Vec<CodeDependencySymbol> {
    extract_dependency_symbols_with_patterns(content, RUST_DEPENDENCY_PATTERNS.as_slice())
}

fn extract_python_dependency_symbols(content: &str) -> Vec<CodeDependencySymbol> {
    extract_dependency_symbols_with_patterns(content, PYTHON_DEPENDENCY_PATTERNS.as_slice())
}

fn extract_dependency_symbols_with_patterns(
    content: &str,
    patterns: &[(SymbolKind, regex::Regex)],
) -> Vec<CodeDependencySymbol> {
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            patterns.iter().find_map(|(kind, regex)| {
                regex.captures(line).map(|captures| CodeDependencySymbol {
                    name: captures[1].to_string(),
                    kind: kind.clone(),
                    line: index + 1,
                })
            })
        })
        .collect()
}
