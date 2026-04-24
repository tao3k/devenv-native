use super::normalize::{normalize_package_id, normalize_relative_asset_path};
use super::types::{AssetRequest, WendaoAssetHandle};
use crate::skill_runtime::SkillRuntimeError;

impl WendaoAssetHandle {
    /// Builds one skill reference request:
    /// `wendao://skills/<semantic_name>/references/<relative_reference_path>`.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when semantic name or relative path is invalid.
    pub fn skill_reference_asset(
        semantic_name: &str,
        relative_reference_path: &str,
    ) -> Result<AssetRequest, SkillRuntimeError> {
        let normalized_semantic = normalize_package_id(semantic_name).ok_or_else(|| {
            SkillRuntimeError::InvalidUri(format!(
                "wendao://skills/{semantic_name}/references/{relative_reference_path}"
            ))
        })?;
        let normalized_reference = normalize_relative_asset_path(relative_reference_path)?;
        Ok(AssetRequest::new(format!(
            "wendao://skills/{normalized_semantic}/references/{normalized_reference}"
        )))
    }
}
