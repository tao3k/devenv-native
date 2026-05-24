//! Arrow/Parquet read-model publication for ontology candidate rows.

use std::{fs::File, path::Path, sync::Arc};

use anyhow::{Context, Result};
use arrow::{
    array::{ArrayRef, BooleanArray, Int64Array, StringArray},
    datatypes::{DataType, Field, Schema, SchemaRef},
    record_batch::RecordBatch,
};
use parquet::arrow::ArrowWriter;

use super::super::model::{
    CandidateEvidenceRow, CandidateGenerationOutputPaths, CandidateObjectRow, CandidateRelationRow,
    CandidateRows,
};

pub(in crate::ontology::candidates) fn write_candidate_read_model(
    paths: &CandidateGenerationOutputPaths,
    rows: &CandidateRows,
) -> Result<()> {
    let object_batch = object_rows_batch(&rows.objects)?;
    let relation_batch = relation_rows_batch(&rows.relations)?;
    let evidence_batch = evidence_rows_batch(&rows.evidence)?;
    write_parquet(paths.objects_parquet.as_path(), &object_batch)?;
    write_parquet(paths.relations_parquet.as_path(), &relation_batch)?;
    write_parquet(paths.evidence_parquet.as_path(), &evidence_batch)?;
    Ok(())
}

fn object_rows_batch(rows: &[CandidateObjectRow]) -> Result<RecordBatch> {
    let schema = schema_ref([
        string_field("candidate_id"),
        string_field("candidate_kind"),
        string_field("status"),
        string_field("label"),
        string_field("suggested_term_key"),
        string_field("suggested_term_label"),
        string_field("source_file_id"),
        string_field("source_queue_id"),
        string_field("source_path"),
        string_field("category"),
        string_field("language"),
        string_field("extraction_route"),
        string_field("extraction_run_id"),
        string_field("source_sha256"),
        string_field("evidence_sha256"),
        int64_field("text_char_count"),
        string_field("review_status"),
        string_field("promotion_status"),
        bool_field("raw_to_rdf_promotion_allowed"),
        bool_field("ontology_truth"),
    ]);
    let text_counts = parse_text_counts(
        rows.iter()
            .map(|row| row.text_char_count.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
    )?;
    let columns = vec![
        strings(rows.iter().map(|row| row.candidate_id.as_str())),
        strings(rows.iter().map(|row| row.candidate_kind)),
        strings(rows.iter().map(|row| row.status)),
        strings(rows.iter().map(|row| row.label.as_str())),
        strings(rows.iter().map(|row| row.suggested_term_key.as_str())),
        strings(rows.iter().map(|row| row.suggested_term_label.as_str())),
        strings(rows.iter().map(|row| row.source_file_id.as_str())),
        strings(rows.iter().map(|row| row.source_queue_id.as_str())),
        strings(rows.iter().map(|row| row.source_path.as_str())),
        strings(rows.iter().map(|row| row.category.as_str())),
        strings(rows.iter().map(|row| row.language.as_str())),
        strings(rows.iter().map(|row| row.extraction_route.as_str())),
        strings(rows.iter().map(|row| row.extraction_run_id.as_str())),
        strings(rows.iter().map(|row| row.source_sha256.as_str())),
        strings(rows.iter().map(|row| row.evidence_sha256.as_str())),
        Arc::new(Int64Array::from(text_counts)) as ArrayRef,
        strings(rows.iter().map(|row| row.review_status)),
        strings(rows.iter().map(|row| row.promotion_status)),
        Arc::new(BooleanArray::from(
            rows.iter()
                .map(|row| row.raw_to_rdf_promotion_allowed)
                .collect::<Vec<_>>(),
        )),
        Arc::new(BooleanArray::from(
            rows.iter()
                .map(|row| row.ontology_truth)
                .collect::<Vec<_>>(),
        )),
    ];
    RecordBatch::try_new(schema, columns).context("failed to build candidate object read-model")
}

fn relation_rows_batch(rows: &[CandidateRelationRow]) -> Result<RecordBatch> {
    let schema = schema_ref([
        string_field("candidate_id"),
        string_field("relation_kind"),
        string_field("source_candidate_id"),
        string_field("target_candidate_id"),
        string_field("source_file_id"),
        string_field("source_queue_id"),
        string_field("extraction_run_id"),
        string_field("evidence_sha256"),
        string_field("review_status"),
        string_field("promotion_status"),
        bool_field("ontology_truth"),
    ]);
    let columns = vec![
        strings(rows.iter().map(|row| row.candidate_id.as_str())),
        strings(rows.iter().map(|row| row.relation_kind)),
        strings(rows.iter().map(|row| row.source_candidate_id.as_str())),
        strings(rows.iter().map(|row| row.target_candidate_id.as_str())),
        strings(rows.iter().map(|row| row.source_file_id.as_str())),
        strings(rows.iter().map(|row| row.source_queue_id.as_str())),
        strings(rows.iter().map(|row| row.extraction_run_id.as_str())),
        strings(rows.iter().map(|row| row.evidence_sha256.as_str())),
        strings(rows.iter().map(|row| row.review_status)),
        strings(rows.iter().map(|row| row.promotion_status)),
        Arc::new(BooleanArray::from(
            rows.iter()
                .map(|row| row.ontology_truth)
                .collect::<Vec<_>>(),
        )),
    ];
    RecordBatch::try_new(schema, columns).context("failed to build candidate relation read-model")
}

fn evidence_rows_batch(rows: &[CandidateEvidenceRow]) -> Result<RecordBatch> {
    let schema = schema_ref([
        string_field("evidence_id"),
        string_field("evidence_kind"),
        string_field("source_file_id"),
        string_field("source_queue_id"),
        string_field("source_path"),
        string_field("source_sha256"),
        string_field("extraction_run_id"),
        string_field("cache_output_path"),
        string_field("evidence_sha256"),
        int64_field("text_char_count"),
        string_field("review_status"),
        string_field("promotion_status"),
        bool_field("ontology_truth"),
    ]);
    let text_counts = parse_text_counts(
        rows.iter()
            .map(|row| row.text_char_count.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
    )?;
    let columns = vec![
        strings(rows.iter().map(|row| row.evidence_id.as_str())),
        strings(rows.iter().map(|row| row.evidence_kind)),
        strings(rows.iter().map(|row| row.source_file_id.as_str())),
        strings(rows.iter().map(|row| row.source_queue_id.as_str())),
        strings(rows.iter().map(|row| row.source_path.as_str())),
        strings(rows.iter().map(|row| row.source_sha256.as_str())),
        strings(rows.iter().map(|row| row.extraction_run_id.as_str())),
        strings(rows.iter().map(|row| row.cache_output_path.as_str())),
        strings(rows.iter().map(|row| row.evidence_sha256.as_str())),
        Arc::new(Int64Array::from(text_counts)) as ArrayRef,
        strings(rows.iter().map(|row| row.review_status)),
        strings(rows.iter().map(|row| row.promotion_status)),
        Arc::new(BooleanArray::from(
            rows.iter()
                .map(|row| row.ontology_truth)
                .collect::<Vec<_>>(),
        )),
    ];
    RecordBatch::try_new(schema, columns).context("failed to build candidate evidence read-model")
}

fn write_parquet(path: &Path, batch: &RecordBatch) -> Result<()> {
    let file =
        File::create(path).with_context(|| format!("failed to create `{}`", path.display()))?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None)
        .context("failed to create Parquet writer")?;
    writer
        .write(batch)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writer
        .close()
        .with_context(|| format!("failed to close `{}`", path.display()))?;
    Ok(())
}

fn schema_ref<const N: usize>(fields: [Field; N]) -> SchemaRef {
    Arc::new(Schema::new(fields.into_iter().collect::<Vec<_>>()))
}

fn string_field(name: &'static str) -> Field {
    Field::new(name, DataType::Utf8, false)
}

fn int64_field(name: &'static str) -> Field {
    Field::new(name, DataType::Int64, false)
}

fn bool_field(name: &'static str) -> Field {
    Field::new(name, DataType::Boolean, false)
}

fn strings<'a>(values: impl Iterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from(values.collect::<Vec<_>>()))
}

fn parse_text_counts(values: &[&str]) -> Result<Vec<i64>> {
    values
        .iter()
        .map(|value| {
            value
                .parse::<i64>()
                .with_context(|| format!("invalid read-model text_char_count `{value}`"))
        })
        .collect()
}
