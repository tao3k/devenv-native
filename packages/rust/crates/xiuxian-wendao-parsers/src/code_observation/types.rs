//! Code observation DTOs parsed from Markdown property drawers.

use super::glob::{find_closing_quote, path_matches_scope};
use serde::{Deserialize, Serialize};

struct ParsedObservationInput<'a> {
    language: String,
    scope: Option<String>,
    pattern: String,
    raw_value: &'a str,
}

/// Parsed code observation entry from `:OBSERVE:` property drawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeObservation {
    /// Target language for the pattern (e.g., "rust", "python", "typescript").
    pub language: String,
    /// The sgrep/ast-grep pattern to match in source code.
    pub pattern: String,
    /// Optional scope filter to restrict pattern matching to specific paths.
    ///
    /// Supports glob patterns such as:
    /// - `"src/api/**"`
    /// - `"packages/core/**/*.rs"`
    /// - `"**/handler.rs"`
    pub scope: Option<String>,
    /// The original raw value from the property drawer.
    pub raw_value: String,
    /// Line number within the document where this observation was declared.
    pub line_number: Option<usize>,
    /// Whether the pattern has been validated by an external language provider.
    pub is_validated: bool,
    /// Validation error message if pattern validation failed.
    pub validation_error: Option<String>,
}

impl CodeObservation {
    /// Create a new code observation.
    #[must_use]
    pub fn new(language: String, pattern: String, raw_value: String) -> Self {
        Self {
            language,
            pattern,
            scope: None,
            raw_value,
            line_number: None,
            is_validated: false,
            validation_error: None,
        }
    }

    /// Create a code observation with scope filter.
    #[must_use]
    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Create a code observation with line number.
    #[must_use]
    pub fn with_line(mut self, line_number: usize) -> Self {
        self.line_number = Some(line_number);
        self
    }

    /// Mark this observation as validated.
    #[must_use]
    pub fn validated(mut self) -> Self {
        self.is_validated = true;
        self
    }

    /// Mark this observation as having a validation error.
    #[must_use]
    pub fn with_error(mut self, error: String) -> Self {
        self.validation_error = Some(error);
        self
    }

    /// Check if a file path matches this observation's scope.
    #[must_use]
    pub fn matches_scope(&self, file_path: &str) -> bool {
        match &self.scope {
            None => true,
            Some(scope) => path_matches_scope(file_path, scope),
        }
    }

    /// Parse a `:OBSERVE:` value string into a `CodeObservation`.
    ///
    /// # Format
    ///
    /// - `lang:<language> "<pattern>"`
    /// - `lang:<language> scope:"<filter>" "<pattern>"`
    ///
    /// # Examples
    ///
    /// ```
    /// use xiuxian_wendao_parsers::CodeObservation;
    ///
    /// let obs = CodeObservation::parse(r#"lang:rust "fn $NAME()""#);
    /// assert!(obs.is_some());
    ///
    /// let obs = CodeObservation::parse(r#"lang:rust scope:"src/api/**" "fn $NAME()""#);
    /// assert!(obs.is_some());
    /// ```
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        let parsed = parse_observation_input(value)?;
        Some(build_observation(parsed))
    }

    /// Report that local AST pattern validation is no longer available.
    ///
    /// # Errors
    ///
    /// Always returns an error because local tree-sitter/ast-grep validation
    /// was retired from this crate. Code intelligence will be provided by the
    /// external language-provider boundary.
    pub fn validate_pattern(&self) -> Result<(), String> {
        Err(format!(
            "code observation validation for `{}` is retired; use the language-provider boundary",
            self.language
        ))
    }
}

fn parse_observation_input(value: &str) -> Option<ParsedObservationInput<'_>> {
    let after_lang = value.trim().strip_prefix("lang:")?;
    let (language, rest) = parse_language_prefix(after_lang)?;
    let (scope, rest) = parse_optional_scope(rest);
    let pattern = parse_quoted_pattern(rest)?;

    Some(ParsedObservationInput {
        language,
        scope,
        pattern,
        raw_value: value,
    })
}

fn parse_language_prefix(after_lang: &str) -> Option<(String, &str)> {
    let space_pos = after_lang.find(' ')?;
    let language = after_lang[..space_pos].trim().to_string();
    (!language.is_empty()).then_some((language, after_lang[space_pos..].trim()))
}

fn parse_optional_scope(rest: &str) -> (Option<String>, &str) {
    let Some(scope_str) = rest.strip_prefix("scope:\"") else {
        return (None, rest);
    };
    find_closing_quote(scope_str).map_or((None, rest), |end_quote| {
        (
            Some(scope_str[..end_quote].replace("\\\"", "\"")),
            scope_str[end_quote + 1..].trim(),
        )
    })
}

fn parse_quoted_pattern(rest: &str) -> Option<String> {
    let pattern_str = rest.strip_prefix('"')?;
    let end_pos = find_closing_quote(pattern_str)?;
    Some(pattern_str[..end_pos].replace("\\\"", "\""))
}

fn build_observation(parsed: ParsedObservationInput<'_>) -> CodeObservation {
    let observation = CodeObservation::new(
        parsed.language,
        parsed.pattern,
        parsed.raw_value.to_string(),
    );
    match parsed.scope {
        Some(scope) => observation.with_scope(scope),
        None => observation,
    }
}
