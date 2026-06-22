use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use tempfile::tempdir;

use super::build_wendaograph_ontology_read_model_quality_request_batches_from_structural_facts_artifacts;
use super::support::string_column_value;

#[test]
fn ontology_read_model_quality_accepts_structural_facts_artifacts() -> io::Result<()> {
    let temp = tempdir()?;
    write_structural_facts_artifacts(temp.path(), "structural.anchor.root")?;

    let request_batches =
        build_wendaograph_ontology_read_model_quality_request_batches_from_structural_facts_artifacts(
            temp.path(),
        )
        .map_err(io::Error::other)?;

    assert_eq!(request_batches.row_counts(), [2, 1, 1]);
    assert_eq!(
        string_column_value(&request_batches.objects, "id", 0),
        "structural.document.policy"
    );
    assert_eq!(
        string_column_value(&request_batches.objects, "evidence_id", 0),
        "sha256:1111222233334444"
    );
    assert_eq!(
        string_column_value(&request_batches.objects, "promotion_decision", 1),
        "blocked_structure_only"
    );
    assert_eq!(
        string_column_value(&request_batches.relations, "target_rdf_file", 0),
        "structural_facts_rdf_seed.ttl"
    );
    assert_eq!(
        string_column_value(&request_batches.projection_state, "projection", 0),
        "structural_facts_seed_read_model"
    );

    Ok(())
}

#[test]
fn ontology_read_model_quality_rejects_structural_facts_unknown_relation_endpoint() -> io::Result<()>
{
    let temp = tempdir()?;
    write_structural_facts_artifacts(temp.path(), "structural.anchor.missing")?;

    let Err(error) =
        build_wendaograph_ontology_read_model_quality_request_batches_from_structural_facts_artifacts(
            temp.path(),
        )
    else {
        panic!("structural facts with unknown relation endpoint should be rejected");
    };

    assert!(
        error.contains("unknown target `structural.anchor.missing`"),
        "unexpected error: {error}"
    );
    Ok(())
}

#[test]
fn ontology_read_model_quality_accepts_structural_facts_artifacts_from_env() -> io::Result<()> {
    let Some(run_dir) = env::var_os("WENDAO_GRAPH_ONTOLOGY_STRUCTURAL_FACTS_RUN_DIR") else {
        return Ok(());
    };

    let request_batches =
        build_wendaograph_ontology_read_model_quality_request_batches_from_structural_facts_artifacts(
            Path::new(&run_dir),
        )
        .map_err(io::Error::other)?;

    assert_eq!(
        request_batches.projection_state.num_rows(),
        1,
        "structural facts run should expose one projection state row"
    );
    assert!(
        request_batches.objects.num_rows() >= 2,
        "structural facts run should expose object rows"
    );
    assert!(
        request_batches.relations.num_rows() >= 1,
        "structural facts run should expose relation rows"
    );

    Ok(())
}

fn write_structural_facts_artifacts(root: &Path, relation_target: &str) -> io::Result<()> {
    write_parquet(
        root.join("structural_facts_read_model_objects.parquet")
            .as_path(),
        &RecordBatch::try_new(
            structural_object_schema(),
            vec![
                strings(["structural.document.policy", "structural.anchor.root"]),
                strings(["source_document", "document_root"]),
                strings(["Shanghai LTC Policy", "Shanghai LTC Policy root"]),
                strings(["fresh", "fresh"]),
                strings([
                    "episteme://medical-episteme/10_LongTermCare",
                    "episteme://medical-episteme/10_LongTermCare",
                ]),
                strings(["private_ltc.corpus.v1", "private_ltc.corpus.v1"]),
                strings(["structural.document.policy", "structural.document.policy"]),
                strings(["ltc.file.policy", "ltc.file.policy"]),
                strings(["policy.docx", "policy.docx"]),
                strings(["1111222233334444", "1111222233334444"]),
                booleans([false, false]),
                strings(["active", "active"]),
            ],
        )
        .map_err(io::Error::other)?,
    )?;
    write_parquet(
        root.join("structural_facts_read_model_relations.parquet")
            .as_path(),
        &RecordBatch::try_new(
            structural_relation_schema(),
            vec![
                strings(["structural.relation.document_root"]),
                strings(["has_document_root"]),
                strings(["structural.document.policy"]),
                strings([relation_target]),
                strings(["fresh"]),
                strings(["episteme://medical-episteme/10_LongTermCare"]),
                strings(["private_ltc.corpus.v1"]),
                strings(["policy.docx"]),
                booleans([false]),
                strings(["active"]),
            ],
        )
        .map_err(io::Error::other)?,
    )?;
    fs::write(
        root.join("structural_facts_read_model_projection_state.json"),
        r#"[
  {
    "projection": "structural_facts_seed_read_model",
    "status": "active",
    "staleness": "fresh",
    "sourceObjectCount": 2,
    "sourceRelationCount": 1,
    "sourceDocumentCount": 1,
    "sourceAnchorCount": 1
  }
]"#,
    )
}

fn write_parquet(path: &Path, batch: &RecordBatch) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None).map_err(io::Error::other)?;
    writer.write(batch).map_err(io::Error::other)?;
    writer.close().map_err(io::Error::other)?;
    Ok(())
}

fn structural_object_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        string_field("id"),
        string_field("kind"),
        string_field("title"),
        string_field("read_model_projection_staleness"),
        string_field("domain_id"),
        string_field("source_contract_id"),
        string_field("document_id"),
        string_field("file_id"),
        string_field("relative_path"),
        string_field("source_content_hash"),
        bool_field("ontology_truth"),
        string_field("status"),
    ]))
}

fn structural_relation_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        string_field("id"),
        string_field("kind"),
        string_field("source"),
        string_field("target"),
        string_field("read_model_projection_staleness"),
        string_field("domain_id"),
        string_field("source_contract_id"),
        string_field("evidence_path"),
        bool_field("ontology_truth"),
        string_field("status"),
    ]))
}

fn string_field(name: &'static str) -> Field {
    Field::new(name, DataType::Utf8, false)
}

fn bool_field(name: &'static str) -> Field {
    Field::new(name, DataType::Boolean, false)
}

fn strings<const N: usize>(values: [&str; N]) -> ArrayRef {
    Arc::new(StringArray::from(values.to_vec()))
}

fn booleans<const N: usize>(values: [bool; N]) -> ArrayRef {
    Arc::new(BooleanArray::from(values.to_vec()))
}
