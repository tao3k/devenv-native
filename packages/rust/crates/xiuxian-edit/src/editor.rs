//! Core structural editor implementation.
//!
//! Provides AST-based code modification using ast-grep patterns.
//! Part of The Surgeon.

use std::fmt::Write as _;
use std::path::Path;
use std::str::FromStr;

// Use omni-ast for unified ast-grep (re-exports Pattern, SupportLang, LanguageExt)
use xiuxian_ast::{AstLanguage, LanguageExt, MatcherExt, Pattern, SupportLang};

use crate::capture::substitute_captures;
use crate::diff::generate_unified_diff;
use crate::error::EditError;
use crate::types::{EditConfig, EditLocation, EditResult};

struct ReplacementMatch {
    start: usize,
    end: usize,
    original_text: String,
    new_text: String,
}

/// `StructuralEditor` - AST-based code modification engine.
///
/// Uses ast-grep patterns for surgical precision in code refactoring.
/// Part of The Surgeon.
///
/// # Example
///
/// ```rust,ignore
/// use omni_edit::StructuralEditor;
///
/// // Rename function calls (use $$$ for variadic args)
/// let result = StructuralEditor::replace(
///     "x = connect(host, port)",
///     "connect($$$)",
///     "async_connect($$$)",
///     "python"
/// )?;
/// assert!(result.modified.contains("async_connect"));
/// ```
pub struct StructuralEditor;

/// Named request for structural replacement on a file.
#[derive(Clone, Copy)]
pub struct ReplaceInFileRequest<'a> {
    /// Path to the source file.
    pub path: &'a Path,
    /// `ast-grep` pattern to match.
    pub pattern: &'a str,
    /// Replacement pattern.
    pub replacement: &'a str,
    /// Optional language hint. When omitted, the language is inferred from the path.
    pub language: Option<&'a str>,
    /// Edit configuration controlling preview and size limits.
    pub config: &'a EditConfig,
}

impl StructuralEditor {
    /// Perform structural replace on content.
    ///
    /// # Arguments
    /// * `content` - Source code content
    /// * `pattern` - ast-grep pattern to match (e.g., `connect($$$)`)
    /// * `replacement` - Replacement pattern (e.g., `async_connect($$$)`)
    /// * `language` - Programming language (python, rust, javascript, typescript)
    ///
    /// # Returns
    /// `EditResult` containing original, modified content, diff, and edit locations.
    ///
    /// # Errors
    ///
    /// Returns [`EditError`] when the language or pattern is invalid.
    pub fn replace(
        content: &str,
        pattern: &str,
        replacement: &str,
        language: &str,
    ) -> Result<EditResult, EditError> {
        Self::replace_with_language(content, pattern, replacement, parse_language(language)?)
    }

    fn replace_with_language(
        content: &str,
        pattern: &str,
        replacement: &str,
        lang: SupportLang,
    ) -> Result<EditResult, EditError> {
        let search_pattern = compile_search_pattern(pattern, lang)?;
        let matches = collect_replacement_matches(content, lang, &search_pattern, replacement);

        if matches.is_empty() {
            return Ok(unchanged_result(content));
        }

        Ok(apply_replacements(content, matches))
    }

    /// Perform structural replace on a file.
    ///
    /// # Arguments
    /// * `request` - Named file replacement request.
    ///
    /// # Returns
    /// `EditResult` with changes (file is modified only if `config.preview_only` is false).
    ///
    /// # Errors
    ///
    /// Returns [`EditError`] when the file cannot be read, written, or structurally rewritten.
    pub fn replace_in_file(request: ReplaceInFileRequest<'_>) -> Result<EditResult, EditError> {
        replace_in_file_internal(request)
    }

    fn replace_in_file_path<P: AsRef<Path>>(
        path: P,
        pattern: &str,
        replacement: &str,
        language: Option<&str>,
        config: &EditConfig,
    ) -> Result<EditResult, EditError> {
        Self::replace_in_file(ReplaceInFileRequest {
            path: path.as_ref(),
            pattern,
            replacement,
            language,
            config,
        })
    }
}

fn replace_in_file_internal(request: ReplaceInFileRequest<'_>) -> Result<EditResult, EditError> {
    let content = xiuxian_io::read_text_safe(request.path, request.config.max_file_size)?;

    let lang_str = match request.language {
        Some(l) => l.to_string(),
        None => {
            if let Some(lang) = SupportLang::from_path(request.path) {
                format!("{lang:?}").to_lowercase()
            } else {
                let ext = request.path.extension().map_or_else(
                    || "unknown".to_string(),
                    |e| e.to_string_lossy().to_string(),
                );
                return Err(EditError::UnsupportedLanguage(ext));
            }
        }
    };

    let result =
        StructuralEditor::replace(&content, request.pattern, request.replacement, &lang_str)?;

    if !request.config.preview_only && result.count > 0 {
        std::fs::write(request.path, &result.modified)
            .map_err(|e| EditError::Replacement(format!("Failed to write file: {e}")))?;
    }

    Ok(result)
}

impl StructuralEditor {
    /// Preview structural replace (no file modification).
    ///
    /// Convenience method that always previews without modifying files.
    ///
    /// # Errors
    ///
    /// Returns [`EditError`] when the file cannot be read or the rewrite cannot be computed.
    pub fn preview<P: AsRef<Path>>(
        path: P,
        pattern: &str,
        replacement: &str,
        language: Option<&str>,
    ) -> Result<EditResult, EditError> {
        Self::replace_in_file_path(
            path,
            pattern,
            replacement,
            language,
            &EditConfig {
                preview_only: true,
                ..Default::default()
            },
        )
    }

    /// Apply structural replace (modify file).
    ///
    /// **Use with caution** - this modifies the file in place.
    ///
    /// # Errors
    ///
    /// Returns [`EditError`] when the file cannot be read, written, or structurally rewritten.
    pub fn apply<P: AsRef<Path>>(
        path: P,
        pattern: &str,
        replacement: &str,
        language: Option<&str>,
    ) -> Result<EditResult, EditError> {
        Self::replace_in_file_path(
            path,
            pattern,
            replacement,
            language,
            &EditConfig {
                preview_only: false,
                ..Default::default()
            },
        )
    }

    /// Format edit result for display.
    ///
    /// Returns a human-readable summary of the changes.
    #[must_use]
    pub fn format_result(result: &EditResult, path: Option<&str>) -> String {
        let mut output = String::new();

        if let Some(p) = path {
            let _ = writeln!(output, "// EDIT: {p}");
        }
        let _ = writeln!(output, "// Replacements: {}", result.count);

        if result.count == 0 {
            output.push_str("[No matches found]\n");
            return output;
        }

        output.push_str("\n// Changes:\n");
        for edit in &result.edits {
            let _ = writeln!(
                output,
                "L{}: \"{}\" -> \"{}\"",
                edit.line, edit.original_text, edit.new_text
            );
        }

        output.push_str("\n// Diff:\n");
        output.push_str(&result.diff);

        output
    }
}

fn parse_language(language: &str) -> Result<SupportLang, EditError> {
    SupportLang::from_str(language)
        .map_err(|_| EditError::UnsupportedLanguage(language.to_string()))
}

fn compile_search_pattern(pattern: &str, lang: SupportLang) -> Result<Pattern, EditError> {
    Pattern::try_new(pattern, lang).map_err(|e| EditError::Pattern(e.to_string()))
}

fn collect_replacement_matches(
    content: &str,
    lang: SupportLang,
    search_pattern: &Pattern,
    replacement: &str,
) -> Vec<ReplacementMatch> {
    let root = lang.ast_grep(content);
    root.root()
        .dfs()
        .filter_map(|node| {
            let matched = search_pattern.match_node(node.clone())?;
            let original_text = matched.text().to_string();
            Some(ReplacementMatch {
                start: matched.range().start,
                end: matched.range().end,
                new_text: substitute_captures(replacement, matched.get_env(), &original_text),
                original_text,
            })
        })
        .collect()
}

fn unchanged_result(content: &str) -> EditResult {
    EditResult {
        original: content.to_string(),
        modified: content.to_string(),
        count: 0,
        diff: String::new(),
        edits: Vec::new(),
    }
}

fn apply_replacements(content: &str, mut matches: Vec<ReplacementMatch>) -> EditResult {
    matches.sort_by_key(|entry| std::cmp::Reverse(entry.start));

    let edits = matches
        .iter()
        .rev()
        .map(|replacement| edit_location(content, replacement))
        .collect();
    let modified = matches
        .iter()
        .fold(content.to_string(), apply_single_replacement);

    build_edit_result(content, modified, edits)
}

fn apply_single_replacement(mut modified: String, replacement: &ReplacementMatch) -> String {
    modified.replace_range(replacement.start..replacement.end, &replacement.new_text);
    modified
}

fn edit_location(content: &str, replacement: &ReplacementMatch) -> EditLocation {
    let line = content[..replacement.start].matches('\n').count() + 1;
    let last_newline = content[..replacement.start]
        .rfind('\n')
        .map_or(0, |idx| idx + 1);
    EditLocation {
        line,
        column: replacement.start - last_newline + 1,
        original_text: replacement.original_text.clone(),
        new_text: replacement.new_text.clone(),
    }
}

fn build_edit_result(content: &str, modified: String, edits: Vec<EditLocation>) -> EditResult {
    EditResult {
        original: content.to_string(),
        diff: generate_unified_diff(content, &modified),
        count: edits.len(),
        modified,
        edits,
    }
}

#[cfg(test)]
#[path = "../tests/unit/editor.rs"]
mod tests;
