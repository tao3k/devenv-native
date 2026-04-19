use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use include_dir::Dir;
use xiuxian_wendao_core::WendaoResourceUri;

use super::super::{InternalSkillManifest, SkillNamespaceIndex, SkillVfsError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::skill_vfs::resolver) struct EmbeddedSemanticMount {
    pub(in crate::skill_vfs::resolver) crate_id: String,
    pub(in crate::skill_vfs::resolver) references_dir: PathBuf,
}

/// Semantic resource resolver for `wendao://skills/.../references/...`.
#[derive(Debug, Clone, Default)]
pub struct SkillVfsResolver {
    pub(in crate::skill_vfs::resolver) index: SkillNamespaceIndex,
    pub(in crate::skill_vfs::resolver) mounts: HashMap<String, &'static Dir<'static>>,
    pub(in crate::skill_vfs::resolver) embedded_mounts_by_semantic:
        HashMap<String, Vec<EmbeddedSemanticMount>>,
    pub(in crate::skill_vfs::resolver) content_cache: Arc<RwLock<HashMap<String, Arc<str>>>>,
    pub(in crate::skill_vfs::resolver) internal_roots: Vec<PathBuf>,
}

impl SkillVfsResolver {
    /// Build resolver by scanning one or more skill roots.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when namespace indexing fails.
    pub fn from_roots(roots: &[PathBuf]) -> Result<Self, SkillVfsError> {
        Ok(Self {
            index: SkillNamespaceIndex::build_from_roots(roots)?,
            mounts: HashMap::new(),
            embedded_mounts_by_semantic: HashMap::new(),
            content_cache: Arc::new(RwLock::new(HashMap::new())),
            internal_roots: Vec::new(),
        })
    }

    /// Build resolver by scanning roots and enabling embedded resource mount.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when namespace indexing fails.
    pub fn from_roots_with_embedded(roots: &[PathBuf]) -> Result<Self, SkillVfsError> {
        Self::from_roots(roots).map(Self::mount_embedded_dir)
    }

    /// Build resolver by scanning both regular and internal skill roots.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when namespace indexing fails.
    pub fn from_roots_with_internal(
        roots: &[PathBuf],
        internal_roots: &[PathBuf],
    ) -> Result<Self, SkillVfsError> {
        let mut resolver = Self::from_roots(roots)?;
        resolver.internal_roots = internal_roots.to_vec();
        for root in internal_roots {
            resolver.index.index_root(root, true)?;
        }
        Ok(resolver)
    }

    /// Build resolver by scanning roots, internal roots, and enabling embedded resource mount.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when namespace indexing fails.
    pub fn from_roots_with_embedded_and_internal(
        roots: &[PathBuf],
        internal_roots: &[PathBuf],
    ) -> Result<Self, SkillVfsError> {
        Self::from_roots_with_internal(roots, internal_roots).map(Self::mount_embedded_dir)
    }

    /// Access the underlying semantic namespace index.
    #[must_use]
    pub fn index(&self) -> &SkillNamespaceIndex {
        &self.index
    }

    /// Access the mounted internal skill roots.
    #[must_use]
    pub fn internal_roots(&self) -> &[PathBuf] {
        &self.internal_roots
    }

    /// List all semantic URIs for discovered internal manifests.
    #[must_use]
    pub fn list_internal_manifest_uris(&self) -> Vec<String> {
        self.index
            .all_uris()
            .into_iter()
            .filter(|uri| uri.starts_with("wendao://skills-internal/"))
            .collect()
    }

    /// Load an internal skill manifest by its semantic URI.
    ///
    /// # Errors
    /// Returns [`SkillVfsError`] if the resource is not found or invalid.
    pub fn load_internal_skill_manifest(
        &self,
        uri: &str,
    ) -> Result<InternalSkillManifest, SkillVfsError> {
        let parsed = WendaoResourceUri::parse(uri)?;
        let path = self.resolve_parsed_uri(&parsed)?;
        crate::skill_vfs::internal_manifest::load_internal_skill_manifest_from_path(&path).map_err(
            |e| SkillVfsError::ReadResource {
                path,
                source: std::io::Error::other(e.to_string()),
            },
        )
    }

    /// Scan all mounted internal roots for authorized manifests.
    #[must_use]
    pub fn scan_internal_manifests(&self) -> xiuxian_skills::InternalSkillManifestScan {
        self.scan_authorized_internal_manifests()
            .map(Into::into)
            .unwrap_or_default()
    }
}
