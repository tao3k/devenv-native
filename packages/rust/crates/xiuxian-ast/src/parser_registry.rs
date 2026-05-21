//! Parser registry for general AST baselines and local authority overrides.

use std::collections::HashMap;
use std::path::Path;

use crate::Lang;

/// Describes which parser should provide effective evidence for one source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstParserResolution {
    /// The parser that owns the effective structure for this source.
    pub effective_parser: String,
    /// The general `xiuxian-ast` baseline parser, when the source language is
    /// supported by ast-grep.
    pub baseline_parser: Option<String>,
    /// The priority class used to choose the effective parser.
    pub priority: AstParserPriority,
}

/// Parser priority selected by [`AstParserRegistry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstParserPriority {
    /// A local native or plugin parser overrides the general AST baseline.
    LocalOverride,
    /// The general `xiuxian-ast` baseline is the effective parser.
    GeneralBaseline,
    /// No structured parser is registered for this path.
    PlainText,
}

/// Registry of parser ownership rules for AST-backed evidence surfaces.
#[derive(Debug, Clone, Default)]
pub struct AstParserRegistry {
    overrides_by_extension: HashMap<String, String>,
}

impl AstParserRegistry {
    /// Creates an empty parser registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a local native/plugin parser as the effective parser for an
    /// extension.
    ///
    /// The leading dot is optional. Parser names should use stable
    /// `<language>-lang-parser` style identifiers such as `rust-lang-parser`,
    /// `julia-lang-parser`, or `modelica-lang-parser`.
    #[must_use]
    pub fn with_extension_override(
        mut self,
        extension: impl AsRef<str>,
        parser_name: impl Into<String>,
    ) -> Self {
        self.insert_extension_override(extension, parser_name);
        self
    }

    /// Registers a local native/plugin parser as the effective parser for an
    /// extension.
    pub fn insert_extension_override(
        &mut self,
        extension: impl AsRef<str>,
        parser_name: impl Into<String>,
    ) {
        let extension = normalized_extension_key(extension.as_ref());
        if extension.is_empty() {
            return;
        }
        self.overrides_by_extension
            .insert(extension, parser_name.into());
    }

    /// Resolves the effective parser and optional general AST baseline for a
    /// source path.
    #[must_use]
    pub fn resolve_path(&self, path: impl AsRef<Path>) -> AstParserResolution {
        let path = path.as_ref();
        let baseline = Lang::from_path(path).map(baseline_parser_name);
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(normalized_extension_key)
            .unwrap_or_default();

        if let Some(parser) = self.overrides_by_extension.get(&extension) {
            return AstParserResolution {
                effective_parser: parser.clone(),
                baseline_parser: baseline,
                priority: AstParserPriority::LocalOverride,
            };
        }

        if let Some(parser) = baseline.clone() {
            return AstParserResolution {
                effective_parser: parser,
                baseline_parser: baseline,
                priority: AstParserPriority::GeneralBaseline,
            };
        }

        AstParserResolution {
            effective_parser: "plain-text-parser".to_owned(),
            baseline_parser: None,
            priority: AstParserPriority::PlainText,
        }
    }
}

/// Returns the general `xiuxian-ast` parser name for a supported language.
#[must_use]
pub fn baseline_parser_name(language: Lang) -> String {
    format!("xiuxian-ast:{}", language.as_str())
}

fn normalized_extension_key(extension: &str) -> String {
    extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}
