//! xiuxian-ast - Unified AST utilities using ast-grep.
//!
//! This crate provides a unified interface for AST-based code analysis across
//! the Omni `DevEnv` project.
//!
//! ## Architecture
//!
//! ```text
//! xiuxian-ast/src/
//! ├── lib.rs                # Re-exports (entry point)
//! ├── re_exports.rs         # ast-grep re-exports
//! ├── lang.rs               # Lang enum and language support
//! ├── match.rs              # Match struct
//! ├── scan.rs               # Pattern utilities
//! ├── python.rs             # Python-specific utilities
//! └── python_tree_sitter.rs # Tree-sitter Python parser
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use xiuxian_ast::{scan, Lang};
//!
//! let content = "def hello(): pass";
//! let matches = scan(content, "def $NAME", Lang::Python)
//!     .unwrap_or_else(|error| panic!("scan failed: {error}"));
//! ```
//!
#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

// ============================================================================
// Module Declarations
// ============================================================================

mod chunk;
mod extract;
mod fingerprint;
mod item;
mod lang;
mod python;
mod python_tree_sitter;
mod re_exports;
mod scan;
mod security;

// ============================================================================
// Re-exports (for backwards compatibility)
// ============================================================================

pub use re_exports::{
    AstLanguage, DeserializeEnv, Doc, LangParser, LanguageExt, MatcherExt, MetaVarEnv,
    MetaVariable, NodeMatch, Pattern, RuleCore, SerializableRuleCore, SupportLang,
};

// Lang enum
pub use lang::Lang;

// Match struct
pub use item::Match;

// Scan functions (both direct and re-exported from scan module)
pub use scan::{extract, pattern, scan, scan_with_lang};

// Python utilities
pub use python::{
    extract_docstring_from_match, extract_python_docstring, find_python_async_functions,
    find_python_classes, find_python_decorated_by_any, find_python_decorated_functions,
    find_python_functions,
};

// Security scanner for harvested skills
pub use security::{SecurityConfig, SecurityScanner, SecurityViolation};

// Code extraction utilities
pub use extract::{ExtractResult, extract_items, extract_skeleton, get_skeleton_patterns};
pub use fingerprint::{semantic_fingerprint, supports_semantic_fingerprint};

// Code chunking utilities
pub use chunk::{CodeChunk, chunk_code};

// Tree-sitter based Python parser for robust decorator extraction
pub use python_tree_sitter::{
    DecoratedFunction, DecoratorArguments, DecoratorInfo, ParameterInfo, TreeSitterPythonParser,
};
