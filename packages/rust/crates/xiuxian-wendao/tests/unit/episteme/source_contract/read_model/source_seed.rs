use std::fs;

#[cfg(feature = "julia")]
use crate::episteme::source_contract::support::decode_single_arrow_batch;
use crate::episteme::source_contract::support::{
    EpistemeFixture, i64_column, string_column, table,
};
#[cfg(feature = "julia")]
use xiuxian_julia_core::integration_support::{
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_flight_request_batch,
};
#[cfg(feature = "julia")]
use xiuxian_wendao::episteme::build_episteme_wendaograph_quality_request_batches;
use xiuxian_wendao::episteme::{
    EpistemeReadModelRequest, materialize_episteme_read_model_seed,
    validate_episteme_read_model_relation_endpoints,
};

#[test]
fn episteme_source_contract_materializes_read_model_seed() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let materialization = materialize_episteme_read_model_seed(&EpistemeReadModelRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
    ))?;
    validate_episteme_read_model_relation_endpoints(&materialization)?;

    assert!(materialization.source_revision.starts_with("sha256:"));
    assert_eq!(materialization.row_counts(), [4, 2, 1]);
    assert_eq!(materialization.tables[0].table_name(), "semantic_objects");
    assert_eq!(materialization.tables[1].table_name(), "semantic_relations");
    assert_eq!(
        materialization.tables[2].table_name(),
        "semantic_projection_state"
    );

    let objects = table(&materialization, "semantic_objects");
    assert_eq!(string_column(objects, "id").value(0), "episteme.file.a");
    assert_eq!(
        string_column(objects, "kind").value(0),
        "episteme_source_contract.source_file"
    );
    assert_eq!(
        string_column(objects, "source_path").value(0),
        "WENDAO_SYNTHETIC_EPISTEME_CORPUS_ROOT/docs/a.docx"
    );
    assert_eq!(i64_column(objects, "owner_count").value(0), 1);
    assert_eq!(i64_column(objects, "relation_count").value(0), 1);
    assert!(
        !string_column(objects, "verification_evidence_json")
            .value(0)
            .contains("fixture content"),
        "read-model seed must not embed raw source corpus text"
    );

    let relations = table(&materialization, "semantic_relations");
    assert_eq!(
        string_column(relations, "source").value(0),
        "episteme.extract.a"
    );
    assert_eq!(
        string_column(relations, "kind").value(0),
        "episteme_source_contract.extraction_task.has_source_file"
    );
    assert_eq!(
        string_column(relations, "target").value(0),
        "episteme.file.a"
    );

    let projection = table(&materialization, "semantic_projection_state");
    assert_eq!(
        string_column(projection, "projection").value(0),
        "episteme_source_contract.source_contract_read_model_seed.v1"
    );
    assert_eq!(i64_column(projection, "source_object_count").value(0), 4);

    Ok(())
}

#[test]
#[cfg(feature = "julia")]
fn episteme_source_contract_read_model_seed_builds_wendaograph_quality_request()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let materialization = materialize_episteme_read_model_seed(&EpistemeReadModelRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
    ))?;
    let quality_batches = build_episteme_wendaograph_quality_request_batches(&materialization)?;

    assert_eq!(quality_batches.row_counts(), [4, 2, 1]);

    let request = build_wendaograph_ontology_read_model_quality_arrow_request(&quality_batches)?;
    let bundle = build_wendaograph_ontology_read_model_quality_flight_request_batch(&request)?;
    assert_eq!(bundle.num_rows(), 1);
    assert!(
        request
            .payload_byte_sizes()
            .into_iter()
            .all(|size| size > 0)
    );

    let objects = decode_single_arrow_batch(request.semantic_objects_payload.as_slice())?;
    assert_eq!(string_column(&objects, "id").value(0), "episteme.file.a");
    assert_eq!(
        string_column(&objects, "read_model_projection_staleness").value(0),
        "fresh"
    );

    Ok(())
}

#[test]
fn episteme_source_contract_read_model_rejects_hash_drift() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fs::write(fixture.corpus_root.join("docs/a.docx"), "changed")?;

    let Err(error) = materialize_episteme_read_model_seed(&EpistemeReadModelRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
    )) else {
        return Err("hash drift should prevent read-model materialization".into());
    };
    assert!(error.to_string().contains("invalid"));

    Ok(())
}
