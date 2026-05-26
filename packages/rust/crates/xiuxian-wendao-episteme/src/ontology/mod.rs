//! Ontology source-contract admission.

#[cfg(feature = "foyer-artifact-cache")]
mod artifact_bundle;
mod bootstrap_pipeline;
mod candidate_review;
mod candidates;
mod extension_validation;
mod manifest;
mod promotion_apply_plan;
mod promotion_review;
mod qianji_review_candidates;
mod rdf_draft;
mod reasoning_context_shard;
mod reasoning_target;
mod registry;
mod review_ledger;
mod search_strategy_oracle;
mod source_patch_apply;
mod source_patch_apply_plan;
mod source_patch_draft;
mod source_patch_preflight;
mod source_patch_rdf_read_model;
mod source_patch_review_packet;
mod source_patch_semantic_preview;
mod structural_facts;
mod structural_facts_reasoning_fill_plan;
mod structural_facts_reasoning_ledger_seed;
mod structural_facts_reasoning_packet;
mod structural_facts_reasoning_qianji_schedule_plan;

#[cfg(feature = "foyer-artifact-cache")]
pub use artifact_bundle::{
    EpistemeOntologyArtifactBundleIdentity, EpistemeOntologyArtifactBundleKind,
    EpistemeOntologyArtifactBundleRestoreReport, EpistemeOntologyArtifactBundleWriteReport,
    restore_episteme_ontology_artifact_bundle, write_episteme_ontology_artifact_bundle,
};
#[cfg(feature = "foyer-artifact-cache")]
pub use bootstrap_pipeline::{
    EpistemeOntologyBootstrapArtifactCacheOptions,
    EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome,
    EpistemeOntologyBootstrapArtifactCacheReadThroughReport,
    EpistemeOntologyBootstrapArtifactCacheReport,
    EpistemeOntologyBootstrapArtifactCacheRestoreMiss,
    EpistemeOntologyBootstrapArtifactCacheRestoreReport,
    EpistemeOntologyBootstrapArtifactCacheStage,
    admit_episteme_ontology_bootstrap_artifact_cache_options,
    read_through_episteme_ontology_bootstrap_artifacts,
    restore_episteme_ontology_bootstrap_pipeline_artifacts,
    run_episteme_ontology_bootstrap_pipeline_with_artifact_cache,
};
pub use bootstrap_pipeline::{
    EpistemeOntologyBootstrapPipelineReport, EpistemeOntologyBootstrapPipelineRequest,
    EpistemeOntologyBootstrapPipelineSafetyFlags, run_episteme_ontology_bootstrap_pipeline,
};
pub use candidate_review::{
    EpistemeOntologyCandidateReviewReport, EpistemeOntologyCandidateReviewRequest,
    review_episteme_ontology_candidates,
};
pub use candidates::{
    EpistemeOntologyCandidateGenerationReport, EpistemeOntologyCandidateGenerationRequest,
    EpistemeOntologyCandidateReadModelMissingEndpoint,
    EpistemeOntologyCandidateReadModelSummaryReport,
    EpistemeOntologyCandidateReadModelSummaryRequest, generate_episteme_ontology_candidates,
    summarize_episteme_ontology_candidate_read_model,
};
pub use extension_validation::{
    EpistemeExtensionValidationMode, EpistemeExtensionValidationReport,
    EpistemeExtensionValidationRequest, validate_episteme_extension_contract,
};
pub use manifest::{
    EpistemeOntologyApiSurface, EpistemeOntologyArtifactMode, EpistemeOntologyBoundaries,
    EpistemeOntologyContractReport, EpistemeOntologyDomain, EpistemeOntologyDomainCategory,
    EpistemeOntologyError, EpistemeOntologyExtends, EpistemeOntologyExtensionContract,
    EpistemeOntologyManifest, ONTOLOGY_MANIFEST_RELATIVE_PATH, ontology_manifest_path,
    read_ontology_manifest, validate_ontology_contract,
};
pub use promotion_apply_plan::{
    EpistemeOntologyPromotionApplyPlanReport, EpistemeOntologyPromotionApplyPlanRequest,
    write_episteme_ontology_promotion_apply_plan,
};
pub use promotion_review::{
    EpistemeOntologyPromotionReviewPacketReport, EpistemeOntologyPromotionReviewPacketRequest,
    write_episteme_ontology_promotion_review_packet,
};
pub use qianji_review_candidates::{
    EpistemeOntologyQianjiReviewCandidateImportReport,
    EpistemeOntologyQianjiReviewCandidateImportRequest,
    import_episteme_ontology_qianji_review_candidates,
};
pub use rdf_draft::{
    EpistemeOntologyRdfDraftExportReport, EpistemeOntologyRdfDraftExportRequest,
    export_episteme_ontology_rdf_draft,
};
pub use registry::{
    EpistemeOntologyRegistryActionType, EpistemeOntologyRegistryApiSurface,
    EpistemeOntologyRegistryArtifactMode, EpistemeOntologyRegistryCategory,
    EpistemeOntologyRegistryDatasetMapping, EpistemeOntologyRegistryDomain,
    EpistemeOntologyRegistryError, EpistemeOntologyRegistryInterfaceType,
    EpistemeOntologyRegistryKind, EpistemeOntologyRegistryLinkType,
    EpistemeOntologyRegistryObjectPropertyTerm, EpistemeOntologyRegistryObjectType,
    EpistemeOntologyRegistryObjectTypeRef, EpistemeOntologyRegistryPolicy,
    EpistemeOntologyRegistryQueryType, EpistemeOntologyRegistryRdfClassTerm,
    EpistemeOntologyRegistryRdfTerms, EpistemeOntologyRegistryReadModelInput,
    EpistemeOntologyRegistryRule, EpistemeOntologyRegistrySnapshot,
    EpistemeOntologyRegistrySnapshotReport, EpistemeOntologyRegistrySourceContract,
    ONTOLOGY_REGISTRY_RELATIVE_PATH, admit_ontology_registry_snapshot, ontology_registry_path,
    read_ontology_registry_snapshot, validate_ontology_registry_snapshot,
};
pub use search_strategy_oracle::{
    EpistemeSearchStrategyOracleReport, EpistemeSearchStrategyOracleRequest,
    write_episteme_search_strategy_oracle,
};
pub use source_patch_apply::{
    EpistemeOntologySourcePatchAppliedTarget, EpistemeOntologySourcePatchApplyPreviewReport,
    EpistemeOntologySourcePatchApplyPreviewRequest, EpistemeOntologySourcePatchApplyPreviewTarget,
    EpistemeOntologySourcePatchApplyReport, EpistemeOntologySourcePatchApplyRequest,
    apply_episteme_ontology_source_patch, write_episteme_ontology_source_patch_apply_preview,
};
pub use source_patch_apply_plan::{
    EpistemeOntologySourcePatchApplyPlanReport, EpistemeOntologySourcePatchApplyPlanRequest,
    write_episteme_ontology_source_patch_apply_plan,
};
pub use source_patch_draft::{
    EpistemeOntologySourcePatchDraftReport, EpistemeOntologySourcePatchDraftRequest,
    export_episteme_ontology_source_patch_draft,
};
pub use source_patch_preflight::{
    EpistemeOntologySourcePatchPreflightReport, EpistemeOntologySourcePatchPreflightRequest,
    write_episteme_ontology_source_patch_preflight,
};
pub use source_patch_rdf_read_model::{
    EpistemeOntologySourcePatchRdfReadModelReport, EpistemeOntologySourcePatchRdfReadModelRequest,
    write_episteme_ontology_source_patch_rdf_read_model,
};
pub use source_patch_review_packet::{
    EpistemeOntologySourcePatchReviewPacketReport, EpistemeOntologySourcePatchReviewPacketRequest,
    EpistemeOntologySourcePatchReviewPacketTarget,
    write_episteme_ontology_source_patch_review_packet,
};
pub use source_patch_semantic_preview::{
    EpistemeOntologySemanticEvidenceRow, EpistemeOntologySemanticObjectRow,
    EpistemeOntologySemanticProjectionStateRow, EpistemeOntologySemanticRelationRow,
    EpistemeOntologySourcePatchSemanticPreviewReport,
    EpistemeOntologySourcePatchSemanticPreviewRequest,
    write_episteme_ontology_source_patch_semantic_preview,
};
pub use structural_facts::{
    EpistemeOntologyStructuralFactsConfiguredRequest, EpistemeOntologyStructuralFactsReport,
    EpistemeOntologyStructuralFactsRequest, EpistemeOntologyStructuralFactsValidationMode,
    write_episteme_ontology_structural_facts, write_episteme_ontology_structural_facts_from_config,
};
pub use structural_facts_reasoning_fill_plan::{
    EpistemeOntologyStructuralFactsReasoningFillPlanReport,
    EpistemeOntologyStructuralFactsReasoningFillPlanRequest,
    write_episteme_ontology_structural_facts_reasoning_fill_plan,
};
pub use structural_facts_reasoning_ledger_seed::{
    EpistemeOntologyStructuralFactsReasoningLedgerSeedReport,
    EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest,
    write_episteme_ontology_structural_facts_reasoning_ledger_seed,
};
pub use structural_facts_reasoning_packet::{
    EpistemeOntologyStructuralFactsReasoningPacketReport,
    EpistemeOntologyStructuralFactsReasoningPacketRequest,
    write_episteme_ontology_structural_facts_reasoning_packet,
};
pub use structural_facts_reasoning_qianji_schedule_plan::{
    EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanReport,
    EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest,
    write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan,
};
