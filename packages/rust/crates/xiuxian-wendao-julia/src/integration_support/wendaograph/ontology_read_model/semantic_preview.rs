//! Semantic preview artifact conversion for `WendaoGraph` ontology quality checks.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use serde::Deserialize;

use super::types::WendaoGraphOntologyReadModelQualityRequestBatches;

const SEMANTIC_OBJECTS_TSV: &str = "semantic_objects.tsv";
const SEMANTIC_RELATIONS_TSV: &str = "semantic_relations.tsv";
const SEMANTIC_PROJECTION_STATE_JSON: &str = "semantic_projection_state.json";
const RDF_SOURCE_SEMANTIC_OBJECTS_TSV: &str = "rdf_source_semantic_objects.tsv";
const RDF_SOURCE_SEMANTIC_RELATIONS_TSV: &str = "rdf_source_semantic_relations.tsv";
const RDF_SOURCE_SEMANTIC_PROJECTION_STATE_JSON: &str = "rdf_source_projection_state.json";

const SEMANTIC_OBJECT_COLUMNS: &[SemanticPreviewColumn] = &[
    SemanticPreviewColumn::string("id"),
    SemanticPreviewColumn::string("kind"),
    SemanticPreviewColumn::string("title"),
    SemanticPreviewColumn::string("domain"),
    SemanticPreviewColumn::string("evidence_id"),
    SemanticPreviewColumn::string("evidence_status"),
    SemanticPreviewColumn::string("target_rdf_file"),
    SemanticPreviewColumn::string("review_decision"),
    SemanticPreviewColumn::string("promotion_decision"),
    SemanticPreviewColumn::string("reviewer_id"),
    SemanticPreviewColumn::int64("relation_count"),
    SemanticPreviewColumn::string("status"),
    SemanticPreviewColumn::string("read_model_projection_staleness"),
];

const SEMANTIC_RELATION_COLUMNS: &[SemanticPreviewColumn] = &[
    SemanticPreviewColumn::string("id"),
    SemanticPreviewColumn::string("kind"),
    SemanticPreviewColumn::string("source"),
    SemanticPreviewColumn::string("target"),
    SemanticPreviewColumn::string("domain"),
    SemanticPreviewColumn::string("evidence_id"),
    SemanticPreviewColumn::string("evidence_status"),
    SemanticPreviewColumn::string("target_rdf_file"),
    SemanticPreviewColumn::string("review_decision"),
    SemanticPreviewColumn::string("promotion_decision"),
    SemanticPreviewColumn::string("reviewer_id"),
    SemanticPreviewColumn::string("status"),
    SemanticPreviewColumn::string("read_model_projection_staleness"),
];

const SEMANTIC_PROJECTION_STATE_COLUMNS: &[SemanticPreviewColumn] = &[
    SemanticPreviewColumn::string("projection"),
    SemanticPreviewColumn::string("status"),
    SemanticPreviewColumn::string("staleness"),
    SemanticPreviewColumn::int64("source_object_count"),
    SemanticPreviewColumn::int64("source_relation_count"),
    SemanticPreviewColumn::int64("source_evidence_count"),
];

/// Build `WendaoGraph` quality request tables from compiled Episteme semantic preview artifacts.
///
/// This converter accepts only generated read-model artifacts. It does not read
/// private corpus files, RDF, `episteme.toml`, or `wendao.toml`.
///
/// # Errors
///
/// Returns an error when required artifact files are missing, TSV/JSON content
/// is malformed, required columns are absent, numeric fields are invalid, or a
/// semantic relation points to an object ID absent from `semantic_objects.tsv`.
pub fn build_wendaograph_ontology_read_model_quality_request_batches_from_semantic_preview_artifacts(
    run_dir: impl AsRef<Path>,
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, String> {
    build_request_batches_from_artifacts(
        run_dir.as_ref(),
        SemanticArtifactPaths {
            objects_tsv: SEMANTIC_OBJECTS_TSV,
            relations_tsv: SEMANTIC_RELATIONS_TSV,
            projection_state_json: SEMANTIC_PROJECTION_STATE_JSON,
            artifact_label: "semantic preview",
        },
    )
}

/// Build `WendaoGraph` quality request tables from applied-RDF source read-model artifacts.
///
/// This converter accepts only generated read-model artifacts. It does not read
/// private corpus files, RDF, `episteme.toml`, or `wendao.toml`.
///
/// # Errors
///
/// Returns an error when required artifact files are missing, TSV/JSON content
/// is malformed, required columns are absent, numeric fields are invalid, or a
/// semantic relation points to an object ID absent from the RDF-source object table.
pub fn build_wendaograph_ontology_read_model_quality_request_batches_from_rdf_source_artifacts(
    run_dir: impl AsRef<Path>,
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, String> {
    build_request_batches_from_artifacts(
        run_dir.as_ref(),
        SemanticArtifactPaths {
            objects_tsv: RDF_SOURCE_SEMANTIC_OBJECTS_TSV,
            relations_tsv: RDF_SOURCE_SEMANTIC_RELATIONS_TSV,
            projection_state_json: RDF_SOURCE_SEMANTIC_PROJECTION_STATE_JSON,
            artifact_label: "RDF source read-model",
        },
    )
}

#[derive(Debug, Clone, Copy)]
struct SemanticArtifactPaths {
    objects_tsv: &'static str,
    relations_tsv: &'static str,
    projection_state_json: &'static str,
    artifact_label: &'static str,
}

fn build_request_batches_from_artifacts(
    run_dir: &Path,
    artifacts: SemanticArtifactPaths,
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, String> {
    let object_rows = read_tsv_rows(
        &run_dir.join(artifacts.objects_tsv),
        artifacts.artifact_label,
        "semantic_objects",
        SEMANTIC_OBJECT_COLUMNS,
    )?;
    let relation_rows = read_tsv_rows(
        &run_dir.join(artifacts.relations_tsv),
        artifacts.artifact_label,
        "semantic_relations",
        SEMANTIC_RELATION_COLUMNS,
    )?;
    let projection_rows = read_projection_state_rows(
        &run_dir.join(artifacts.projection_state_json),
        artifacts.artifact_label,
    )?;

    validate_relation_endpoints(&object_rows, &relation_rows)?;

    Ok(WendaoGraphOntologyReadModelQualityRequestBatches::new(
        semantic_tsv_batch("semantic_objects", SEMANTIC_OBJECT_COLUMNS, &object_rows)?,
        semantic_tsv_batch(
            "semantic_relations",
            SEMANTIC_RELATION_COLUMNS,
            &relation_rows,
        )?,
        semantic_projection_state_batch(&projection_rows)?,
    ))
}

#[derive(Debug, Clone, Copy)]
struct SemanticPreviewColumn {
    name: &'static str,
    data_type: SemanticPreviewDataType,
}

#[derive(Debug, Clone, Copy)]
enum SemanticPreviewDataType {
    String,
    Int64,
}

impl SemanticPreviewColumn {
    const fn string(name: &'static str) -> Self {
        Self {
            name,
            data_type: SemanticPreviewDataType::String,
        }
    }

    const fn int64(name: &'static str) -> Self {
        Self {
            name,
            data_type: SemanticPreviewDataType::Int64,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SemanticPreviewProjectionStateRow {
    projection: String,
    status: String,
    staleness: String,
    source_object_count: i64,
    source_relation_count: i64,
    source_evidence_count: i64,
}

fn read_tsv_rows(
    path: &Path,
    artifact_label: &str,
    table_name: &str,
    required_columns: &[SemanticPreviewColumn],
) -> Result<Vec<BTreeMap<String, String>>, String> {
    let body = fs::read_to_string(path)
        .map_err(|error| format!("read `{}` {artifact_label} TSV: {error}", path.display()))?;
    let mut lines = body.lines();
    let header = lines
        .next()
        .ok_or_else(|| format!("semantic preview `{table_name}` TSV is empty"))?;
    let columns = header.split('\t').map(str::to_owned).collect::<Vec<_>>();
    require_columns(table_name, &columns, required_columns)?;

    let mut rows = Vec::new();
    for (line_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let values = line.split('\t').map(unescape_tsv).collect::<Vec<_>>();
        if values.len() != columns.len() {
            return Err(format!(
                "semantic preview `{table_name}` TSV row {} has {} values for {} columns",
                line_index + 2,
                values.len(),
                columns.len()
            ));
        }
        let row = columns
            .iter()
            .cloned()
            .zip(values)
            .collect::<BTreeMap<_, _>>();
        require_nonblank_values(table_name, line_index + 2, &row, required_columns)?;
        rows.push(row);
    }

    if rows.is_empty() {
        return Err(format!(
            "semantic preview `{table_name}` TSV must contain at least one data row"
        ));
    }

    Ok(rows)
}

fn read_projection_state_rows(
    path: &Path,
    artifact_label: &str,
) -> Result<Vec<SemanticPreviewProjectionStateRow>, String> {
    let body = fs::read_to_string(path).map_err(|error| {
        format!(
            "read `{}` {artifact_label} projection state JSON: {error}",
            path.display()
        )
    })?;
    let rows =
        serde_json::from_str::<Vec<SemanticPreviewProjectionStateRow>>(&body).map_err(|error| {
            format!(
                "semantic preview `{}` projection state JSON is invalid: {error}",
                path.display()
            )
        })?;
    if rows.is_empty() {
        return Err(
            "semantic preview projection state JSON must contain at least one row".to_string(),
        );
    }
    for (index, row) in rows.iter().enumerate() {
        require_projection_nonblank(index + 1, "projection", row.projection.as_str())?;
        require_projection_nonblank(index + 1, "status", row.status.as_str())?;
        require_projection_nonblank(index + 1, "staleness", row.staleness.as_str())?;
    }
    Ok(rows)
}

fn require_columns(
    table_name: &str,
    columns: &[String],
    required_columns: &[SemanticPreviewColumn],
) -> Result<(), String> {
    let present = columns.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let missing = required_columns
        .iter()
        .filter_map(|column| (!present.contains(column.name)).then_some(column.name))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "semantic preview `{table_name}` TSV is missing required column(s): {}",
            missing.join(", ")
        ))
    }
}

fn require_nonblank_values(
    table_name: &str,
    row_number: usize,
    row: &BTreeMap<String, String>,
    required_columns: &[SemanticPreviewColumn],
) -> Result<(), String> {
    for column in required_columns {
        let value = required_value(row, column.name, table_name, row_number)?;
        if value.trim().is_empty() {
            return Err(format!(
                "semantic preview `{table_name}` TSV row {row_number} has blank `{}`",
                column.name
            ));
        }
    }
    Ok(())
}

fn require_projection_nonblank(
    row_number: usize,
    field_name: &str,
    value: &str,
) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!(
            "semantic preview projection state row {row_number} has blank `{field_name}`"
        ));
    }
    Ok(())
}

fn validate_relation_endpoints(
    object_rows: &[BTreeMap<String, String>],
    relation_rows: &[BTreeMap<String, String>],
) -> Result<(), String> {
    let object_ids = object_rows
        .iter()
        .map(|row| required_value(row, "id", "semantic_objects", 0))
        .collect::<Result<BTreeSet<_>, _>>()?;
    for (index, relation) in relation_rows.iter().enumerate() {
        let row_number = index + 2;
        let source = required_value(relation, "source", "semantic_relations", row_number)?;
        let target = required_value(relation, "target", "semantic_relations", row_number)?;
        if !object_ids.contains(source) {
            return Err(format!(
                "semantic preview `semantic_relations` TSV row {row_number} references unknown source `{source}`"
            ));
        }
        if !object_ids.contains(target) {
            return Err(format!(
                "semantic preview `semantic_relations` TSV row {row_number} references unknown target `{target}`"
            ));
        }
    }
    Ok(())
}

fn semantic_tsv_batch(
    table_name: &str,
    columns: &[SemanticPreviewColumn],
    rows: &[BTreeMap<String, String>],
) -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(
        columns
            .iter()
            .map(|column| Field::new(column.name, column.arrow_data_type(), false))
            .collect::<Vec<_>>(),
    ));
    let arrays = columns
        .iter()
        .map(|column| column.array_from_tsv_rows(table_name, rows))
        .collect::<Result<Vec<_>, _>>()?;

    RecordBatch::try_new(schema, arrays)
        .map_err(|error| format!("build semantic preview `{table_name}` batch: {error}"))
}

fn semantic_projection_state_batch(
    rows: &[SemanticPreviewProjectionStateRow],
) -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(
        SEMANTIC_PROJECTION_STATE_COLUMNS
            .iter()
            .map(|column| Field::new(column.name, column.arrow_data_type(), false))
            .collect::<Vec<_>>(),
    ));

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.projection.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.status.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.staleness.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Int64Array::from(
                rows.iter()
                    .map(|row| row.source_object_count)
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Int64Array::from(
                rows.iter()
                    .map(|row| row.source_relation_count)
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Int64Array::from(
                rows.iter()
                    .map(|row| row.source_evidence_count)
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build semantic preview `semantic_projection_state` batch: {error}"))
}

impl SemanticPreviewColumn {
    fn arrow_data_type(self) -> DataType {
        match self.data_type {
            SemanticPreviewDataType::String => DataType::Utf8,
            SemanticPreviewDataType::Int64 => DataType::Int64,
        }
    }

    fn array_from_tsv_rows(
        self,
        table_name: &str,
        rows: &[BTreeMap<String, String>],
    ) -> Result<ArrayRef, String> {
        match self.data_type {
            SemanticPreviewDataType::String => {
                let values = rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| required_value(row, self.name, table_name, index + 2))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Arc::new(StringArray::from(values)) as ArrayRef)
            }
            SemanticPreviewDataType::Int64 => {
                let values = rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| {
                        let value = required_value(row, self.name, table_name, index + 2)?;
                        value.parse::<i64>().map_err(|error| {
                            format!(
                                "semantic preview `{table_name}` TSV row {} field `{}` must be int64: {error}",
                                index + 2,
                                self.name
                            )
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Arc::new(Int64Array::from(values)) as ArrayRef)
            }
        }
    }
}

fn required_value<'a>(
    row: &'a BTreeMap<String, String>,
    column_name: &str,
    table_name: &str,
    row_number: usize,
) -> Result<&'a str, String> {
    row.get(column_name).map(String::as_str).ok_or_else(|| {
        let row_label = if row_number == 0 {
            "unknown".to_string()
        } else {
            row_number.to_string()
        };
        format!("semantic preview `{table_name}` TSV row {row_label} is missing `{column_name}`")
    })
}

fn unescape_tsv(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('t') => output.push('\t'),
                Some('n') => output.push('\n'),
                Some('r') => output.push('\r'),
                Some('\\') | None => output.push('\\'),
                Some(other) => {
                    output.push('\\');
                    output.push(other);
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}
