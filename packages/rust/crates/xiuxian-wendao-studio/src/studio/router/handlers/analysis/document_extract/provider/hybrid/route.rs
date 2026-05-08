use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Instant;

#[cfg(feature = "document-extract-pdf-render")]
use futures::future::{BoxFuture, FutureExt};
#[cfg(feature = "document-extract-pdf-render")]
use futures::stream::{FuturesUnordered, StreamExt};
#[cfg(any(feature = "document-extract-pdf-render", test))]
use serde_json::Value;
use serde_json::json;
#[cfg(any(feature = "document-extract-pdf-render", test))]
use sha2::{Digest, Sha256};
#[cfg(any(feature = "document-extract-pdf-render", test))]
use std::path::PathBuf;
use xiuxian_wendao_attachments::pdf::metrics::PdfOcrShardMetric;
#[cfg(any(feature = "document-extract-pdf-source-range", test))]
use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE, PdfOcrShardInput,
    PdfOcrShardResult, PdfOcrShardResultStatus, build_ocr_result_resource_batch,
    decode_ocr_shard_input_batches, is_hosted_vlm_direct_profile,
};
#[cfg(any(feature = "document-extract-pdf-render", test))]
use xiuxian_wendao_attachments::pdf::ocr::{
    downgrade_hosted_vlm_region_parent_page_inputs, hosted_vlm_region_parent_page_shards,
    merge_hosted_vlm_recovery_region_inputs, prepare_hosted_vlm_recovery_region_inputs,
};
#[cfg(any(feature = "document-extract-pdf-render", test))]
use xiuxian_wendao_attachments::pdf::profile::{
    PdfSourcePageProfile, source_pdf_page_profiles_cached,
};
use xiuxian_wendao_attachments::pdf::render::PdfPageRenderShardReport;
#[cfg(any(feature = "document-extract-pdf-render", test))]
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageRegionRenderRequest, PdfPageRenderProfile, PdfRenderRoutingDecision, PdfRenderStatus,
};
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::render::{
    page_region_render_request_chunks_all, page_region_render_request_chunks_by_page,
    page_region_render_request_chunks_by_page_area_desc,
    page_region_render_request_chunks_by_page_max_area_desc,
    page_region_render_request_chunks_by_region, render_pdf_page_shards_for_page_indices,
    render_pdf_region_shards,
};
use xiuxian_wendao_server::transport::{
    DocumentExtractFlightRequest, DocumentExtractFlightRouteProvider,
    DocumentExtractFlightRouteResponse,
};

use super::precision_gate::{
    validate_hybrid_page_coverage, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results,
};
use super::profile::apply_hybrid_page_ocr_profile_plan;
#[cfg(feature = "document-extract-pdf-render")]
use super::render::hybrid_page_ocr_render_profile_with_lookup;
#[cfg(any(feature = "document-extract-pdf-render", test))]
use super::render::{
    automatic_ocr2_recovery_region_requests_for_source_with_lookup,
    hybrid_page_ocr_region_requests_for_source_with_lookup,
};
use super::render::{
    hybrid_page_ocr_input_arrow_path, hybrid_page_ocr_request_paths, render_hybrid_page_ocr_shards,
};
use super::structure::write_hybrid_document_resource_artifacts;
#[cfg(any(feature = "document-extract-pdf-render", test))]
use super::types::DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV;
use super::types::HybridDocumentResourceBatch;
#[cfg(any(feature = "document-extract-pdf-render", test))]
use super::types::{
    HybridPdfOcr2RegionPipelineMode, HybridPdfOcr2RegionRenderChunkMode, HybridPdfOcr2ScaffoldMode,
    hybrid_page_ocr2_region_pipeline_mode_with_lookup,
    hybrid_page_ocr2_region_render_chunk_mode_with_lookup,
    hybrid_page_ocr2_scaffold_mode_with_lookup,
};
use crate::studio::document_extract_pdf_ocr_client::PdfOcrShardSchedulerTrace;
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    read_arrow_file, read_cached_document_batches,
};
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_order::order_ocr_results_by_inputs;
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::{
    PdfOcrWorkerScheduler, pdf_ocr_endpoint_urls,
};
use crate::studio::router::handlers::analysis::document_extract::provider::{
    DEFAULT_DOCUMENT_EXTRACT_ENDPOINT, StudioDocumentExtractFlightRouteProvider,
};

const HYBRID_PAGE_OCR_FALLBACK_REPORT_NAME: &str = "_hybrid_page_ocr_fallback.json";
const HYBRID_PAGE_OCR_TIMING_REPORT_NAME: &str = "_hybrid_page_ocr_timing.json";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_REGION_RENDER_CACHE_DIR_NAME: &str = "hosted-vlm-region-renders";
#[cfg(feature = "document-extract-pdf-render")]
const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR_SHARD_MANIFEST_ARROW_NAME: &str = "_ocr_shards.arrow";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR_SHARD_INPUT_ARROW_NAME: &str = "_ocr_input.arrow";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR_PENDING_RESOURCE_ARROW_NAME: &str = "_ocr_pending.arrow";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_REGION_SCAFFOLD_FILE_NAME: &str = "_hosted_vlm_region_scaffolds.json";
const DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY";
const FAILED_PAGE_RECOVERY_HOSTED_VLM_PAGE_MODE: &str = "hosted-vlm-page";
const HOSTED_VLM_DIRECT_OCR_ENGINE: &str = "hosted-vlm-direct-ocr";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HybridPdfFailedPageRecoveryMode {
    Disabled,
    HostedVlmPage,
}

impl HybridPdfFailedPageRecoveryMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::HostedVlmPage => FAILED_PAGE_RECOVERY_HOSTED_VLM_PAGE_MODE,
        }
    }
}

impl StudioDocumentExtractFlightRouteProvider {
    pub(crate) async fn hybrid_page_ocr_document_extract_batch(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let total_started = Instant::now();
        let mut phase_elapsed_ms = BTreeMap::new();
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

        let phase_started = Instant::now();
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
        record_phase_elapsed(&mut phase_elapsed_ms, "renderShardInputs", phase_started);

        let ocr2_region_materialization_stats: Ocr2RegionMaterializationStats;
        let (resource_batch, ocr_scheduler_trace) = {
            let phase_started = Instant::now();
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
            record_phase_elapsed(&mut phase_elapsed_ms, "decodeOcrInputs", phase_started);

            let phase_started = Instant::now();
            let inputs = apply_hybrid_page_ocr_profile_plan(inputs);
            record_phase_elapsed(&mut phase_elapsed_ms, "profilePlan", phase_started);

            if ocr2_region_pipeline_enabled() {
                #[cfg(feature = "document-extract-pdf-render")]
                {
                    let phase_started = Instant::now();
                    let pipeline =
                        match materialize_hybrid_page_ocr_resource_batch_with_region_pipeline(
                            &render_report,
                            inputs,
                            &self.runtime.pdf_ocr_scheduler,
                        )
                        .await
                        {
                            Ok(pipeline) => pipeline,
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
                    ocr2_region_materialization_stats = pipeline.stats;
                    phase_elapsed_ms.extend(pipeline.phase_elapsed_ms);
                    record_phase_elapsed(&mut phase_elapsed_ms, "regionPipeline", phase_started);
                    (pipeline.resource_batch, pipeline.scheduler_trace)
                }
                #[cfg(not(feature = "document-extract-pdf-render"))]
                {
                    return self
                        .fallback_python_document_extract(
                            request,
                            output.as_path(),
                            "hosted VLM/OCR region pipeline requires the `document-extract-pdf-render` feature",
                        )
                        .await;
                }
            } else {
                let phase_started = Instant::now();
                let region_materialization =
                    match materialize_ocr2_recovery_region_images(&render_report, inputs).await {
                        Ok(materialization) => materialization,
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
                let Ocr2RegionMaterialization {
                    inputs,
                    stats,
                    phase_elapsed_ms: region_phase_elapsed_ms,
                } = region_materialization;
                ocr2_region_materialization_stats = stats;
                phase_elapsed_ms.extend(region_phase_elapsed_ms);
                record_phase_elapsed(&mut phase_elapsed_ms, "regionMaterialize", phase_started);

                let phase_started = Instant::now();
                let inputs =
                    match materialize_ocr2_recovery_page_images(&render_report, inputs).await {
                        Ok(inputs) => inputs,
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
                record_phase_elapsed(&mut phase_elapsed_ms, "pageMaterialize", phase_started);

                let phase_started = Instant::now();
                let batch = match materialize_hybrid_page_ocr_resource_batch(
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
                };
                record_phase_elapsed(&mut phase_elapsed_ms, "ocrScheduler", phase_started);
                (batch, Vec::new())
            }
        };
        let phase_started = Instant::now();
        if let Err(reason) = write_hybrid_document_resource_artifacts(
            output.as_path(),
            source.as_path(),
            &resource_batch,
        ) {
            return self
                .fallback_python_document_extract(request, output.as_path(), reason.as_str())
                .await;
        }
        record_phase_elapsed(&mut phase_elapsed_ms, "writeArtifacts", phase_started);
        let total_elapsed_ms = total_started.elapsed().as_secs_f64() * 1000.0;
        phase_elapsed_ms.insert("total".to_string(), total_elapsed_ms);
        write_hybrid_page_ocr_timing_report(
            source.as_path(),
            output.as_path(),
            &resource_batch,
            &ocr2_region_materialization_stats,
            &phase_elapsed_ms,
            ocr_scheduler_trace.as_slice(),
            total_elapsed_ms,
        )
        .await;
        tokio::fs::File::create(output.join("_complete.marker"))
            .await
            .map_err(|error| format!("touch hybrid PDF OCR complete marker: {error}"))?;

        Ok(DocumentExtractFlightRouteResponse::new(
            resource_batch.batch,
        ))
    }

    async fn fallback_python_document_extract(
        &self,
        request: &DocumentExtractFlightRequest,
        output: &Path,
        reason: &str,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        write_hybrid_page_ocr_fallback_report(request, output, reason).await;
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
            request.profile.as_str(),
        )
        .await
    }
}

fn record_phase_elapsed(
    phase_elapsed_ms: &mut BTreeMap<String, f64>,
    phase: &str,
    started: Instant,
) {
    phase_elapsed_ms.insert(phase.to_string(), started.elapsed().as_secs_f64() * 1000.0);
}

fn ocr2_region_pipeline_enabled() -> bool {
    #[cfg(any(feature = "document-extract-pdf-render", test))]
    {
        hybrid_page_ocr2_region_pipeline_mode_with_lookup(&|key| std::env::var(key).ok())
            == HybridPdfOcr2RegionPipelineMode::RenderDispatch
    }
    #[cfg(not(any(feature = "document-extract-pdf-render", test)))]
    {
        false
    }
}

fn ocr2_region_pipeline_mode_label() -> &'static str {
    #[cfg(any(feature = "document-extract-pdf-render", test))]
    {
        hybrid_page_ocr2_region_pipeline_mode_with_lookup(&|key| std::env::var(key).ok()).as_str()
    }
    #[cfg(not(any(feature = "document-extract-pdf-render", test)))]
    {
        "disabled"
    }
}

fn ocr2_region_render_chunk_mode_label() -> &'static str {
    #[cfg(any(feature = "document-extract-pdf-render", test))]
    {
        hybrid_page_ocr2_region_render_chunk_mode_with_lookup(&|key| std::env::var(key).ok())
            .as_str()
    }
    #[cfg(not(any(feature = "document-extract-pdf-render", test)))]
    {
        "page"
    }
}

fn failed_page_recovery_mode_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> HybridPdfFailedPageRecoveryMode {
    match lookup(DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV)
        .unwrap_or_default()
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .as_str()
    {
        FAILED_PAGE_RECOVERY_HOSTED_VLM_PAGE_MODE => HybridPdfFailedPageRecoveryMode::HostedVlmPage,
        _ => HybridPdfFailedPageRecoveryMode::Disabled,
    }
}

fn failed_page_recovery_mode() -> HybridPdfFailedPageRecoveryMode {
    failed_page_recovery_mode_with_lookup(&|key| std::env::var(key).ok())
}

fn failed_page_recovery_mode_label() -> &'static str {
    failed_page_recovery_mode().as_str()
}

#[cfg(feature = "document-extract-pdf-render")]
fn ocr2_region_render_request_chunks_with_lookup(
    regions: &[PdfPageRegionRenderRequest],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Vec<Vec<PdfPageRegionRenderRequest>> {
    match hybrid_page_ocr2_region_render_chunk_mode_with_lookup(lookup) {
        HybridPdfOcr2RegionRenderChunkMode::All => page_region_render_request_chunks_all(regions),
        HybridPdfOcr2RegionRenderChunkMode::PageAreaDesc => {
            page_region_render_request_chunks_by_page_area_desc(regions)
        }
        HybridPdfOcr2RegionRenderChunkMode::PageMaxAreaDesc => {
            page_region_render_request_chunks_by_page_max_area_desc(regions)
        }
        HybridPdfOcr2RegionRenderChunkMode::Region => {
            page_region_render_request_chunks_by_region(regions)
        }
        HybridPdfOcr2RegionRenderChunkMode::Page => {
            page_region_render_request_chunks_by_page(regions)
        }
    }
}

#[cfg(feature = "document-extract-pdf-render")]
fn ocr2_region_render_ahead_limit_with_lookup(
    chunk_count: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> usize {
    let requested = lookup(DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD_ENV)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(1);
    requested.clamp(1, chunk_count.max(1))
}

#[derive(Debug, Clone, Copy, Default)]
struct Ocr2RegionMaterializationStats {
    requested_region_count: usize,
    rendered_region_count: usize,
    render_cache_hit_count: usize,
    render_cache_miss_count: usize,
    render_reported_elapsed_ms: f64,
    pipeline_render_chunk_count: usize,
    pipeline_region_dispatch_count: usize,
    pipeline_base_result_count: usize,
    pipeline_base_result_shard_count: usize,
    pipeline_region_result_count: usize,
    pipeline_region_result_shard_count: usize,
}

#[derive(Debug)]
struct Ocr2RegionMaterialization {
    inputs: Vec<PdfOcrShardInput>,
    stats: Ocr2RegionMaterializationStats,
    phase_elapsed_ms: BTreeMap<String, f64>,
}

impl Ocr2RegionMaterialization {
    fn new(inputs: Vec<PdfOcrShardInput>) -> Self {
        Self {
            inputs,
            stats: Ocr2RegionMaterializationStats::default(),
            phase_elapsed_ms: BTreeMap::new(),
        }
    }

    fn record_phase_elapsed(&mut self, phase: &str, started: Instant) {
        record_phase_elapsed(&mut self.phase_elapsed_ms, phase, started);
    }
}

async fn write_hybrid_page_ocr_fallback_report(
    request: &DocumentExtractFlightRequest,
    output: &Path,
    reason: &str,
) {
    let report = json!({
        "schema": "xiuxian_wendao.hybrid_page_ocr_fallback.v1",
        "sourcePath": request.source_path,
        "outputDir": output.to_string_lossy(),
        "reason": reason,
    });
    let path = output.join(HYBRID_PAGE_OCR_FALLBACK_REPORT_NAME);
    match serde_json::to_vec_pretty(&report) {
        Ok(bytes) => {
            if let Err(error) = tokio::fs::write(path.as_path(), bytes).await {
                log::warn!(
                    "failed to write hybrid PDF OCR fallback report `{}`: {error}",
                    path.display()
                );
            }
        }
        Err(error) => {
            log::warn!("failed to serialize hybrid PDF OCR fallback report: {error}");
        }
    }
}

async fn write_hybrid_page_ocr_timing_report(
    source: &Path,
    output: &Path,
    resource_batch: &HybridDocumentResourceBatch,
    region_materialization_stats: &Ocr2RegionMaterializationStats,
    phase_elapsed_ms: &BTreeMap<String, f64>,
    scheduler_trace: &[PdfOcrShardSchedulerTrace],
    total_elapsed_ms: f64,
) {
    let report = json!({
        "schema": "xiuxian_wendao.hybrid_page_ocr_timing.v1",
        "sourcePath": source.to_string_lossy(),
        "outputDir": output.to_string_lossy(),
        "pageCount": resource_batch.page_count,
        "ocrShardCount": resource_batch.ocr_inputs.len(),
        "ocr2RegionShardCount": resource_batch
            .ocr_inputs
            .iter()
            .filter(|input| input.shard_type == "region"
                && is_hosted_vlm_direct_profile(input.ocr_profile.as_str()))
            .count(),
        "ocr2RegionRequestCount": region_materialization_stats.requested_region_count,
        "ocr2RegionRenderedShardCount": region_materialization_stats.rendered_region_count,
        "ocr2RegionRenderCacheHitCount": region_materialization_stats.render_cache_hit_count,
        "ocr2RegionRenderCacheMissCount": region_materialization_stats.render_cache_miss_count,
        "ocr2RegionRenderReportedElapsedMs": region_materialization_stats.render_reported_elapsed_ms,
        "ocr2RegionPipelineRenderChunkCount": region_materialization_stats.pipeline_render_chunk_count,
        "ocr2RegionPipelineRegionDispatchCount": region_materialization_stats.pipeline_region_dispatch_count,
        "ocr2RegionPipelineBaseResultCount": region_materialization_stats.pipeline_base_result_count,
        "ocr2RegionPipelineBaseResultShardCount": region_materialization_stats.pipeline_base_result_shard_count,
        "ocr2RegionPipelineRegionResultCount": region_materialization_stats.pipeline_region_result_count,
        "ocr2RegionPipelineRegionResultShardCount": region_materialization_stats.pipeline_region_result_shard_count,
        "ocr2RegionPipelineMode": ocr2_region_pipeline_mode_label(),
        "ocr2RegionRenderChunkMode": ocr2_region_render_chunk_mode_label(),
        "failedPageRecoveryMode": failed_page_recovery_mode_label(),
        "failedPageRecoveryHostedVlmPageShardCount": resource_batch
            .ocr_inputs
            .iter()
            .filter(|input| input.shard_type == "page"
                && is_hosted_vlm_direct_profile(input.ocr_profile.as_str()))
            .count(),
        "ocrSchedulerTrace": scheduler_trace,
        "totalElapsedMs": total_elapsed_ms,
        "phaseElapsedMs": phase_elapsed_ms,
    });
    let path = output.join(HYBRID_PAGE_OCR_TIMING_REPORT_NAME);
    match serde_json::to_vec_pretty(&report) {
        Ok(bytes) => {
            if let Err(error) = tokio::fs::write(path.as_path(), bytes).await {
                log::warn!(
                    "failed to write hybrid PDF OCR timing report `{}`: {error}",
                    path.display()
                );
            }
        }
        Err(error) => {
            log::warn!("failed to serialize hybrid PDF OCR timing report: {error}");
        }
    }
}

async fn materialize_ocr2_recovery_page_images(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
) -> Result<Vec<PdfOcrShardInput>, String> {
    let recovery_pages = inputs
        .iter()
        .filter(|input| {
            input.shard_type == "page"
                && is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
                && !Path::new(input.image_path.as_str()).is_file()
        })
        .map(|input| input.page_index)
        .collect::<BTreeSet<_>>();
    if recovery_pages.is_empty() {
        return Ok(inputs);
    }

    #[cfg(feature = "document-extract-pdf-render")]
    {
        let source_path = Path::new(render_report.source_path.as_str()).to_path_buf();
        let output_dir = Path::new(render_report.output_dir.as_str()).join("_ocr2-page-renders");
        let page_indices = recovery_pages.iter().copied().collect::<Vec<_>>();
        let render_profile =
            hybrid_page_ocr_render_profile_with_lookup(true, &|key| std::env::var(key).ok());
        let page_render_report = tokio::task::spawn_blocking(move || {
            render_pdf_page_shards_for_page_indices(
                source_path.as_path(),
                output_dir.as_path(),
                &render_profile,
                page_indices.as_slice(),
            )
        })
        .await
        .map_err(|error| format!("join hosted VLM/OCR recovery page render task: {error}"))??;
        let ocr_input_path = hybrid_page_ocr_input_arrow_path(&page_render_report)?;
        let input_batches = read_arrow_file(ocr_input_path.as_path())?;
        let rendered_inputs = decode_ocr_shard_input_batches(&input_batches)?;
        merge_ocr2_recovery_page_inputs(inputs, rendered_inputs)
    }

    #[cfg(not(feature = "document-extract-pdf-render"))]
    {
        let _ = render_report;
        Err(
            "hosted VLM/OCR recovery pages require the `document-extract-pdf-render` feature"
                .to_string(),
        )
    }
}

async fn materialize_ocr2_recovery_region_images(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
) -> Result<Ocr2RegionMaterialization, String> {
    #[cfg(feature = "document-extract-pdf-render")]
    {
        let source_path = Path::new(render_report.source_path.as_str()).to_path_buf();
        let mut materialization = Ocr2RegionMaterialization::new(inputs);
        let phase_started = Instant::now();
        let (explicit_regions, regions) = ocr2_recovery_region_requests_for_inputs(
            source_path.as_path(),
            materialization.inputs.as_slice(),
        )?;
        materialization.stats.requested_region_count = regions.len();
        materialization.record_phase_elapsed("regionMaterializePlan", phase_started);
        if regions.is_empty() {
            return Ok(materialization);
        }

        #[cfg(feature = "document-extract-pdf-render")]
        {
            let phase_started = Instant::now();
            let request_count = regions.len();
            let region_pages = regions
                .iter()
                .map(|region| region.page_index)
                .collect::<BTreeSet<_>>();
            let render_profile =
                hybrid_page_ocr_render_profile_with_lookup(true, &|key| std::env::var(key).ok());
            let output_dir = ocr2_region_render_cache_dir(
                source_path.as_path(),
                &render_profile,
                regions.as_slice(),
            )?;
            let cached_region_render_report = cached_ocr2_region_render_report(
                source_path.as_path(),
                output_dir.as_path(),
                render_report.page_count,
                &render_profile,
                request_count,
            );
            let render_cache_hit = cached_region_render_report.is_some();
            let region_render_report = if let Some(report) = cached_region_render_report {
                report
            } else {
                let source_for_render = source_path.clone();
                let output_for_render = output_dir.clone();
                let regions_for_render = regions.clone();
                tokio::task::spawn_blocking(move || {
                    render_pdf_region_shards(
                        source_for_render.as_path(),
                        output_for_render.as_path(),
                        &render_profile,
                        regions_for_render.as_slice(),
                    )
                })
                .await
                .map_err(|error| {
                    format!("join hosted VLM/OCR recovery region render task: {error}")
                })??
            };
            materialization.record_phase_elapsed("regionMaterializeRender", phase_started);
            materialization.stats.render_reported_elapsed_ms = region_render_report.elapsed_ms;

            let phase_started = Instant::now();
            let ocr_input_path = hybrid_page_ocr_input_arrow_path(&region_render_report)?;
            let input_batches = read_arrow_file(ocr_input_path.as_path())?;
            let rendered_inputs = decode_ocr_shard_input_batches(&input_batches)?;
            materialization.stats.rendered_region_count = rendered_inputs.len();
            if render_cache_hit {
                materialization.stats.render_cache_hit_count = rendered_inputs.len();
            } else {
                materialization.stats.render_cache_miss_count = rendered_inputs.len();
            }
            let existing_inputs = std::mem::take(&mut materialization.inputs);
            let merged_inputs = merge_hosted_vlm_recovery_region_inputs(
                existing_inputs,
                rendered_inputs,
                &region_pages,
            )?;
            materialization.record_phase_elapsed("regionMaterializeMerge", phase_started);

            let phase_started = Instant::now();
            write_ocr2_region_scaffold_sidecar_with_lookup(
                source_path.as_path(),
                output_dir.as_path(),
                merged_inputs.as_slice(),
                explicit_regions,
                &|key| std::env::var(key).ok(),
            )?;
            materialization.inputs = merged_inputs;
            materialization.record_phase_elapsed("regionMaterializeScaffold", phase_started);
            return Ok(materialization);
        }
    }

    #[cfg(not(feature = "document-extract-pdf-render"))]
    {
        let _ = render_report;
        let _ = inputs;
        Err(
            "hosted VLM/OCR recovery regions require the `document-extract-pdf-render` feature"
                .to_string(),
        )
    }
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn ocr2_recovery_region_requests_for_inputs(
    source_path: &Path,
    inputs: &[PdfOcrShardInput],
) -> Result<(bool, Vec<PdfPageRegionRenderRequest>), String> {
    let explicit_regions = std::env::var(DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).is_ok();
    if explicit_regions {
        return hybrid_page_ocr_region_requests_for_source_with_lookup(source_path, &|key| {
            std::env::var(key).ok()
        })
        .map(|regions| (explicit_regions, regions));
    }
    if !has_ocr2_recovery_page_candidates(inputs) {
        return Ok((explicit_regions, Vec::new()));
    }
    Ok((
        explicit_regions,
        automatic_ocr2_recovery_region_requests_for_source_with_lookup(
            source_path,
            inputs,
            &|key| std::env::var(key).ok(),
        ),
    ))
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn ocr2_region_render_cache_dir(
    source: &Path,
    profile: &PdfPageRenderProfile,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<PathBuf, String> {
    Ok(ocr2_region_render_cache_root()
        .join(ocr2_region_render_cache_key(source, profile, regions)?))
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn ocr2_region_render_cache_root() -> PathBuf {
    if let Some(root) = std::env::var_os(DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT_ENV) {
        let root = PathBuf::from(root);
        return root.parent().map_or_else(
            || root.join(OCR2_REGION_RENDER_CACHE_DIR_NAME),
            |parent| parent.join(OCR2_REGION_RENDER_CACHE_DIR_NAME),
        );
    }
    let cache_root =
        std::env::var_os("PRJ_CACHE_HOME").map_or_else(|| PathBuf::from(".cache"), PathBuf::from);
    cache_root
        .join("wendao-document-extract")
        .join(OCR2_REGION_RENDER_CACHE_DIR_NAME)
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn ocr2_region_render_cache_key(
    source: &Path,
    profile: &PdfPageRenderProfile,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<String, String> {
    let source_content_hash = sha256_file_hex(source)?;
    let payload = serde_json::to_vec(&json!({
        "schema": "xiuxian_wendao.ocr2_region_render_cache_key.v1",
        "sourceContentHash": source_content_hash,
        "renderProfile": profile,
        "regions": regions,
    }))
    .map_err(|error| format!("serialize hosted VLM/OCR region render cache key: {error}"))?;
    Ok(sha256_hex(payload.as_slice()))
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn cached_ocr2_region_render_report(
    source: &Path,
    output_dir: &Path,
    page_count: u32,
    profile: &PdfPageRenderProfile,
    request_count: usize,
) -> Option<PdfPageRenderShardReport> {
    let manifest_arrow_path = output_dir.join(OCR_SHARD_MANIFEST_ARROW_NAME);
    let ocr_input_arrow_path = output_dir.join(OCR_SHARD_INPUT_ARROW_NAME);
    let pending_resource_arrow_path = output_dir.join(OCR_PENDING_RESOURCE_ARROW_NAME);
    if !manifest_arrow_path.is_file()
        || !ocr_input_arrow_path.is_file()
        || !pending_resource_arrow_path.is_file()
    {
        return None;
    }

    let Ok(input_batches) = read_arrow_file(ocr_input_arrow_path.as_path()) else {
        return None;
    };
    let Ok(inputs) = decode_ocr_shard_input_batches(&input_batches) else {
        return None;
    };
    if inputs.len() != request_count
        || inputs.iter().any(|input| {
            input.shard_type != "region" || !Path::new(input.image_path.as_str()).is_file()
        })
    {
        return None;
    }

    Some(PdfPageRenderShardReport {
        source_path: source.to_string_lossy().to_string(),
        output_dir: output_dir.to_string_lossy().to_string(),
        page_count,
        shard_count: u32::try_from(inputs.len()).unwrap_or(u32::MAX),
        manifest_arrow_path: Some(manifest_arrow_path.to_string_lossy().to_string()),
        ocr_input_arrow_path: Some(ocr_input_arrow_path.to_string_lossy().to_string()),
        pending_resource_arrow_path: Some(
            pending_resource_arrow_path.to_string_lossy().to_string(),
        ),
        render_profile: profile.profile_id.clone(),
        render_selection: "region_shards".to_string(),
        status: PdfRenderStatus::Rendered.as_str().to_string(),
        routing_decision: PdfRenderRoutingDecision::HybridPageOcrCandidate
            .as_str()
            .to_string(),
        elapsed_ms: 0.0,
        error_message: None,
    })
}

pub(crate) fn has_ocr2_recovery_page_candidates(inputs: &[PdfOcrShardInput]) -> bool {
    inputs.iter().any(|input| {
        input.shard_type == "page" && is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
    })
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn write_ocr2_region_scaffold_sidecar_with_lookup(
    source: &Path,
    output_dir: &Path,
    inputs: &[PdfOcrShardInput],
    explicit_regions: bool,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<(), String> {
    let Some(payload) = ocr2_region_scaffold_payload(source, inputs, explicit_regions, lookup)
    else {
        return Ok(());
    };
    std::fs::create_dir_all(output_dir)
        .map_err(|error| format!("create hosted VLM/OCR scaffold output directory: {error}"))?;
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|error| format!("serialize hosted VLM/OCR region scaffold sidecar: {error}"))?;
    std::fs::write(output_dir.join(OCR2_REGION_SCAFFOLD_FILE_NAME), bytes)
        .map_err(|error| format!("write hosted VLM/OCR region scaffold sidecar: {error}"))
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn sha256_file_hex(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| {
        format!(
            "read hosted VLM/OCR region render cache source `{}`: {error}",
            path.display()
        )
    })?;
    Ok(sha256_hex(bytes.as_slice()))
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn ocr2_region_scaffold_payload(
    source: &Path,
    inputs: &[PdfOcrShardInput],
    explicit_regions: bool,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<Value> {
    if hybrid_page_ocr2_scaffold_mode_with_lookup(lookup)
        != HybridPdfOcr2ScaffoldMode::RegionTableJson
    {
        return None;
    }
    let region_inputs = inputs
        .iter()
        .filter(|input| {
            input.shard_type == "region" && is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
        })
        .collect::<Vec<_>>();
    if region_inputs.is_empty() {
        return None;
    }

    let profiles = source_pdf_page_profiles_cached(source).unwrap_or_default();
    let profiles_by_page = profiles
        .iter()
        .map(|profile| (profile.page_index, profile))
        .collect::<BTreeMap<_, _>>();
    let items = region_inputs
        .iter()
        .map(|input| {
            let profile = profiles_by_page.get(&input.page_index).copied();
            json!({
                "scaffoldKind": ocr2_region_scaffold_kind(profile, explicit_regions),
                "shardElementId": input.shard_element_id,
                "parentShardElementId": input.parent_shard_element_id,
                "pageIndex": input.page_index,
                "regionIndex": input.region_index,
                "sourcePath": input.source_path,
                "sourceContentHash": input.source_content_hash,
                "rasterSha256": input.raster_sha256,
                "renderDpi": input.render_dpi,
                "imagePath": input.image_path,
                "cropBox": {
                    "left": input.crop_left,
                    "bottom": input.crop_bottom,
                    "right": input.crop_right,
                    "top": input.crop_top,
                },
                "sourcePagePixelBox": {
                    "left": input.source_page_pixel_left,
                    "top": input.source_page_pixel_top,
                    "right": input.source_page_pixel_right,
                    "bottom": input.source_page_pixel_bottom,
                },
                "sourcePageProfile": profile.map(source_page_profile_json),
            })
        })
        .collect::<Vec<_>>();
    Some(json!({
        "schema": "xiuxian_wendao.hosted_vlm_region_scaffold.v1",
        "mode": "region-table-json",
        "sourcePath": source.to_string_lossy(),
        "items": items,
    }))
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn ocr2_region_scaffold_kind(
    profile: Option<&PdfSourcePageProfile>,
    explicit_regions: bool,
) -> &'static str {
    if explicit_regions {
        return "manual_region_candidate";
    }
    let Some(profile) = profile else {
        return "complex_layout_candidate";
    };
    if profile.rectangle_ops > 0 || profile.path_ops >= 64 {
        "table_candidate"
    } else {
        "complex_layout_candidate"
    }
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn source_page_profile_json(profile: &PdfSourcePageProfile) -> Value {
    json!({
        "pageIndex": profile.page_index,
        "contentBytes": profile.content_bytes,
        "operationCount": profile.operation_count,
        "textShowOps": profile.text_show_ops,
        "pathOps": profile.path_ops,
        "rectangleOps": profile.rectangle_ops,
        "drawObjectOps": profile.draw_object_ops,
        "estimatedWeight": profile.estimated_weight,
    })
}

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/provider/hybrid/route.rs"]
mod tests;

#[cfg(feature = "document-extract-pdf-render")]
struct Ocr2RegionPipelineBatch {
    resource_batch: HybridDocumentResourceBatch,
    stats: Ocr2RegionMaterializationStats,
    phase_elapsed_ms: BTreeMap<String, f64>,
    scheduler_trace: Vec<PdfOcrShardSchedulerTrace>,
}

#[cfg(feature = "document-extract-pdf-render")]
#[derive(Debug)]
struct Ocr2RegionRenderChunk {
    output_dir: PathBuf,
    render_cache_hit: bool,
    report: PdfPageRenderShardReport,
}

#[cfg(feature = "document-extract-pdf-render")]
#[derive(Debug)]
struct ScheduledOcrBatch {
    kind: Ocr2RegionPipelineBatchKind,
    inputs: Vec<PdfOcrShardInput>,
    results: Vec<PdfOcrShardResult>,
    scheduler_trace: Vec<PdfOcrShardSchedulerTrace>,
}

#[cfg(feature = "document-extract-pdf-render")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ocr2RegionPipelineBatchKind {
    Base,
    Region,
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(crate) fn merge_ocr2_recovery_page_inputs(
    mut inputs: Vec<PdfOcrShardInput>,
    rendered_inputs: Vec<PdfOcrShardInput>,
) -> Result<Vec<PdfOcrShardInput>, String> {
    let rendered_by_page = rendered_inputs
        .into_iter()
        .map(|input| (input.page_index, input))
        .collect::<BTreeMap<_, _>>();
    for input in &mut inputs {
        if input.shard_type != "page" || !is_hosted_vlm_direct_profile(input.ocr_profile.as_str()) {
            continue;
        }
        let Some(rendered) = rendered_by_page.get(&input.page_index) else {
            return Err(format!(
                "hosted VLM/OCR recovery render did not produce page {}",
                input.page_index
            ));
        };
        let ocr_profile = input.ocr_profile.clone();
        let ocr_engine = input.ocr_engine.clone();
        *input = rendered.clone();
        input.ocr_profile = ocr_profile;
        input.ocr_engine = ocr_engine;
    }
    Ok(inputs)
}

fn failed_page_recovery_input(input: &PdfOcrShardInput) -> PdfOcrShardInput {
    let mut recovery_input = input.clone();
    recovery_input.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
    recovery_input.ocr_engine = HOSTED_VLM_DIRECT_OCR_ENGINE.to_string();
    recovery_input
}

fn failed_page_recovery_candidates(
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
) -> Vec<(usize, PdfOcrShardInput)> {
    inputs
        .iter()
        .zip(results.iter())
        .enumerate()
        .filter(|(_, (input, result))| is_failed_page_recovery_candidate(input, result))
        .map(|(index, (input, _result))| (index, failed_page_recovery_input(input)))
        .collect()
}

fn is_failed_page_recovery_candidate(input: &PdfOcrShardInput, result: &PdfOcrShardResult) -> bool {
    input.shard_type == "page"
        && !is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
        && input.ocr_profile != PDF_OCR_BACKEND_TEXT_PROFILE
        && (result.status != PdfOcrShardResultStatus::Succeeded
            || result
                .text
                .as_deref()
                .is_none_or(|text| text.trim().is_empty()))
}

async fn recover_failed_page_ocr_results(
    render_report: &PdfPageRenderShardReport,
    endpoint_urls: &[String],
    pdf_ocr_scheduler: &PdfOcrWorkerScheduler,
    inputs: &mut [PdfOcrShardInput],
    results: &mut [PdfOcrShardResult],
) -> Result<(), String> {
    if failed_page_recovery_mode() != HybridPdfFailedPageRecoveryMode::HostedVlmPage {
        return Ok(());
    }
    let candidates = failed_page_recovery_candidates(inputs, results);
    if candidates.is_empty() {
        return Ok(());
    }
    let positions = candidates
        .iter()
        .map(|(position, _input)| *position)
        .collect::<Vec<_>>();
    let recovery_inputs = candidates
        .into_iter()
        .map(|(_position, input)| input)
        .collect::<Vec<_>>();
    let recovery_inputs =
        materialize_ocr2_recovery_page_images(render_report, recovery_inputs).await?;
    let response = pdf_ocr_scheduler
        .request_shards_with_endpoints(endpoint_urls, recovery_inputs.as_slice())
        .await?;
    let recovery_results =
        order_ocr_results_by_inputs(recovery_inputs.as_slice(), response.results)?;
    for ((position, recovery_input), recovery_result) in positions
        .into_iter()
        .zip(recovery_inputs.into_iter())
        .zip(recovery_results.into_iter())
    {
        inputs[position] = recovery_input;
        results[position] = recovery_result;
    }
    Ok(())
}

#[cfg(feature = "document-extract-pdf-render")]
async fn materialize_hybrid_page_ocr_resource_batch_with_region_pipeline(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
    pdf_ocr_scheduler: &PdfOcrWorkerScheduler,
) -> Result<Ocr2RegionPipelineBatch, String> {
    let source_path = Path::new(render_report.source_path.as_str()).to_path_buf();
    let mut phase_elapsed_ms = BTreeMap::new();
    let mut stats = Ocr2RegionMaterializationStats::default();

    let phase_started = Instant::now();
    let (explicit_regions, regions) =
        ocr2_recovery_region_requests_for_inputs(source_path.as_path(), inputs.as_slice())?;
    stats.requested_region_count = regions.len();
    record_phase_elapsed(
        &mut phase_elapsed_ms,
        "regionMaterializePlan",
        phase_started,
    );

    if regions.is_empty() {
        let phase_started = Instant::now();
        let inputs = materialize_ocr2_recovery_page_images(render_report, inputs).await?;
        record_phase_elapsed(&mut phase_elapsed_ms, "pageMaterialize", phase_started);

        let phase_started = Instant::now();
        let resource_batch =
            materialize_hybrid_page_ocr_resource_batch(render_report, inputs, pdf_ocr_scheduler)
                .await?;
        record_phase_elapsed(&mut phase_elapsed_ms, "ocrScheduler", phase_started);
        return Ok(Ocr2RegionPipelineBatch {
            resource_batch,
            stats,
            phase_elapsed_ms,
            scheduler_trace: Vec::new(),
        });
    }

    let region_pages = regions
        .iter()
        .map(|region| region.page_index)
        .collect::<BTreeSet<_>>();
    let mut base_inputs = inputs;
    downgrade_hosted_vlm_region_parent_page_inputs(&mut base_inputs, &region_pages);
    let parent_page_shards = hosted_vlm_region_parent_page_shards(base_inputs.as_slice());

    let phase_started = Instant::now();
    let base_inputs = materialize_ocr2_recovery_page_images(render_report, base_inputs).await?;
    record_phase_elapsed(&mut phase_elapsed_ms, "pageMaterialize", phase_started);

    let endpoint_url = std::env::var("WENDAO_DOCUMENT_EXTRACT_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_DOCUMENT_EXTRACT_ENDPOINT.to_string());
    let endpoint_urls = pdf_ocr_endpoint_urls(endpoint_url.as_str());
    let render_profile =
        hybrid_page_ocr_render_profile_with_lookup(true, &|key| std::env::var(key).ok());
    let region_chunks = ocr2_region_render_request_chunks_with_lookup(regions.as_slice(), &|key| {
        std::env::var(key).ok()
    });
    let render_ahead_limit =
        ocr2_region_render_ahead_limit_with_lookup(region_chunks.len(), &|key| {
            std::env::var(key).ok()
        });
    let mut chunk_index = 0usize;
    let mut pending_ocr: FuturesUnordered<BoxFuture<'_, Result<ScheduledOcrBatch, String>>> =
        FuturesUnordered::new();
    let mut active_renders: FuturesUnordered<
        tokio::task::JoinHandle<Result<Ocr2RegionRenderChunk, String>>,
    > = FuturesUnordered::new();
    let mut all_inputs = base_inputs.clone();
    let mut all_results = Vec::new();
    let mut scheduler_trace = Vec::new();
    let scheduler_started = Instant::now();

    if !base_inputs.is_empty() {
        phase_elapsed_ms.insert(
            "regionPipelineBaseDispatch".to_string(),
            scheduler_started.elapsed().as_secs_f64() * 1000.0,
        );
        pending_ocr.push(schedule_ocr_input_batch(
            pdf_ocr_scheduler,
            endpoint_urls.as_slice(),
            Ocr2RegionPipelineBatchKind::Base,
            base_inputs,
        ));
    }
    fill_ocr2_region_render_ahead(
        source_path.as_path(),
        render_report.page_count,
        &render_profile,
        region_chunks.as_slice(),
        &mut chunk_index,
        render_ahead_limit,
        &mut active_renders,
    );

    while !active_renders.is_empty() || !pending_ocr.is_empty() {
        tokio::select! {
            render_join = active_renders.next(), if !active_renders.is_empty() => {
                let render_join = render_join
                    .ok_or_else(|| "hosted VLM/OCR region pipeline render queue ended unexpectedly".to_string())?;
                let render_chunk = render_join
                    .map_err(|error| format!("join hosted VLM/OCR region pipeline render task: {error}"))??;
                let render_ready_elapsed_ms =
                    scheduler_started.elapsed().as_secs_f64() * 1000.0;
                stats.pipeline_render_chunk_count =
                    stats.pipeline_render_chunk_count.saturating_add(1);
                if stats.pipeline_render_chunk_count == 1 {
                    phase_elapsed_ms.insert(
                        "regionPipelineFirstRegionReady".to_string(),
                        render_ready_elapsed_ms,
                    );
                }
                phase_elapsed_ms.insert(
                    "regionPipelineLastRegionReady".to_string(),
                    render_ready_elapsed_ms,
                );
                let region_inputs = decode_ocr2_region_render_chunk(
                    source_path.as_path(),
                    &render_chunk,
                    &parent_page_shards,
                    explicit_regions,
                    &mut stats,
                )?;
                if !region_inputs.is_empty() {
                    all_inputs.extend(region_inputs.clone());
                    let dispatch_elapsed_ms =
                        scheduler_started.elapsed().as_secs_f64() * 1000.0;
                    stats.pipeline_region_dispatch_count = stats
                        .pipeline_region_dispatch_count
                        .saturating_add(1);
                    if stats.pipeline_region_dispatch_count == 1 {
                        phase_elapsed_ms.insert(
                            "regionPipelineFirstRegionDispatch".to_string(),
                            dispatch_elapsed_ms,
                        );
                    }
                    phase_elapsed_ms.insert(
                        "regionPipelineLastRegionDispatch".to_string(),
                        dispatch_elapsed_ms,
                    );
                    pending_ocr.push(schedule_ocr_input_batch(
                        pdf_ocr_scheduler,
                        endpoint_urls.as_slice(),
                        Ocr2RegionPipelineBatchKind::Region,
                        region_inputs,
                    ));
                }
                fill_ocr2_region_render_ahead(
                    source_path.as_path(),
                    render_report.page_count,
                    &render_profile,
                    region_chunks.as_slice(),
                    &mut chunk_index,
                    render_ahead_limit,
                    &mut active_renders,
                );
            }
            scheduled = pending_ocr.next(), if !pending_ocr.is_empty() => {
                let scheduled = scheduled
                    .ok_or_else(|| "hosted VLM/OCR region pipeline request queue ended unexpectedly".to_string())??;
                let completed_elapsed_ms = scheduler_started.elapsed().as_secs_f64() * 1000.0;
                record_ocr2_region_pipeline_batch_result(
                    &mut phase_elapsed_ms,
                    &mut stats,
                    scheduled.kind,
                    scheduled.inputs.len(),
                    completed_elapsed_ms,
                );
                collect_scheduled_ocr_batch(&mut all_results, &mut scheduler_trace, scheduled)?;
            }
        }
    }

    let scheduler_elapsed_ms = scheduler_started.elapsed().as_secs_f64() * 1000.0;
    phase_elapsed_ms.insert("ocrScheduler".to_string(), scheduler_elapsed_ms);
    all_inputs.sort_by(|left, right| left.reading_order_key.cmp(&right.reading_order_key));
    let resource_batch = materialize_hybrid_page_ocr_resource_batch_from_results(
        render_report,
        all_inputs,
        all_results,
        scheduler_elapsed_ms,
    )?;
    Ok(Ocr2RegionPipelineBatch {
        resource_batch,
        stats,
        phase_elapsed_ms,
        scheduler_trace,
    })
}

#[cfg(feature = "document-extract-pdf-render")]
fn fill_ocr2_region_render_ahead(
    source_path: &Path,
    page_count: u32,
    render_profile: &PdfPageRenderProfile,
    region_chunks: &[Vec<PdfPageRegionRenderRequest>],
    chunk_index: &mut usize,
    render_ahead_limit: usize,
    active_renders: &mut FuturesUnordered<
        tokio::task::JoinHandle<Result<Ocr2RegionRenderChunk, String>>,
    >,
) {
    while active_renders.len() < render_ahead_limit {
        let Some(render) = spawn_next_ocr2_region_render_chunk(
            source_path,
            page_count,
            render_profile,
            region_chunks,
            chunk_index,
        ) else {
            break;
        };
        active_renders.push(render);
    }
}

#[cfg(feature = "document-extract-pdf-render")]
fn schedule_ocr_input_batch<'a>(
    pdf_ocr_scheduler: &'a PdfOcrWorkerScheduler,
    endpoint_urls: &'a [String],
    kind: Ocr2RegionPipelineBatchKind,
    inputs: Vec<PdfOcrShardInput>,
) -> BoxFuture<'a, Result<ScheduledOcrBatch, String>> {
    async move {
        let response = pdf_ocr_scheduler
            .request_shards_with_endpoints(endpoint_urls, inputs.as_slice())
            .await?;
        Ok(ScheduledOcrBatch {
            kind,
            inputs,
            results: response.results,
            scheduler_trace: response.scheduler_trace,
        })
    }
    .boxed()
}

#[cfg(feature = "document-extract-pdf-render")]
fn record_ocr2_region_pipeline_batch_result(
    phase_elapsed_ms: &mut BTreeMap<String, f64>,
    stats: &mut Ocr2RegionMaterializationStats,
    kind: Ocr2RegionPipelineBatchKind,
    input_count: usize,
    completed_elapsed_ms: f64,
) {
    match kind {
        Ocr2RegionPipelineBatchKind::Base => {
            stats.pipeline_base_result_count = stats.pipeline_base_result_count.saturating_add(1);
            stats.pipeline_base_result_shard_count = stats
                .pipeline_base_result_shard_count
                .saturating_add(input_count);
            if stats.pipeline_base_result_count == 1 {
                phase_elapsed_ms.insert(
                    "regionPipelineFirstBaseResult".to_string(),
                    completed_elapsed_ms,
                );
            }
            phase_elapsed_ms.insert(
                "regionPipelineLastBaseResult".to_string(),
                completed_elapsed_ms,
            );
        }
        Ocr2RegionPipelineBatchKind::Region => {
            stats.pipeline_region_result_count =
                stats.pipeline_region_result_count.saturating_add(1);
            stats.pipeline_region_result_shard_count = stats
                .pipeline_region_result_shard_count
                .saturating_add(input_count);
            if stats.pipeline_region_result_count == 1 {
                phase_elapsed_ms.insert(
                    "regionPipelineFirstRegionResult".to_string(),
                    completed_elapsed_ms,
                );
            }
            phase_elapsed_ms.insert(
                "regionPipelineLastRegionResult".to_string(),
                completed_elapsed_ms,
            );
        }
    }
}

#[cfg(feature = "document-extract-pdf-render")]
fn collect_scheduled_ocr_batch(
    all_results: &mut Vec<PdfOcrShardResult>,
    scheduler_trace: &mut Vec<PdfOcrShardSchedulerTrace>,
    scheduled: ScheduledOcrBatch,
) -> Result<(), String> {
    let ordered = order_ocr_results_by_inputs(scheduled.inputs.as_slice(), scheduled.results)?;
    all_results.extend(ordered);
    scheduler_trace.extend(scheduled.scheduler_trace);
    Ok(())
}

#[cfg(feature = "document-extract-pdf-render")]
fn spawn_next_ocr2_region_render_chunk(
    source_path: &Path,
    page_count: u32,
    render_profile: &PdfPageRenderProfile,
    region_chunks: &[Vec<PdfPageRegionRenderRequest>],
    chunk_index: &mut usize,
) -> Option<tokio::task::JoinHandle<Result<Ocr2RegionRenderChunk, String>>> {
    let regions = region_chunks.get(*chunk_index)?.clone();
    *chunk_index = (*chunk_index).saturating_add(1);
    let source_path = source_path.to_path_buf();
    let render_profile = render_profile.clone();
    Some(tokio::spawn(async move {
        let output_dir = ocr2_region_render_cache_dir(
            source_path.as_path(),
            &render_profile,
            regions.as_slice(),
        )?;
        if let Some(report) = cached_ocr2_region_render_report(
            source_path.as_path(),
            output_dir.as_path(),
            page_count,
            &render_profile,
            regions.len(),
        ) {
            return Ok(Ocr2RegionRenderChunk {
                output_dir,
                render_cache_hit: true,
                report,
            });
        }
        let source_for_render = source_path;
        let output_for_render = output_dir.clone();
        let render_profile_for_render = render_profile;
        let regions_for_render = regions;
        let report = tokio::task::spawn_blocking(move || {
            render_pdf_region_shards(
                source_for_render.as_path(),
                output_for_render.as_path(),
                &render_profile_for_render,
                regions_for_render.as_slice(),
            )
        })
        .await
        .map_err(|error| {
            format!("join hosted VLM/OCR region pipeline blocking render task: {error}")
        })??;
        Ok(Ocr2RegionRenderChunk {
            output_dir,
            render_cache_hit: false,
            report,
        })
    }))
}

#[cfg(feature = "document-extract-pdf-render")]
fn decode_ocr2_region_render_chunk(
    source_path: &Path,
    render_chunk: &Ocr2RegionRenderChunk,
    parent_page_shards: &BTreeMap<u32, String>,
    explicit_regions: bool,
    stats: &mut Ocr2RegionMaterializationStats,
) -> Result<Vec<PdfOcrShardInput>, String> {
    let ocr_input_path = hybrid_page_ocr_input_arrow_path(&render_chunk.report)?;
    let input_batches = read_arrow_file(ocr_input_path.as_path())?;
    let rendered_inputs = decode_ocr_shard_input_batches(&input_batches)?;
    stats.render_reported_elapsed_ms += render_chunk.report.elapsed_ms;
    stats.rendered_region_count = stats
        .rendered_region_count
        .saturating_add(rendered_inputs.len());
    if render_chunk.render_cache_hit {
        stats.render_cache_hit_count = stats
            .render_cache_hit_count
            .saturating_add(rendered_inputs.len());
    } else {
        stats.render_cache_miss_count = stats
            .render_cache_miss_count
            .saturating_add(rendered_inputs.len());
    }
    let region_inputs =
        prepare_hosted_vlm_recovery_region_inputs(parent_page_shards, rendered_inputs)?;
    write_ocr2_region_scaffold_sidecar_with_lookup(
        source_path,
        render_chunk.output_dir.as_path(),
        region_inputs.as_slice(),
        explicit_regions,
        &|key| std::env::var(key).ok(),
    )?;
    Ok(region_inputs)
}

async fn materialize_hybrid_page_ocr_resource_batch(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
    pdf_ocr_scheduler: &PdfOcrWorkerScheduler,
) -> Result<HybridDocumentResourceBatch, String> {
    let endpoint_url = std::env::var("WENDAO_DOCUMENT_EXTRACT_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_DOCUMENT_EXTRACT_ENDPOINT.to_string());
    let endpoint_urls = pdf_ocr_endpoint_urls(endpoint_url.as_str());
    let scheduler_started = Instant::now();
    let response = pdf_ocr_scheduler
        .request_shards_with_endpoints(endpoint_urls.as_slice(), inputs.as_slice())
        .await?;
    let mut inputs = inputs;
    let mut results = order_ocr_results_by_inputs(inputs.as_slice(), response.results)?;
    recover_failed_page_ocr_results(
        render_report,
        endpoint_urls.as_slice(),
        pdf_ocr_scheduler,
        inputs.as_mut_slice(),
        results.as_mut_slice(),
    )
    .await?;
    let scheduler_elapsed_ms = scheduler_started.elapsed().as_secs_f64() * 1000.0;
    materialize_hybrid_page_ocr_resource_batch_from_results(
        render_report,
        inputs,
        results,
        scheduler_elapsed_ms,
    )
}

fn materialize_hybrid_page_ocr_resource_batch_from_results(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
    results: Vec<PdfOcrShardResult>,
    scheduler_elapsed_ms: f64,
) -> Result<HybridDocumentResourceBatch, String> {
    let results = order_ocr_results_by_inputs(inputs.as_slice(), results)?;
    validate_successful_ocr_results(
        results.as_slice(),
        render_report.page_count,
        u32::try_from(inputs.len()).unwrap_or(u32::MAX),
    )?;
    validate_ocr_results_match_inputs(inputs.as_slice(), results.as_slice())?;
    let has_region_shards = inputs.iter().any(|input| input.shard_type == "region");
    let resource_batch = build_ocr_result_resource_batch(results.as_slice())?;

    if render_report.shard_count == render_report.page_count && !has_region_shards {
        validate_hybrid_page_coverage(render_report.page_count, &[], results.as_slice())?;
        let metrics = results
            .iter()
            .zip(inputs.iter())
            .map(|(result, input)| {
                PdfOcrShardMetric::from_ocr_result(
                    input,
                    result,
                    render_report.page_count,
                    Some(scheduler_elapsed_ms),
                )
            })
            .collect::<Vec<_>>();
        return Ok(HybridDocumentResourceBatch::new(
            resource_batch,
            inputs,
            results,
            metrics,
            render_report.page_count,
            Vec::new(),
        ));
    }

    validate_hybrid_shard_coverage(
        render_report.page_count,
        &[],
        inputs.as_slice(),
        results.as_slice(),
    )?;
    let metrics = results
        .iter()
        .zip(inputs.iter())
        .map(|(result, input)| {
            PdfOcrShardMetric::from_ocr_result(
                input,
                result,
                render_report.page_count,
                Some(scheduler_elapsed_ms),
            )
        })
        .collect::<Vec<_>>();
    Ok(HybridDocumentResourceBatch::new(
        resource_batch,
        inputs,
        results,
        metrics,
        render_report.page_count,
        Vec::new(),
    ))
}
