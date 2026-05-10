//! Parser evidence signals for code-intelligence consumers.

use std::path::Path;
use std::sync::OnceLock;

use xiuxian_ast::{AstParserPriority, AstParserRegistry, AstParserResolution, Lang};

use crate::types::CodeLanguageId;

/// Evidence describing which parser owns code structure for one source path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeParserEvidence {
    /// Parser selected as the effective source of structure.
    pub effective_parser: String,
    /// General `xiuxian-ast` baseline parser, when available.
    pub baseline_parser: Option<String>,
    /// Parser priority used to choose the effective parser.
    pub priority: CodeParserPriority,
    /// Compact edge-kind tags for graph and reasoning-tree consumers.
    pub edge_kinds: Vec<String>,
}

/// Parser priority selected for code-intelligence evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeParserPriority {
    /// A local native or plugin parser overrides the general AST baseline.
    LocalOverride,
    /// The general `xiuxian-ast` baseline is the effective parser.
    GeneralBaseline,
    /// No structured parser is registered for this path.
    PlainText,
}

/// Registry for resolving source paths into code parser evidence.
#[derive(Debug, Clone)]
pub struct CodeParserEvidenceRegistry {
    registry: AstParserRegistry,
}

impl CodeParserEvidenceRegistry {
    /// Create a registry from a prepared AST parser registry.
    #[must_use]
    pub fn new(registry: AstParserRegistry) -> Self {
        Self { registry }
    }

    /// Create the default Agent-search parser evidence registry.
    #[must_use]
    pub fn agent_search_defaults() -> &'static Self {
        static REGISTRY: OnceLock<CodeParserEvidenceRegistry> = OnceLock::new();
        REGISTRY.get_or_init(|| {
            Self::new(
                AstParserRegistry::new()
                    .with_extension_override("md", "markdown-lang-parser")
                    .with_extension_override("markdown", "markdown-lang-parser")
                    .with_extension_override("rs", "rust-lang-parser")
                    .with_extension_override("jl", "julia-lang-parser")
                    .with_extension_override("mo", "modelica-lang-parser"),
            )
        })
    }

    /// Resolve parser evidence for one path.
    #[must_use]
    pub fn resolve_path(&self, path: &str) -> CodeParserEvidence {
        let resolution = self.registry.resolve_path(path);
        CodeParserEvidence {
            effective_parser: resolution.effective_parser.clone(),
            baseline_parser: resolution.baseline_parser.clone(),
            priority: CodeParserPriority::from_ast_priority(resolution.priority),
            edge_kinds: edge_kinds_for_resolution(&resolution),
        }
    }

    /// Resolve compact graph edge-kind evidence for one path.
    #[must_use]
    pub fn resolve_path_edge_kinds(&self, path: &str) -> Vec<String> {
        self.resolve_path(path).edge_kinds
    }
}

impl CodeParserPriority {
    fn from_ast_priority(priority: AstParserPriority) -> Self {
        match priority {
            AstParserPriority::LocalOverride => Self::LocalOverride,
            AstParserPriority::GeneralBaseline => Self::GeneralBaseline,
            AstParserPriority::PlainText => Self::PlainText,
        }
    }
}

/// Normalize a language or parser identifier into a compact language id.
#[must_use]
pub fn normalize_code_language_identifier(identifier: &str) -> String {
    let normalized = identifier.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return normalized;
    }
    if let Ok(lang) = Lang::try_from(normalized.as_str()) {
        return lang.as_str().to_owned();
    }
    for suffix in ["-lang-parser", "-code-parser", "-parser"] {
        if let Some(language) = normalized.strip_suffix(suffix)
            && (Lang::try_from(language).is_ok() || is_known_external_code_language(language))
        {
            return language.to_owned();
        }
    }
    normalized
}

/// Resolve the source-code language for a path using the shared AST language
/// table.
#[must_use]
pub fn code_language_from_path(path: &Path) -> Option<Lang> {
    Lang::from_path(path)
}

/// Resolve the source-code language id for a path.
#[must_use]
pub fn code_language_id_from_path(path: &Path) -> Option<&'static str> {
    code_language_from_path(path).map(Lang::as_str)
}

/// Resolve languages supported by generic code structure and reference scans.
#[must_use]
pub fn supported_code_language_from_path(path: &Path) -> Option<Lang> {
    match code_language_from_path(path)? {
        Lang::Python
        | Lang::Rust
        | Lang::JavaScript
        | Lang::TypeScript
        | Lang::Bash
        | Lang::Go
        | Lang::Java
        | Lang::C
        | Lang::Cpp
        | Lang::CSharp
        | Lang::Ruby
        | Lang::Swift
        | Lang::Kotlin
        | Lang::Lua
        | Lang::Php => code_language_from_path(path),
        _ => None,
    }
}

/// Resolve the supported source-code language id for a path.
#[must_use]
pub fn supported_code_language_id_from_path(path: &Path) -> Option<&'static str> {
    supported_code_language_from_path(path).map(Lang::as_str)
}

/// Return every language id known to the generic code-intelligence parser
/// table.
#[must_use]
pub fn all_code_language_ids() -> Vec<CodeLanguageId> {
    Lang::all()
        .iter()
        .copied()
        .map(Lang::as_str)
        .map(CodeLanguageId::from)
        .collect()
}

fn is_known_external_code_language(language: &str) -> bool {
    matches!(language, "julia" | "modelica" | "markdown")
}

fn edge_kinds_for_resolution(resolution: &AstParserResolution) -> Vec<String> {
    let mut kinds = match resolution.priority {
        AstParserPriority::LocalOverride => vec![
            "parser-priority:local-override".to_owned(),
            format!("effective-parser:{}", resolution.effective_parser),
        ],
        AstParserPriority::GeneralBaseline => vec![
            "parser-priority:general-baseline".to_owned(),
            format!("effective-parser:{}", resolution.effective_parser),
        ],
        AstParserPriority::PlainText => vec![
            "parser-priority:plain-text".to_owned(),
            format!("effective-parser:{}", resolution.effective_parser),
        ],
    };

    if let Some(baseline) = resolution.baseline_parser.as_deref() {
        kinds.push("general-ast-baseline".to_owned());
        kinds.push(format!("baseline-parser:{baseline}"));
    }

    if resolution.effective_parser == "markdown-lang-parser"
        || resolution.effective_parser == "rust-lang-parser"
    {
        kinds.push("native-parser-override".to_owned());
    }
    if resolution.effective_parser == "julia-lang-parser"
        || resolution.effective_parser == "modelica-lang-parser"
    {
        kinds.push("plugin-parser-override".to_owned());
    }

    kinds.sort();
    kinds.dedup();
    kinds
}
