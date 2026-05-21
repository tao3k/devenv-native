//! Studio document-extraction API DTOs for browser-facing job state.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{
    StudioContractContentType, StudioContractId, StudioContractMillisecondsI64,
    StudioContractMillisecondsU64, StudioContractMimeType, StudioContractPath,
    StudioContractStatus,
};

/// Document extraction result returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentExtractResult {
    /// Source document path.
    pub source_path: String,
    /// Lowercase source document format inferred from the source extension.
    pub source_format: String,
    /// Total number of extracted resource rows.
    pub total_resources: usize,
    /// Page-like count inferred from resource page indexes when present.
    pub total_pages: usize,
    /// Unix timestamp when extraction completed (from marker file).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_at: Option<i64>,
    /// Extracted structured resources.
    pub resources: Vec<DocumentExtractResource>,
}

/// One extracted resource from a document.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentExtractResource {
    /// Resource type: "document" | "image" | "table" | "formula".
    pub resource_type: StudioContractContentType,
    /// VFS path to the extracted file (empty for inline text).
    pub resource_path: StudioContractPath,
    /// Page index (0-based).
    pub page_index: usize,
    /// Caption or title.
    pub caption: String,
    /// Text / HTML / LaTeX content.
    pub content: String,
    /// MIME type.
    pub mime_type: StudioContractMimeType,
    /// Extraction status: "ok" | "error" | "skipped".
    pub status: StudioContractStatus,
    /// Element ID from the extractor.
    pub element_id: StudioContractId,
}

/// Browser-facing request for submitting an async document extraction job.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentExtractJobSubmitRequest {
    /// Source document path. A VFS path is resolved through Studio when the raw
    /// path does not exist on disk.
    pub source_path: String,
    /// Optional output directory for extracted resources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_dir: Option<String>,
    /// Force reconversion when an existing cache artifact is present.
    #[serde(default)]
    pub force: bool,
    /// Async wait budget in milliseconds.
    #[serde(default)]
    pub wait_ms: u64,
}

/// Browser-facing document extraction job status.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentExtractJobStatus {
    /// Stable job id derived from content hash and converter profile.
    pub job_id: StudioContractId,
    /// Source document path.
    pub source_path: StudioContractPath,
    /// Output directory for extracted resources.
    pub output_dir: String,
    /// SHA-256 content hash for the source document.
    pub content_hash: String,
    /// Job status: queued, running, succeeded, or failed.
    pub status: StudioContractStatus,
    /// Number of conversion attempts.
    pub attempt_count: i32,
    /// Creation timestamp in epoch milliseconds.
    pub created_at_ms: StudioContractMillisecondsI64,
    /// Start timestamp in epoch milliseconds.
    pub started_at_ms: StudioContractMillisecondsI64,
    /// Finish timestamp in epoch milliseconds.
    pub finished_at_ms: StudioContractMillisecondsI64,
    /// Failure message when status is failed.
    pub error_message: String,
}

/// Browser-facing runtime snapshot for async document extraction capacity.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentExtractJobsStatus {
    /// Maximum concurrent cold conversions admitted by the Rust provider.
    pub max_running_conversions: usize,
    /// Currently available conversion permits.
    pub available_conversion_permits: usize,
    /// In-process running conversions inferred from the permit pool.
    pub in_process_running_conversions: usize,
    /// Deployment upper bound for Rust-scheduled PDF OCR workers.
    #[serde(default)]
    pub max_pdf_ocr_workers: usize,
    /// Current adaptive PDF OCR worker budget selected by Rust.
    #[serde(default)]
    pub current_pdf_ocr_worker_budget: usize,
    /// Currently available PDF OCR worker permits.
    #[serde(default)]
    pub available_pdf_ocr_worker_permits: usize,
    /// In-process PDF OCR workers inferred from the permit pool.
    #[serde(default)]
    pub in_process_pdf_ocr_workers: usize,
    /// OCR shard keys currently owned by live Rust scheduler requests.
    #[serde(default)]
    pub in_flight_pdf_ocr_shards: usize,
    /// Cumulative shard cache hits observed by this provider process.
    #[serde(default)]
    pub pdf_ocr_cache_hits: u64,
    /// Cumulative shard cache misses observed by this provider process.
    #[serde(default)]
    pub pdf_ocr_cache_misses: u64,
    /// Cumulative live Python OCR calls issued by this provider process.
    #[serde(default)]
    pub pdf_ocr_live_requests: u64,
    /// Rolling p50 wait before receiving a PDF OCR permit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_ocr_queue_wait_p50_ms: Option<StudioContractMillisecondsU64>,
    /// Rolling p95 wait before receiving a PDF OCR permit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_ocr_queue_wait_p95_ms: Option<StudioContractMillisecondsU64>,
    /// Rolling p50 Python OCR latency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_ocr_latency_p50_ms: Option<StudioContractMillisecondsU64>,
    /// Rolling p95 Python OCR latency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_ocr_latency_p95_ms: Option<StudioContractMillisecondsU64>,
    /// Cumulative source-PDF page-range OCR shard count.
    #[serde(default)]
    pub pdf_ocr_source_pdf_page_range_shards: u64,
    /// Cumulative rendered page OCR shard count.
    #[serde(default)]
    pub pdf_ocr_rendered_page_shards: u64,
    /// Cumulative rendered region OCR shard count.
    #[serde(default)]
    pub pdf_ocr_rendered_region_shards: u64,
    /// Adaptive budget increase events observed by this provider process.
    #[serde(default)]
    pub pdf_ocr_budget_increase_events: u64,
    /// Adaptive budget decrease events observed by this provider process.
    #[serde(default)]
    pub pdf_ocr_budget_decrease_events: u64,
    /// In-process scheduled job tasks waiting or running in this provider.
    pub in_process_scheduled_jobs: usize,
    /// Total persisted jobs in the `DuckDB` registry.
    pub total_jobs: usize,
    /// Persisted queued job count.
    pub queued_jobs: usize,
    /// Persisted running job count.
    pub running_jobs: usize,
    /// Persisted succeeded job count.
    pub succeeded_jobs: usize,
    /// Persisted failed job count.
    pub failed_jobs: usize,
    /// Most recently finished job id, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_finished_job_id: Option<StudioContractId>,
    /// Most recently finished job status, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_finished_status: Option<StudioContractStatus>,
    /// Most recently finished conversion duration in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_conversion_duration_ms: Option<StudioContractMillisecondsI64>,
    /// Maximum finished conversion duration in milliseconds across persisted jobs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_conversion_duration_ms: Option<StudioContractMillisecondsI64>,
}
