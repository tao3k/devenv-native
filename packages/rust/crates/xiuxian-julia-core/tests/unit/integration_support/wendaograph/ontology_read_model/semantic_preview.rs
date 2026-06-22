use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
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
    write_read_model_artifacts(
        root,
        "semantic_objects.parquet",
        "semantic_relations.parquet",
        "semantic_projection_state.json",
        "source_patch_semantic_read_model_preview",
        relation_target,
    )
}

fn write_rdf_source_artifacts(root: &Path, relation_target: &str) -> io::Result<()> {
    write_read_model_artifacts(
        root,
        "rdf_source_semantic_objects.parquet",
        "rdf_source_semantic_relations.parquet",
        "rdf_source_projection_state.json",
        "source_patch_rdf_source_read_model",
        relation_target,
    )
}

fn write_read_model_artifacts(
    root: &Path,
    object_file: &str,
    relation_file: &str,
    projection_file: &str,
    projection: &str,
    relation_target: &str,
) -> io::Result<()> {
    write_parquet(
        root.join(object_file).as_path(),
        &RecordBatch::try_new(
            semantic_object_schema(),
            vec![
                strings(["demo.policy.shanghai_ltc_trial", "demo.city.shanghai"]),
                strings(["demo.policy_document", "demo.pilot_city"]),
                strings(["Shanghai LTC Trial Policy", "Shanghai"]),
                strings([
                    "episteme://private/demo/10_LongTermCare",
                    "episteme://private/demo/10_LongTermCare",
                ]),
                strings(["evidence:demo.policy", "evidence:demo.policy"]),
                strings(["accepted", "accepted"]),
                strings([
                    "10_LongTermCare/ontology.rdf",
                    "10_LongTermCare/ontology.rdf",
                ]),
                strings(["accepted_evidence_candidate", "accepted_evidence_candidate"]),
                strings(["approved", "approved"]),
                strings(["reviewer.demo", "reviewer.demo"]),
                ints([1, 1]),
                strings(["active", "active"]),
                strings(["fresh", "fresh"]),
            ],
        )
        .map_err(io::Error::other)?,
    )?;
    write_parquet(
        root.join(relation_file).as_path(),
        &RecordBatch::try_new(
            semantic_relation_schema(),
            vec![
                strings(["demo.relation.policy_city"]),
                strings(["demo.applies_to_city"]),
                strings(["demo.policy.shanghai_ltc_trial"]),
                strings([relation_target]),
                strings(["episteme://private/demo/10_LongTermCare"]),
                strings(["evidence:demo.policy"]),
                strings(["accepted"]),
                strings(["10_LongTermCare/ontology.rdf"]),
                strings(["accepted_evidence_candidate"]),
                strings(["approved"]),
                strings(["reviewer.demo"]),
                strings(["active"]),
                strings(["fresh"]),
            ],
        )
        .map_err(io::Error::other)?,
    )?;
    fs::write(
        root.join(projection_file),
        format!(
            r#"[
  {{
    "projection": "{projection}",
    "status": "active",
    "staleness": "fresh",
    "sourceObjectCount": 2,
    "sourceRelationCount": 1,
    "sourceEvidenceCount": 1
  }}
]"#
        ),
    )
}

fn write_parquet(path: &Path, batch: &RecordBatch) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None).map_err(io::Error::other)?;
    writer.write(batch).map_err(io::Error::other)?;
    writer.close().map_err(io::Error::other)?;
    Ok(())
}

fn semantic_object_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        string_field("id"),
        string_field("kind"),
        string_field("title"),
        string_field("domain"),
        string_field("evidence_id"),
        string_field("evidence_status"),
        string_field("target_rdf_file"),
        string_field("review_decision"),
        string_field("promotion_decision"),
        string_field("reviewer_id"),
        int64_field("relation_count"),
        string_field("status"),
        string_field("read_model_projection_staleness"),
    ]))
}

fn semantic_relation_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        string_field("id"),
        string_field("kind"),
        string_field("source"),
        string_field("target"),
        string_field("domain"),
        string_field("evidence_id"),
        string_field("evidence_status"),
        string_field("target_rdf_file"),
        string_field("review_decision"),
        string_field("promotion_decision"),
        string_field("reviewer_id"),
        string_field("status"),
        string_field("read_model_projection_staleness"),
    ]))
}

fn string_field(name: &'static str) -> Field {
    Field::new(name, DataType::Utf8, false)
}

fn int64_field(name: &'static str) -> Field {
    Field::new(name, DataType::Int64, false)
}

fn strings<const N: usize>(values: [&str; N]) -> ArrayRef {
    Arc::new(StringArray::from(values.to_vec()))
}

fn ints<const N: usize>(values: [i64; N]) -> ArrayRef {
    Arc::new(Int64Array::from(values.to_vec()))
}
