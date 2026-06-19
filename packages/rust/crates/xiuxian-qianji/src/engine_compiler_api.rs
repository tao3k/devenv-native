//! Engine compiler api surface for `xiuxian-qianji`.

use crate::engine::QianjiEngine;
use crate::error::QianjiError;
#[cfg(feature = "wendao-integration")]
use std::sync::Arc;
#[cfg(feature = "wendao-integration")]
use xiuxian_wendao::LinkGraphIndex;

use super::compiler::compile_manifest;

/// Orchestrates the conversion of TOML manifests into executable engines.
pub struct QianjiCompiler {
    #[cfg(feature = "wendao-integration")]
    pub(super) index: Arc<LinkGraphIndex>,
}

impl QianjiCompiler {
    /// Creates a new compiler with provided Wendao dependencies.
    #[cfg(feature = "wendao-integration")]
    #[must_use]
    pub fn new(index: Arc<LinkGraphIndex>) -> Self {
        Self { index }
    }

    /// Creates a new compiler for Qianji-only manifests.
    #[cfg(not(feature = "wendao-integration"))]
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Compiles a TOML manifest into a ready-to-run `QianjiEngine`.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when TOML parsing fails, a task type is unsupported,
    /// required dependencies are missing, manifest edges reference unknown nodes,
    /// or the graph contains static cycles.
    pub fn compile(&self, manifest_toml: &str) -> Result<QianjiEngine, QianjiError> {
        compile_manifest(self, manifest_toml)
    }
}
