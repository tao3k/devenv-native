use std::env;
use std::fs;
use std::io;
use std::path::Path;

use tempfile::tempdir;

use super::support::{decode_single_batch, string_column_value};
use super::{
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_request_batches_from_rdf_source_artifacts,
    build_wendaograph_ontology_read_model_quality_request_batches_from_semantic_preview_artifacts,
};

#[test]
fn ontology_read_model_quality_accepts_semantic_preview_artifacts() -> io::Result<()> {
    let temp = tempdir()?;
    write_semantic_preview_artifacts(temp.path(), "demo.city.shanghai")?;

    let request_batches =
        build_wendaograph_ontology_read_model_quality_request_batches_from_semantic_preview_artifacts(
            temp.path(),
        )
        .map_err(io::Error::other)?;

    assert_eq!(request_batches.row_counts(), [2, 1, 1]);
    assert_eq!(
        string_column_value(&request_batches.objects, "id", 0),
        "demo.policy.shanghai_ltc_trial"
    );
    assert_eq!(
        string_column_value(&request_batches.objects, "kind", 1),
        "demo.pilot_city"
    );
    assert_eq!(
        string_column_value(&request_batches.relations, "target", 0),
        "demo.city.shanghai"
    );
    assert_eq!(
        string_column_value(&request_batches.projection_state, "staleness", 0),
        "fresh"
    );

    let request = build_wendaograph_ontology_read_model_quality_arrow_request(&request_batches)
        .map_err(io::Error::other)?;
    let relations = decode_single_batch(
        request.semantic_relations_payload.as_slice(),
        "semantic_relations",
    );
    let projection_state = decode_single_batch(
        request.semantic_projection_state_payload.as_slice(),
        "semantic_projection_state",
    );

    assert_eq!(
        string_column_value(&relations, "read_model_projection_staleness", 0),
        "fresh"
    );
    assert_eq!(
        string_column_value(&projection_state, "projection", 0),
        "source_patch_semantic_read_model_preview"
    );

    Ok(())
}

#[test]
fn ontology_read_model_quality_rejects_semantic_preview_unknown_relation_endpoint() -> io::Result<()>
{
    let temp = tempdir()?;
    write_semantic_preview_artifacts(temp.path(), "demo.city.missing")?;

    let Err(error) =
        build_wendaograph_ontology_read_model_quality_request_batches_from_semantic_preview_artifacts(
            temp.path(),
        )
    else {
        panic!("semantic preview with unknown relation endpoint should be rejected");
    };

    assert!(
        error.contains("unknown target `demo.city.missing`"),
        "unexpected error: {error}"
    );
    Ok(())
}

#[test]
fn ontology_read_model_quality_accepts_semantic_preview_artifacts_from_env() -> io::Result<()> {
    let Some(run_dir) = env::var_os("WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PREVIEW_RUN_DIR") else {
        return Ok(());
    };

    let request_batches =
        build_wendaograph_ontology_read_model_quality_request_batches_from_semantic_preview_artifacts(
            Path::new(&run_dir),
        )
        .map_err(io::Error::other)?;

    assert_eq!(
        request_batches.projection_state.num_rows(),
        1,
        "semantic preview run should expose one projection state row"
    );
    assert!(
        request_batches.objects.num_rows() >= 2,
        "semantic preview run should expose object rows"
    );
    assert!(
        request_batches.relations.num_rows() >= 1,
        "semantic preview run should expose relation rows"
    );

    Ok(())
}

#[test]
fn ontology_read_model_quality_accepts_rdf_source_artifacts() -> io::Result<()> {
    let temp = tempdir()?;
    write_rdf_source_artifacts(temp.path(), "demo.city.shanghai")?;

    let request_batches =
        build_wendaograph_ontology_read_model_quality_request_batches_from_rdf_source_artifacts(
            temp.path(),
        )
        .map_err(io::Error::other)?;

    assert_eq!(request_batches.row_counts(), [2, 1, 1]);
    assert_eq!(
        string_column_value(&request_batches.projection_state, "projection", 0),
        "source_patch_rdf_source_read_model"
    );

    Ok(())
}

#[test]
fn ontology_read_model_quality_accepts_rdf_source_artifacts_from_env() -> io::Result<()> {
    let Some(run_dir) = env::var_os("WENDAO_GRAPH_ONTOLOGY_RDF_SOURCE_RUN_DIR") else {
        return Ok(());
    };

    let request_batches =
        build_wendaograph_ontology_read_model_quality_request_batches_from_rdf_source_artifacts(
            Path::new(&run_dir),
        )
        .map_err(io::Error::other)?;

    assert_eq!(
        request_batches.projection_state.num_rows(),
        1,
        "RDF source run should expose one projection state row"
    );
    assert!(
        request_batches.objects.num_rows() >= 2,
        "RDF source run should expose object rows"
    );
    assert!(
        request_batches.relations.num_rows() >= 1,
        "RDF source run should expose relation rows"
    );

    Ok(())
}

fn write_semantic_preview_artifacts(root: &Path, relation_target: &str) -> io::Result<()> {
    fs::write(
        root.join("semantic_objects.tsv"),
        "\
id\tkind\ttitle\tdomain\tevidence_id\tevidence_status\ttarget_rdf_file\treview_decision\tpromotion_decision\treviewer_id\trelation_count\tstatus\tread_model_projection_staleness
demo.policy.shanghai_ltc_trial\tdemo.policy_document\tShanghai LTC Trial Policy\tepisteme://private/demo/10_LongTermCare\tevidence:demo.policy\taccepted\t10_LongTermCare/ontology.rdf\taccepted_evidence_candidate\tapproved\treviewer.demo\t1\tactive\tfresh
demo.city.shanghai\tdemo.pilot_city\tShanghai\tepisteme://private/demo/10_LongTermCare\tevidence:demo.policy\taccepted\t10_LongTermCare/ontology.rdf\taccepted_evidence_candidate\tapproved\treviewer.demo\t1\tactive\tfresh
",
    )?;
    fs::write(
        root.join("semantic_relations.tsv"),
        format!(
            "\
id\tkind\tsource\ttarget\tdomain\tevidence_id\tevidence_status\ttarget_rdf_file\treview_decision\tpromotion_decision\treviewer_id\tstatus\tread_model_projection_staleness
demo.relation.policy_city\tdemo.applies_to_city\tdemo.policy.shanghai_ltc_trial\t{relation_target}\tepisteme://private/demo/10_LongTermCare\tevidence:demo.policy\taccepted\t10_LongTermCare/ontology.rdf\taccepted_evidence_candidate\tapproved\treviewer.demo\tactive\tfresh
"
        ),
    )?;
    fs::write(
        root.join("semantic_projection_state.json"),
        r#"[
  {
    "projection": "source_patch_semantic_read_model_preview",
    "status": "active",
    "staleness": "fresh",
    "sourceObjectCount": 2,
    "sourceRelationCount": 1,
    "sourceEvidenceCount": 1
  }
]"#,
    )
}

fn write_rdf_source_artifacts(root: &Path, relation_target: &str) -> io::Result<()> {
    fs::write(
        root.join("rdf_source_semantic_objects.tsv"),
        "\
id\tkind\ttitle\tdomain\tevidence_id\tevidence_status\ttarget_rdf_file\treview_decision\tpromotion_decision\treviewer_id\trelation_count\tstatus\tread_model_projection_staleness
demo.policy.shanghai_ltc_trial\tdemo.policy_document\tShanghai LTC Trial Policy\tepisteme://private/demo/10_LongTermCare\tevidence:demo.policy\taccepted\t10_LongTermCare/ontology.rdf\taccepted_evidence_candidate\tapproved\treviewer.demo\t1\tactive\tfresh
demo.city.shanghai\tdemo.pilot_city\tShanghai\tepisteme://private/demo/10_LongTermCare\tevidence:demo.policy\taccepted\t10_LongTermCare/ontology.rdf\taccepted_evidence_candidate\tapproved\treviewer.demo\t1\tactive\tfresh
",
    )?;
    fs::write(
        root.join("rdf_source_semantic_relations.tsv"),
        format!(
            "\
id\tkind\tsource\ttarget\tdomain\tevidence_id\tevidence_status\ttarget_rdf_file\treview_decision\tpromotion_decision\treviewer_id\tstatus\tread_model_projection_staleness
demo.relation.policy_city\tdemo.applies_to_city\tdemo.policy.shanghai_ltc_trial\t{relation_target}\tepisteme://private/demo/10_LongTermCare\tevidence:demo.policy\taccepted\t10_LongTermCare/ontology.rdf\taccepted_evidence_candidate\tapproved\treviewer.demo\tactive\tfresh
"
        ),
    )?;
    fs::write(
        root.join("rdf_source_projection_state.json"),
        r#"[
  {
    "projection": "source_patch_rdf_source_read_model",
    "status": "active",
    "staleness": "fresh",
    "sourceObjectCount": 2,
    "sourceRelationCount": 1,
    "sourceEvidenceCount": 1
  }
]"#,
    )
}
