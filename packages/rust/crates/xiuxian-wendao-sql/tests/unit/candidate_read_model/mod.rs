use std::{fs::File, path::Path, sync::Arc};

use arrow::{
    array::{ArrayRef, BooleanArray, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use parquet::arrow::ArrowWriter;

use crate::candidate_read_model::{
    CandidateReadModelDuckDbInspectionRequest, inspect_candidate_read_model_with_duckdb,
};

#[test]
fn candidate_read_model_duckdb_inspection_reports_counts() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let paths = CandidatePaths::new(temp.path());
    write_candidate_tables(&paths, "candidate:source")?;

    let report = inspect_candidate_read_model_with_duckdb(&paths.request())?;

    assert!(report.inspection_passed);
    assert_eq!(report.execution_engine, "duckdb");
    assert_eq!(report.registration_strategy, "duckdb_read_parquet_view");
    assert_eq!(report.candidate_object_count, 2);
    assert_eq!(report.candidate_relation_count, 1);
    assert_eq!(report.candidate_evidence_count, 1);
    assert_eq!(
        report.object_kind_counts[0].kind,
        "ontology_candidate.object_term"
    );
    assert_eq!(report.object_kind_counts[0].row_count, 1);
    assert_eq!(
        report.relation_kind_counts[0].kind,
        "ontology_candidate.source_artifact.suggested_object_type"
    );
    assert_eq!(report.missing_relation_endpoint_count, 0);
    assert_eq!(report.review_status_violation_count, 0);
    assert_eq!(report.promotion_status_violation_count, 0);
    assert_eq!(report.ontology_truth_violation_count, 0);
    assert_eq!(report.raw_to_rdf_promotion_violation_count, 0);
    Ok(())
}

#[test]
fn candidate_read_model_duckdb_inspection_reports_missing_relation_endpoint()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let paths = CandidatePaths::new(temp.path());
    write_candidate_tables(&paths, "candidate:missing")?;

    let report = inspect_candidate_read_model_with_duckdb(&paths.request())?;

    assert!(!report.inspection_passed);
    assert_eq!(report.missing_relation_endpoint_count, 1);
    assert_eq!(report.missing_relation_endpoints[0].endpoint_role, "source");
    assert_eq!(
        report.missing_relation_endpoints[0].endpoint_candidate_id,
        "candidate:missing"
    );
    Ok(())
}

struct CandidatePaths {
    objects: std::path::PathBuf,
    relations: std::path::PathBuf,
    evidence: std::path::PathBuf,
}

impl CandidatePaths {
    fn new(root: &Path) -> Self {
        Self {
            objects: root.join("ontology_candidate_objects.parquet"),
            relations: root.join("ontology_candidate_relations.parquet"),
            evidence: root.join("ontology_candidate_evidence.parquet"),
        }
    }

    fn request(&self) -> CandidateReadModelDuckDbInspectionRequest {
        let run_dir = self.objects.parent().expect("test path has a parent");
        CandidateReadModelDuckDbInspectionRequest::from_candidate_run_dir(run_dir)
    }
}

fn write_candidate_tables(
    paths: &CandidatePaths,
    relation_source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(paths.objects.as_path(), &object_batch()?)?;
    write_parquet(paths.relations.as_path(), &relation_batch(relation_source)?)?;
    write_parquet(paths.evidence.as_path(), &evidence_batch()?)?;
    Ok(())
}

fn object_batch() -> Result<RecordBatch, Box<dyn std::error::Error>> {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            string_field("candidate_id"),
            string_field("candidate_kind"),
            string_field("review_status"),
            string_field("promotion_status"),
            bool_field("ontology_truth"),
            bool_field("raw_to_rdf_promotion_allowed"),
        ])),
        vec![
            strings(["candidate:source", "candidate:target"]),
            strings([
                "ontology_candidate.object_term",
                "ontology_candidate.source_artifact",
            ]),
            strings(["review_required", "review_required"]),
            strings(["blocked_pending_review", "blocked_pending_review"]),
            bools([false, false]),
            bools([false, false]),
        ],
    )
    .map_err(Into::into)
}

fn relation_batch(relation_source: &str) -> Result<RecordBatch, Box<dyn std::error::Error>> {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            string_field("candidate_id"),
            string_field("relation_kind"),
            string_field("source_candidate_id"),
            string_field("target_candidate_id"),
            string_field("review_status"),
            string_field("promotion_status"),
            bool_field("ontology_truth"),
        ])),
        vec![
            strings(["relation:source-target"]),
            strings(["ontology_candidate.source_artifact.suggested_object_type"]),
            strings([relation_source]),
            strings(["candidate:target"]),
            strings(["review_required"]),
            strings(["blocked_pending_review"]),
            bools([false]),
        ],
    )
    .map_err(Into::into)
}

fn evidence_batch() -> Result<RecordBatch, Box<dyn std::error::Error>> {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            string_field("evidence_kind"),
            string_field("review_status"),
            string_field("promotion_status"),
            bool_field("ontology_truth"),
        ])),
        vec![
            strings(["extraction_cache_text_hash"]),
            strings(["review_required"]),
            strings(["blocked_pending_review"]),
            bools([false]),
        ],
    )
    .map_err(Into::into)
}

fn write_parquet(path: &Path, batch: &RecordBatch) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None)?;
    writer.write(batch)?;
    writer.close()?;
    Ok(())
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

fn bools<const N: usize>(values: [bool; N]) -> ArrayRef {
    Arc::new(BooleanArray::from(values.to_vec()))
}
