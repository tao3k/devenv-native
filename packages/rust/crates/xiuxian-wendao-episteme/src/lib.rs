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
    EPISTEME_IMAGE_OCR_ROUTE, EPISTEME_IMAGE_OCR_WRAPPER_SCHEMA, EpistemeCacheTask,
    EpistemeCacheTaskCategory, EpistemeCacheTaskStatus, EpistemeDoclingDocumentCacheBridgeReport,
    EpistemeImageOcrCacheBridgeReport, read_docling_document_tasks_tsv, read_image_ocr_tasks_tsv,
    skipped_docling_document_cache_bridge_report, skipped_image_ocr_cache_bridge_report,
    validate_docling_document_tasks, validate_image_ocr_tasks,
    write_docling_document_cache_outputs, write_image_ocr_cache_outputs,
};
pub use ontology::{
    EpistemeOntologyApiSurface, EpistemeOntologyArtifactMode, EpistemeOntologyBoundaries,
    EpistemeOntologyContractReport, EpistemeOntologyDomain, EpistemeOntologyDomainCategory,
    EpistemeOntologyError, EpistemeOntologyExtensionContract, EpistemeOntologyManifest,
    ONTOLOGY_MANIFEST_RELATIVE_PATH, ontology_manifest_path, read_ontology_manifest,
    validate_ontology_contract,
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
