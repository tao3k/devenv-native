//! Rust-owned Episteme source-contract runtime boundary.
//!
//! Parser syntax is delegated to `xiuxian-wendao-parsers`; this crate owns the
//! deterministic Rust admission surface for Episteme source contracts.

mod cache;
mod ontology;
mod source_contract;

pub use cache::{
    EPISTEME_DOCLING_DOCUMENT_RESULTS_JSONL, EPISTEME_DOCLING_DOCUMENT_ROUTE,
    EPISTEME_DOCLING_DOCUMENT_WRAPPER_SCHEMA, EPISTEME_IMAGE_OCR_RESULTS_JSONL,
    EPISTEME_IMAGE_OCR_ROUTE, EPISTEME_IMAGE_OCR_WRAPPER_SCHEMA,
    EPISTEME_LEGACY_OFFICE_CONVERSION_RECEIPT_JSON,
    EPISTEME_LEGACY_OFFICE_CONVERSION_WRAPPER_SCHEMA, EPISTEME_LEGACY_OFFICE_DOCUMENT_ROUTE,
    EpistemeCacheTask, EpistemeCacheTaskCategory, EpistemeCacheTaskStatus,
    EpistemeDoclingDocumentCacheBridgeReport, EpistemeImageOcrCacheBridgeReport,
    EpistemeLegacyOfficeConversionReport, EpistemeLegacyOfficeConversionRequest,
    convert_legacy_office_tasks, read_docling_document_tasks_tsv, read_image_ocr_tasks_tsv,
    read_legacy_office_conversion_tasks_tsv, skipped_docling_document_cache_bridge_report,
    skipped_image_ocr_cache_bridge_report, validate_docling_document_tasks,
    validate_image_ocr_tasks, validate_legacy_office_conversion_tasks,
    write_docling_document_cache_outputs, write_image_ocr_cache_outputs,
};
pub use ontology::{
    EpistemeOntologyApiSurface, EpistemeOntologyArtifactMode, EpistemeOntologyBoundaries,
    EpistemeOntologyCandidateGenerationReport, EpistemeOntologyCandidateGenerationRequest,
    EpistemeOntologyCandidateReviewReport, EpistemeOntologyCandidateReviewRequest,
    EpistemeOntologyContractReport, EpistemeOntologyDomain, EpistemeOntologyDomainCategory,
    EpistemeOntologyError, EpistemeOntologyExtends, EpistemeOntologyExtensionContract,
    EpistemeOntologyManifest, EpistemeOntologyPromotionApplyPlanReport,
    EpistemeOntologyPromotionApplyPlanRequest, EpistemeOntologyPromotionReviewPacketReport,
    EpistemeOntologyPromotionReviewPacketRequest, EpistemeOntologyRdfDraftExportReport,
    EpistemeOntologyRdfDraftExportRequest, EpistemeOntologyRegistryActionType,
    EpistemeOntologyRegistryApiSurface, EpistemeOntologyRegistryArtifactMode,
    EpistemeOntologyRegistryCategory, EpistemeOntologyRegistryDatasetMapping,
    EpistemeOntologyRegistryDomain, EpistemeOntologyRegistryError,
    EpistemeOntologyRegistryInterfaceType, EpistemeOntologyRegistryKind,
    EpistemeOntologyRegistryLinkType, EpistemeOntologyRegistryObjectPropertyTerm,
    EpistemeOntologyRegistryObjectType, EpistemeOntologyRegistryObjectTypeRef,
    EpistemeOntologyRegistryPolicy, EpistemeOntologyRegistryQueryType,
    EpistemeOntologyRegistryRdfClassTerm, EpistemeOntologyRegistryRdfTerms,
    EpistemeOntologyRegistryReadModelInput, EpistemeOntologyRegistryRule,
    EpistemeOntologyRegistrySnapshot, EpistemeOntologyRegistrySnapshotReport,
    EpistemeOntologyRegistrySourceContract, EpistemeOntologySemanticEvidenceRow,
    EpistemeOntologySemanticObjectRow, EpistemeOntologySemanticProjectionStateRow,
    EpistemeOntologySemanticRelationRow, EpistemeOntologySourcePatchAppliedTarget,
    EpistemeOntologySourcePatchApplyPlanReport, EpistemeOntologySourcePatchApplyPlanRequest,
    EpistemeOntologySourcePatchApplyPreviewReport, EpistemeOntologySourcePatchApplyPreviewRequest,
    EpistemeOntologySourcePatchApplyPreviewTarget, EpistemeOntologySourcePatchApplyReport,
    EpistemeOntologySourcePatchApplyRequest, EpistemeOntologySourcePatchDraftReport,
    EpistemeOntologySourcePatchDraftRequest, EpistemeOntologySourcePatchPreflightReport,
    EpistemeOntologySourcePatchPreflightRequest, EpistemeOntologySourcePatchRdfReadModelReport,
    EpistemeOntologySourcePatchRdfReadModelRequest, EpistemeOntologySourcePatchReviewPacketReport,
    EpistemeOntologySourcePatchReviewPacketRequest, EpistemeOntologySourcePatchReviewPacketTarget,
    EpistemeOntologySourcePatchSemanticPreviewReport,
    EpistemeOntologySourcePatchSemanticPreviewRequest,
    EpistemeOntologyStructuralIdfReasoningFillPlanReport,
    EpistemeOntologyStructuralIdfReasoningFillPlanRequest,
    EpistemeOntologyStructuralIdfReasoningLedgerSeedReport,
    EpistemeOntologyStructuralIdfReasoningLedgerSeedRequest,
    EpistemeOntologyStructuralIdfReasoningPacketReport,
    EpistemeOntologyStructuralIdfReasoningPacketRequest,
    EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanReport,
    EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest,
    EpistemeOntologyStructuralIdfReport, EpistemeOntologyStructuralIdfRequest,
    EpistemeOntologyStructuralIdfValidationMode, ONTOLOGY_MANIFEST_RELATIVE_PATH,
    ONTOLOGY_REGISTRY_RELATIVE_PATH, admit_ontology_registry_snapshot,
    apply_episteme_ontology_source_patch, export_episteme_ontology_rdf_draft,
    export_episteme_ontology_source_patch_draft, generate_episteme_ontology_candidates,
    ontology_manifest_path, ontology_registry_path, read_ontology_manifest,
    read_ontology_registry_snapshot, review_episteme_ontology_candidates,
    validate_ontology_contract, validate_ontology_registry_snapshot,
    write_episteme_ontology_promotion_apply_plan, write_episteme_ontology_promotion_review_packet,
    write_episteme_ontology_source_patch_apply_plan,
    write_episteme_ontology_source_patch_apply_preview,
    write_episteme_ontology_source_patch_preflight,
    write_episteme_ontology_source_patch_rdf_read_model,
    write_episteme_ontology_source_patch_review_packet,
    write_episteme_ontology_source_patch_semantic_preview, write_episteme_ontology_structural_idf,
    write_episteme_ontology_structural_idf_reasoning_fill_plan,
    write_episteme_ontology_structural_idf_reasoning_ledger_seed,
    write_episteme_ontology_structural_idf_reasoning_packet,
    write_episteme_ontology_structural_idf_reasoning_qianji_schedule_plan,
};
pub use source_contract::{
    EpistemeActiveSourceContract, EpistemeDomainManifest, EpistemeError,
    EpistemeSourceContractPaths, configured_episteme_corpus_root_env, read_source_manifest,
    source_contract_paths,
};

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for cargo-test verification"),
        )
    }
);
