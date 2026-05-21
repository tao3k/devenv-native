//! `skill_runtime::index::types` owns Wendao skill runtime index types behavior.

use crate::skill_runtime::inventory::preload::{preload_reference_dir, semantic_resource_uri_key};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use xiuxian_wendao_core::WendaoResourceUri;

use crate::skill_runtime::SkillRuntimeError;

/// One mounted semantic namespace in skill runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInventoryMount {
    /// Semantic namespace from SKILL frontmatter `name`.
    pub semantic_name: String,
    /// Descriptor path that declared the namespace.
    pub skill_doc: PathBuf,
    /// Relative resource root (`references/`) for this namespace.
    pub references_dir: PathBuf,
}

/// In-memory semantic namespace index built from skill roots.
#[derive(Debug, Clone, Default)]
pub struct SkillInventory {
    pub(in crate::skill_runtime::inventory) mounts_by_name:
        HashMap<String, Vec<SkillInventoryMount>>,
    pub(in crate::skill_runtime::inventory) paths_by_uri: HashMap<String, PathBuf>,
}

impl SkillInventory {
    /// Resolve all mounts by semantic namespace (case-insensitive).
    #[must_use]
    pub fn mounts_for(&self, semantic_name: &str) -> Option<&[SkillInventoryMount]> {
        let key = semantic_name.trim().to_ascii_lowercase();
        self.mounts_by_name.get(&key).map(Vec::as_slice)
    }

    /// Returns total number of indexed semantic namespaces.
    #[must_use]
    pub fn namespace_count(&self) -> usize {
        self.mounts_by_name.len()
    }

    /// Resolve one concrete path from parsed semantic URI.
    #[must_use]
    pub fn path_for_uri(&self, uri: &WendaoResourceUri) -> Option<&PathBuf> {
        let key = semantic_resource_uri_key(uri.semantic_name(), uri.entity_name());
        self.paths_by_uri.get(key.as_str())
    }

    /// Index a single root directory into the namespace.
    ///
    /// # Errors
    /// Returns [`SkillRuntimeError`] if descriptor scanning or parsing fails.
    pub fn index_root(&mut self, root: &Path) -> Result<(), SkillRuntimeError> {
        let other = Self::build_from_roots(&[root.to_path_buf()])?;
        for (name, mounts) in other.mounts_by_name {
            self.mounts_by_name.entry(name).or_default().extend(mounts);
        }
        for (uri, path) in other.paths_by_uri {
            self.paths_by_uri.insert(uri, path);
        }
        Ok(())
    }

    /// Return all unique semantic resource URIs currently indexed.
    #[must_use]
    pub fn all_uris(&self) -> Vec<String> {
        self.paths_by_uri.keys().cloned().collect()
    }

    /// Preload references for one semantic namespace.
    pub(in crate::skill_runtime::inventory) fn preload_references_for_semantic(
        &mut self,
        semantic_name: &str,
    ) {
        let Some(mounts) = self.mounts_by_name.get(semantic_name) else {
            return;
        };
        let references_roots = mounts
            .iter()
            .map(|mount| mount.references_dir.clone())
            .collect::<Vec<_>>();
        for references_dir in references_roots {
            preload_reference_dir(self, semantic_name, references_dir.as_path());
        }
    }
}

/// Compatibility alias for the previous inventory type name.
pub type SkillNamespaceIndex = SkillInventory;

/// Compatibility alias for the previous inventory mount type name.
pub type SkillNamespaceMount = SkillInventoryMount;
