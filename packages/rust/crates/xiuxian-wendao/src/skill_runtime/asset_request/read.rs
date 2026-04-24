use std::sync::Arc;

use super::types::AssetRequest;
use crate::skill_runtime::{SkillRuntimeError, SkillRuntimeResolver};

impl AssetRequest {
    /// Resolve the asset and return UTF-8 text using the provided resolver.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when URI resolution fails or file read fails.
    pub fn read_utf8(&self, resolver: &SkillRuntimeResolver) -> Result<String, SkillRuntimeError> {
        let uri = self.uri();
        resolver.read_utf8(uri).map(|text| text.trim().to_string())
    }

    /// Resolve the asset and return shared UTF-8 text using the provided resolver.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when URI resolution fails or file read fails.
    pub fn read_utf8_shared(
        &self,
        resolver: &SkillRuntimeResolver,
    ) -> Result<Arc<str>, SkillRuntimeError> {
        resolver.read_utf8_shared(self.uri())
    }

    /// Convenience wrapper to read text and trim it.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when URI resolution fails or file read fails.
    pub fn read_trimmed(
        &self,
        resolver: &SkillRuntimeResolver,
    ) -> Result<String, SkillRuntimeError> {
        self.read_utf8(resolver).map(|text| text.trim().to_string())
    }
}
