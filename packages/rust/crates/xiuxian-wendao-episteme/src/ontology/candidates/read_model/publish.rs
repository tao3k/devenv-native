//! Arrow/Parquet read-model publication for ontology candidate rows.

use std::{collections::HashMap, fs::File, path::Path, sync::Arc};

use anyhow::{Context, Result};
use arrow::{
    array::{ArrayRef, BooleanArray, Int64Array, StringArray},
    datatypes::SchemaRef,
    record_batch::RecordBatch,
};
use parquet::arrow::ArrowWriter;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

use super::super::model::{
    CandidateEvidenceRow, CandidateGenerationOutputPaths, CandidateObjectRow, CandidateRelationRow,
    CandidateRows,
};

const OBJECTS_TABLE: &str = "ontology_candidate_objects";
const RELATIONS_TABLE: &str = "ontology_candidate_relations";
const EVIDENCE_TABLE: &str = "ontology_candidate_evidence";

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
    let contract = ArrowSchemaContract::new(
        OBJECTS_TABLE,
        true,
        vec![
            string_column("candidate_id"),
            string_column("candidate_kind"),
            string_column("status"),
            string_column("label"),
            string_column("suggested_term_key"),
            string_column("suggested_term_label"),
            string_column("source_file_id"),
            string_column("source_queue_id"),
            string_column("source_path"),
            string_column("category"),
            string_column("language"),
            string_column("extraction_route"),
            string_column("extraction_run_id"),
            string_column("source_sha256"),
            string_column("evidence_sha256"),
            int64_column("text_char_count"),
            string_column("review_status"),
            string_column("promotion_status"),
            bool_column("raw_to_rdf_promotion_allowed"),
            bool_column("ontology_truth"),
        ],
    );
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
    record_batch(
        &contract,
        columns,
        "failed to build candidate object read-model",
    )
}

fn relation_rows_batch(rows: &[CandidateRelationRow]) -> Result<RecordBatch> {
    let contract = ArrowSchemaContract::new(
        RELATIONS_TABLE,
        true,
        vec![
            string_column("candidate_id"),
            string_column("relation_kind"),
            string_column("source_candidate_id"),
            string_column("target_candidate_id"),
            string_column("source_file_id"),
            string_column("source_queue_id"),
            string_column("extraction_run_id"),
            string_column("evidence_sha256"),
            string_column("review_status"),
            string_column("promotion_status"),
            bool_column("ontology_truth"),
        ],
    );
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
    record_batch(
        &contract,
        columns,
        "failed to build candidate relation read-model",
    )
}

fn evidence_rows_batch(rows: &[CandidateEvidenceRow]) -> Result<RecordBatch> {
    let contract = ArrowSchemaContract::new(
        EVIDENCE_TABLE,
        true,
        vec![
            string_column("evidence_id"),
            string_column("evidence_kind"),
            string_column("source_file_id"),
            string_column("source_queue_id"),
            string_column("source_path"),
            string_column("source_sha256"),
            string_column("extraction_run_id"),
            string_column("cache_output_path"),
            string_column("evidence_sha256"),
            int64_column("text_char_count"),
            string_column("review_status"),
            string_column("promotion_status"),
            bool_column("ontology_truth"),
        ],
    );
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
    record_batch(
        &contract,
        columns,
        "failed to build candidate evidence read-model",
    )
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

fn record_batch(
    contract: &ArrowSchemaContract,
    columns: Vec<ArrayRef>,
    build_context: &'static str,
) -> Result<RecordBatch> {
    let schema = schema_ref(contract);
    let batch = RecordBatch::try_new(schema, columns).context(build_context)?;
    validate_record_batch_schema_with_options(
        &batch,
        contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .context("candidate read-model schema validation failed")?;
    Ok(batch)
}

fn schema_ref(contract: &ArrowSchemaContract) -> SchemaRef {
    Arc::new(build_arrow_schema(
        contract,
        [(
            WENDAO_TABLE_METADATA_KEY.to_string(),
            contract.table_name().to_string(),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    ))
}

const fn string_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

const fn int64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Int64)
}

const fn bool_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Boolean)
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
