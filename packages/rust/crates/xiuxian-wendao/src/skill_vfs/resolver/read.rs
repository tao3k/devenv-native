use std::path::Path;
use std::sync::Arc;
use xiuxian_wendao_core::WendaoResourceUri;

use super::super::SkillVfsError;
use super::core::SkillVfsResolver;

impl SkillVfsResolver {
    /// Resolve one Wendao URI and return UTF-8 file content.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when URI resolution fails or content lookup fails.
    pub fn read_utf8(&self, uri: &str) -> Result<String, SkillVfsError> {
        self.read_semantic(uri)
            .map(|text: Arc<str>| text.as_ref().to_string())
    }

    /// Resolve one Wendao URI and return shared UTF-8 content.
    ///
    /// Runtime lookup is cache-backed and performs lazy disk reads on cache miss.
    ///
    /// Lookup order for semantic URIs:
    /// 1. `content_cache`
    /// 2. Local semantic reference path indexed by [`super::super::SkillNamespaceIndex`]
    /// 3. Embedded `include_dir` resources (when enabled)
    ///
    /// Internal URIs resolve from mounted internal skill roots.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when URI resolution fails or content lookup fails.
    pub fn read_semantic(&self, uri: &str) -> Result<Arc<str>, SkillVfsError> {
        let parsed = WendaoResourceUri::parse(uri)?;
        let canonical_uri = parsed.canonical_uri();
        let cached = self
            .content_cache
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(canonical_uri.as_str())
            .cloned();
        if let Some(cached) = cached {
            return Ok(cached);
        }

        if parsed.is_internal_skill() {
            return self.read_internal_resource(&parsed, canonical_uri.as_str());
        }

        if let Some(shared) = self.read_local_semantic(&parsed, canonical_uri.as_str())? {
            return Ok(shared);
        }

        if let Some(shared) = self.read_mounted_semantic(&parsed, canonical_uri.as_str()) {
            return Ok(shared);
        }

        let semantic_known = self.index.mounts_for(parsed.semantic_name()).is_some()
            || self
                .embedded_mounts_by_semantic
                .contains_key(parsed.semantic_name());
        if !semantic_known {
            return Err(SkillVfsError::UnknownSemanticSkill {
                semantic_name: parsed.semantic_name().to_string(),
            });
        }
        Err(SkillVfsError::ResourceNotFound {
            semantic_name: parsed.semantic_name().to_string(),
            entity_name: parsed.entity_name().to_string(),
        })
    }

    /// Compatibility alias for existing callers that expect shared UTF-8 reads.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when URI resolution fails or content lookup fails.
    pub fn read_utf8_shared(&self, uri: &str) -> Result<Arc<str>, SkillVfsError> {
        self.read_semantic(uri)
    }

    fn read_internal_resource(
        &self,
        parsed: &WendaoResourceUri,
        canonical_uri: &str,
    ) -> Result<Arc<str>, SkillVfsError> {
        let path = self.resolve_parsed_uri(parsed)?;
        let content = std::fs::read_to_string(path.as_path()).map_err(|source| {
            SkillVfsError::ReadResource {
                path: path.clone(),
                source,
            }
        })?;
        let shared = Arc::<str>::from(content);
        self.content_cache
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(canonical_uri.to_string(), Arc::clone(&shared));
        Ok(shared)
    }

    fn read_mounted_semantic(
        &self,
        parsed: &WendaoResourceUri,
        canonical_uri: &str,
    ) -> Option<Arc<str>> {
        let mounts = self
            .embedded_mounts_by_semantic
            .get(parsed.semantic_name())?;
        let relative_entity = parsed.entity_relative_path();
        for mount in mounts {
            let Some(dir) = self.mounts.get(mount.crate_id.as_str()) else {
                continue;
            };
            let candidate = normalize_embedded_resource_path(
                mount.references_dir.join(relative_entity).as_path(),
            );
            let Some(file) = dir.get_file(candidate.as_str()) else {
                continue;
            };
            let Some(content) = file.contents_utf8() else {
                continue;
            };
            if content.trim().is_empty() {
                continue;
            }
            let shared = Arc::<str>::from(content);
            self.content_cache
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(canonical_uri.to_string(), Arc::clone(&shared));
            return Some(shared);
        }
        None
    }

    fn read_local_semantic(
        &self,
        parsed: &WendaoResourceUri,
        canonical_uri: &str,
    ) -> Result<Option<Arc<str>>, SkillVfsError> {
        let Some(path) = self.index.path_for_uri(parsed).cloned() else {
            return Ok(None);
        };
        let content = std::fs::read_to_string(path.as_path()).map_err(|source| {
            SkillVfsError::ReadResource {
                path: path.clone(),
                source,
            }
        })?;
        let shared = Arc::<str>::from(content);
        self.content_cache
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(canonical_uri.to_string(), Arc::clone(&shared));
        Ok(Some(shared))
    }
}

fn normalize_embedded_resource_path(path: &Path) -> String {
    path.to_string_lossy()
        .trim()
        .trim_start_matches("./")
        .replace('\\', "/")
}
