//! Query and quality summary for candidate Arrow/Parquet read-model tables.

use std::{collections::BTreeSet, fs::File, path::Path};

use anyhow::{Context, Result};
use arrow::{
    array::{Array, BooleanArray, StringArray},
    datatypes::DataType,
    record_batch::RecordBatch,
};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use crate::ontology::candidates::{
    api::{
        EpistemeOntologyCandidateReadModelMissingEndpoint,
        EpistemeOntologyCandidateReadModelSummaryReport,
        EpistemeOntologyCandidateReadModelSummaryRequest,
    },
    model::{PROMOTION_STATUS, REVIEW_STATUS},
};

const READ_MODEL_SUMMARY_SCHEMA: &str =
    "xiuxian_wendao.episteme_ontology_candidate_read_model_summary.v1";

/// Summarize and validate candidate Parquet read-model tables.
///
/// # Errors
///
/// Returns an error when a Parquet file cannot be opened or decoded, when a
/// required column is missing, or when a required column has an unexpected
/// Arrow type.
pub fn summarize_episteme_ontology_candidate_read_model(
    request: &EpistemeOntologyCandidateReadModelSummaryRequest,
) -> Result<EpistemeOntologyCandidateReadModelSummaryReport> {
    let objects = read_parquet_batches(request.objects.as_path())?;
    let relations = read_parquet_batches(request.relations.as_path())?;
    let evidence = read_parquet_batches(request.evidence.as_path())?;

    let object_ids = string_values(&objects, "candidate_id")?;
    let object_id_set = object_ids.iter().cloned().collect::<BTreeSet<_>>();
    let missing_relation_endpoints = missing_relation_endpoints(&relations, &object_id_set)?;

    let review_status_violation_count =
        count_unexpected_string(&objects, "review_status", REVIEW_STATUS)?
            + count_unexpected_string(&relations, "review_status", REVIEW_STATUS)?
            + count_unexpected_string(&evidence, "review_status", REVIEW_STATUS)?;
    let promotion_status_violation_count =
        count_unexpected_string(&objects, "promotion_status", PROMOTION_STATUS)?
            + count_unexpected_string(&relations, "promotion_status", PROMOTION_STATUS)?
            + count_unexpected_string(&evidence, "promotion_status", PROMOTION_STATUS)?;
    let ontology_truth_violation_count = count_true(&objects, "ontology_truth")?
        + count_true(&relations, "ontology_truth")?
        + count_true(&evidence, "ontology_truth")?;
    let raw_to_rdf_promotion_violation_count =
        count_true(&objects, "raw_to_rdf_promotion_allowed")?;
    let candidate_object_count = row_count(&objects);
    let read_model_gate_passed = candidate_object_count > 0
        && review_status_violation_count == 0
        && promotion_status_violation_count == 0
        && ontology_truth_violation_count == 0
        && raw_to_rdf_promotion_violation_count == 0
        && missing_relation_endpoints.is_empty();

    Ok(EpistemeOntologyCandidateReadModelSummaryReport {
        schema_version: READ_MODEL_SUMMARY_SCHEMA,
        candidate_objects_parquet: request.objects.clone(),
        candidate_relations_parquet: request.relations.clone(),
        candidate_evidence_parquet: request.evidence.clone(),
        candidate_object_count,
        candidate_relation_count: row_count(&relations),
        candidate_evidence_count: row_count(&evidence),
        review_status_violation_count,
        promotion_status_violation_count,
        ontology_truth_violation_count,
        raw_to_rdf_promotion_violation_count,
        missing_relation_endpoint_count: missing_relation_endpoints.len(),
        missing_relation_endpoints,
        read_model_gate_passed,
    })
}

fn read_parquet_batches(path: &Path) -> Result<Vec<RecordBatch>> {
    let file = File::open(path).with_context(|| format!("failed to open `{}`", path.display()))?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .with_context(|| format!("failed to read Parquet metadata from `{}`", path.display()))?
        .build()
        .with_context(|| format!("failed to build Parquet reader for `{}`", path.display()))?;
    reader
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("failed to read Parquet batches from `{}`", path.display()))
}

fn row_count(batches: &[RecordBatch]) -> usize {
    batches.iter().map(RecordBatch::num_rows).sum()
}

fn missing_relation_endpoints(
    relations: &[RecordBatch],
    object_ids: &BTreeSet<String>,
) -> Result<Vec<EpistemeOntologyCandidateReadModelMissingEndpoint>> {
    let relation_ids = string_values(relations, "candidate_id")?;
    let source_ids = string_values(relations, "source_candidate_id")?;
    let target_ids = string_values(relations, "target_candidate_id")?;
    let mut missing = Vec::new();
    for ((relation_candidate_id, source_id), target_id) in relation_ids
        .iter()
        .zip(source_ids.iter())
        .zip(target_ids.iter())
    {
        if !object_ids.contains(source_id) {
            missing.push(missing_endpoint(relation_candidate_id, "source", source_id));
        }
        if !object_ids.contains(target_id) {
            missing.push(missing_endpoint(relation_candidate_id, "target", target_id));
        }
    }
    Ok(missing)
}

fn missing_endpoint(
    relation_candidate_id: &str,
    endpoint_role: &str,
    endpoint_candidate_id: &str,
) -> EpistemeOntologyCandidateReadModelMissingEndpoint {
    EpistemeOntologyCandidateReadModelMissingEndpoint {
        relation_candidate_id: relation_candidate_id.to_string(),
        endpoint_role: endpoint_role.to_string(),
        endpoint_candidate_id: endpoint_candidate_id.to_string(),
    }
}

fn count_unexpected_string(
    batches: &[RecordBatch],
    column_name: &str,
    expected: &str,
) -> Result<usize> {
    Ok(string_values(batches, column_name)?
        .iter()
        .filter(|value| value.as_str() != expected)
        .count())
}

fn count_true(batches: &[RecordBatch], column_name: &str) -> Result<usize> {
    batches.iter().try_fold(0, |count, batch| {
        let values = boolean_column(batch, column_name)?;
        Ok(count
            + (0..values.len())
                .filter(|index| values.value(*index))
                .count())
    })
}

fn string_values(batches: &[RecordBatch], column_name: &str) -> Result<Vec<String>> {
    let mut values = Vec::new();
    for batch in batches {
        let strings = string_column(batch, column_name)?;
        values.extend((0..strings.len()).map(|index| strings.value(index).to_string()));
    }
    Ok(values)
}

fn string_column<'a>(batch: &'a RecordBatch, column_name: &str) -> Result<&'a StringArray> {
    ensure_column_type(batch, column_name, &DataType::Utf8)?;
    batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .with_context(|| format!("`{column_name}` is not a Utf8 column"))
}

fn boolean_column<'a>(batch: &'a RecordBatch, column_name: &str) -> Result<&'a BooleanArray> {
    ensure_column_type(batch, column_name, &DataType::Boolean)?;
    batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<BooleanArray>())
        .with_context(|| format!("`{column_name}` is not a Boolean column"))
}

fn ensure_column_type(
    batch: &RecordBatch,
    column_name: &str,
    expected_type: &DataType,
) -> Result<()> {
    let schema = batch.schema();
    let field = schema.field_with_name(column_name).with_context(|| {
        format!(
            "missing `{column_name}` column in `{}`",
            schema_columns(schema.fields().iter().map(|field| field.name().as_str()))
        )
    })?;
    if field.data_type() != expected_type {
        anyhow::bail!(
            "`{column_name}` has type `{:?}`, expected `{:?}`",
            field.data_type(),
            expected_type
        );
    }
    Ok(())
}

fn schema_columns<'a>(names: impl Iterator<Item = &'a str>) -> String {
    names.collect::<Vec<_>>().join(", ")
}
