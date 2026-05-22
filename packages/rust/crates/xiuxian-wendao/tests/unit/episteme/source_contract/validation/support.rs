pub(super) use std::fs;

pub(super) use crate::episteme::source_contract::support::{
    EpistemeFixture, SYNTHETIC_MAPPING_LEDGER,
};
pub(super) use xiuxian_wendao::episteme::{
    EpistemeReadModelRequest, EpistemeRunPlanRequest,
    materialize_episteme_read_model_seed_with_validation_hash_cache, plan_episteme_extraction_run,
    validate_episteme_source_contract, validate_episteme_source_contract_with_hash_cache,
};

pub(super) fn add_doc_to_docling_route(
    fixture: &EpistemeFixture,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = fixture
        .episteme_root
        .join("ontology/SourceContract/corpus/source_manifest.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    fs::write(
        manifest_path,
        manifest.replace(
            "document_text_evidence = [\"docx\", \"txt\"]",
            "document_text_evidence = [\"doc\", \"docx\", \"txt\"]",
        ),
    )?;
    Ok(())
}
