use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use xiuxian_llm::model_routing::WendaoModelRoutingTomlConfig;
use xiuxian_wendao_server::transport::{
    DOCUMENT_EXTRACT_FULL_PROFILE, DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE,
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
            model_routing_config: Arc::clone(&state.studio.model_routing_config),
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
            model_routing_config: Arc::new(Ok(None)),
        }
    }

    #[cfg(test)]
    pub(super) fn from_registry_with_document_extract_endpoint(
        registry: Result<DocumentExtractJobRegistry, String>,
        conversion_limit: usize,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            runtime: Arc::new(DocumentExtractProviderRuntime::new(
                registry,
                conversion_limit,
            )),
            configured_default_endpoint: Some(endpoint.into()),
            model_routing_config: Arc::new(Ok(None)),
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
            model_routing_config: Arc::new(Ok(None)),
        }
    }

    #[cfg(all(test, feature = "document-extract-audio-shards"))]
    pub(super) fn from_registry_with_audio_worker_limit(
        registry: Result<DocumentExtractJobRegistry, String>,
        conversion_limit: usize,
        audio_worker_limit: usize,
    ) -> Self {
        Self {
            runtime: Arc::new(DocumentExtractProviderRuntime::new_with_audio_worker_limit(
                registry,
                conversion_limit,
                audio_worker_limit,
            )),
            configured_default_endpoint: None,
            model_routing_config: Arc::new(Ok(None)),
        }
    }

    pub(crate) fn status(&self, job_id: &str) -> Result<Option<DocumentExtractJobStatus>, String> {
        let _registry_guard = self.registry_lock();
        self.registry()?.status(job_id)
    }

    pub(super) fn model_routing_config(
        &self,
    ) -> Result<Option<WendaoModelRoutingTomlConfig>, String> {
        (*self.model_routing_config).clone()
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
        #[cfg(feature = "document-extract-audio-shards")]
        let audio_snapshot = self.runtime.audio_capacity.snapshot();
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
            #[cfg(feature = "document-extract-audio-shards")]
            max_audio_shard_workers: audio_snapshot.max_worker_bound,
            #[cfg(feature = "document-extract-audio-shards")]
            current_audio_shard_worker_budget: audio_snapshot.current_worker_budget,
            #[cfg(feature = "document-extract-audio-shards")]
            audio_shard_healthy_streak: audio_snapshot.healthy_streak,
            #[cfg(feature = "document-extract-audio-shards")]
            audio_shard_budget_increase_events: audio_snapshot.budget_increase_events,
            #[cfg(feature = "document-extract-audio-shards")]
            audio_shard_budget_decrease_events: audio_snapshot.budget_decrease_events,
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
        profile: &str,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        self.sync_document_extract_batch(source_path, output_dir, force, error_row, profile)
            .await
    }

    async fn document_extract_batch_for_request(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        match gateway_document_extract_mode(request) {
            DocumentExtractMode::Sync => {
                let profile = gateway_document_extract_profile_for_source(
                    request.source_path.as_str(),
                    request.profile.as_str(),
                );
                self.document_extract_batch(
                    request.source_path.as_str(),
                    request.output_dir.as_str(),
                    request.force,
                    request.error_row,
                    profile.as_str(),
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
            DocumentExtractMode::AudioShards => {
                #[cfg(feature = "document-extract-audio-shards")]
                {
                    self.audio_shards_document_extract_batch(request).await
                }
                #[cfg(not(feature = "document-extract-audio-shards"))]
                {
                    Err(
                        "`audio-shards` document extraction requires the `document-extract-audio-shards` feature"
                            .to_string(),
                    )
                }
            }
            DocumentExtractMode::Auto => {
                unreachable!("gateway auto mode must resolve before route dispatch")
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

fn gateway_document_extract_mode(request: &DocumentExtractFlightRequest) -> DocumentExtractMode {
    match request.mode {
        DocumentExtractMode::Auto => gateway_document_extract_mode_for_source(&request.source_path),
        mode => mode,
    }
}

pub(crate) fn gateway_document_extract_mode_for_source(source_path: &str) -> DocumentExtractMode {
    if is_audio_source_path(Path::new(source_path)) {
        return DocumentExtractMode::AudioShards;
    }
    DocumentExtractMode::Sync
}

pub(crate) fn gateway_document_extract_profile_for_source(
    source_path: &str,
    requested_profile: &str,
) -> String {
    if requested_profile == DOCUMENT_EXTRACT_FULL_PROFILE
        && is_image_source_path(Path::new(source_path))
    {
        return DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE.to_string();
    }
    requested_profile.to_string()
}

fn is_audio_source_path(source_path: &Path) -> bool {
    let Some(extension) = source_path
        .extension()
        .and_then(|extension| extension.to_str())
    else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "aac" | "flac" | "m4a" | "mp3" | "ogg" | "wav"
    )
}

pub(super) fn is_image_source_path(source_path: &Path) -> bool {
    let Some(extension) = source_path
        .extension()
        .and_then(|extension| extension.to_str())
    else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "bmp" | "gif" | "jpeg" | "jpg" | "png" | "tif" | "tiff" | "webp"
    )
}
