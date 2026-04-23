use super::build;
use super::presets::{MEMORY_PROMOTION_PIPELINE_TOML, RESEARCH_TRINITY_TOML};
use crate::QianjiLlmClient;
use crate::consensus::ConsensusManager;
use crate::error::QianjiError;
use crate::scheduler::QianjiScheduler;
use std::sync::Arc;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_wendao::link_graph::LinkGraphIndex;

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
        manifest_toml: &str,
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        llm_client: Option<Arc<QianjiLlmClient>>,
    ) -> Result<QianjiScheduler, QianjiError> {
        Self::create_pipeline_from_manifest_with_consensus(
            manifest_toml,
            index,
            orchestrator,
            registry,
            llm_client,
            None,
        )
    }

    /// Creates a scheduler from one TOML manifest payload with optional
    /// distributed consensus manager.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when manifest compilation fails due to invalid
    /// topology, unsupported mechanisms, or dependency checks.
    pub fn create_pipeline_from_manifest_with_consensus(
        manifest_toml: &str,
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        llm_client: Option<Arc<QianjiLlmClient>>,
        consensus_manager: Option<Arc<ConsensusManager>>,
    ) -> Result<QianjiScheduler, QianjiError> {
        build::compile_scheduler(
            manifest_toml,
            index,
            orchestrator,
            registry,
            llm_client,
            consensus_manager,
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
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        llm_client: Option<Arc<QianjiLlmClient>>,
    ) -> Result<QianjiScheduler, QianjiError> {
        Self::create_research_pipeline_with_consensus(
            index,
            orchestrator,
            registry,
            llm_client,
            None,
        )
    }

    /// Creates a standard high-precision research scheduler with optional
    /// distributed consensus manager.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when manifest compilation fails due to invalid
    /// topology, unsupported mechanism configuration, or dependency checks.
    pub fn create_research_pipeline_with_consensus(
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        llm_client: Option<Arc<QianjiLlmClient>>,
        consensus_manager: Option<Arc<ConsensusManager>>,
    ) -> Result<QianjiScheduler, QianjiError> {
        build::compile_scheduler(
            RESEARCH_TRINITY_TOML,
            index,
            orchestrator,
            registry,
            llm_client,
            consensus_manager,
        )
    }

    /// Creates a standard `MemRL` promotion scheduler.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when manifest compilation fails due to invalid
    /// topology, unsupported mechanisms, or dependency checks.
    pub fn create_memory_promotion_pipeline(
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        llm_client: Option<Arc<QianjiLlmClient>>,
    ) -> Result<QianjiScheduler, QianjiError> {
        Self::create_memory_promotion_pipeline_with_consensus(
            index,
            orchestrator,
            registry,
            llm_client,
            None,
        )
    }

    /// Creates a standard `MemRL` promotion scheduler with optional
    /// distributed consensus manager.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when manifest compilation fails due to invalid
    /// topology, unsupported mechanisms, or dependency checks.
    pub fn create_memory_promotion_pipeline_with_consensus(
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        llm_client: Option<Arc<QianjiLlmClient>>,
        consensus_manager: Option<Arc<ConsensusManager>>,
    ) -> Result<QianjiScheduler, QianjiError> {
        build::compile_scheduler(
            MEMORY_PROMOTION_PIPELINE_TOML,
            index,
            orchestrator,
            registry,
            llm_client,
            consensus_manager,
        )
    }
}
