use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Arc;

use async_trait::async_trait;
use xiuxian_wendao_server::transport::{
    DocumentExtractFlightRequest, DocumentExtractFlightRouteProvider,
    DocumentExtractFlightRouteResponse, DocumentExtractMode,
};

#[cfg(test)]
use super::core::DocumentExtractProviderRuntime;
use super::core::{DocumentExtractRuntimeSnapshot, StudioDocumentExtractFlightRouteProvider};
use super::runtime::shared_document_extract_provider_runtime;
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::build_status_batch;
#[cfg(test)]
use crate::studio::router::handlers::analysis::document_extract::registry::DocumentExtractJobRegistry;
use crate::studio::router::handlers::analysis::document_extract::registry::DocumentExtractJobStatus;
use crate::studio::router::{GatewayState, load_document_extract_endpoint_from_wendao_toml};

impl StudioDocumentExtractFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: &GatewayState) -> Self {
        Self {
            runtime: shared_document_extract_provider_runtime(state.studio.project_root.as_path()),
            configured_default_endpoint: load_document_extract_endpoint_from_wendao_toml(
                state.studio.config_root.as_path(),
            ),
        }
    }

    #[cfg(test)]
    pub(super) fn from_registry(
        registry: Result<DocumentExtractJobRegistry, String>,
        conversion_limit: usize,
    ) -> Self {
        Self {
            runtime: Arc::new(DocumentExtractProviderRuntime::new(
                registry,
                conversion_limit,
            )),
            configured_default_endpoint: None,
        }
    }

    #[cfg(all(test, feature = "document-extract-pdf-source-range"))]
    pub(super) fn from_registry_with_pdf_ocr_worker_limit(
        registry: Result<DocumentExtractJobRegistry, String>,
        conversion_limit: usize,
        pdf_ocr_worker_limit: usize,
    ) -> Self {
        Self {
            runtime: Arc::new(
                DocumentExtractProviderRuntime::new_with_pdf_ocr_worker_limit(
                    registry,
                    conversion_limit,
                    pdf_ocr_worker_limit,
                ),
            ),
            configured_default_endpoint: None,
        }
    }

    pub(crate) fn status(&self, job_id: &str) -> Result<Option<DocumentExtractJobStatus>, String> {
        let _registry_guard = self.registry_lock();
        self.registry()?.status(job_id)
    }

    pub(crate) fn succeeded_output_dir_for_source(
        &self,
        source_path: &Path,
    ) -> Result<Option<PathBuf>, String> {
        let _registry_guard = self.registry_lock();
        self.registry()?
            .latest_succeeded_status_for_source(source_path)
            .map(|status: Option<DocumentExtractJobStatus>| {
                status.map(|status| PathBuf::from(status.output_dir))
            })
    }

    pub(crate) async fn runtime_snapshot(&self) -> Result<DocumentExtractRuntimeSnapshot, String> {
        let scheduled_count = self.runtime.scheduled.lock().await.len();
        let available_conversion_permits = self.runtime.conversion_permits.available_permits();
        #[cfg(feature = "document-extract-pdf-source-range")]
        let pdf_ocr_snapshot = self.runtime.pdf_ocr_scheduler.snapshot();
        let registry_snapshot = {
            let _registry_guard = self.registry_lock();
            self.registry()?.snapshot()?
        };
        Ok(DocumentExtractRuntimeSnapshot {
            max_running_conversions: self.runtime.conversion_limit,
            available_conversion_permits,
            in_process_running_conversions: self
                .runtime
                .conversion_limit
                .saturating_sub(available_conversion_permits),
            #[cfg(feature = "document-extract-pdf-source-range")]
            max_pdf_ocr_workers: pdf_ocr_snapshot.max_worker_bound,
            #[cfg(feature = "document-extract-pdf-source-range")]
            current_pdf_ocr_worker_budget: pdf_ocr_snapshot.current_worker_budget,
            #[cfg(feature = "document-extract-pdf-source-range")]
            available_pdf_ocr_worker_permits: pdf_ocr_snapshot.available_worker_permits,
            #[cfg(feature = "document-extract-pdf-source-range")]
            in_process_pdf_ocr_workers: pdf_ocr_snapshot.in_process_workers,
            #[cfg(feature = "document-extract-pdf-source-range")]
            in_flight_pdf_ocr_shards: pdf_ocr_snapshot.in_flight_shards,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_cache_hits: pdf_ocr_snapshot.cache_hits,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_cache_misses: pdf_ocr_snapshot.cache_misses,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_live_requests: pdf_ocr_snapshot.live_requests,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_queue_wait_p50_ms: pdf_ocr_snapshot.queue_wait_p50_ms,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_queue_wait_p95_ms: pdf_ocr_snapshot.queue_wait_p95_ms,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_latency_p50_ms: pdf_ocr_snapshot.ocr_latency_p50_ms,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_latency_p95_ms: pdf_ocr_snapshot.ocr_latency_p95_ms,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_source_pdf_page_range_shards: pdf_ocr_snapshot.source_pdf_page_range_shards,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_rendered_page_shards: pdf_ocr_snapshot.rendered_page_shards,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_rendered_region_shards: pdf_ocr_snapshot.rendered_region_shards,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_budget_increase_events: pdf_ocr_snapshot.budget_increase_events,
            #[cfg(feature = "document-extract-pdf-source-range")]
            pdf_ocr_budget_decrease_events: pdf_ocr_snapshot.budget_decrease_events,
            in_process_scheduled_jobs: scheduled_count,
            registry: registry_snapshot,
        })
    }
}

#[async_trait]
impl DocumentExtractFlightRouteProvider for StudioDocumentExtractFlightRouteProvider {
    async fn document_extract_batch(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        self.sync_document_extract_batch(source_path, output_dir, force, error_row)
            .await
    }

    async fn document_extract_batch_for_request(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        match request.mode {
            DocumentExtractMode::Sync => {
                self.document_extract_batch(
                    request.source_path.as_str(),
                    request.output_dir.as_str(),
                    request.force,
                    request.error_row,
                )
                .await
            }
            DocumentExtractMode::Async => self.async_document_extract_batch(request).await,
            DocumentExtractMode::HybridPageOcr => {
                #[cfg(feature = "document-extract-pdf-source-range")]
                {
                    self.hybrid_page_ocr_document_extract_batch(request).await
                }
                #[cfg(not(feature = "document-extract-pdf-source-range"))]
                {
                    Err(
                        "`hybrid-page-ocr` document extraction requires the `document-extract-pdf-source-range` feature"
                            .to_string(),
                    )
                }
            }
        }
    }

    async fn document_extract_status_batch(
        &self,
        job_id: &str,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let status = self
            .status(job_id)?
            .ok_or_else(|| format!("unknown document extract job id `{job_id}`"))?;
        Ok(DocumentExtractFlightRouteResponse::new(build_status_batch(
            &status,
        )?))
    }
}
