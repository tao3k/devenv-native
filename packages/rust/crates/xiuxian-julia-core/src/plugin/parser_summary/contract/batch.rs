//! Julia parser-summary request and response batch contract.

use std::sync::Arc;

use arrow::array::StringArray;
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::columns::JuliaParserSummaryResponseColumns;
use super::values::{
    julia_parser_summary_request_schema, parser_summary_contract_error,
    parser_summary_request_error, required_utf8_values,
    validate_julia_parser_summary_request_schema,
};
use super::{
    JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN, JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN,
    JULIA_PARSER_SUMMARY_SOURCE_TEXT_COLUMN, JuliaParserSummaryRequestRow,
    JuliaParserSummaryResponseRow,
};

pub(crate) fn build_julia_parser_summary_request_batch(
    rows: &[JuliaParserSummaryRequestRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let batch = RecordBatch::try_new(
        julia_parser_summary_request_schema(),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.request_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.source_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.source_text.as_str())
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| parser_summary_request_error(error.to_string()))?;
    validate_julia_parser_summary_request_schema(&batch)?;
    validate_julia_parser_summary_request_batches(std::slice::from_ref(&batch))?;
    Ok(batch)
}

pub(crate) fn validate_julia_parser_summary_request_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        if batch.num_rows() == 0 {
            return Err(parser_summary_contract_error(
                "request",
                "parser-summary request batch must contain at least one row".to_string(),
            ));
        }

        let _request_id =
            required_utf8_values(batch, JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN, "request")?;
        let _source_id =
            required_utf8_values(batch, JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN, "request")?;
        let _source_text =
            required_utf8_values(batch, JULIA_PARSER_SUMMARY_SOURCE_TEXT_COLUMN, "request")?;
    }

    Ok(())
}

pub(crate) fn validate_julia_parser_summary_response_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    if batches.is_empty() {
        return Err(parser_summary_contract_error(
            "response",
            "parser-summary response stream returned no record batches; the Flight transport likely terminated before emitting the first schema-bearing response batch".to_string(),
        ));
    }

    for batch in batches {
        if batch.num_rows() == 0 {
            return Err(parser_summary_contract_error(
                "response",
                "parser-summary response batch must contain at least one row".to_string(),
            ));
        }
        let _ = JuliaParserSummaryResponseColumns::read(batch)?;
    }

    Ok(())
}

pub(crate) fn decode_julia_parser_summary_response_rows(
    batches: &[RecordBatch],
) -> Result<Vec<JuliaParserSummaryResponseRow>, RepoIntelligenceError> {
    let mut rows = Vec::new();

    for batch in batches {
        rows.extend(JuliaParserSummaryResponseColumns::read(batch)?.into_rows());
    }

    Ok(rows)
}
