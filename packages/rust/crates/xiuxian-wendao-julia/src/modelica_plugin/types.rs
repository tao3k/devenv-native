//! Shared types for Modelica repository intelligence.

use std::collections::BTreeMap;

use serde::Serialize;
use xiuxian_wendao_core::repo_intelligence::{DocRecord, ImportKind, RepoSymbolKind};

/// Borrowed Modelica repository source identifier accepted by public helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelicaSourceId<'a>(&'a str);

impl<'a> ModelicaSourceId<'a> {
    /// Return the raw repository source identifier.
    #[must_use]
    pub fn as_str(self) -> &'a str {
        self.0
    }
}

impl<'a> From<&'a str> for ModelicaSourceId<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

/// Collected documentation record with target IDs.
#[derive(Debug, Clone)]
pub(crate) struct CollectedDoc {
    pub(crate) record: DocRecord,
    pub(crate) target_ids: Vec<String>,
}

/// Parsed import statement from Modelica source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ParsedImport {
    /// Imported package/class name.
    pub(crate) name: String,
    /// Optional alias for the import.
    pub(crate) alias: Option<String>,
    /// Normalized import kind.
    pub(crate) kind: ImportKind,
    /// Source location: starting line number (1-based).
    pub(crate) line_start: Option<usize>,
    /// Parser-owned detail attributes preserved for downstream AST consumers.
    pub(crate) attributes: BTreeMap<String, String>,
}

/// Parsed symbol declaration from Modelica source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ParsedDeclaration {
    /// Symbol display name.
    pub(crate) name: String,
    /// Normalized symbol kind.
    pub(crate) kind: RepoSymbolKind,
    /// Signature snippet (first line of declaration).
    pub(crate) signature: String,
    /// Source location: starting line number (1-based).
    pub(crate) line_start: Option<usize>,
    /// Source location: ending line number (1-based).
    pub(crate) line_end: Option<usize>,
    /// Mathematical equations within this declaration.
    pub(crate) equations: Vec<String>,
    /// Parser-owned detail attributes preserved for downstream AST consumers.
    pub(crate) attributes: BTreeMap<String, String>,
}
