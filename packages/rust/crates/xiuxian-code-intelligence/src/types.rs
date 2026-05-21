//! Type definitions for symbols and search results.
//!
//! Core data structures for the code-intelligence signal system.

use serde::Serialize;
use std::collections::HashMap;

/// Current code-intelligence signal schema version.
pub const CODE_INTELLIGENCE_SIGNAL_SCHEMA_VERSION: &str = "xiuxian.code_intelligence.signal.v1";

/// Code-intelligence language identifier.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CodeLanguageId(String);

impl CodeLanguageId {
    /// Returns the underlying normalized language identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for CodeLanguageId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Symbol kind enumeration
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum SymbolKind {
    /// Function definition
    Function,
    /// Class definition
    Class,
    /// Struct definition
    Struct,
    /// Method within a class
    Method,
    /// Trait definition
    Trait,
    /// Impl block
    Impl,
    /// Module or namespace
    Module,
    /// Constant declaration
    Const,
    /// Static declaration
    Static,
    /// Type alias declaration
    TypeAlias,
    /// Async function definition
    AsyncFunction,
    /// Enum definition
    Enum,
    /// Interface or type alias
    Interface,
    /// Unknown or unrecognized symbol
    Unknown,
}

impl From<&str> for SymbolKind {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fn" | "def" | "function" | "method" => SymbolKind::Function,
            "class" => SymbolKind::Class,
            "struct" => SymbolKind::Struct,
            "impl" => SymbolKind::Impl,
            "trait" => SymbolKind::Trait,
            "mod" | "module" => SymbolKind::Module,
            "const" => SymbolKind::Const,
            "static" => SymbolKind::Static,
            "type" | "typealias" | "type_alias" => SymbolKind::TypeAlias,
            "enum" => SymbolKind::Enum,
            "interface" => SymbolKind::Interface,
            _ => SymbolKind::Unknown,
        }
    }
}

/// A symbol extracted from source code
#[derive(Debug, Clone, Serialize)]
pub struct Symbol {
    /// Name of the symbol
    pub name: String,
    /// Kind of symbol (function, class, etc.)
    pub kind: SymbolKind,
    /// Line number where the symbol is defined
    pub line: usize,
    /// Signature or declaration string
    pub signature: String,
}

/// A typed symbol node suitable for indexing and reasoning-tree retrieval.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CodeSymbolNode {
    /// Signal schema version.
    pub schema_version: &'static str,
    /// Source file path.
    pub source_path: String,
    /// Source language identifier.
    pub language: String,
    /// Symbol name.
    pub name: String,
    /// Symbol kind.
    pub kind: SymbolKind,
    /// One-indexed line number where the symbol starts.
    pub line: usize,
    /// Compact declaration or signature.
    pub signature: String,
}

impl CodeSymbolNode {
    /// Build a symbol node from a local symbol extraction result.
    #[must_use]
    pub fn from_symbol(source_path: String, language: String, symbol: Symbol) -> Self {
        Self {
            schema_version: CODE_INTELLIGENCE_SIGNAL_SCHEMA_VERSION,
            source_path,
            language,
            name: symbol.name,
            kind: symbol.kind,
            line: symbol.line,
            signature: symbol.signature,
        }
    }
}

// ============================================================================
// Search Results (Phase 51: The Hunter)
// ============================================================================

/// A single search match result
#[derive(Debug, Clone, Serialize)]
pub struct SearchMatch {
    /// Path to the file
    pub path: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Matched content/snippet
    pub content: String,
    /// Captured variables (if any)
    pub captures: HashMap<String, String>,
}

/// A typed structural search hit suitable for indexing and Agent evidence.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CodeStructureHit {
    /// Signal schema version.
    pub schema_version: &'static str,
    /// Source file path.
    pub source_path: String,
    /// Source language identifier.
    pub language: String,
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed when known).
    pub column: usize,
    /// Matched content/snippet.
    pub content: String,
    /// Captured variables, if any.
    pub captures: HashMap<String, String>,
}

/// A language skeleton symbol extracted from source code.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CodeStructureSymbol {
    /// Symbol name or fallback signature.
    pub name: String,
    /// First declaration/signature line.
    pub signature: String,
    /// One-indexed start line.
    pub line_start: usize,
    /// One-indexed end line.
    pub line_end: usize,
    /// Captured ast-grep variables, if any.
    pub captures: HashMap<String, String>,
}

/// A dependency-indexing symbol extracted from source text.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CodeDependencySymbol {
    /// Symbol identifier.
    pub name: String,
    /// Symbol classification.
    pub kind: SymbolKind,
    /// One-indexed source line.
    pub line: usize,
}

/// Source file content resolved for source-code analysis.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CodeSourceFile {
    /// File path for diagnostics and downstream projection.
    pub path: String,
    /// Source code content.
    pub content: String,
}

/// A structural pattern match extracted from source content.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CodePatternMatch {
    /// Matched source text.
    pub text: String,
    /// Byte offset start position.
    pub start: usize,
    /// Byte offset end position.
    pub end: usize,
    /// One-indexed start line.
    pub line_start: usize,
    /// One-indexed end line.
    pub line_end: usize,
    /// Captured ast-grep variables, if any.
    pub captures: HashMap<String, String>,
}

impl CodeStructureHit {
    /// Build a typed structural hit from a raw search match.
    #[must_use]
    pub fn from_match(language: String, search_match: SearchMatch) -> Self {
        Self {
            schema_version: CODE_INTELLIGENCE_SIGNAL_SCHEMA_VERSION,
            source_path: search_match.path,
            language,
            line: search_match.line,
            column: search_match.column,
            content: search_match.content,
            captures: search_match.captures,
        }
    }
}

/// Return the first non-empty signature line from extracted code text.
#[must_use]
pub fn first_code_signature_line(text: &str) -> &str {
    text.lines().next().map(str::trim).unwrap_or_default()
}

/// Return a pattern-oriented signature line for source-code skeleton matching.
#[must_use]
pub fn code_pattern_signature_line(text: &str, lang: xiuxian_ast::Lang) -> String {
    let first_line = text.lines().next().unwrap_or(text);

    match lang {
        xiuxian_ast::Lang::Python
        | xiuxian_ast::Lang::Ruby
        | xiuxian_ast::Lang::Lua
        | xiuxian_ast::Lang::Bash => {
            if let Some(colon_pos) = first_line.find(':') {
                format!("{} $$$BODY", first_line[..=colon_pos].trim())
            } else {
                first_line.trim().to_string()
            }
        }
        xiuxian_ast::Lang::Rust
        | xiuxian_ast::Lang::C
        | xiuxian_ast::Lang::Cpp
        | xiuxian_ast::Lang::CSharp
        | xiuxian_ast::Lang::Java
        | xiuxian_ast::Lang::Go
        | xiuxian_ast::Lang::Swift
        | xiuxian_ast::Lang::Kotlin
        | xiuxian_ast::Lang::Php
        | xiuxian_ast::Lang::JavaScript
        | xiuxian_ast::Lang::TypeScript => {
            if let Some(brace_pos) = first_line.find('{') {
                format!("{} {{ $$$BODY }}", first_line[..brace_pos].trim())
            } else {
                first_line.trim().to_string()
            }
        }
        _ => first_line.trim().to_string(),
    }
}

/// Return a pattern-oriented signature line using a code-intelligence language id.
#[must_use]
pub fn code_pattern_signature_line_for_language_id(
    text: &str,
    language_id: &CodeLanguageId,
) -> String {
    xiuxian_ast::Lang::try_from(language_id.as_str()).map_or_else(
        |_| text.lines().next().unwrap_or(text).trim().to_string(),
        |lang| code_pattern_signature_line(text, lang),
    )
}

/// Score a code-structure result against a user query.
///
/// Returns `None` when the query does not match the symbol name, signature, or
/// relative path closely enough to keep the result.
#[must_use]
pub fn score_code_structure_query(
    search_term: Option<&str>,
    relative_path: &str,
    name: &str,
    signature: &str,
) -> Option<f64> {
    let Some(search_term) = search_term
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
    else {
        return Some(0.72);
    };
    let normalized_name = name.to_ascii_lowercase();
    let normalized_signature = signature.to_ascii_lowercase();
    let normalized_path = relative_path.to_ascii_lowercase();

    if normalized_name == search_term {
        return Some(1.0);
    }
    if normalized_name.contains(search_term.as_str()) {
        return Some(0.97);
    }
    if normalized_signature.contains(search_term.as_str()) {
        return Some(0.91);
    }
    if normalized_path.contains(search_term.as_str()) {
        return Some(0.84);
    }

    None
}

/// Result of a code search
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Total number of matches
    pub count: usize,
    /// Individual matches
    pub matches: Vec<SearchMatch>,
}

/// Directory walker configuration
pub struct SearchConfig {
    /// File patterns to include (e.g., "**/*.py")
    pub file_pattern: String,
    /// Maximum file size in bytes (default 1MB)
    pub max_file_size: u64,
    /// Maximum number of matches per file
    pub max_matches_per_file: usize,
    /// Languages to search (empty means auto-detect)
    pub languages: Vec<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            file_pattern: "**/*".to_string(),
            max_file_size: 1024 * 1024, // 1MB
            max_matches_per_file: 100,
            languages: Vec::new(),
        }
    }
}
