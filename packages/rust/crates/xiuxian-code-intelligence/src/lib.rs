#![allow(clippy::doc_markdown)]

//! xiuxian-code-intelligence - code structure signals for Agent search.
//!
//! Features:
//! - AST-based symbol extraction using xiuxian-ast (ast-grep)
//! - Syntax-aware matching for Python, Rust, JavaScript, TypeScript
//! - Compact outlines and structural search results for reasoning-tree search
//!
//! # Architecture
//!
//! ```text
//! xiuxian-code-intelligence/src/
//! ├── lib.rs      # Re-exports (this file)
//! ├── analysis.rs # Generic code-structure analysis helpers
//! ├── error.rs    # CodeIntelligenceError, CodeSearchError
//! ├── types.rs    # SymbolKind, Symbol, SearchMatch, SearchConfig
//! ├── patterns/   # ast-grep pattern constants
//! ├── parser_evidence.rs # Parser ownership evidence
//! └── extractor.rs # CodeIntelligenceExtractor
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use xiuxian_code_intelligence::CodeIntelligenceExtractor;
//!
//! let outline = CodeIntelligenceExtractor::outline_file("src/main.py", Some("python"))?;
//! println!("{}", outline);
//! ```

// ============================================================================
// Module Declarations (ODF-REP: Atomic Structure)
// ============================================================================

mod analysis;
mod error;
mod extractor;
mod parser_evidence;
/// Language-specific ast-grep pattern constants and grouped pattern tables.
pub mod patterns;
mod types;

// ============================================================================
// Public Re-exports
// ============================================================================

pub use analysis::{
    code_semantic_fingerprint, code_semantic_fingerprint_language_id_from_identifier,
    code_semantic_fingerprint_language_id_from_path, count_code_pattern_matches,
    count_code_pattern_matches_for_language_id, extract_code_dependency_symbols,
    extract_code_pattern_matches, extract_code_structure_symbols,
    extract_code_structure_symbols_for_language_id, resolve_code_source_files,
    resolve_code_source_files_for_language_id, supports_code_semantic_fingerprint,
};
pub use error::{CodeIntelligenceError, CodeSearchError};
pub use extractor::CodeIntelligenceExtractor;
pub use parser_evidence::{
    CodeParserEvidence, CodeParserEvidenceRegistry, CodeParserPriority, all_code_language_ids,
    code_language_from_path, code_language_id_from_path, normalize_code_language_identifier,
    supported_code_language_from_path, supported_code_language_id_from_path,
};
pub use patterns::{
    JS_CLASS, JS_FN, PYTHON_ASYNC_DEF, PYTHON_CLASS, PYTHON_DEF, RUST_ENUM, RUST_FN, RUST_IMPL,
    RUST_STRUCT, RUST_TRAIT, TS_INTERFACE,
};
pub use types::{
    CODE_INTELLIGENCE_SIGNAL_SCHEMA_VERSION, CodeDependencySymbol, CodeLanguageId,
    CodePatternMatch, CodeSourceFile, CodeStructureHit, CodeStructureSymbol, CodeSymbolNode,
    SearchConfig, SearchMatch, SearchResult, Symbol, SymbolKind, code_pattern_signature_line,
    code_pattern_signature_line_for_language_id, first_code_signature_line,
    score_code_structure_query,
};
