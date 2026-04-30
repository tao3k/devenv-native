use std::collections::{HashMap, HashSet};
#[cfg(feature = "document-extract-pdf-source-range")]
use std::fs::File;
#[cfg(feature = "document-extract-pdf-source-range")]
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex, OnceLock, Weak};
use std::time::{Duration, Instant};

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_flight::client::FlightClient;
use arrow_flight::flight_service_client::FlightServiceClient as TonicFlightServiceClient;
use async_trait::async_trait;
use futures::TryStreamExt;
#[cfg(feature = "document-extract-pdf-source-range")]
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, Semaphore};
use tonic::transport::{Channel, Endpoint};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE, DocumentExtractFlightRequest,
    DocumentExtractFlightRouteProvider, DocumentExtractFlightRouteResponse, DocumentExtractMode,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER, WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
};

#[cfg(feature = "document-extract-pdf-source-range")]
use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, PdfOcrShardResultStatus, decode_ocr_shard_input_batches,
};
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::render::render_pdf_region_shards;
#[cfg(feature = "document-extract-pdf-source-range")]
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageRegionRenderRequest, PdfPageRenderProfile, PdfPageRenderSelection,
    PdfPageRenderShardReport, PdfRenderRoutingDecision, PdfRenderStatus,
    prepare_pdf_source_page_range_ocr_shards_with_selection,
};
#[cfg(feature = "document-extract-pdf-source-range")]
use xiuxian_wendao_attachments::pdf::structure::{
    DOCUMENT_STRUCTURE_ARROW_CACHE_NAME, DocumentStructureBlock, build_document_structure_batch,
    document_resource_batch_to_structure_blocks,
};

use super::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, build_error_resource_batch, build_job_resource_batch,
    build_status_batch, mirror_artifact_to_output, read_arrow_file, read_cached_document_batches,
    write_arrow_file,
};
#[cfg(feature = "document-extract-pdf-source-range")]
use super::pdf_ocr_scheduler::PdfOcrWorkerScheduler;
use super::registry::{
    DocumentExtractJobRegistry, DocumentExtractJobRegistrySnapshot, DocumentExtractJobStatus,
    artifact_ready, default_output_dir,
};
use crate::gateway::studio::router::GatewayState;

const DEFAULT_DOCUMENT_EXTRACT_ENDPOINT: &str = "http://localhost:50051";
const DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES: usize = 256 * 1024 * 1024;
const DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS";
#[cfg(feature = "document-extract-pdf-source-range")]
const DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION";
#[cfg(feature = "document-extract-pdf-source-range")]
const DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON";
const DEFAULT_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS: usize = 4;

#[cfg(feature = "document-extract-pdf-source-range")]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct HybridPdfRegionInput {
    source: PathBuf,
    regions: Vec<PdfPageRegionRenderRequest>,
}

#[cfg(feature = "document-extract-pdf-source-range")]
struct HybridDocumentResourceBatch {
    batch: EngineRecordBatch,
    ocr_inputs: Vec<PdfOcrShardInput>,
    ocr_results: Vec<PdfOcrShardResult>,
}

#[cfg(feature = "document-extract-pdf-source-range")]
impl HybridDocumentResourceBatch {
    #[cfg(test)]
    fn native(batch: EngineRecordBatch) -> Self {
        Self {
            batch,
            ocr_inputs: Vec::new(),
            ocr_results: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct StudioDocumentExtractFlightRouteProvider {
    runtime: Arc<DocumentExtractProviderRuntime>,
}

struct DocumentExtractProviderRuntime {
    channel: Arc<Mutex<Option<CachedDocumentExtractChannel>>>,
    registry: Arc<Result<DocumentExtractJobRegistry, String>>,
    registry_lock: Arc<StdMutex<()>>,
    scheduled: Arc<Mutex<HashSet<String>>>,
    submit_lock: Arc<Mutex<()>>,
    artifact_lock: Arc<Mutex<()>>,
    conversion_permits: Arc<Semaphore>,
    conversion_limit: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pdf_ocr_scheduler: PdfOcrWorkerScheduler,
}

#[derive(Clone)]
struct CachedDocumentExtractChannel {
    endpoint_url: String,
    channel: Channel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentExtractRuntimeSnapshot {
    pub(crate) max_running_conversions: usize,
    pub(crate) available_conversion_permits: usize,
    pub(crate) in_process_running_conversions: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) max_pdf_ocr_workers: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) available_pdf_ocr_worker_permits: usize,
    #[cfg(feature = "document-extract-pdf-source-range")]
    pub(crate) in_process_pdf_ocr_workers: usize,
    pub(crate) in_process_scheduled_jobs: usize,
    pub(crate) registry: DocumentExtractJobRegistrySnapshot,
}

static DOCUMENT_EXTRACT_PROVIDER_RUNTIMES: OnceLock<
    StdMutex<HashMap<PathBuf, Weak<DocumentExtractProviderRuntime>>>,
> = OnceLock::new();

impl StudioDocumentExtractFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: &GatewayState) -> Self {
        Self {
            runtime: shared_document_extract_provider_runtime(state.studio.project_root.as_path()),
        }
    }

    #[cfg(test)]
    fn from_registry(
        registry: Result<DocumentExtractJobRegistry, String>,
        conversion_limit: usize,
    ) -> Self {
        Self {
            runtime: Arc::new(DocumentExtractProviderRuntime::new(
                registry,
                conversion_limit,
            )),
        }
    }

    #[cfg(all(test, feature = "document-extract-pdf-source-range"))]
    fn from_registry_with_pdf_ocr_worker_limit(
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
        }
    }

    pub(crate) async fn submit_document_extract_job(
        &self,
        source_path: &str,
        output_dir: Option<&str>,
        force: bool,
        wait_ms: u64,
    ) -> Result<DocumentExtractJobStatus, String> {
        let source = PathBuf::from(source_path);
        let output = output_dir
            .filter(|value| !value.trim().is_empty())
            .map_or_else(|| default_output_dir(source.as_path()), PathBuf::from);
        let registry = self.registry()?;
        let status = {
            let _guard = self.runtime.submit_lock.lock().await;
            let _registry_guard = self.registry_lock();
            registry.submit(source.as_path(), output.as_path(), force)?
        };
        if matches!(status.status.as_str(), "queued" | "running") {
            self.schedule_job(status.job_id.clone()).await;
        }
        if wait_ms == 0 {
            return Ok(status);
        }
        self.wait_for_terminal_status(status, wait_ms).await
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
            .map(|status| status.map(|status| PathBuf::from(status.output_dir)))
    }

    pub(crate) async fn runtime_snapshot(&self) -> Result<DocumentExtractRuntimeSnapshot, String> {
        let scheduled_count = self.runtime.scheduled.lock().await.len();
        let available_conversion_permits = self.runtime.conversion_permits.available_permits();
        #[cfg(feature = "document-extract-pdf-source-range")]
        let available_pdf_ocr_worker_permits = self.runtime.pdf_ocr_scheduler.available_permits();
        #[cfg(feature = "document-extract-pdf-source-range")]
        let max_pdf_ocr_workers = self.runtime.pdf_ocr_scheduler.worker_limit();
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
            max_pdf_ocr_workers,
            #[cfg(feature = "document-extract-pdf-source-range")]
            available_pdf_ocr_worker_permits,
            #[cfg(feature = "document-extract-pdf-source-range")]
            in_process_pdf_ocr_workers: max_pdf_ocr_workers
                .saturating_sub(available_pdf_ocr_worker_permits),
            in_process_scheduled_jobs: scheduled_count,
            registry: registry_snapshot,
        })
    }

    async fn channel_for_endpoint(&self, endpoint_url: &str) -> Result<Channel, String> {
        {
            let cached = self.runtime.channel.lock().await;
            if let Some(cached) = cached.as_ref()
                && cached.endpoint_url == endpoint_url
            {
                return Ok(cached.channel.clone());
            }
        }

        let endpoint = Endpoint::from_shared(endpoint_url.to_string()).map_err(|error| {
            format!("invalid document extract endpoint `{endpoint_url}`: {error}")
        })?;

        let channel = endpoint.connect().await.map_err(|error| {
            format!("failed to connect to document extract endpoint `{endpoint_url}`: {error}")
        })?;

        let mut cached = self.runtime.channel.lock().await;
        *cached = Some(CachedDocumentExtractChannel {
            endpoint_url: endpoint_url.to_string(),
            channel: channel.clone(),
        });
        Ok(channel)
    }

    async fn async_document_extract_batch(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let source = PathBuf::from(request.source_path.as_str());
        let output = if request.output_dir.trim().is_empty() {
            default_output_dir(source.as_path())
        } else {
            PathBuf::from(request.output_dir.as_str())
        };
        if source.exists()
            && !request.force
            && let Some(batches) = read_cached_document_batches(source.as_path(), output.as_path())?
        {
            return Ok(DocumentExtractFlightRouteResponse::from_batches(batches));
        }

        let output_string = output.to_string_lossy().to_string();
        let mut status = self
            .submit_document_extract_job(
                request.source_path.as_str(),
                Some(output_string.as_str()),
                request.force,
                request.wait_ms,
            )
            .await?;

        if status.status == "succeeded" {
            let _guard = self.runtime.artifact_lock.lock().await;
            Self::mirror_and_read_succeeded(&status, output.as_path())
        } else if status.status == "failed" {
            if request.error_row {
                Ok(DocumentExtractFlightRouteResponse::new(
                    build_error_resource_batch(&status)?,
                ))
            } else {
                Err(status.error_message)
            }
        } else {
            if request.wait_ms > 0
                && let Some(current) = self.status(status.job_id.as_str())?
            {
                status = current;
            }
            Ok(DocumentExtractFlightRouteResponse::new(
                build_job_resource_batch(&status)?,
            ))
        }
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    async fn hybrid_page_ocr_document_extract_batch(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let (source, output) = hybrid_page_ocr_request_paths(request);
        if source.exists()
            && !request.force
            && let Some(batches) = read_cached_document_batches(source.as_path(), output.as_path())?
        {
            return Ok(DocumentExtractFlightRouteResponse::from_batches(batches));
        }

        tokio::fs::create_dir_all(output.as_path())
            .await
            .map_err(|error| {
                format!(
                    "create hybrid PDF OCR output directory `{}`: {error}",
                    output.display()
                )
            })?;

        let render_report = match render_hybrid_page_ocr_shards(source.as_path(), output.as_path())
            .await
        {
            Ok(report) => report,
            Err(reason) => {
                return self
                    .fallback_python_document_extract(request, output.as_path(), reason.as_str())
                    .await;
            }
        };

        let resource_batch = {
            let ocr_input_path = match hybrid_page_ocr_input_arrow_path(&render_report) {
                Ok(path) => path,
                Err(reason) => {
                    return self
                        .fallback_python_document_extract(
                            request,
                            output.as_path(),
                            reason.as_str(),
                        )
                        .await;
                }
            };

            let input_batches = read_arrow_file(ocr_input_path.as_path())?;
            let inputs = decode_ocr_shard_input_batches(&input_batches)?;
            if inputs.is_empty() {
                return self
                    .fallback_python_document_extract(
                        request,
                        output.as_path(),
                        "hybrid PDF OCR route found no OCR shard inputs",
                    )
                    .await;
            }

            match materialize_hybrid_page_ocr_resource_batch(
                source.as_path(),
                &render_report,
                inputs,
                &self.runtime.pdf_ocr_scheduler,
            )
            .await
            {
                Ok(batch) => batch,
                Err(reason) => {
                    return self
                        .fallback_python_document_extract(
                            request,
                            output.as_path(),
                            reason.as_str(),
                        )
                        .await;
                }
            }
        };
        write_hybrid_document_resource_artifacts(
            output.as_path(),
            source.as_path(),
            &resource_batch,
        )?;
        tokio::fs::File::create(output.join("_complete.marker"))
            .await
            .map_err(|error| format!("touch hybrid PDF OCR complete marker: {error}"))?;

        Ok(DocumentExtractFlightRouteResponse::new(
            resource_batch.batch,
        ))
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    async fn fallback_python_document_extract(
        &self,
        request: &DocumentExtractFlightRequest,
        output: &Path,
        reason: &str,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        log::info!(
            "hybrid PDF OCR route fell back to full Docling extraction for `{}`: {reason}",
            request.source_path
        );
        let output_string = output.to_string_lossy().to_string();
        self.document_extract_batch(
            request.source_path.as_str(),
            output_string.as_str(),
            request.force,
            request.error_row,
        )
        .await
    }

    async fn wait_for_terminal_status(
        &self,
        status: DocumentExtractJobStatus,
        wait_ms: u64,
    ) -> Result<DocumentExtractJobStatus, String> {
        let deadline = Instant::now() + Duration::from_millis(wait_ms);
        let mut current = status;
        loop {
            if matches!(current.status.as_str(), "succeeded" | "failed") {
                return Ok(current);
            }
            if Instant::now() >= deadline {
                return Ok(current);
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
            if let Some(next) = self.status(current.job_id.as_str())? {
                current = next;
            }
        }
    }

    async fn schedule_job(&self, job_id: String) {
        let mut scheduled = self.runtime.scheduled.lock().await;
        if !scheduled.insert(job_id.clone()) {
            return;
        }
        drop(scheduled);

        let provider = self.clone();
        tokio::spawn(async move {
            if let Err(error) = provider.run_job(job_id.as_str()).await {
                log::warn!("document extract async job `{job_id}` failed: {error}");
            }
            provider
                .runtime
                .scheduled
                .lock()
                .await
                .remove(job_id.as_str());
        });
    }

    async fn run_job(&self, job_id: &str) -> Result<(), String> {
        let _permit = Arc::clone(&self.runtime.conversion_permits)
            .acquire_owned()
            .await
            .map_err(|error| format!("acquire document extract conversion permit: {error}"))?;
        let Some(status) = ({
            let _registry_guard = self.registry_lock();
            self.registry()?.start_job(job_id)?
        }) else {
            return Ok(());
        };
        let artifact_dir = PathBuf::from(status.artifact_dir.as_str());
        if artifact_dir.exists() {
            tokio::fs::remove_dir_all(artifact_dir.as_path())
                .await
                .map_err(|error| {
                    format!(
                        "remove stale document extract artifact `{}`: {error}",
                        artifact_dir.display()
                    )
                })?;
        }
        tokio::fs::create_dir_all(artifact_dir.as_path())
            .await
            .map_err(|error| {
                format!(
                    "create document extract artifact `{}`: {error}",
                    artifact_dir.display()
                )
            })?;

        let conversion = self
            .request_python_document_extract(
                status.source_path.as_str(),
                status.artifact_dir.as_str(),
                true,
                false,
            )
            .await;

        match conversion {
            Ok(batches) => {
                let resources_path = artifact_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
                if !resources_path.exists() {
                    write_arrow_file(resources_path.as_path(), &batches)?;
                    tokio::fs::File::create(artifact_dir.join("_complete.marker"))
                        .await
                        .map_err(|error| {
                            format!("touch document extract artifact marker: {error}")
                        })?;
                }
                mirror_artifact_to_output(
                    artifact_dir.as_path(),
                    Path::new(status.output_dir.as_str()),
                )?;
                let _registry_guard = self.registry_lock();
                self.registry()?.mark_succeeded(job_id)
            }
            Err(error) => {
                let _registry_guard = self.registry_lock();
                self.registry()?.mark_failed(job_id, error.as_str())?;
                Err(error)
            }
        }
    }

    fn mirror_and_read_succeeded(
        status: &DocumentExtractJobStatus,
        output_dir: &Path,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let artifact_dir = Path::new(status.artifact_dir.as_str());
        if artifact_ready(status) {
            mirror_artifact_to_output(artifact_dir, output_dir)?;
        }
        let resources_path = output_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
        let batches = read_arrow_file(resources_path.as_path())?;
        if batches.is_empty() {
            return Err(format!(
                "document extract cache `{}` contained no batches",
                resources_path.display()
            ));
        }
        Ok(DocumentExtractFlightRouteResponse::from_batches(batches))
    }

    async fn request_python_document_extract(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
    ) -> Result<Vec<EngineRecordBatch>, String> {
        let endpoint_url = std::env::var("WENDAO_DOCUMENT_EXTRACT_ENDPOINT")
            .unwrap_or_else(|_| DEFAULT_DOCUMENT_EXTRACT_ENDPOINT.to_string());

        let channel = self.channel_for_endpoint(&endpoint_url).await?;

        let inner_client = TonicFlightServiceClient::new(channel)
            .max_encoding_message_size(DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES)
            .max_decoding_message_size(DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES);
        let mut client = FlightClient::new_from_inner(inner_client);
        client
            .add_header(WENDAO_SCHEMA_VERSION_HEADER, "v2")
            .map_err(|error| format!("invalid schema version header: {error}"))?;
        client
            .add_header(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER, source_path)
            .map_err(|error| format!("invalid source path header: {error}"))?;
        client
            .add_header(WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, output_dir)
            .map_err(|error| format!("invalid output dir header: {error}"))?;
        client
            .add_header(
                WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
                if force { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid force header: {error}"))?;
        client
            .add_header(
                WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
                if error_row { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid error-row header: {error}"))?;

        let descriptor = FlightDescriptor::new_path(
            ANALYSIS_DOCUMENT_EXTRACT_ROUTE
                .trim_start_matches('/')
                .split('/')
                .map(ToString::to_string)
                .collect(),
        );
        let flight_info = client
            .get_flight_info(descriptor)
            .await
            .map_err(|error| format!("document extract get_flight_info failed: {error}"))?;

        let ticket = flight_info
            .endpoint
            .first()
            .and_then(|endpoint| endpoint.ticket.clone())
            .ok_or_else(|| "document extract flight info missing ticket".to_string())?;

        let stream = client
            .do_get(ticket)
            .await
            .map_err(|error| format!("document extract do_get failed: {error}"))?;

        let engine_batches: Vec<EngineRecordBatch> = stream
            .try_collect()
            .await
            .map_err(|error| format!("document extract stream decode failed: {error}"))?;

        if engine_batches.is_empty() {
            return Err("document extract returned no record batches".to_string());
        }
        Ok(engine_batches)
    }

    fn registry(&self) -> Result<&DocumentExtractJobRegistry, String> {
        self.runtime
            .registry
            .as_ref()
            .as_ref()
            .map_err(Clone::clone)
    }

    fn registry_lock(&self) -> std::sync::MutexGuard<'_, ()> {
        self.runtime
            .registry_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(feature = "document-extract-pdf-source-range")]
async fn render_hybrid_page_ocr_shards(
    source: &Path,
    output: &Path,
) -> Result<PdfPageRenderShardReport, String> {
    let selection = hybrid_page_ocr_render_selection();
    let regions = if selection == PdfPageRenderSelection::RegionShards {
        Some(hybrid_page_ocr_region_requests_for_source(source)?)
    } else {
        None
    };
    let source_for_render = source.to_path_buf();
    let output_for_render = output.to_path_buf();
    tokio::task::spawn_blocking(move || {
        if let Some(regions) = regions {
            #[cfg(feature = "document-extract-pdf-render")]
            return render_pdf_region_shards(
                source_for_render.as_path(),
                output_for_render.as_path(),
                &PdfPageRenderProfile::ocr_default(),
                regions.as_slice(),
            );
            #[cfg(not(feature = "document-extract-pdf-render"))]
            let _ = regions;
            #[cfg(not(feature = "document-extract-pdf-render"))]
            return Err(format!(
                "hybrid PDF region shards for `{}` require the `document-extract-pdf-render` feature",
                source_for_render.display()
            ));
        }
        prepare_pdf_source_page_range_ocr_shards_with_selection(
            source_for_render.as_path(),
            output_for_render.as_path(),
            &PdfPageRenderProfile::ocr_default(),
            selection,
        )
    })
    .await
    .map_err(|error| format!("join hybrid PDF OCR render task: {error}"))?
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn hybrid_page_ocr_render_selection() -> PdfPageRenderSelection {
    hybrid_page_ocr_render_selection_with_lookup(&|key| std::env::var(key).ok())
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn hybrid_page_ocr_render_selection_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> PdfPageRenderSelection {
    match lookup(DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV)
        .unwrap_or_default()
        .trim()
        .replace('-', "_")
        .as_str()
    {
        "all_pages" => PdfPageRenderSelection::AllPages,
        "region_shards" => PdfPageRenderSelection::RegionShards,
        _ => PdfPageRenderSelection::ShardFallbackPages,
    }
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn hybrid_page_ocr_region_requests_for_source(
    source: &Path,
) -> Result<Vec<PdfPageRegionRenderRequest>, String> {
    hybrid_page_ocr_region_requests_for_source_with_lookup(source, &|key| std::env::var(key).ok())
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn hybrid_page_ocr_region_requests_for_source_with_lookup(
    source: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<PdfPageRegionRenderRequest>, String> {
    let regions_json = lookup(DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).ok_or_else(|| {
        format!("{DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV} is required for region_shards")
    })?;
    let region_inputs = serde_json::from_str::<Vec<HybridPdfRegionInput>>(regions_json.as_str())
        .map_err(|error| format!("parse hybrid PDF region JSON: {error}"))?;
    let mut matching_regions = None;
    for input in region_inputs {
        if paths_match(source, input.source.as_path()) {
            if input.regions.is_empty() {
                return Err(format!(
                    "hybrid PDF region fixture has no regions for `{}`",
                    input.source.display()
                ));
            }
            if matching_regions.replace(input.regions).is_some() {
                return Err(format!(
                    "duplicate hybrid PDF region fixture for `{}`",
                    source.display()
                ));
            }
        }
    }
    matching_regions.ok_or_else(|| {
        format!(
            "no hybrid PDF region fixture matched source `{}`",
            source.display()
        )
    })
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn paths_match(left: &Path, right: &Path) -> bool {
    left == right
        || match (left.canonicalize(), right.canonicalize()) {
            (Ok(left), Ok(right)) => left == right,
            _ => false,
        }
}

#[cfg(feature = "document-extract-pdf-source-range")]
async fn materialize_hybrid_page_ocr_resource_batch(
    _source: &Path,
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
    pdf_ocr_scheduler: &PdfOcrWorkerScheduler,
) -> Result<HybridDocumentResourceBatch, String> {
    let endpoint_url = std::env::var("WENDAO_DOCUMENT_EXTRACT_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_DOCUMENT_EXTRACT_ENDPOINT.to_string());
    let response = pdf_ocr_scheduler
        .request_shards(endpoint_url, inputs.as_slice())
        .await?;
    validate_successful_ocr_results(
        response.results.as_slice(),
        render_report.page_count,
        render_report.shard_count,
    )?;
    validate_ocr_results_match_inputs(inputs.as_slice(), response.results.as_slice())?;
    let has_region_shards = inputs.iter().any(|input| input.shard_type == "region");

    if render_report.shard_count == render_report.page_count && !has_region_shards {
        validate_hybrid_page_coverage(render_report.page_count, &[], response.results.as_slice())?;
        return Ok(HybridDocumentResourceBatch {
            batch: response.resource_batch,
            ocr_inputs: inputs,
            ocr_results: response.results,
        });
    }

    Err(
        "hybrid PDF OCR partial or region coverage requires native text merge support; falling back to Docling"
            .to_string(),
    )
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn write_hybrid_document_resource_artifacts(
    output: &Path,
    source: &Path,
    resource_batch: &HybridDocumentResourceBatch,
) -> Result<(), String> {
    write_arrow_file(
        output.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME).as_path(),
        std::slice::from_ref(&resource_batch.batch),
    )?;
    let source_content_hash = sha256_file_hex(source)?;
    let structure_blocks = hybrid_document_structure_blocks(
        resource_batch,
        source_content_hash.as_str(),
        "wendao-hybrid-page-ocr",
    )?;
    let structure_batch = build_document_structure_batch(structure_blocks.as_slice())?;
    write_arrow_file(
        output.join(DOCUMENT_STRUCTURE_ARROW_CACHE_NAME).as_path(),
        std::slice::from_ref(&structure_batch),
    )
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn hybrid_document_structure_blocks(
    resource_batch: &HybridDocumentResourceBatch,
    source_content_hash: &str,
    engine: &str,
) -> Result<Vec<DocumentStructureBlock>, String> {
    let mut blocks = document_resource_batch_to_structure_blocks(
        &resource_batch.batch,
        source_content_hash,
        engine,
    )?;
    if resource_batch.ocr_inputs.is_empty() {
        return Ok(blocks);
    }

    let inputs_by_shard = resource_batch
        .ocr_inputs
        .iter()
        .map(|input| (input.shard_element_id.as_str(), input))
        .collect::<HashMap<_, _>>();
    let results_by_element = resource_batch
        .ocr_results
        .iter()
        .map(|result| (result.element_id.as_str(), result))
        .collect::<HashMap<_, _>>();

    for block in &mut blocks {
        let Some(result) = results_by_element.get(block.resource_element_id.as_str()) else {
            continue;
        };
        let Some(input) = inputs_by_shard.get(result.shard_element_id.as_str()) else {
            continue;
        };
        block.reading_order_key = input.reading_order_key.clone();
        if let Some(block_index) = parse_reading_order_block_index(input.reading_order_key.as_str())
        {
            block.block_index = block_index;
        }
        block.block_type = match input.shard_type.as_str() {
            "region" => "ocr_region".to_string(),
            "page" => "ocr_page".to_string(),
            other => format!("ocr_{other}"),
        };
        block.parent_block_id = input.parent_shard_element_id.clone();
        block.confidence = result.confidence;
        block.bbox_left = Some(input.crop_left);
        block.bbox_top = Some(input.crop_top);
        block.bbox_right = Some(input.crop_right);
        block.bbox_bottom = Some(input.crop_bottom);
        block.provenance = serde_json::json!({
            "source": "pdf_ocr_shard",
            "shardType": input.shard_type,
            "regionIndex": input.region_index,
            "shardElementId": input.shard_element_id,
            "parentShardElementId": input.parent_shard_element_id,
            "readingOrderKey": input.reading_order_key,
            "rasterSha256": input.raster_sha256,
            "imagePath": input.image_path,
        })
        .to_string();
    }
    Ok(blocks)
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn parse_reading_order_block_index(reading_order_key: &str) -> Option<i32> {
    reading_order_key
        .split('.')
        .nth(1)
        .and_then(|value| value.parse::<i32>().ok())
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn sha256_file_hex(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("open `{}` for hashing: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("read `{}` for hashing: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

impl DocumentExtractProviderRuntime {
    fn new(registry: Result<DocumentExtractJobRegistry, String>, conversion_limit: usize) -> Self {
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
    fn new_with_pdf_ocr_worker_limit(
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

fn shared_document_extract_provider_runtime(
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

fn document_extract_conversion_concurrency_limit() -> usize {
    document_extract_conversion_concurrency_limit_with_lookup(
        &|key| std::env::var(key).ok(),
        std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .ok(),
    )
}

fn document_extract_conversion_concurrency_limit_with_lookup(
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

#[async_trait]
impl DocumentExtractFlightRouteProvider for StudioDocumentExtractFlightRouteProvider {
    async fn document_extract_batch(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let engine_batches = self
            .request_python_document_extract(source_path, output_dir, force, error_row)
            .await?;
        Ok(DocumentExtractFlightRouteResponse::from_batches(
            engine_batches,
        ))
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

#[cfg(feature = "document-extract-pdf-source-range")]
fn hybrid_page_ocr_request_paths(request: &DocumentExtractFlightRequest) -> (PathBuf, PathBuf) {
    let source = PathBuf::from(request.source_path.as_str());
    let output = if request.output_dir.trim().is_empty() {
        default_output_dir(source.as_path())
    } else {
        PathBuf::from(request.output_dir.as_str())
    };
    (source, output)
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn hybrid_page_ocr_input_arrow_path(report: &PdfPageRenderShardReport) -> Result<PathBuf, String> {
    if report.status != PdfRenderStatus::Rendered.as_str() {
        return Err(format!(
            "render status `{}` is not eligible for hybrid OCR",
            report.status
        ));
    }
    if report.routing_decision != PdfRenderRoutingDecision::HybridPageOcrCandidate.as_str() {
        return Err(format!(
            "routing decision `{}` is not eligible for hybrid OCR",
            report.routing_decision
        ));
    }
    if report.page_count == 0 {
        return Err("hybrid OCR render report has no pages".to_string());
    }
    report
        .ocr_input_arrow_path
        .as_ref()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "hybrid OCR render report is missing `_ocr_input.arrow`".to_string())
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn validate_successful_ocr_results(
    results: &[PdfOcrShardResult],
    page_count: u32,
    shard_count: u32,
) -> Result<(), String> {
    if results.len() != usize::try_from(shard_count).unwrap_or(usize::MAX) {
        return Err(format!(
            "OCR worker returned {} rows for {shard_count} rendered shards",
            results.len()
        ));
    }
    for result in results {
        if result.page_index >= page_count {
            return Err(format!(
                "OCR worker returned out-of-range page {} for {page_count} page PDF",
                result.page_index
            ));
        }
        if result.status != PdfOcrShardResultStatus::Succeeded {
            return Err(format!(
                "OCR worker returned non-success status `{}` for page {}",
                result.status.as_str(),
                result.page_index
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn validate_ocr_results_match_inputs(
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
) -> Result<(), String> {
    if inputs.len() != results.len() {
        return Err(format!(
            "OCR worker returned {} rows for {} inputs",
            results.len(),
            inputs.len()
        ));
    }
    let mut inputs_by_shard = HashMap::new();
    for input in inputs {
        if inputs_by_shard
            .insert(input.shard_element_id.as_str(), input)
            .is_some()
        {
            return Err(format!(
                "duplicate OCR shard input id `{}`",
                input.shard_element_id
            ));
        }
    }
    let mut result_shards = HashSet::new();
    for result in results {
        if !result_shards.insert(result.shard_element_id.as_str()) {
            return Err(format!(
                "duplicate OCR shard result id `{}`",
                result.shard_element_id
            ));
        }
        let input = inputs_by_shard
            .get(result.shard_element_id.as_str())
            .ok_or_else(|| {
                format!(
                    "OCR worker returned unknown shard id `{}`",
                    result.shard_element_id
                )
            })?;
        if input.page_index != result.page_index {
            return Err(format!(
                "OCR worker returned page {} for shard `{}` but input page was {}",
                result.page_index, result.shard_element_id, input.page_index
            ));
        }
        if input.raster_sha256 != result.raster_sha256 {
            return Err(format!(
                "OCR worker returned raster hash `{}` for shard `{}` but input hash was `{}`",
                result.raster_sha256, result.shard_element_id, input.raster_sha256
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn validate_hybrid_page_coverage(
    page_count: u32,
    text_page_indices: &[u32],
    ocr_results: &[PdfOcrShardResult],
) -> Result<(), String> {
    if let Some(page_index) = text_page_indices
        .iter()
        .copied()
        .find(|page_index| *page_index >= page_count)
    {
        return Err(format!(
            "native text page {page_index} is out of range for {page_count} page PDF"
        ));
    }
    let mut covered = text_page_indices.iter().copied().collect::<HashSet<_>>();
    for result in ocr_results {
        if covered.contains(&result.page_index) {
            return Err(format!(
                "hybrid merge has duplicate page coverage for page {}",
                result.page_index
            ));
        }
        covered.insert(result.page_index);
    }
    let missing = (0..page_count)
        .filter(|page_index| !covered.contains(page_index))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "hybrid merge is missing page coverage: {missing:?}"
        ));
    }
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[cfg(test)]
fn validate_hybrid_shard_coverage(
    page_count: u32,
    text_page_indices: &[u32],
    ocr_inputs: &[PdfOcrShardInput],
    ocr_results: &[PdfOcrShardResult],
) -> Result<(), String> {
    validate_ocr_results_match_inputs(ocr_inputs, ocr_results)?;
    if let Some(page_index) = text_page_indices
        .iter()
        .copied()
        .find(|page_index| *page_index >= page_count)
    {
        return Err(format!(
            "native text page {page_index} is out of range for {page_count} page PDF"
        ));
    }

    let mut covered_pages = HashSet::new();
    for page_index in text_page_indices {
        if !covered_pages.insert(*page_index) {
            return Err(format!(
                "hybrid merge has duplicate native text page coverage for page {page_index}"
            ));
        }
    }

    for input in ocr_inputs {
        match input.shard_type.as_str() {
            "page" => {
                if !covered_pages.insert(input.page_index) {
                    return Err(format!(
                        "hybrid merge has duplicate page coverage for page {}",
                        input.page_index
                    ));
                }
            }
            "region" => {
                if !covered_pages.contains(&input.page_index) {
                    return Err(format!(
                        "region OCR shard `{}` has no native text coverage for page {}",
                        input.shard_element_id, input.page_index
                    ));
                }
            }
            other => {
                return Err(format!(
                    "unsupported OCR shard input type `{other}` for shard `{}`",
                    input.shard_element_id
                ));
            }
        }
    }

    let missing = (0..page_count)
        .filter(|page_index| !covered_pages.contains(page_index))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "hybrid merge is missing page coverage: {missing:?}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tokio::time::{Duration, sleep};

    use super::*;

    #[test]
    fn document_extract_conversion_limit_defaults_to_bounded_parallelism() {
        let limit = document_extract_conversion_concurrency_limit_with_lookup(&|_| None, Some(12));

        assert_eq!(limit, DEFAULT_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS);
    }

    #[test]
    fn document_extract_conversion_limit_accepts_positive_override() {
        let limit = document_extract_conversion_concurrency_limit_with_lookup(
            &|key| (key == DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV).then(|| "7".to_string()),
            Some(12),
        );

        assert_eq!(limit, 7);
    }

    #[test]
    fn document_extract_conversion_limit_ignores_invalid_override() {
        let limit = document_extract_conversion_concurrency_limit_with_lookup(
            &|key| (key == DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV).then(|| "0".to_string()),
            Some(2),
        );

        assert_eq!(limit, 2);
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_render_selection_defaults_to_shard_fallback() {
        let selection = hybrid_page_ocr_render_selection_with_lookup(&|_| None);

        assert_eq!(selection, PdfPageRenderSelection::ShardFallbackPages);
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_render_selection_accepts_all_pages_override() {
        let selection = hybrid_page_ocr_render_selection_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV).then(|| "all-pages".to_string())
        });

        assert_eq!(selection, PdfPageRenderSelection::AllPages);
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_render_selection_accepts_region_shards_override() {
        let selection = hybrid_page_ocr_render_selection_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV).then(|| "region-shards".to_string())
        });

        assert_eq!(selection, PdfPageRenderSelection::RegionShards);
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_region_requests_match_selected_source() -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let source = temp.path().join("source.pdf");
        fs::write(source.as_path(), b"%PDF").map_err(|error| error.to_string())?;
        let source_text = source.to_string_lossy();
        let regions_json = format!(
            r#"[{{"source":"{source_text}","regions":[{{"pageIndex":0,"regionIndex":3,"regionBox":{{"left":72.0,"bottom":72.0,"right":144.0,"top":144.0}},"readingOrderKey":"000000.000003"}}]}}]"#
        );

        let regions =
            hybrid_page_ocr_region_requests_for_source_with_lookup(source.as_path(), &|key| {
                (key == DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).then(|| regions_json.clone())
            })?;

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].page_index, 0);
        assert_eq!(regions[0].region_index, 3);
        assert_eq!(
            regions[0].reading_order_key.as_deref(),
            Some("000000.000003")
        );
        Ok(())
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_region_requests_reject_missing_source() {
        let regions_json = r#"[{"source":"/tmp/other.pdf","regions":[{"pageIndex":0,"regionIndex":0,"regionBox":{"left":0.0,"bottom":0.0,"right":10.0,"top":10.0}}]}]"#;

        let error = match hybrid_page_ocr_region_requests_for_source_with_lookup(
            Path::new("/tmp/source.pdf"),
            &|key| (key == DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).then(|| regions_json.into()),
        ) {
            Ok(regions) => panic!("expected missing source to fail: {regions:?}"),
            Err(error) => error,
        };

        assert!(error.contains("no hybrid PDF region fixture matched"));
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_input_arrow_path_accepts_complete_render() -> Result<(), String> {
        let report = sample_hybrid_page_ocr_report(
            PdfRenderStatus::Rendered,
            PdfRenderRoutingDecision::HybridPageOcrCandidate,
            2,
            2,
            Some("/tmp/out/_ocr_input.arrow"),
        );

        let path = hybrid_page_ocr_input_arrow_path(&report)?;

        assert_eq!(path, PathBuf::from("/tmp/out/_ocr_input.arrow"));
        Ok(())
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_input_arrow_path_accepts_partial_page_render() -> Result<(), String> {
        let report = sample_hybrid_page_ocr_report(
            PdfRenderStatus::Rendered,
            PdfRenderRoutingDecision::HybridPageOcrCandidate,
            3,
            1,
            Some("/tmp/out/_ocr_input.arrow"),
        );

        let path = hybrid_page_ocr_input_arrow_path(&report)?;

        assert_eq!(path, PathBuf::from("/tmp/out/_ocr_input.arrow"));
        Ok(())
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_input_arrow_path_rejects_fallback_report() {
        let report = sample_hybrid_page_ocr_report(
            PdfRenderStatus::Fallback,
            PdfRenderRoutingDecision::FullDoclingFallback,
            1,
            0,
            None,
        );

        let error = match hybrid_page_ocr_input_arrow_path(&report) {
            Ok(path) => panic!(
                "fallback report should not become OCR input: {}",
                path.display()
            ),
            Err(error) => error,
        };

        assert!(error.contains("not eligible for hybrid OCR"));
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_validation_rejects_skipped_ocr_rows() {
        let error = match validate_successful_ocr_results(&[sample_ocr_result(1, false)], 3, 1) {
            Ok(()) => panic!("expected non-success OCR status to fail"),
            Err(error) => error,
        };

        assert!(error.contains("non-success status"));
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_coverage_accepts_text_and_ocr_pages() -> Result<(), String> {
        validate_hybrid_page_coverage(3, &[0, 2], &[sample_ocr_result(1, true)])
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_coverage_accepts_text_only_pages() -> Result<(), String> {
        validate_hybrid_page_coverage(3, &[0, 1, 2], &[])
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_coverage_rejects_missing_pages() {
        let error = match validate_hybrid_page_coverage(3, &[0], &[sample_ocr_result(1, true)]) {
            Ok(()) => panic!("expected missing page coverage to fail"),
            Err(error) => error,
        };

        assert!(error.contains("missing page coverage"));
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_coverage_rejects_duplicate_pages() {
        let error = match validate_hybrid_page_coverage(3, &[0, 1], &[sample_ocr_result(1, true)]) {
            Ok(()) => panic!("expected duplicate page coverage to fail"),
            Err(error) => error,
        };

        assert!(error.contains("duplicate page coverage"));
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_region_coverage_keeps_native_text_page() -> Result<(), String> {
        let input = sample_ocr_input(1, "region");
        let result = sample_ocr_result(1, true);

        validate_hybrid_shard_coverage(3, &[0, 1, 2], &[input], &[result])
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_region_coverage_requires_native_text_page() {
        let input = sample_ocr_input(1, "region");
        let result = sample_ocr_result(1, true);

        let error = match validate_hybrid_shard_coverage(3, &[0, 2], &[input], &[result]) {
            Ok(()) => panic!("expected region without native text page to fail"),
            Err(error) => error,
        };

        assert!(error.contains("has no native text coverage"));
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_page_shard_coverage_still_replaces_full_page() -> Result<(), String> {
        let input = sample_ocr_input(1, "page");
        let result = sample_ocr_result(1, true);

        validate_hybrid_shard_coverage(3, &[0, 2], &[input], &[result])
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_validation_rejects_unknown_shard_result() {
        let input = sample_ocr_input(1, "region");
        let mut result = sample_ocr_result(1, true);
        result.shard_element_id = "unknown-shard".to_string();

        let error = match validate_ocr_results_match_inputs(&[input], &[result]) {
            Ok(()) => panic!("expected unknown OCR result shard to fail"),
            Err(error) => error,
        };

        assert!(error.contains("unknown shard id"));
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_writes_structure_sidecar() -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let source = temp.path().join("source.pdf");
        let output = temp.path().join("out");
        fs::write(source.as_path(), b"%PDF-1.4\n").map_err(|error| error.to_string())?;
        fs::create_dir_all(output.as_path()).map_err(|error| error.to_string())?;
        let resource_batch = HybridDocumentResourceBatch::native(test_resource_batch(&[(
            "text_page",
            0,
            "text-0",
        )])?);

        write_hybrid_document_resource_artifacts(
            output.as_path(),
            source.as_path(),
            &resource_batch,
        )?;

        let structure_path = output
            .join(xiuxian_wendao_attachments::pdf::structure::DOCUMENT_STRUCTURE_ARROW_CACHE_NAME);
        let structure_batches = read_arrow_file(structure_path.as_path())?;
        assert_eq!(structure_batches.len(), 1);
        assert_eq!(structure_batches[0].num_rows(), 1);
        Ok(())
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[test]
    fn hybrid_page_ocr_structure_sidecar_preserves_region_provenance() -> Result<(), String> {
        let mut input = sample_ocr_input(0, "region");
        input.crop_left = 72.0;
        input.crop_bottom = 80.0;
        input.crop_right = 240.0;
        input.crop_top = 320.0;
        input.reading_order_key = "000000.000004".to_string();
        let mut result = sample_ocr_result(0, true);
        result.confidence = Some(0.87);
        let resource_batch = HybridDocumentResourceBatch {
            batch: test_resource_batch(&[
                ("text_page", 0, "text-0"),
                ("ocr_text", 0, result.element_id.as_str()),
            ])?,
            ocr_inputs: vec![input],
            ocr_results: vec![result],
        };

        let blocks =
            hybrid_document_structure_blocks(&resource_batch, "sourcehash", "wendao-hybrid")?;
        let structure_batch = build_document_structure_batch(blocks.as_slice())?;

        assert_eq!(
            structure_string_column(&structure_batch, "blockId")?.value(1),
            "ocr-0"
        );
        assert_eq!(
            structure_string_column(&structure_batch, "readingOrderKey")?.value(1),
            "000000.000004"
        );
        assert_eq!(
            structure_string_column(&structure_batch, "blockType")?.value(1),
            "ocr_region"
        );
        assert_eq!(
            structure_string_column(&structure_batch, "parentBlockId")?.value(1),
            "page-shard-0"
        );
        assert_close(
            structure_float64_column(&structure_batch, "bboxLeft")?.value(1),
            72.0,
        );
        assert_close(
            structure_float64_column(&structure_batch, "bboxBottom")?.value(1),
            80.0,
        );
        assert_close(
            structure_float64_column(&structure_batch, "bboxRight")?.value(1),
            240.0,
        );
        assert_close(
            structure_float64_column(&structure_batch, "bboxTop")?.value(1),
            320.0,
        );
        assert_close(
            structure_float64_column(&structure_batch, "confidence")?.value(1),
            0.87,
        );
        assert!(
            structure_string_column(&structure_batch, "provenance")?
                .value(1)
                .contains(r#""shardType":"region""#)
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn document_extract_job_remains_queued_until_conversion_permit_is_available()
    -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let source = temp.path().join("manual.pdf");
        fs::write(source.as_path(), b"pdf fixture").map_err(|error| error.to_string())?;
        let registry = DocumentExtractJobRegistry::new(
            temp.path().join("jobs.duckdb"),
            temp.path().join("artifacts"),
        )?;
        let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 1);
        let held_permit = Arc::clone(&provider.runtime.conversion_permits)
            .acquire_owned()
            .await
            .map_err(|error| error.to_string())?;
        let queued = provider.registry()?.submit(
            source.as_path(),
            temp.path().join("out").as_path(),
            false,
        )?;
        let job_id = queued.job_id.clone();
        let running_provider = provider.clone();

        let handle = tokio::spawn(async move { running_provider.run_job(job_id.as_str()).await });
        sleep(Duration::from_millis(50)).await;
        let status = provider
            .status(queued.job_id.as_str())?
            .ok_or_else(|| "job should still exist".to_string())?;

        assert_eq!(status.status, "queued");
        assert_eq!(status.attempt_count, 0);

        handle.abort();
        drop(held_permit);
        Ok(())
    }

    #[tokio::test]
    async fn document_extract_runtime_snapshot_reports_capacity_and_registry_counts()
    -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let source = temp.path().join("manual.pdf");
        fs::write(source.as_path(), b"pdf fixture").map_err(|error| error.to_string())?;
        let registry = DocumentExtractJobRegistry::new(
            temp.path().join("jobs.duckdb"),
            temp.path().join("artifacts"),
        )?;
        let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 2);
        let _held_permit = Arc::clone(&provider.runtime.conversion_permits)
            .acquire_owned()
            .await
            .map_err(|error| error.to_string())?;
        let queued = provider.registry()?.submit(
            source.as_path(),
            temp.path().join("out").as_path(),
            false,
        )?;
        provider.schedule_job(queued.job_id.clone()).await;

        let snapshot = provider.runtime_snapshot().await?;

        assert_eq!(snapshot.max_running_conversions, 2);
        assert_eq!(snapshot.available_conversion_permits, 1);
        assert_eq!(snapshot.in_process_running_conversions, 1);
        assert_eq!(snapshot.in_process_scheduled_jobs, 1);
        assert_eq!(snapshot.registry.queued_jobs, 1);
        assert_eq!(snapshot.registry.total_jobs, 1);
        Ok(())
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    #[tokio::test]
    async fn document_extract_runtime_snapshot_reports_pdf_ocr_worker_capacity()
    -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let registry = DocumentExtractJobRegistry::new(
            temp.path().join("jobs.duckdb"),
            temp.path().join("artifacts"),
        )?;
        let provider =
            StudioDocumentExtractFlightRouteProvider::from_registry_with_pdf_ocr_worker_limit(
                Ok(registry),
                2,
                5,
            );
        let _held_permits = provider
            .runtime
            .pdf_ocr_scheduler
            .permits_for_tests()
            .acquire_many_owned(2)
            .await
            .map_err(|error| error.to_string())?;

        let snapshot = provider.runtime_snapshot().await?;

        assert_eq!(snapshot.max_pdf_ocr_workers, 5);
        assert_eq!(snapshot.available_pdf_ocr_worker_permits, 3);
        assert_eq!(snapshot.in_process_pdf_ocr_workers, 2);
        Ok(())
    }

    #[test]
    fn document_extract_provider_reuses_runtime_for_same_project_root() {
        let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create tempdir: {error}"));
        let first = shared_document_extract_provider_runtime(temp.path());
        let second = shared_document_extract_provider_runtime(temp.path());

        assert!(Arc::ptr_eq(&first, &second));
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    fn sample_hybrid_page_ocr_report(
        status: PdfRenderStatus,
        routing_decision: PdfRenderRoutingDecision,
        page_count: u32,
        shard_count: u32,
        ocr_input_arrow_path: Option<&str>,
    ) -> PdfPageRenderShardReport {
        PdfPageRenderShardReport {
            source_path: "/tmp/source.pdf".to_string(),
            output_dir: "/tmp/out".to_string(),
            page_count,
            shard_count,
            manifest_arrow_path: None,
            ocr_input_arrow_path: ocr_input_arrow_path.map(ToString::to_string),
            pending_resource_arrow_path: None,
            render_profile: "pdfium-render-page-shards-v1".to_string(),
            render_selection: PdfPageRenderSelection::ShardFallbackPages
                .as_str()
                .to_string(),
            status: status.as_str().to_string(),
            routing_decision: routing_decision.as_str().to_string(),
            elapsed_ms: 1.0,
            error_message: None,
        }
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    fn sample_ocr_result(page_index: u32, succeeded: bool) -> PdfOcrShardResult {
        PdfOcrShardResult {
            contract_version:
                xiuxian_wendao_attachments::pdf::ocr::PDF_OCR_SHARD_RESULT_SCHEMA_VERSION
                    .to_string(),
            source_path: "/tmp/source.pdf".to_string(),
            source_content_hash: "sourcehash".to_string(),
            page_index,
            image_path: format!("/tmp/out/page-{page_index}.png"),
            image_mime_type: "image/png".to_string(),
            raster_sha256: format!("raster-{page_index}"),
            render_profile: "pdfium-render-page-shards-v1".to_string(),
            ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
            status: if succeeded {
                PdfOcrShardResultStatus::Succeeded
            } else {
                PdfOcrShardResultStatus::Skipped
            },
            text: succeeded.then(|| format!("OCR text {page_index}")),
            text_mime_type: "text/plain".to_string(),
            confidence: succeeded.then_some(0.99),
            error_message: (!succeeded).then(|| "worker skipped".to_string()),
            shard_element_id: format!("shard-{page_index}"),
            element_id: format!("ocr-{page_index}"),
        }
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    fn sample_ocr_input(page_index: u32, shard_type: &str) -> PdfOcrShardInput {
        PdfOcrShardInput {
            contract_version:
                xiuxian_wendao_attachments::pdf::ocr::PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
            source_path: "/tmp/source.pdf".to_string(),
            source_content_hash: "sourcehash".to_string(),
            page_index,
            image_path: format!("/tmp/out/page-{page_index}.png"),
            image_mime_type: "image/png".to_string(),
            raster_sha256: format!("raster-{page_index}"),
            render_profile: "pdfium-render-page-shards-v1".to_string(),
            ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
            ocr_engine: "docling-compatible-ocr".to_string(),
            preferred_languages: vec!["auto".to_string()],
            min_confidence: 0.0,
            preserve_layout: true,
            raster_width_px: 1000,
            raster_height_px: 1000,
            render_dpi: 216,
            rotation_degrees: 0,
            crop_left: 0.0,
            crop_bottom: 0.0,
            crop_right: 612.0,
            crop_top: 792.0,
            point_to_pixel_scale_x: 3.0,
            point_to_pixel_scale_y: 3.0,
            shard_element_id: format!("shard-{page_index}"),
            shard_type: shard_type.to_string(),
            region_index: u32::from(shard_type == "region"),
            parent_shard_element_id: if shard_type == "region" {
                format!("page-shard-{page_index}")
            } else {
                String::new()
            },
            reading_order_key: format!("{page_index:06}.000001"),
            source_page_pixel_left: 0,
            source_page_pixel_top: 0,
            source_page_pixel_right: 1000,
            source_page_pixel_bottom: 1000,
        }
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    fn test_resource_batch(rows: &[(&str, i32, &str)]) -> Result<EngineRecordBatch, String> {
        arrow::record_batch::RecordBatch::try_new(
            std::sync::Arc::new(arrow::datatypes::Schema::new(vec![
                arrow::datatypes::Field::new("sourcePath", arrow::datatypes::DataType::Utf8, true),
                arrow::datatypes::Field::new(
                    "resourceType",
                    arrow::datatypes::DataType::Utf8,
                    true,
                ),
                arrow::datatypes::Field::new(
                    "resourcePath",
                    arrow::datatypes::DataType::Utf8,
                    true,
                ),
                arrow::datatypes::Field::new("pageIndex", arrow::datatypes::DataType::Int32, true),
                arrow::datatypes::Field::new("caption", arrow::datatypes::DataType::Utf8, true),
                arrow::datatypes::Field::new("content", arrow::datatypes::DataType::Utf8, true),
                arrow::datatypes::Field::new("mimeType", arrow::datatypes::DataType::Utf8, true),
                arrow::datatypes::Field::new("status", arrow::datatypes::DataType::Utf8, true),
                arrow::datatypes::Field::new("elementId", arrow::datatypes::DataType::Utf8, true),
            ])),
            vec![
                std::sync::Arc::new(arrow::array::StringArray::from(vec![
                    "/tmp/source.pdf";
                    rows.len()
                ])) as arrow::array::ArrayRef,
                std::sync::Arc::new(arrow::array::StringArray::from(
                    rows.iter().map(|row| row.0).collect::<Vec<_>>(),
                )) as arrow::array::ArrayRef,
                std::sync::Arc::new(arrow::array::StringArray::from(vec![
                    "/tmp/source.pdf";
                    rows.len()
                ])) as arrow::array::ArrayRef,
                std::sync::Arc::new(arrow::array::Int32Array::from(
                    rows.iter().map(|row| row.1).collect::<Vec<_>>(),
                )) as arrow::array::ArrayRef,
                std::sync::Arc::new(arrow::array::StringArray::from(vec![""; rows.len()]))
                    as arrow::array::ArrayRef,
                std::sync::Arc::new(arrow::array::StringArray::from(vec!["content"; rows.len()]))
                    as arrow::array::ArrayRef,
                std::sync::Arc::new(arrow::array::StringArray::from(vec![
                    "text/markdown";
                    rows.len()
                ])) as arrow::array::ArrayRef,
                std::sync::Arc::new(arrow::array::StringArray::from(vec!["ok"; rows.len()]))
                    as arrow::array::ArrayRef,
                std::sync::Arc::new(arrow::array::StringArray::from(
                    rows.iter().map(|row| row.2).collect::<Vec<_>>(),
                )) as arrow::array::ArrayRef,
            ],
        )
        .map_err(|error| error.to_string())
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    fn structure_string_column<'a>(
        batch: &'a EngineRecordBatch,
        name: &str,
    ) -> Result<&'a arrow::array::StringArray, String> {
        batch
            .column_by_name(name)
            .ok_or_else(|| format!("missing structure `{name}` column"))?
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .ok_or_else(|| format!("structure `{name}` column is not Utf8"))
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    fn structure_float64_column<'a>(
        batch: &'a EngineRecordBatch,
        name: &str,
    ) -> Result<&'a arrow::array::Float64Array, String> {
        batch
            .column_by_name(name)
            .ok_or_else(|| format!("missing structure `{name}` column"))?
            .as_any()
            .downcast_ref::<arrow::array::Float64Array>()
            .ok_or_else(|| format!("structure `{name}` column is not Float64"))
    }

    #[cfg(feature = "document-extract-pdf-source-range")]
    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "expected {actual} to be close to {expected}"
        );
    }
}
