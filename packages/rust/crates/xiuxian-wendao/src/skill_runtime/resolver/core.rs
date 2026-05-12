//! `skill_runtime::resolver::core` owns Wendao skill runtime resolver core behavior.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use include_dir::Dir;
use xiuxian_wendao_core::WendaoResourceUri;

use crate::skill_runtime::{SkillInventory, SkillManifest, SkillManifestScan, SkillRuntimeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::skill_runtime::resolver) struct EmbeddedSemanticMount {
    pub(in crate::skill_runtime::resolver) crate_id: String,
    pub(in crate::skill_runtime::resolver) references_dir: PathBuf,
}

/// Semantic resource resolver for `wendao://skills/.../references/...`.
#[derive(Debug, Clone, Default)]
pub struct SkillRuntimeResolver {
    pub(in crate::skill_runtime::resolver) index: SkillInventory,
    pub(in crate::skill_runtime::resolver) mounts: HashMap<String, &'static Dir<'static>>,
    pub(in crate::skill_runtime::resolver) embedded_mounts_by_semantic:
        HashMap<String, Vec<EmbeddedSemanticMount>>,
    pub(in crate::skill_runtime::resolver) content_cache: Arc<RwLock<HashMap<String, Arc<str>>>>,
    pub(in crate::skill_runtime::resolver) runtime_roots: Vec<PathBuf>,
}

impl SkillRuntimeResolver {
    /// Build resolver by scanning one or more skill roots.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when namespace indexing fails.
    pub fn from_roots(roots: &[PathBuf]) -> Result<Self, SkillRuntimeError> {
        Ok(Self {
            index: SkillInventory::build_from_roots(roots)?,
            mounts: HashMap::new(),
            embedded_mounts_by_semantic: HashMap::new(),
            content_cache: Arc::new(RwLock::new(HashMap::new())),
            runtime_roots: Vec::new(),
        })
    }

    /// Build resolver by scanning roots and enabling embedded resource mount.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when namespace indexing fails.
    pub fn from_roots_with_embedded(roots: &[PathBuf]) -> Result<Self, SkillRuntimeError> {
        Self::from_roots(roots).map(Self::mount_embedded_dir)
    }

    /// Build resolver by scanning both regular and runtime skill roots.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when namespace indexing fails.
    pub fn from_roots_with_runtime(
        roots: &[PathBuf],
        runtime_roots: &[PathBuf],
    ) -> Result<Self, SkillRuntimeError> {
        let mut resolver = Self::from_roots(roots)?;
        resolver.runtime_roots = runtime_roots.to_vec();
        for root in runtime_roots {
            resolver.index.index_root(root)?;
        }
        Ok(resolver)
    }

    /// Build resolver by scanning roots, runtime roots, and enabling embedded resource mount.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when namespace indexing fails.
    pub fn from_roots_with_embedded_and_runtime(
        roots: &[PathBuf],
        runtime_roots: &[PathBuf],
    ) -> Result<Self, SkillRuntimeError> {
        Self::from_roots_with_runtime(roots, runtime_roots).map(Self::mount_embedded_dir)
    }

    /// Access the underlying semantic namespace index.
    #[must_use]
    pub fn index(&self) -> &SkillInventory {
        &self.index
    }

    /// Access the mounted runtime skill roots.
    #[must_use]
    pub fn runtime_roots(&self) -> &[PathBuf] {
        &self.runtime_roots
    }

    /// List all semantic URIs for discovered runtime manifests.
    #[must_use]
    pub fn list_manifest_uris(&self) -> Vec<String> {
        self.index
            .all_uris()
            .into_iter()
            .filter(|uri| uri.ends_with("/qianji.toml"))
            .collect()
    }

    /// Load a runtime skill manifest by its semantic URI.
    ///
    /// # Errors
    /// Returns [`SkillRuntimeError`] if the resource is not found or invalid.
    pub fn load_skill_manifest(&self, uri: &str) -> Result<SkillManifest, SkillRuntimeError> {
        let parsed = WendaoResourceUri::parse(uri)?;
        let path = self.resolve_parsed_uri(&parsed)?;
        crate::skill_runtime::manifest::load_skill_manifest_from_path(&path).map_err(|e| {
            SkillRuntimeError::ReadResource {
                path,
                source: std::io::Error::other(e.to_string()),
            }
        })
    }

    /// Scan all mounted runtime roots for authorized manifests.
    #[must_use]
    pub fn scan_manifests(&self) -> SkillManifestScan {
        self.scan_authorized_manifests()
            .map(Into::into)
            .unwrap_or_default()
    }
}
