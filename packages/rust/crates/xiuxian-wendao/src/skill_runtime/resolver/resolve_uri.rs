use std::path::PathBuf;
use xiuxian_wendao_core::WendaoResourceUri;

use super::core::SkillRuntimeResolver;
use crate::skill_runtime::SkillRuntimeError;

impl SkillRuntimeResolver {
    /// Resolve one semantic URI to concrete file path.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when URI parsing fails, namespace is unknown,
    /// or no matching reference document exists.
    pub fn resolve_path(&self, uri: &str) -> Result<PathBuf, SkillRuntimeError> {
        let parsed = WendaoResourceUri::parse(uri)?;
        self.resolve_parsed_uri(&parsed)
    }

    /// Resolve one parsed URI to concrete file path.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when namespace is unknown or resource is missing.
    pub fn resolve_parsed_uri(
        &self,
        uri: &WendaoResourceUri,
    ) -> Result<PathBuf, SkillRuntimeError> {
        let Some(path) = self.index.path_for_uri(uri).cloned() else {
            let Some(_mounts) = self.index.mounts_for(uri.semantic_name()) else {
                return Err(SkillRuntimeError::UnknownSemanticSkill {
                    semantic_name: uri.semantic_name().to_string(),
                });
            };
            return Err(SkillRuntimeError::ResourceNotFound {
                semantic_name: uri.semantic_name().to_string(),
                entity_name: uri.entity_name().to_string(),
            });
        };

        Ok(path)
    }
}
