//! Typed record contracts emitted by repository-intelligence analyzers.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::config::RegisteredRepository;

macro_rules! repo_record_string_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Debug,
            Clone,
            Hash,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Serialize,
            Deserialize,
            JsonSchema,
            Default,
        )]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Borrows the serialized value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl std::borrow::Borrow<str> for $name {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.as_str() == other
            }
        }

        impl PartialEq<String> for $name {
            fn eq(&self, other: &String) -> bool {
                self.as_str() == other
            }
        }

        impl PartialEq<$name> for String {
            fn eq(&self, other: &$name) -> bool {
                self == other.as_str()
            }
        }

        impl $name {
            /// Returns true when the serialized value is empty.
            #[must_use]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }
    };
}

repo_record_string_type!(RepoRecordId, "Stable repository identifier.");
repo_record_string_type!(
    RepoRecordPath,
    "Repository-relative or local repository path."
);
repo_record_string_type!(DocRecordId, "Stable documentation identifier.");
repo_record_string_type!(ExampleRecordId, "Stable example identifier.");
repo_record_string_type!(ModuleRecordId, "Stable module identifier.");
repo_record_string_type!(
    RepoIntelligencePluginId,
    "Stable repository-intelligence plugin identifier."
);
repo_record_string_type!(SymbolRecordId, "Stable symbol identifier.");
repo_record_string_type!(
    RepoAuditStatus,
    "Stable repository-intelligence audit status."
);
repo_record_string_type!(
    RepoVerificationState,
    "Stable repository-intelligence verification state."
);
repo_record_string_type!(DocTargetKind, "Stable documentation target kind.");

/// Record representing a repository in analysis results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepositoryRecord {
    /// Stable repository identifier.
    pub repo_id: RepoRecordId,
    /// Display name for the repository.
    pub name: String,
    /// Local checkout or source path.
    pub path: RepoRecordPath,
    /// Upstream repository URL when known.
    pub url: Option<String>,
    /// Current checked-out revision when known.
    pub revision: Option<String>,
    /// Declared package version when known.
    pub version: Option<String>,
    /// Declared package UUID when known.
    pub uuid: Option<String>,
    /// Declared package dependencies.
    pub dependencies: Vec<String>,
}

impl From<&RegisteredRepository> for RepositoryRecord {
    fn from(reg: &RegisteredRepository) -> Self {
        Self {
            repo_id: reg.id.clone().into(),
            name: reg.id.clone(),
            path: reg
                .path
                .as_ref()
                .map_or_else(String::new, |p| p.to_string_lossy().to_string())
                .into(),
            url: reg.url.clone(),
            ..Self::default()
        }
    }
}

/// Record representing a module extracted from analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ModuleRecord {
    /// Stable repository identifier.
    pub repo_id: RepoRecordId,
    /// Stable module identifier.
    pub module_id: ModuleRecordId,
    /// Fully-qualified module name.
    pub qualified_name: String,
    /// Repository-relative module path.
    pub path: RepoRecordPath,
}

/// Record representing a symbolic entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct SymbolRecord {
    /// Stable repository identifier.
    pub repo_id: RepoRecordId,
    /// Stable symbol identifier.
    pub symbol_id: SymbolRecordId,
    /// Owning module identifier when known.
    pub module_id: Option<ModuleRecordId>,
    /// Short symbol name.
    pub name: String,
    /// Fully-qualified symbol name.
    pub qualified_name: String,
    /// Normalized symbol kind.
    pub kind: RepoSymbolKind,
    /// Repository-relative symbol path.
    pub path: RepoRecordPath,
    /// Optional 1-based starting line.
    pub line_start: Option<usize>,
    /// Optional 1-based ending line.
    pub line_end: Option<usize>,
    /// Optional rendered signature.
    pub signature: Option<String>,
    /// Optional audit status carried into downstream payloads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_status: Option<RepoAuditStatus>,
    /// Optional skeptic verification state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_state: Option<RepoVerificationState>,
    /// Free-form symbol attributes emitted by analyzers.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

/// Normalized symbol kind for repository intelligence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum RepoSymbolKind {
    /// Callable function or method.
    #[default]
    Function,
    /// Type, struct, or class-like symbol.
    Type,
    /// Constant or immutable value.
    Constant,
    /// Symbol exported by a module.
    ModuleExport,
    /// Any symbol kind not mapped into a more specific bucket.
    Other,
}

/// Relationship between repository entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RelationRecord {
    /// Stable repository identifier.
    pub repo_id: RepoRecordId,
    /// Source entity identifier.
    pub source_id: String,
    /// Target entity identifier.
    pub target_id: String,
    /// Relationship kind between the source and target.
    pub kind: RelationKind,
}

/// Kind of relationship between entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RelationKind {
    /// Source module contains target symbol.
    Contains,
    /// Source symbol calls target symbol.
    Calls,
    /// Source symbol uses target type.
    Uses,
    /// Source document describes target entity.
    Documents,
    /// Source example demonstrates target entity.
    ExampleOf,
    /// Source entity declares target entity.
    Declares,
    /// Source entity implements target entity.
    Implements,
    /// Source module imports target symbol/module.
    Imports,
}

/// Parser-owned target metadata attached to a documentation entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocTargetRecord {
    /// Stable parser-owned target kind such as `symbol` or `module`.
    pub kind: DocTargetKind,
    /// Display name of the documented target.
    pub name: String,
    /// Optional parser-owned qualified target path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Optional 1-based declaration start line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_start: Option<usize>,
    /// Optional 1-based declaration end line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_end: Option<usize>,
}

/// Record representing a documentation entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocRecord {
    /// Stable repository identifier.
    pub repo_id: RepoRecordId,
    /// Stable documentation identifier.
    pub doc_id: DocRecordId,
    /// Human-readable document title.
    pub title: String,
    /// Repository-relative document path.
    pub path: RepoRecordPath,
    /// Optional normalized document format.
    pub format: Option<String>,
    /// Optional parser-owned target metadata for code-attached docs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_target: Option<DocTargetRecord>,
}

/// Record representing a code example.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExampleRecord {
    /// Repository identifier.
    pub repo_id: RepoRecordId,
    /// Example identifier.
    pub example_id: ExampleRecordId,
    /// Example title.
    pub title: String,
    /// File path to the example.
    pub path: RepoRecordPath,
    /// Optional summary of the example.
    pub summary: Option<String>,
}

/// Diagnostic message emitted during analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticRecord {
    /// Repository identifier.
    pub repo_id: RepoRecordId,
    /// File path where the diagnostic was emitted.
    pub path: RepoRecordPath,
    /// Line number where the diagnostic was emitted.
    pub line: usize,
    /// Diagnostic message.
    pub message: String,
    /// Severity level.
    pub severity: String,
}

/// Kind of an external import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum ImportKind {
    /// Direct symbolic import.
    #[default]
    Symbol,
    /// Module-level import.
    Module,
    /// Re-export of an import.
    Reexport,
}

/// Record representing an external import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImportRecord {
    /// Repository identifier.
    pub repo_id: RepoRecordId,
    /// Module identifier where the import occurs.
    pub module_id: ModuleRecordId,
    /// Repository-relative source path where the import occurs.
    #[serde(default, skip_serializing_if = "RepoRecordPath::is_empty")]
    pub path: RepoRecordPath,
    /// Name of the imported symbol or module.
    pub import_name: String,
    /// Target package being imported from.
    pub target_package: String,
    /// Source module within the target package.
    pub source_module: String,
    /// Kind of import.
    pub kind: ImportKind,
    /// Optional 1-based starting line where the import occurs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_start: Option<usize>,
    /// Resolved identifier if the import was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_id: Option<ModuleRecordId>,
    /// Free-form parser-owned import attributes emitted by analyzers.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}
