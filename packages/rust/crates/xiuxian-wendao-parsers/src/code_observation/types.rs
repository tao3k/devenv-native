//! Code observation DTOs parsed from Markdown property drawers.

use super::glob::{find_closing_quote, path_matches_scope};
use serde::{Deserialize, Serialize};

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
    /// Whether the pattern has been validated by `xiuxian-ast`.
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
    #[allow(clippy::too_many_lines)]
    pub fn parse(value: &str) -> Option<Self> {
        let trimmed = value.trim();
        if !trimmed.starts_with("lang:") {
            return None;
        }

        let after_lang = &trimmed[5..];
        let space_pos = after_lang.find(' ')?;

        let language = after_lang[..space_pos].trim().to_string();
        if language.is_empty() {
            return None;
        }

        let mut rest = after_lang[space_pos..].trim();
        let mut scope = None;

        if rest.starts_with("scope:\"") {
            let scope_str = &rest[7..];
            if let Some(end_quote) = find_closing_quote(scope_str) {
                scope = Some(scope_str[..end_quote].replace("\\\"", "\""));
                rest = scope_str[end_quote + 1..].trim();
            }
        }

        if !rest.starts_with('"') {
            return None;
        }

        let pattern_str = &rest[1..];
        let end_pos = find_closing_quote(pattern_str)?;
        let pattern = pattern_str[..end_pos].replace("\\\"", "\"");

        let mut observation = Self::new(language, pattern, value.to_string());
        if let Some(scope) = scope {
            observation = observation.with_scope(scope);
        }

        Some(observation)
    }

    /// Get the language for `xiuxian-ast` queries.
    #[must_use]
    pub fn ast_language(&self) -> Option<xiuxian_ast::Lang> {
        xiuxian_ast::Lang::try_from(self.language.as_str()).ok()
    }

    /// Validate the pattern using `xiuxian-ast`.
    ///
    /// # Errors
    ///
    /// Returns an error when the observation language is not supported by
    /// `xiuxian-ast` or when the configured pattern is not accepted by the
    /// target parser.
    pub fn validate_pattern(&self) -> Result<(), String> {
        let language = self
            .ast_language()
            .ok_or_else(|| format!("Unsupported language: {}", self.language))?;

        xiuxian_ast::pattern(&self.pattern, language)
            .map_err(|error| format!("Invalid pattern: {error}"))?;

        Ok(())
    }
}
