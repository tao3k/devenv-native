use std::sync::Arc;

use arrow::array::{ArrayRef, Int32Array, Int64Array};
use arrow::record_batch::RecordBatch;

use super::schema::{document_extract_status_schema, document_resource_schema, string_column};
use crate::studio::router::handlers::analysis::document_extract::registry::DocumentExtractJobStatus;

pub(crate) fn build_job_resource_batch(
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

pub(crate) fn build_error_resource_batch(
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

#[cfg(all(test, feature = "document-extract-audio-shards"))]
pub(crate) fn build_audio_transcript_resource_batch(
    source_path: &str,
    output_dir: &str,
    transcript: &str,
    element_id: &str,
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_resource_schema(),
        vec![
            string_column([source_path]),
            string_column(["audio-transcript"]),
            string_column([output_dir]),
            Arc::new(Int32Array::from(vec![0])) as ArrayRef,
            string_column(["audio transcript"]),
            string_column([transcript]),
            string_column(["text/plain"]),
            string_column(["ok"]),
            string_column([element_id]),
        ],
    )
    .map_err(|error| format!("build audio transcript resource batch: {error}"))
}

#[cfg(feature = "document-extract-audio-shards")]
pub(crate) fn build_audio_transcript_with_org_resource_batch(
    source_path: &str,
    output_dir: &str,
    transcript: &str,
    org_ledger: &str,
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_resource_schema(),
        vec![
            string_column([source_path, source_path]),
            string_column(["audio-transcript", "audio-transcript-ledger"]),
            string_column([output_dir, output_dir]),
            Arc::new(Int32Array::from(vec![0, 0])) as ArrayRef,
            string_column(["audio transcript", "audio transcript Org ledger"]),
            string_column([transcript, org_ledger]),
            string_column(["text/plain", "text/org"]),
            string_column(["ok", "ok"]),
            string_column(["_audio_transcript", "_audio_transcript_org"]),
        ],
    )
    .map_err(|error| format!("build audio transcript with Org resource batch: {error}"))
}

pub(crate) fn build_status_batch(status: &DocumentExtractJobStatus) -> Result<RecordBatch, String> {
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
