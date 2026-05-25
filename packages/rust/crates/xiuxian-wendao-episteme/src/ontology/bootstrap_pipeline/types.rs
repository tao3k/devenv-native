use std::path::{Path, PathBuf};

use serde::{Serialize, Serializer, ser::SerializeStruct};

use super::{
    reasoning_fill_plan_stage_run_id, reasoning_ledger_seed_stage_run_id,
    reasoning_packet_stage_run_id, structural_facts_stage_run_id,
};
#[cfg(feature = "artifact-cache")]
use crate::{
    EpistemeOntologyArtifactBundleRestoreReport, EpistemeOntologyArtifactBundleWriteReport,
};
use crate::{
    EpistemeOntologyStructuralFactsReasoningFillPlanReport,
    EpistemeOntologyStructuralFactsReasoningLedgerSeedReport,
    EpistemeOntologyStructuralFactsReasoningPacketReport, EpistemeOntologyStructuralFactsReport,
    EpistemeOntologyStructuralFactsValidationMode,
};

/// Request for running the deterministic ontology bootstrap pipeline.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyBootstrapPipelineRequest {
    episteme_root: PathBuf,
    corpus_root: Option<PathBuf>,
    structure_run_root: Option<PathBuf>,
    ontology_generation_run_root: Option<PathBuf>,
    run_id: String,
    structural_facts_run_id: String,
    reasoning_packet_run_id: String,
    reasoning_ledger_seed_run_id: String,
    reasoning_fill_plan_run_id: String,
    validation_mode: EpistemeOntologyStructuralFactsValidationMode,
    category: Option<String>,
    route: Option<String>,
    reasoning_packet_limit: usize,
    reasoning_ledger_seed_limit: usize,
    reasoning_fill_plan_limit: usize,
}

impl EpistemeOntologyBootstrapPipelineRequest {
    /// Create a deterministic ontology bootstrap pipeline request.
    #[must_use]
    pub fn new(episteme_root: impl Into<PathBuf>, run_id: impl Into<String>) -> Self {
        let run_id = run_id.into();
        Self {
            episteme_root: episteme_root.into(),
            corpus_root: None,
            structure_run_root: None,
            ontology_generation_run_root: None,
            structural_facts_run_id: structural_facts_stage_run_id(run_id.as_str()),
            reasoning_packet_run_id: reasoning_packet_stage_run_id(run_id.as_str()),
            reasoning_ledger_seed_run_id: reasoning_ledger_seed_stage_run_id(run_id.as_str()),
            reasoning_fill_plan_run_id: reasoning_fill_plan_stage_run_id(run_id.as_str()),
            run_id,
            validation_mode: EpistemeOntologyStructuralFactsValidationMode::default(),
            category: None,
            route: None,
            reasoning_packet_limit: 256,
            reasoning_ledger_seed_limit: 512,
            reasoning_fill_plan_limit: 1024,
        }
    }

    /// Override the source corpus root instead of resolving it from config.
    #[must_use]
    pub fn with_corpus_root(mut self, corpus_root: impl Into<PathBuf>) -> Self {
        self.corpus_root = Some(corpus_root.into());
        self
    }

    /// Override the structure run root instead of resolving it from config.
    #[must_use]
    pub fn with_structure_run_root(mut self, run_root: impl Into<PathBuf>) -> Self {
        self.structure_run_root = Some(run_root.into());
        self
    }

    /// Override the ontology-generation run root instead of resolving it from config.
    #[must_use]
    pub fn with_ontology_generation_run_root(mut self, run_root: impl Into<PathBuf>) -> Self {
        self.ontology_generation_run_root = Some(run_root.into());
        self
    }

    /// Override deterministic stage run ids.
    #[must_use]
    pub fn with_stage_run_ids(
        mut self,
        structural_facts_run_id: impl Into<String>,
        reasoning_packet_run_id: impl Into<String>,
        reasoning_ledger_seed_run_id: impl Into<String>,
        reasoning_fill_plan_run_id: impl Into<String>,
    ) -> Self {
        self.structural_facts_run_id = structural_facts_run_id.into();
        self.reasoning_packet_run_id = reasoning_packet_run_id.into();
        self.reasoning_ledger_seed_run_id = reasoning_ledger_seed_run_id.into();
        self.reasoning_fill_plan_run_id = reasoning_fill_plan_run_id.into();
        self
    }

    /// Set structural facts validation mode.
    #[must_use]
    pub fn with_validation_mode(
        mut self,
        validation_mode: EpistemeOntologyStructuralFactsValidationMode,
    ) -> Self {
        self.validation_mode = validation_mode;
        self
    }

    /// Restrict reasoning packet rows to one source category.
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Restrict reasoning packet rows to one extraction route.
    #[must_use]
    pub fn with_route(mut self, route: impl Into<String>) -> Self {
        self.route = Some(route.into());
        self
    }

    /// Set the maximum number of reasoning packet rows.
    #[must_use]
    pub fn with_reasoning_packet_limit(mut self, limit: usize) -> Self {
        self.reasoning_packet_limit = limit;
        self
    }

    /// Set the maximum number of ledger seed packet rows.
    #[must_use]
    pub fn with_reasoning_ledger_seed_limit(mut self, limit: usize) -> Self {
        self.reasoning_ledger_seed_limit = limit;
        self
    }

    /// Set the maximum number of fill-plan seed rows.
    #[must_use]
    pub fn with_reasoning_fill_plan_limit(mut self, limit: usize) -> Self {
        self.reasoning_fill_plan_limit = limit;
        self
    }

    #[must_use]
    pub(super) fn episteme_root(&self) -> &Path {
        self.episteme_root.as_path()
    }

    #[must_use]
    pub(super) fn corpus_root(&self) -> Option<&Path> {
        self.corpus_root.as_deref()
    }

    #[must_use]
    pub(super) fn structure_run_root(&self) -> Option<&Path> {
        self.structure_run_root.as_deref()
    }

    #[must_use]
    pub(super) fn ontology_generation_run_root(&self) -> Option<&Path> {
        self.ontology_generation_run_root.as_deref()
    }

    #[must_use]
    pub(super) fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    #[must_use]
    pub(super) fn structural_facts_run_id(&self) -> &str {
        self.structural_facts_run_id.as_str()
    }

    #[must_use]
    pub(super) fn reasoning_packet_run_id(&self) -> &str {
        self.reasoning_packet_run_id.as_str()
    }

    #[must_use]
    pub(super) fn reasoning_ledger_seed_run_id(&self) -> &str {
        self.reasoning_ledger_seed_run_id.as_str()
    }

    #[must_use]
    pub(super) fn reasoning_fill_plan_run_id(&self) -> &str {
        self.reasoning_fill_plan_run_id.as_str()
    }

    #[must_use]
    pub(super) fn validation_mode(&self) -> EpistemeOntologyStructuralFactsValidationMode {
        self.validation_mode
    }

    #[must_use]
    pub(super) fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    #[must_use]
    pub(super) fn route(&self) -> Option<&str> {
        self.route.as_deref()
    }

    #[must_use]
    pub(super) fn reasoning_packet_limit(&self) -> usize {
        self.reasoning_packet_limit
    }

    #[must_use]
    pub(super) fn reasoning_ledger_seed_limit(&self) -> usize {
        self.reasoning_ledger_seed_limit
    }

    #[must_use]
    pub(super) fn reasoning_fill_plan_limit(&self) -> usize {
        self.reasoning_fill_plan_limit
    }
}

/// Report emitted by the deterministic ontology bootstrap pipeline.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyBootstrapPipelineReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// User-selected pipeline run id.
    pub run_id: String,
    /// Episteme repository root.
    pub episteme_root: PathBuf,
    /// Ontology-generation run root used by downstream deterministic stages.
    pub ontology_generation_run_root: PathBuf,
    /// Structural facts stage report.
    pub structural_facts: EpistemeOntologyStructuralFactsReport,
    /// Reasoning packet stage report.
    pub reasoning_packet: EpistemeOntologyStructuralFactsReasoningPacketReport,
    /// Reasoning ledger seed stage report.
    pub reasoning_ledger_seed: EpistemeOntologyStructuralFactsReasoningLedgerSeedReport,
    /// Reasoning fill-plan stage report.
    pub reasoning_fill_plan: EpistemeOntologyStructuralFactsReasoningFillPlanReport,
    /// Pipeline-level safety flags.
    #[serde(flatten)]
    pub safety: EpistemeOntologyBootstrapPipelineSafetyFlags,
}

/// Artifact-cache identity controls for bootstrap pipeline run bundles.
#[cfg(feature = "artifact-cache")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyBootstrapArtifactCacheOptions {
    /// Source contract, registry, or corpus digest component.
    pub source_digest: String,
    /// Compiler, validation, or ontology profile digest component.
    pub profile_digest: String,
}

#[cfg(feature = "artifact-cache")]
impl EpistemeOntologyBootstrapArtifactCacheOptions {
    /// Create artifact-cache options for bootstrap pipeline bundles.
    #[must_use]
    pub fn new(source_digest: impl Into<String>, profile_digest: impl Into<String>) -> Self {
        Self {
            source_digest: source_digest.into(),
            profile_digest: profile_digest.into(),
        }
    }
}

/// Report emitted by the artifact-cache bootstrap wrapper.
#[cfg(feature = "artifact-cache")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemeOntologyBootstrapArtifactCacheReport {
    /// Original deterministic pipeline report.
    pub pipeline: EpistemeOntologyBootstrapPipelineReport,
    /// Artifact bundle writes produced from generated run directories.
    pub bundles: Vec<EpistemeOntologyArtifactBundleWriteReport>,
}

/// Bootstrap stage represented in artifact-cache reports.
#[cfg(feature = "artifact-cache")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemeOntologyBootstrapArtifactCacheStage {
    /// Structural facts stage directory.
    StructuralFacts,
    /// Reasoning packet stage directory.
    ReasoningPacket,
    /// Reasoning ledger seed stage directory.
    ReasoningLedgerSeed,
    /// Reasoning fill-plan stage directory.
    ReasoningFillPlan,
}

/// Missing bootstrap stage bundle during artifact-cache restore.
#[cfg(feature = "artifact-cache")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemeOntologyBootstrapArtifactCacheRestoreMiss {
    /// Missing stage.
    pub stage: EpistemeOntologyBootstrapArtifactCacheStage,
    /// Stage-qualified run digest used for the artifact key.
    pub run_digest: String,
    /// Directory that would receive restored files.
    pub target_dir: PathBuf,
}

/// Report emitted after restoring bootstrap stage bundles from artifact cache.
#[cfg(feature = "artifact-cache")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemeOntologyBootstrapArtifactCacheRestoreReport {
    /// Stage bundles restored into deterministic run directories.
    pub restored: Vec<EpistemeOntologyArtifactBundleRestoreReport>,
    /// Stage bundles not found in the cache backend.
    pub missing: Vec<EpistemeOntologyBootstrapArtifactCacheRestoreMiss>,
}

#[cfg(feature = "artifact-cache")]
impl EpistemeOntologyBootstrapArtifactCacheRestoreReport {
    /// Whether every expected bootstrap stage bundle was restored.
    #[must_use]
    pub fn complete(&self) -> bool {
        self.missing.is_empty()
    }
}

/// Outcome of a bootstrap artifact read-through attempt.
#[cfg(feature = "artifact-cache")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome {
    /// Existing artifact bundles restored all stage directories.
    Restored,
    /// One or more bundles were missing, so the pipeline regenerated artifacts.
    Generated,
}

/// Report emitted by bootstrap artifact read-through.
#[cfg(feature = "artifact-cache")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemeOntologyBootstrapArtifactCacheReadThroughReport {
    /// Whether the read-through restored or generated artifacts.
    pub outcome: EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome,
    /// Restore attempt performed before generation.
    pub restore: EpistemeOntologyBootstrapArtifactCacheRestoreReport,
    /// Generation report when restore was incomplete.
    pub generated: Option<EpistemeOntologyBootstrapArtifactCacheReport>,
}

/// Pipeline-level non-execution and non-promotion safety flags.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyBootstrapPipelineSafetyFlags {
    enabled: &'static [EpistemeOntologyBootstrapPipelineSafetyFlag],
}

impl EpistemeOntologyBootstrapPipelineSafetyFlags {
    /// Return the deterministic non-executing bootstrap safety state.
    #[must_use]
    pub const fn deterministic_non_mutating() -> Self {
        Self { enabled: &[] }
    }

    /// Whether the pipeline read private source text for inference.
    #[must_use]
    pub fn source_text_read(&self) -> bool {
        self.enabled(EpistemeOntologyBootstrapPipelineSafetyFlag::SourceTextRead)
    }

    /// Whether the pipeline called a live LLM.
    #[must_use]
    pub fn llm_executed(&self) -> bool {
        self.enabled(EpistemeOntologyBootstrapPipelineSafetyFlag::LlmExecuted)
    }

    /// Whether the pipeline executed a workflow runtime.
    #[must_use]
    pub fn workflow_executed(&self) -> bool {
        self.enabled(EpistemeOntologyBootstrapPipelineSafetyFlag::WorkflowExecuted)
    }

    /// Whether the pipeline authorizes source mutation.
    #[must_use]
    pub fn source_mutation_allowed(&self) -> bool {
        self.enabled(EpistemeOntologyBootstrapPipelineSafetyFlag::SourceMutationAllowed)
    }

    /// Whether the pipeline authorizes RDF mutation.
    #[must_use]
    pub fn rdf_mutation_allowed(&self) -> bool {
        self.enabled(EpistemeOntologyBootstrapPipelineSafetyFlag::RdfMutationAllowed)
    }

    /// Whether produced rows are ontology truth.
    #[must_use]
    pub fn ontology_truth(&self) -> bool {
        self.enabled(EpistemeOntologyBootstrapPipelineSafetyFlag::OntologyTruth)
    }

    fn enabled(&self, flag: EpistemeOntologyBootstrapPipelineSafetyFlag) -> bool {
        self.enabled.contains(&flag)
    }
}

impl Serialize for EpistemeOntologyBootstrapPipelineSafetyFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state =
            serializer.serialize_struct("EpistemeOntologyBootstrapPipelineSafetyFlags", 6)?;
        state.serialize_field("sourceTextRead", &self.source_text_read())?;
        state.serialize_field("llmExecuted", &self.llm_executed())?;
        state.serialize_field("workflowExecuted", &self.workflow_executed())?;
        state.serialize_field("sourceMutationAllowed", &self.source_mutation_allowed())?;
        state.serialize_field("rdfMutationAllowed", &self.rdf_mutation_allowed())?;
        state.serialize_field("ontologyTruth", &self.ontology_truth())?;
        state.end()
    }
}

/// Bootstrap safety dimensions represented as explicit risk markers.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpistemeOntologyBootstrapPipelineSafetyFlag {
    /// Pipeline read private source text for inference.
    SourceTextRead,
    /// Pipeline called a live LLM.
    LlmExecuted,
    /// Pipeline executed a workflow runtime.
    WorkflowExecuted,
    /// Pipeline authorized source mutation.
    SourceMutationAllowed,
    /// Pipeline authorized RDF mutation.
    RdfMutationAllowed,
    /// Pipeline output was promoted as ontology truth.
    OntologyTruth,
}
