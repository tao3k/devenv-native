//! Wendao-owned episteme source-contract services.

mod source_contract;

pub use xiuxian_wendao_parsers::EpistemeFileRow;

#[cfg(feature = "julia")]
pub use source_contract::build_episteme_wendaograph_quality_request_batches;
pub use source_contract::{
    EpistemeAudioClaimPromotionProposalReport, EpistemeAudioClaimPromotionProposalRequest,
    EpistemeAudioEvidenceReadModelRequest, EpistemeAudioEvidenceSegmentRow,
    EpistemeAudioEvidenceSourceRow, EpistemeAudioReviewedClaimObjectKind,
    EpistemeAudioReviewedClaimReadModelRequest, EpistemeAudioReviewedClaimRow, EpistemeError,
    EpistemeEvidenceByteSizeStatus, EpistemeEvidenceReadReport, EpistemeEvidenceReadRequest,
    EpistemeEvidenceReadValidationMode, EpistemeEvidenceSelectionPlanRequest,
    EpistemeEvidenceSelectionReceipt, EpistemeEvidenceSelectionRow,
    EpistemeEvidenceSelectionValidationMode, EpistemeEvidenceSelectionWriteReport,
    EpistemeEvidenceSha256Status, EpistemeEvidenceSourceAvailability, EpistemeEvidenceSourceRef,
    EpistemeEvidenceTextPreview, EpistemeReadModelMaterialization, EpistemeReadModelRequest,
    EpistemeReadModelTable, EpistemeRegistryDomainId, EpistemeRegistryDuplicateDomainId,
    EpistemeRegistryEntry, EpistemeRegistryError, EpistemeRegistryGitMaterializationError,
    EpistemeRegistryId, EpistemeRegistryInvalidDomainId, EpistemeRegistryLoadReceipt,
    EpistemeRegistryMissingExtensionTarget, EpistemeRegistryReferenceGraphEntry,
    EpistemeRegistryReferenceGraphLink, EpistemeRegistryReferenceGraphReceipt,
    EpistemeRunPlanReceipt, EpistemeRunPlanRequest, EpistemeRunPlanWriteReport, EpistemeRunTask,
    EpistemeRuntimeConfig, EpistemeStructureTocReceipt, EpistemeStructureTocRequest,
    EpistemeStructureTocValidationMode, EpistemeStructureTocWriteReport,
    EpistemeValidationHashCacheReport, EpistemeValidationReport, LoadedEpistemeRegistryEntry,
    LoadedEpistemeSourceKind,
    admit_and_materialize_episteme_ontology_registry_snapshot_read_model_seed,
    configured_episteme_corpus_root_env, load_episteme_registry_entries,
    load_episteme_registry_entries_with_mode, load_episteme_runtime_config,
    materialize_episteme_audio_evidence_review_seed,
    materialize_episteme_audio_reviewed_claim_seed,
    materialize_episteme_ontology_registry_snapshot_read_model_seed,
    materialize_episteme_read_model_seed,
    materialize_episteme_read_model_seed_with_validation_hash_cache,
    materialize_episteme_registry_reference_graph_read_model_seed, plan_episteme_extraction_run,
    read_episteme_evidence, read_episteme_evidence_selection_file_ids,
    validate_episteme_read_model_relation_endpoints, validate_episteme_registry_reference_graph,
    validate_episteme_source_contract, validate_episteme_source_contract_with_hash_cache,
    write_episteme_audio_claim_promotion_proposal, write_episteme_evidence_selection_plan,
    write_episteme_extraction_run_plan, write_episteme_structure_toc,
};
