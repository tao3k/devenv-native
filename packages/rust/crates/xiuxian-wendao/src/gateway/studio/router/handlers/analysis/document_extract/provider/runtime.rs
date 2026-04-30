use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex as StdMutex, Weak};

use tokio::sync::{Mutex, Semaphore};

#[cfg(feature = "document-extract-pdf-source-range")]
use super::super::pdf_ocr_scheduler::PdfOcrWorkerScheduler;
use super::super::registry::DocumentExtractJobRegistry;
use super::{
    DEFAULT_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS, DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV,
    DOCUMENT_EXTRACT_PROVIDER_RUNTIMES, DocumentExtractProviderRuntime,
    StudioDocumentExtractFlightRouteProvider,
};

impl DocumentExtractProviderRuntime {
    pub(super) fn new(
        registry: Result<DocumentExtractJobRegistry, String>,
        conversion_limit: usize,
    ) -> Self {
        #[cfg(feature = "document-extract-pdf-source-range")]
        {
            Self::new_with_pdf_ocr_scheduler(
                registry,
                conversion_limit,
                PdfOcrWorkerScheduler::from_environment(),
            )
        }
        #[cfg(not(feature = "document-extract-pdf-source-range"))]
        {
            Self::new_without_pdf_ocr_scheduler(registry, conversion_limit)
        }
    }

    #[cfg(not(feature = "document-extract-pdf-source-range"))]
    fn new_without_pdf_ocr_scheduler(
        registry: Result<DocumentExtractJobRegistry, String>,
        conversion_limit: usize,
    ) -> Self {
        let conversion_limit = conversion_limit.max(1);
        Self {
            channel: Arc::new(Mutex::new(None)),
            registry: Arc::new(registry),
            registry_lock: Arc::new(StdMutex::new(())),
            scheduled: Arc::new(Mutex::new(HashSet::new())),
            submit_lock: Arc::new(Mutex::new(())),
            artifact_lock: Arc::new(Mutex::new(())),
            conversion_permits: Arc::new(Semaphore::new(conversion_limit)),
            conversion_limit,
        }
    }

    #[cfg(all(test, feature = "document-extract-pdf-source-range"))]
    pub(super) fn new_with_pdf_ocr_worker_limit(
        registry: Result<DocumentExtractJobRegistry, String>,
        conversion_limit: usize,
        pdf_ocr_worker_limit: usize,
    ) -> Self {
        Self::new_with_pdf_ocr_scheduler(
            registry,
            conversion_limit,
            PdfOcrWorkerScheduler::with_limit(pdf_ocr_worker_limit),
        )
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    fn new_with_pdf_ocr_scheduler(
        registry: Result<DocumentExtractJobRegistry, String>,
        conversion_limit: usize,
        pdf_ocr_scheduler: PdfOcrWorkerScheduler,
    ) -> Self {
        let conversion_limit = conversion_limit.max(1);
        Self {
            channel: Arc::new(Mutex::new(None)),
            registry: Arc::new(registry),
            registry_lock: Arc::new(StdMutex::new(())),
            scheduled: Arc::new(Mutex::new(HashSet::new())),
            submit_lock: Arc::new(Mutex::new(())),
            artifact_lock: Arc::new(Mutex::new(())),
            conversion_permits: Arc::new(Semaphore::new(conversion_limit)),
            conversion_limit,
            pdf_ocr_scheduler,
        }
    }
}

pub(super) fn shared_document_extract_provider_runtime(
    project_root: &Path,
) -> Arc<DocumentExtractProviderRuntime> {
    let key = project_root.to_path_buf();
    let runtimes = DOCUMENT_EXTRACT_PROVIDER_RUNTIMES.get_or_init(|| StdMutex::new(HashMap::new()));
    let mut guard = runtimes
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(runtime) = guard.get(&key).and_then(Weak::upgrade) {
        return runtime;
    }

    let runtime = Arc::new(DocumentExtractProviderRuntime::new(
        DocumentExtractJobRegistry::default_for_project(project_root),
        document_extract_conversion_concurrency_limit(),
    ));
    guard.insert(key, Arc::downgrade(&runtime));
    runtime
}

pub(super) fn document_extract_conversion_concurrency_limit() -> usize {
    document_extract_conversion_concurrency_limit_with_lookup(
        &|key| std::env::var(key).ok(),
        std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .ok(),
    )
}

pub(super) fn document_extract_conversion_concurrency_limit_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
    available_parallelism: Option<usize>,
) -> usize {
    if let Some(limit) = lookup(DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|limit| *limit > 0)
    {
        return limit;
    }

    available_parallelism
        .unwrap_or(1)
        .clamp(1, DEFAULT_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS)
}

impl std::fmt::Debug for StudioDocumentExtractFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioDocumentExtractFlightRouteProvider")
            .finish_non_exhaustive()
    }
}
