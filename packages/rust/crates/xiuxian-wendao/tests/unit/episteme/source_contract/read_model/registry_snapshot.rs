#[cfg(feature = "julia")]
use crate::episteme::source_contract::support::decode_single_arrow_batch;
#[cfg(feature = "julia")]
use crate::episteme::source_contract::support::string_column;
#[cfg(feature = "julia")]
use xiuxian_wendao::episteme::{
    build_episteme_wendaograph_quality_request_batches,
    materialize_episteme_ontology_registry_snapshot_read_model_seed,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_episteme::{
    EpistemeOntologyRegistryReadModelInput, EpistemeOntologyRegistrySnapshot,
    EpistemeOntologyRegistrySnapshotReport,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::integration_support::{
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_flight_request_batch,
};

#[test]
#[cfg(feature = "julia")]
fn episteme_registry_snapshot_seed_builds_wendaograph_quality_request()
-> Result<(), Box<dyn std::error::Error>> {
    let materialization = materialize_episteme_ontology_registry_snapshot_read_model_seed(
        &registry_snapshot_input()?,
    )?;
    let quality_batches = build_episteme_wendaograph_quality_request_batches(&materialization)?;

    assert_eq!(quality_batches.row_counts(), [15, 18, 1]);

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
    assert_eq!(
        string_column(&objects, "id").value(0),
        "episteme_registry.snapshot:synthetic"
    );
    assert_eq!(
        string_column(&objects, "source_path").value(0),
        "ontology/registry.json"
    );
    assert_eq!(
        string_column(&objects, "read_model_projection_staleness").value(0),
        "fresh"
    );

    let relations = decode_single_arrow_batch(request.semantic_relations_payload.as_slice())?;
    assert_eq!(
        string_column(&relations, "kind").value(0),
        "episteme_registry.snapshot.declares_domain"
    );
    assert_eq!(
        string_column(&relations, "read_model_projection_staleness").value(0),
        "fresh"
    );

    let projection =
        decode_single_arrow_batch(request.semantic_projection_state_payload.as_slice())?;
    assert_eq!(
        string_column(&projection, "projection").value(0),
        "episteme_registry.snapshot_read_model_seed.v1"
    );

    Ok(())
}

#[cfg(feature = "julia")]
fn registry_snapshot_input() -> Result<EpistemeOntologyRegistryReadModelInput, serde_json::Error> {
    let snapshot = serde_json::from_str::<EpistemeOntologyRegistrySnapshot>(include_str!(
        "../../../../fixtures/episteme_registry_snapshot.json"
    ))?;
    Ok(EpistemeOntologyRegistryReadModelInput {
        snapshot,
        report: EpistemeOntologyRegistrySnapshotReport {
            domains: 2,
            rdf_files: 2,
            rules: 1,
            policies: 1,
            dataset_mappings: 1,
            rdf_classes: 2,
            rdf_object_properties: 1,
            api_objects: 2,
            api_links: 1,
            api_actions: 1,
            api_queries: 1,
            api_interfaces: 1,
            reference_nouns: 2,
        },
    })
}
