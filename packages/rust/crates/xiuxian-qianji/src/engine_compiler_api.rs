//! Engine compiler api surface for `xiuxian-qianji`.

use crate::QianjiLlmClient;
use crate::engine::QianjiEngine;
use crate::error::QianjiError;
use std::sync::Arc;
use xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator;
use xiuxian_qianhuan::persona::PersonaRegistry;
#[cfg(feature = "wendao-integration")]
use xiuxian_wendao::LinkGraphIndex;

use super::compiler::compile_manifest;

/// Orchestrates the conversion of TOML manifests into executable engines.
pub struct QianjiCompiler {
    #[cfg(feature = "wendao-integration")]
    pub(super) index: Arc<LinkGraphIndex>,
    pub(super) orchestrator: Arc<ThousandFacesOrchestrator>,
    pub(super) registry: Arc<PersonaRegistry>,
    #[cfg(feature = "llm")]
    pub(super) llm_client: Option<Arc<QianjiLlmClient>>,
}

impl QianjiCompiler {
    /// Creates a new compiler with provided trinity dependencies.
    #[cfg(all(feature = "llm", feature = "wendao-integration"))]
    #[must_use]
    pub fn new(
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        llm_client: Option<Arc<QianjiLlmClient>>,
    ) -> Self {
        Self {
            index,
            orchestrator,
            registry,
            llm_client,
        }
    }

    /// Creates a new compiler with provided trinity dependencies.
    #[cfg(all(not(feature = "llm"), feature = "wendao-integration"))]
    #[must_use]
    pub fn new(
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        _llm_client: Option<Arc<QianjiLlmClient>>,
    ) -> Self {
        Self {
            index,
            orchestrator,
            registry,
        }
    }

    /// Creates a new compiler for Qianji-only manifests.
    #[cfg(all(feature = "llm", not(feature = "wendao-integration")))]
    #[must_use]
    pub fn new(
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        llm_client: Option<Arc<QianjiLlmClient>>,
    ) -> Self {
        Self {
            orchestrator,
            registry,
            llm_client,
        }
    }

    /// Creates a new compiler for Qianji-only manifests.
    #[cfg(all(not(feature = "llm"), not(feature = "wendao-integration")))]
    #[must_use]
    pub fn new(
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        _llm_client: Option<Arc<QianjiLlmClient>>,
    ) -> Self {
        Self {
            orchestrator,
            registry,
        }
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
