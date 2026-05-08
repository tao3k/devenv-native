//! AST-based code extraction utilities.
//!
//! Provides high-level functions for extracting code elements from source files
//! with precise location information (byte offsets, line numbers) and capture support.

use std::collections::{HashMap, HashSet};

use crate::lang::Lang;
use crate::re_exports::{LanguageExt, MatcherExt, MetaVariable, Pattern, SupportLang};

enum TomlPattern {
    Table {
        name: TomlPatternToken,
    },
    Assignment {
        name: TomlPatternToken,
        value: TomlPatternToken,
    },
}

enum TomlPatternToken {
    Meta(String),
    Literal(String),
}

/// Result of extracting a single code element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractResult {
    /// The matched code text
    pub text: String,
    /// Byte offset start position
    pub start: usize,
    /// Byte offset end position
    pub end: usize,
    /// Line number start (1-indexed)
    pub line_start: usize,
    /// Line number end (1-indexed)
    pub line_end: usize,
    /// Captured variable values (name -> value)
    pub captures: HashMap<String, String>,
}

impl ExtractResult {
    /// Create a new `ExtractResult`.
    #[must_use]
    pub fn new(
        text: String,
        start: usize,
        end: usize,
        line_start: usize,
        line_end: usize,
        captures: HashMap<String, String>,
    ) -> Self {
        Self {
            text,
            start,
            end,
            line_start,
            line_end,
            captures,
        }
    }
}

/// Extract code elements from content using an ast-grep pattern.
///
/// # Arguments
///
/// * `content` - The source code content to search
/// * `pattern` - The ast-grep pattern (e.g., "def $NAME", "class $CLASS")
/// * `lang` - The programming language
/// * `capture_names` - Optional list of capture names to include in results
///
/// # Returns
///
/// A vector of `ExtractResult` containing all matches with location info and captures.
///
/// # Examples
///
/// ```rust
/// use xiuxian_ast::{extract_items, Lang};
///
/// let content = r#"
/// def hello(name: str) -> str:
///     '''Greet someone.'''
///     return f"Hello, {name}!"
///
/// def goodbye():
///     pass
/// "#;
///
/// let results = extract_items(content, "def $NAME", Lang::Python, None);
/// assert_eq!(results.len(), 2);
/// ```
#[must_use]
pub fn extract_items(
    content: &str,
    pattern: &str,
    lang: Lang,
    capture_names: Option<Vec<&str>>,
) -> Vec<ExtractResult> {
    extract_items_for_patterns(content, &[pattern], lang, capture_names)
}

/// Extract code elements from content using multiple ast-grep patterns.
///
/// This parses the source once and applies all valid patterns to the parsed
/// tree. Use it when a caller needs to evaluate a language's full skeleton
/// pattern set over the same file.
#[must_use]
pub fn extract_items_for_patterns(
    content: &str,
    patterns: &[&str],
    lang: Lang,
    capture_names: Option<Vec<&str>>,
) -> Vec<ExtractResult> {
    let capture_names: Option<HashSet<String>> =
        capture_names.map(|v| v.into_iter().map(str::to_string).collect());

    if lang == Lang::Toml {
        return patterns
            .iter()
            .flat_map(|pattern| extract_toml_items(content, pattern, capture_names.as_ref()))
            .collect();
    }

    let lang_str = lang.as_str();
    let support_lang: SupportLang = match lang_str.parse() {
        Ok(l) => l,
        Err(_) => return Vec::new(),
    };

    let grep_result = support_lang.ast_grep(content);
    let root_node = grep_result.root();

    let search_patterns = patterns
        .iter()
        .filter_map(|pattern| Pattern::try_new(pattern, support_lang).ok())
        .collect::<Vec<_>>();
    if search_patterns.is_empty() {
        return Vec::new();
    }

    // Pre-compute line index for fast line number lookup
    let line_offsets: Vec<usize> = content
        .char_indices()
        .filter(|(_, c)| *c == '\n')
        .map(|(i, _)| i)
        .chain(std::iter::once(content.len()))
        .collect();

    let mut results = Vec::new();

    for node in root_node.dfs() {
        for search_pattern in &search_patterns {
            if let Some(m) = search_pattern.match_node(node.clone()) {
                let env = m.get_env();

                // Extract captures based on filter
                let mut captures = HashMap::new();
                for mv in env.get_matched_variables() {
                    let name = match &mv {
                        MetaVariable::Capture(name, _) | MetaVariable::MultiCapture(name) => {
                            name.as_str()
                        }
                        _ => continue,
                    };

                    // Apply capture name filter if provided
                    if let Some(ref filter) = capture_names
                        && !filter.contains(name)
                    {
                        continue;
                    }

                    if let Some(captured) = env.get_match(name) {
                        captures.insert(name.to_string(), captured.text().to_string());
                    }
                }

                // Calculate line numbers from byte offsets
                let start = m.range().start;
                let end = m.range().end;
                let (line_start, line_end) = byte_to_line(start, end, &line_offsets);

                results.push(ExtractResult {
                    text: m.text().to_string(),
                    start,
                    end,
                    line_start,
                    line_end,
                    captures,
                });
            }
        }
    }

    results
}

fn extract_toml_items(
    content: &str,
    pattern: &str,
    capture_names: Option<&HashSet<String>>,
) -> Vec<ExtractResult> {
    let Some(pattern) = parse_toml_pattern(pattern) else {
        return Vec::new();
    };

    let mut results = Vec::new();
    let mut byte_offset = 0;

    for (line_index, raw_line) in content.split_inclusive('\n').enumerate() {
        let trimmed_line = raw_line.trim();
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            byte_offset += raw_line.len();
            continue;
        }

        let Some(mut captures) = match_toml_pattern(&pattern, trimmed_line) else {
            byte_offset += raw_line.len();
            continue;
        };
        if let Some(filter) = capture_names {
            captures.retain(|name, _| filter.contains(name));
        }

        let leading_offset = raw_line.find(trimmed_line).unwrap_or(0);
        let start = byte_offset + leading_offset;
        let end = start + trimmed_line.len();
        let line_number = line_index + 1;

        results.push(ExtractResult::new(
            trimmed_line.to_string(),
            start,
            end,
            line_number,
            line_number,
            captures,
        ));
        byte_offset += raw_line.len();
    }

    results
}

fn parse_toml_pattern(pattern: &str) -> Option<TomlPattern> {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return None;
    }

    if let Some(table_name) = pattern.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        return Some(TomlPattern::Table {
            name: parse_toml_pattern_token(table_name)?,
        });
    }

    let (name, value) = pattern.split_once('=')?;
    Some(TomlPattern::Assignment {
        name: parse_toml_pattern_token(name)?,
        value: parse_toml_pattern_token(value)?,
    })
}

fn parse_toml_pattern_token(token: &str) -> Option<TomlPatternToken> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }

    Some(if let Some(name) = token.strip_prefix('$') {
        TomlPatternToken::Meta(name.to_string())
    } else {
        TomlPatternToken::Literal(token.to_string())
    })
}

fn match_toml_pattern(pattern: &TomlPattern, line: &str) -> Option<HashMap<String, String>> {
    match pattern {
        TomlPattern::Table { name } => {
            let table_name = line
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
                .filter(|value| !value.starts_with('[') && !value.ends_with(']'))?;
            let mut captures = HashMap::new();
            match_toml_token(name, table_name.trim(), &mut captures).then_some(captures)
        }
        TomlPattern::Assignment { name, value } => {
            let (line_name, line_value) = line.split_once('=')?;
            let mut captures = HashMap::new();
            if !match_toml_token(name, line_name.trim(), &mut captures) {
                return None;
            }
            if !match_toml_token(value, line_value.trim(), &mut captures) {
                return None;
            }
            Some(captures)
        }
    }
}

fn match_toml_token(
    token: &TomlPatternToken,
    value: &str,
    captures: &mut HashMap<String, String>,
) -> bool {
    match token {
        TomlPatternToken::Meta(name) => {
            captures.insert(name.clone(), value.to_string());
            true
        }
        TomlPatternToken::Literal(expected) => expected == value,
    }
}

/// Convert byte offsets to line numbers (1-indexed).
fn byte_to_line(byte_start: usize, byte_end: usize, line_offsets: &[usize]) -> (usize, usize) {
    let line_start = line_offsets
        .iter()
        .position(|&offset| offset >= byte_start)
        .map_or(1, |i| i + 1); // Convert to 1-indexed

    let line_end = line_offsets
        .iter()
        .position(|&offset| offset >= byte_end)
        .map_or(line_start, |i| i + 1); // Convert to 1-indexed

    (line_start, line_end)
}

/// Get skeleton patterns for a language.
///
/// These patterns extract only signatures (function/class definitions with docstrings),
/// removing implementation details for lightweight indexing.
#[must_use]
pub fn get_skeleton_patterns(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::Python => &["def $NAME", "class $NAME", "async def $NAME"],
        Lang::Rust => &[
            "fn $NAME",
            "pub fn $NAME",
            "async fn $NAME",
            "pub async fn $NAME",
            "struct $NAME",
            "pub struct $NAME",
            "impl $NAME",
        ],
        Lang::JavaScript | Lang::TypeScript => &[
            "function $NAME",
            "class $NAME",
            "const $NAME = function",
            "const $NAME = (",
        ],
        Lang::Go => &["func $NAME", "type $NAME struct", "type $NAME interface"],
        Lang::Java => &["public $NAME", "class $NAME", "interface $NAME"],
        Lang::C | Lang::Cpp => &["$TYPE $NAME(", "class $NAME", "struct $NAME"],
        Lang::CSharp => &["public $TYPE $NAME", "class $NAME", "interface $NAME"],
        Lang::Ruby => &["def $NAME", "class $NAME"],
        Lang::Swift => &["func $NAME", "class $NAME", "struct $NAME"],
        Lang::Kotlin => &["fun $NAME", "class $NAME", "data class $NAME"],
        Lang::Lua => &["function $NAME", "local $NAME = function"],
        Lang::Php => &["function $NAME", "class $NAME", "public function $NAME"],
        Lang::Bash => &["$NAME()", "function $NAME"],
        Lang::Toml => &["$NAME = $VALUE", "[$NAME]"],
        _ => &["$NAME"],
    }
}

/// Extract skeleton (signatures + docstrings) from source code.
///
/// This function extracts only the structural definitions (function signatures,
/// class definitions) along with their docstrings, removing implementation bodies.
/// It's ideal for lightweight semantic indexing where full code content is not needed.
///
/// # Arguments
///
/// * `content` - The source code content
/// * `lang` - The programming language
///
/// # Returns
///
/// A concatenated string of all skeletons (signatures + docstrings)
///
/// # Examples
///
/// ```rust
/// use xiuxian_ast::{extract_skeleton, Lang};
///
/// let python_code = r#"
/// def hello(name: str) -> str:
///     """Greet someone by name."""
///     return f"Hello, {name}!"
///
/// def goodbye():
///     """Say goodbye."""
///     print("Goodbye")
/// "#;
///
/// let skeleton = extract_skeleton(python_code, Lang::Python);
/// assert!(skeleton.contains("def hello"));
/// ```
#[must_use]
pub fn extract_skeleton(content: &str, lang: Lang) -> String {
    let patterns = get_skeleton_patterns(lang);

    // Extract items for each pattern and combine results
    let mut all_results: Vec<String> = Vec::new();
    for pattern in patterns {
        let results = extract_items(content, pattern, lang, None);
        for result in results {
            if !result.text.is_empty() {
                // Extract just the signature line (before the body starts)
                let signature = extract_signature(&result.text, lang);
                if !signature.is_empty() {
                    all_results.push(signature);
                }
            }
        }
    }

    all_results.join("\n\n")
}

/// Extract just the signature line from a code element, removing the body.
/// For Python: takes everything up to and including the colon on the first line.
/// For Rust/C-like: takes everything up to the first opening brace.
fn extract_signature(text: &str, lang: Lang) -> String {
    let first_line = text.lines().next().unwrap_or(text);

    match lang {
        Lang::Python | Lang::Ruby | Lang::Lua | Lang::Bash => {
            // For Python-like languages, the signature is the entire first line
            first_line.trim().to_string()
        }
        Lang::Rust
        | Lang::C
        | Lang::Cpp
        | Lang::CSharp
        | Lang::Java
        | Lang::Go
        | Lang::Swift
        | Lang::Kotlin
        | Lang::Php
        | Lang::JavaScript
        | Lang::TypeScript => {
            // For C-like languages, truncate at the first '{'
            match first_line.find('{') {
                Some(idx) => first_line[..idx].trim().to_string(),
                None => first_line.trim().to_string(),
            }
        }
        _ => first_line.trim().to_string(),
    }
}
