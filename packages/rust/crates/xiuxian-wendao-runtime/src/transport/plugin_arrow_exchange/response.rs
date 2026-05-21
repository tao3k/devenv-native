//! Arrow response decoding and validation for Plugin Arrow exchange scoring.

use std::collections::{BTreeMap, BTreeSet};

use arrow_array::{Array, Float64Array, RecordBatch, StringArray};
use xiuxian_wendao_core::repo_intelligence::{
    JULIA_ARROW_ANALYZER_SCORE_COLUMN, JULIA_ARROW_DOC_ID_COLUMN, JULIA_ARROW_FINAL_SCORE_COLUMN,
    JULIA_ARROW_TRACE_ID_COLUMN, RepoIntelligenceError,
};

use super::errors::{contract_decode_error, contract_response_error};

/// One typed row materialized from the `WendaoArrow` plugin response contract.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginArrowScoreRow {
    /// Stable document identifier emitted by the Rust request batch.
    pub doc_id: String,
    /// Plugin-side analyzer score for the document.
    pub analyzer_score: f64,
    /// Final score after plugin-side reranking.
    pub final_score: f64,
    /// Optional trace identifier materialized from additive response columns.
    pub trace_id: Option<String>,
}

/// Decode plugin Arrow response batches into a `doc_id` keyed score map.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the response batch shape does not
/// match the stable `WendaoArrow` `v1` response contract.
pub fn decode_plugin_arrow_score_rows(
    batches: &[RecordBatch],
) -> Result<BTreeMap<String, PluginArrowScoreRow>, RepoIntelligenceError> {
    let mut rows = BTreeMap::new();

    for batch in batches {
        let doc_id = batch
            .column_by_name(JULIA_ARROW_DOC_ID_COLUMN)
            .and_then(|array| array.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| contract_decode_error("missing required Utf8 column `doc_id`"))?;
        let analyzer_score = batch
            .column_by_name(JULIA_ARROW_ANALYZER_SCORE_COLUMN)
            .and_then(|array| array.as_any().downcast_ref::<Float64Array>())
            .ok_or_else(|| {
                contract_decode_error("missing required Float64 column `analyzer_score`")
            })?;
        let final_score = batch
            .column_by_name(JULIA_ARROW_FINAL_SCORE_COLUMN)
            .and_then(|array| array.as_any().downcast_ref::<Float64Array>())
            .ok_or_else(|| {
                contract_decode_error("missing required Float64 column `final_score`")
            })?;
        let trace_id = batch
            .column_by_name(JULIA_ARROW_TRACE_ID_COLUMN)
            .map(|array| {
                array
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| contract_decode_error("optional `trace_id` column must be Utf8"))
            })
            .transpose()?;

        for row in 0..batch.num_rows() {
            let doc_id_value = doc_id
                .is_valid(row)
                .then(|| doc_id.value(row).to_string())
                .ok_or_else(|| contract_decode_error("`doc_id` must be non-null"))?;
            let analyzer_score_value = analyzer_score
                .is_valid(row)
                .then(|| analyzer_score.value(row))
                .ok_or_else(|| contract_decode_error("`analyzer_score` must be non-null"))?;
            let final_score_value = final_score
                .is_valid(row)
                .then(|| final_score.value(row))
                .ok_or_else(|| contract_decode_error("`final_score` must be non-null"))?;

            rows.insert(
                doc_id_value.clone(),
                PluginArrowScoreRow {
                    doc_id: doc_id_value,
                    analyzer_score: analyzer_score_value,
                    final_score: final_score_value,
                    trace_id: trace_id.and_then(|array| {
                        array.is_valid(row).then(|| array.value(row).to_string())
                    }),
                },
            );
        }
    }

    Ok(rows)
}

/// Validate one `WendaoArrow` `v1` plugin response batch set.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the response batches are missing
/// required columns, contain duplicate `doc_id` values across batches, or emit
/// invalid `final_score` values.
pub fn validate_plugin_arrow_response_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    let mut seen_doc_ids = BTreeSet::new();
    for batch in batches {
        validate_plugin_arrow_response_batch(batch, &mut seen_doc_ids)?;
    }

    Ok(())
}

struct PluginArrowResponseColumns<'a> {
    doc_id: &'a StringArray,
    analyzer_score: &'a Float64Array,
    final_score: &'a Float64Array,
}

fn validate_plugin_arrow_response_batch(
    batch: &RecordBatch,
    seen_doc_ids: &mut BTreeSet<String>,
) -> Result<(), RepoIntelligenceError> {
    let columns = plugin_arrow_response_columns(batch)?;
    for row in 0..batch.num_rows() {
        validate_plugin_arrow_response_row(&columns, row, seen_doc_ids)?;
    }
    Ok(())
}

fn plugin_arrow_response_columns(
    batch: &RecordBatch,
) -> Result<PluginArrowResponseColumns<'_>, RepoIntelligenceError> {
    let doc_id = batch
        .column_by_name(JULIA_ARROW_DOC_ID_COLUMN)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| contract_response_error("missing required Utf8 column `doc_id`"))?;
    let analyzer_score = batch
        .column_by_name(JULIA_ARROW_ANALYZER_SCORE_COLUMN)
        .and_then(|array| array.as_any().downcast_ref::<Float64Array>())
        .ok_or_else(|| {
            contract_response_error("missing required Float64 column `analyzer_score`")
        })?;
    let final_score = batch
        .column_by_name(JULIA_ARROW_FINAL_SCORE_COLUMN)
        .and_then(|array| array.as_any().downcast_ref::<Float64Array>())
        .ok_or_else(|| contract_response_error("missing required Float64 column `final_score`"))?;
    Ok(PluginArrowResponseColumns {
        doc_id,
        analyzer_score,
        final_score,
    })
}

fn validate_plugin_arrow_response_row(
    columns: &PluginArrowResponseColumns<'_>,
    row: usize,
    seen_doc_ids: &mut BTreeSet<String>,
) -> Result<(), RepoIntelligenceError> {
    validate_plugin_arrow_response_nulls(columns, row)?;
    let doc_id_value = columns.doc_id.value(row).to_string();
    if !seen_doc_ids.insert(doc_id_value.clone()) {
        return Err(contract_response_error(format!(
            "duplicate `doc_id` in plugin analyzer response: {doc_id_value}"
        )));
    }
    if !columns.final_score.value(row).is_finite() {
        return Err(contract_response_error(format!(
            "`final_score` must be finite for doc_id `{doc_id_value}`"
        )));
    }
    Ok(())
}

fn validate_plugin_arrow_response_nulls(
    columns: &PluginArrowResponseColumns<'_>,
    row: usize,
) -> Result<(), RepoIntelligenceError> {
    if columns.doc_id.is_null(row) {
        return Err(contract_response_error("`doc_id` must be non-null"));
    }
    if columns.analyzer_score.is_null(row) {
        return Err(contract_response_error("`analyzer_score` must be non-null"));
    }
    if columns.final_score.is_null(row) {
        return Err(contract_response_error("`final_score` must be non-null"));
    }
    Ok(())
}
