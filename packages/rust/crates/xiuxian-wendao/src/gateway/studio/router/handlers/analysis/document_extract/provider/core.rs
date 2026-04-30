use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex, OnceLock, Weak};

use tokio::sync::{Mutex, Semaphore};
use tonic::transport::Channel;

#[cfg(feature = "document-extract-pdf-source-range")]
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::PdfOcrWorkerScheduler;
use crate::gateway::studio::router::handlers::analysis::document_extract::registry::{
    DocumentExtractJobRegistry, DocumentExtractJobRegistrySnapshot,
};

pub(super) const DEFAULT_DOCUMENT_EXTRACT_ENDPOINT: &str = "http://localhost:50051";
pub(super) const DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES: usize = 256 * 1024 * 1024;
pub(super) const DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS";
pub(super) const DEFAULT_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS: usize = 4;

#[derive(Clone)]
pub(crate) struct StudioDocumentExtractFlightRouteProvider {
    pub(super) runtime: Arc<DocumentExtractProviderRuntime>,
}

pub(super) struct DocumentExtractProviderRuntime {
    pub(super) channel: Arc<Mutex<Option<CachedDocumentExtractChannel>>>,
    pub(super) registry: Arc<Result<DocumentExtractJobRegistry, String>>,
    pub(super) registry_lock: Arc<StdMutex<()>>,
    pub(super) scheduled: Arc<Mutex<HashSet<String>>>,
    pub(super) submit_lock: Arc<Mutex<()>>,
    pub(super) artifact_lock: Arc<Mutex<()>>,
    pub(super) conversion_permits: Arc<Semaphore>,
    pub(super) conversion_limit: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(super) pdf_ocr_scheduler: PdfOcrWorkerScheduler,
}

#[derive(Clone)]
pub(super) struct CachedDocumentExtractChannel {
    pub(super) endpoint_url: String,
    pub(super) channel: Channel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentExtractRuntimeSnapshot {
    pub(crate) max_running_conversions: usize,
    pub(crate) available_conversion_permits: usize,
    pub(crate) in_process_running_conversions: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) max_pdf_ocr_workers: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) current_pdf_ocr_worker_budget: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) available_pdf_ocr_worker_permits: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) in_process_pdf_ocr_workers: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) in_flight_pdf_ocr_shards: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_cache_hits: u64,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_cache_misses: u64,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_live_requests: u64,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_queue_wait_p50_ms: Option<u64>,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_queue_wait_p95_ms: Option<u64>,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_latency_p50_ms: Option<u64>,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_latency_p95_ms: Option<u64>,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_source_pdf_page_range_shards: u64,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_rendered_page_shards: u64,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_rendered_region_shards: u64,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_budget_increase_events: u64,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) pdf_ocr_budget_decrease_events: u64,
    pub(crate) in_process_scheduled_jobs: usize,
    pub(crate) registry: DocumentExtractJobRegistrySnapshot,
}

pub(super) static DOCUMENT_EXTRACT_PROVIDER_RUNTIMES: OnceLock<
    StdMutex<HashMap<PathBuf, Weak<DocumentExtractProviderRuntime>>>,
> = OnceLock::new();
