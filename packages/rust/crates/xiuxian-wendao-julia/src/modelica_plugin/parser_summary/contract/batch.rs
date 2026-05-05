//! Modelica parser-summary request and response batch contract.

use std::sync::Arc;

use arrow::array::StringArray;
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::columns::ModelicaParserSummaryResponseColumns;
use super::values::{
    modelica_parser_summary_request_schema, parser_summary_contract_error,
    parser_summary_request_error, required_utf8_values,
};
use super::{
    MODELICA_PARSER_SUMMARY_REQUEST_ID_COLUMN, MODELICA_PARSER_SUMMARY_SOURCE_ID_COLUMN,
    MODELICA_PARSER_SUMMARY_SOURCE_TEXT_COLUMN, ModelicaParserSummaryRequestRow,
    ModelicaParserSummaryResponseRow,
};

pub(crate) fn build_modelica_parser_summary_request_batch(
    rows: &[ModelicaParserSummaryRequestRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let batch = RecordBatch::try_new(
        modelica_parser_summary_request_schema(),
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
    validate_modelica_parser_summary_request_batches(std::slice::from_ref(&batch))?;
    Ok(batch)
}

pub(crate) fn validate_modelica_parser_summary_request_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        if batch.num_rows() == 0 {
            return Err(parser_summary_contract_error(
                "request",
                "Modelica parser-summary request batch must contain at least one row".to_string(),
            ));
        }
        let _request_id =
            required_utf8_values(batch, MODELICA_PARSER_SUMMARY_REQUEST_ID_COLUMN, "request")?;
        let _source_id =
            required_utf8_values(batch, MODELICA_PARSER_SUMMARY_SOURCE_ID_COLUMN, "request")?;
        let _source_text =
            required_utf8_values(batch, MODELICA_PARSER_SUMMARY_SOURCE_TEXT_COLUMN, "request")?;
    }
    Ok(())
}

pub(crate) fn validate_modelica_parser_summary_response_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        if batch.num_rows() == 0 {
            return Err(parser_summary_contract_error(
                "response",
                "Modelica parser-summary response batch must contain at least one row".to_string(),
            ));
        }
        let _ = ModelicaParserSummaryResponseColumns::read(batch)?;
    }

    Ok(())
}

pub(crate) fn decode_modelica_parser_summary_response_rows(
    batches: &[RecordBatch],
) -> Result<Vec<ModelicaParserSummaryResponseRow>, RepoIntelligenceError> {
    validate_modelica_parser_summary_response_batches(batches)?;
    let mut rows = Vec::new();

    for batch in batches {
        rows.extend(ModelicaParserSummaryResponseColumns::read(batch)?.into_rows());
    }

    Ok(rows)
}
