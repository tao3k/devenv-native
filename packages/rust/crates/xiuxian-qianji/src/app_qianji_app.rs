//! Application-level constructors for standard `Qianji` scheduler pipelines.

use super::build;
use super::presets::{MEMORY_PROMOTION_PIPELINE_TOML, RESEARCH_TRINITY_TOML};
use crate::QianjiLlmClient;
use crate::consensus::ConsensusManager;
use crate::error::QianjiError;
use crate::scheduler::QianjiScheduler;
use std::sync::Arc;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_wendao::link_graph::LinkGraphIndex;

/// Shared dependencies required to build a `Qianji` scheduler pipeline.
#[derive(Clone)]
pub struct QianjiPipelineDependencies {
    /// Search index used by Wendao-backed pipeline mechanisms.
    pub index: Arc<LinkGraphIndex>,
    /// Persona orchestrator used by Qianhuan-backed pipeline mechanisms.
    pub orchestrator: Arc<ThousandFacesOrchestrator>,
    /// Persona registry used for agent resolution.
    pub registry: Arc<PersonaRegistry>,
    /// Optional LLM client injected into LLM-capable nodes.
    pub llm_client: Option<Arc<QianjiLlmClient>>,
    /// Optional consensus manager for distributed calibration.
    pub consensus_manager: Option<Arc<ConsensusManager>>,
}

impl QianjiPipelineDependencies {
    /// Create dependencies without optional LLM or consensus services.
    #[must_use]
    pub fn new(
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
    ) -> Self {
        Self {
            index,
            orchestrator,
            registry,
            llm_client: None,
            consensus_manager: None,
        }
    }

    /// Attach an optional LLM client.
    #[must_use]
    pub fn with_llm_client(mut self, llm_client: Option<Arc<QianjiLlmClient>>) -> Self {
        self.llm_client = llm_client;
        self
    }

    /// Attach an optional consensus manager.
    #[must_use]
    pub fn with_consensus_manager(
        mut self,
        consensus_manager: Option<Arc<ConsensusManager>>,
    ) -> Self {
        self.consensus_manager = consensus_manager;
        self
    }
}

/// Request for building a scheduler from one manifest payload.
pub struct QianjiManifestPipelineRequest<'a> {
    /// TOML manifest payload.
    pub manifest_toml: &'a str,
    /// Pipeline dependencies.
    pub dependencies: QianjiPipelineDependencies,
}

/// Convenient entry point for deploying standard Qianji pipelines.
pub struct QianjiApp;

impl QianjiApp {
    /// Creates a scheduler from one TOML manifest payload.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when manifest compilation fails due to invalid
    /// topology, unsupported mechanisms, or dependency checks.
    pub fn create_pipeline_from_manifest(
        request: QianjiManifestPipelineRequest<'_>,
    ) -> Result<QianjiScheduler, QianjiError> {
        let QianjiManifestPipelineRequest {
            manifest_toml,
            dependencies,
        } = request;
        build::compile_scheduler(
            manifest_toml,
            dependencies.index,
            dependencies.orchestrator,
            dependencies.registry,
            dependencies.llm_client,
            dependencies.consensus_manager,
        )
    }

    /// Creates a standard high-precision research scheduler.
    ///
    /// This pipeline integrates Wendao knowledge search, Qianhuan persona
    /// annotation, and Synapse-Audit adversarial calibration.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when manifest compilation fails due to invalid
    /// topology, unsupported mechanism configuration, or dependency checks.
    pub fn create_research_pipeline(
        dependencies: QianjiPipelineDependencies,
    ) -> Result<QianjiScheduler, QianjiError> {
        Self::create_pipeline_from_manifest(QianjiManifestPipelineRequest {
            manifest_toml: RESEARCH_TRINITY_TOML,
            dependencies,
        })
    }

    /// Creates a standard `MemRL` promotion scheduler.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when manifest compilation fails due to invalid
    /// topology, unsupported mechanisms, or dependency checks.
    pub fn create_memory_promotion_pipeline(
        dependencies: QianjiPipelineDependencies,
    ) -> Result<QianjiScheduler, QianjiError> {
        Self::create_pipeline_from_manifest(QianjiManifestPipelineRequest {
            manifest_toml: MEMORY_PROMOTION_PIPELINE_TOML,
            dependencies,
        })
    }
}
