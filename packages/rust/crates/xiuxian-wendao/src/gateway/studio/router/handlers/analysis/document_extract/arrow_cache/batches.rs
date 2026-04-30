use std::sync::Arc;

use arrow::array::{ArrayRef, Int32Array, Int64Array};
use arrow::record_batch::RecordBatch;

use super::schema::{document_extract_status_schema, document_resource_schema, string_column};
use crate::gateway::studio::router::handlers::analysis::document_extract::registry::DocumentExtractJobStatus;

pub(in super::super) fn build_job_resource_batch(
    status: &DocumentExtractJobStatus,
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_resource_schema(),
        vec![
            string_column([status.source_path.as_str()]),
            string_column(["job"]),
            string_column([status.output_dir.as_str()]),
            Arc::new(Int32Array::from(vec![0])) as ArrayRef,
            string_column(["document extraction job"]),
            string_column([status.status.as_str()]),
            string_column(["application/vnd.xiuxian.document-extract-job"]),
            string_column([status.status.as_str()]),
            string_column([status.job_id.as_str()]),
        ],
    )
    .map_err(|error| format!("build document extract job batch: {error}"))
}

pub(in super::super) fn build_error_resource_batch(
    status: &DocumentExtractJobStatus,
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_resource_schema(),
        vec![
            string_column([status.source_path.as_str()]),
            string_column(["error"]),
            string_column([status.output_dir.as_str()]),
            Arc::new(Int32Array::from(vec![0])) as ArrayRef,
            string_column(["document extraction job failed"]),
            string_column([status.error_message.as_str()]),
            string_column(["text/plain"]),
            string_column(["error"]),
            string_column([status.job_id.as_str()]),
        ],
    )
    .map_err(|error| format!("build document extract error batch: {error}"))
}

pub(in super::super) fn build_status_batch(
    status: &DocumentExtractJobStatus,
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_extract_status_schema(),
        vec![
            string_column([status.job_id.as_str()]),
            string_column([status.source_path.as_str()]),
            string_column([status.output_dir.as_str()]),
            string_column([status.content_hash.as_str()]),
            string_column([status.status.as_str()]),
            Arc::new(Int32Array::from(vec![status.attempt_count])) as ArrayRef,
            Arc::new(Int64Array::from(vec![status.created_at_ms])) as ArrayRef,
            Arc::new(Int64Array::from(vec![status.started_at_ms])) as ArrayRef,
            Arc::new(Int64Array::from(vec![status.finished_at_ms])) as ArrayRef,
            string_column([status.error_message.as_str()]),
        ],
    )
    .map_err(|error| format!("build document extract status batch: {error}"))
}
