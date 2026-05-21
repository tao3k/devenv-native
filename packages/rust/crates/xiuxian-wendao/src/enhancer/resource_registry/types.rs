//! `enhancer::resource_registry::types` owns Wendao enhancer resource registry types behavior.

use std::collections::HashMap;

use crate::enhancer::markdown_config::{MarkdownConfigBlock, MarkdownConfigMemoryIndex};
use include_dir::Dir;
use thiserror::Error;

/// One normalized config link target with optional explicit reference category.
#[derive(Debug, Clone, PartialEq, Eq)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct WendaoResourceLinkTarget {
    /// Normalized target path or semantic URI.
    pub target_path: String,
    /// Optional explicit reference category (for example `template`, `persona`).
    pub reference_type: Option<String>,
}

/// One unresolved link edge found during embedded-registry validation.
#[derive(Debug, Clone, PartialEq, Eq)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct MissingEmbeddedLink {
    /// Markdown source file path inside embedded resources.
    pub source_path: String,
    /// Config block id owning this link scope.
    pub id: String,
    /// Linked target path that was not found.
    pub target_path: String,
}

/// Error type for `WendaoResourceRegistry::build_from_embedded`.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WendaoResourceRegistryError {
    /// Embedded markdown file bytes could not be decoded as UTF-8 text.
    #[error("embedded markdown file is not valid UTF-8: {path}")]
    InvalidUtf8 {
        /// Failing markdown file path inside embedded resources.
        path: String,
    },
    /// One or more link targets declared in markdown could not be resolved.
    #[error("embedded markdown registry found {count} missing linked resource(s)")]
    MissingLinkedResources {
        /// Number of unresolved links.
        count: usize,
        /// Detailed unresolved links.
        missing: Vec<MissingEmbeddedLink>,
    },
}

/// Per-file view for markdown config links extracted from embedded resources.
#[derive(Debug, Clone, Default)]
pub struct WendaoResourceFile {
    pub(crate) path: String,
    pub(crate) links_by_id: HashMap<String, Vec<String>>,
    pub(crate) link_targets_by_id: HashMap<String, Vec<WendaoResourceLinkTarget>>,
}

impl WendaoResourceFile {
    /// File path (relative to embedded resources root).
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns all linked targets for one config `id`.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn links_for_id(&self, id: &str) -> Option<&[String]> {
        self.links_by_id.get(id).map(Vec::as_slice)
    }

    /// Full map of `id -> linked targets` parsed from this file.
    #[must_use]
    pub fn links_by_id(&self) -> &HashMap<String, Vec<String>> {
        &self.links_by_id
    }

    /// Full map of `id -> link targets` with optional explicit reference categories.
    #[must_use]
    pub fn link_targets_by_id(&self) -> &HashMap<String, Vec<WendaoResourceLinkTarget>> {
        &self.link_targets_by_id
    }

    /// Returns link targets for one config `id` including optional explicit reference categories.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn link_targets_for_id(&self, id: &str) -> Option<&[WendaoResourceLinkTarget]> {
        self.link_targets_by_id.get(id).map(Vec::as_slice)
    }

    /// Resolves deduplicated semantic links for one reference type.
    ///
    /// Type matching is ASCII case-insensitive and uses explicit section
    /// metadata, not body-link suffixes.
    #[must_use]
    pub fn links_for_reference_type(&self, reference_type: &str) -> Vec<String> {
        let normalized_type = reference_type.trim().to_ascii_lowercase();
        if normalized_type.is_empty() {
            return Vec::new();
        }

        let mut links = self
            .link_targets_by_id
            .values()
            .flatten()
            .filter(|target| target.reference_type.as_deref() == Some(normalized_type.as_str()))
            .map(|target| target.target_path.clone())
            .collect::<Vec<_>>();
        links.sort();
        links.dedup();
        links
    }
}

/// Embedded markdown registry parsed by Wendao AST utilities.
#[derive(Debug, Clone, Default)]
pub struct WendaoResourceRegistry {
    pub(crate) files_by_path: HashMap<String, WendaoResourceFile>,
    pub(crate) config_index: MarkdownConfigMemoryIndex,
}

impl WendaoResourceRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// O(1) config block lookup by exact `id`.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn get(&self, id: &str) -> Option<&MarkdownConfigBlock> {
        self.config_index.get(id)
    }

    /// Returns one embedded markdown file entry by relative path.
    #[must_use]
    pub fn file(&self, path: &str) -> Option<&WendaoResourceFile> {
        self.files_by_path
            .get(&crate::enhancer::resource_registry::scan::normalize_registry_key(path))
    }

    /// Iterates all embedded markdown file entries.
    pub fn files(&self) -> impl Iterator<Item = &WendaoResourceFile> {
        self.files_by_path.values()
    }

    /// Number of indexed markdown files.
    #[must_use]
    pub fn files_len(&self) -> usize {
        self.files_by_path.len()
    }

    /// Access to the underlying O(1) markdown config index.
    #[must_use]
    pub fn config_index(&self) -> &MarkdownConfigMemoryIndex {
        &self.config_index
    }
}

impl WendaoResourceRegistry {
    /// Builds a registry from compile-time embedded resources.
    ///
    /// This scans markdown files, indexes tagged config blocks in O(1) by `id`,
    /// extracts local linked targets per `id`, and validates each linked target
    /// exists in the same embedded directory tree.
    ///
    /// # Errors
    ///
    /// Returns [`WendaoResourceRegistryError::InvalidUtf8`] when an embedded
    /// markdown file cannot be decoded as UTF-8.
    ///
    /// Returns [`WendaoResourceRegistryError::MissingLinkedResources`] when
    /// markdown links reference files missing from embedded resources.
    pub fn build_from_embedded(embedded: &Dir<'_>) -> Result<Self, WendaoResourceRegistryError> {
        crate::enhancer::resource_registry::registry::build_from_embedded(embedded)
    }
}
