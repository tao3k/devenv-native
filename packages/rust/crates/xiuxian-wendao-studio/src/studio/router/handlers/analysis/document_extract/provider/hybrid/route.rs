//! Hybrid PDF OCR document extraction route.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
#[cfg(feature = "document-extract-pdf-render")]
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use arrow::array::{Array, ArrayRef, Int32Array, StringArray};
use arrow::compute::concat_batches;
use arrow::record_batch::RecordBatch as EngineRecordBatch;
use xiuxian_wendao_attachments::pdf::metrics::PdfOcrShardMetric;
#[cfg(any(feature = "document-extract-pdf-source-range", test))]
use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_DEFAULT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE,
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE, PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput,
    PdfOcrShardResult, PdfOcrShardResultStatus, build_ocr_result_resource_batch,
    decode_ocr_shard_input_batches, is_hosted_vlm_direct_profile,
};
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::ocr::{
    downgrade_hosted_vlm_region_parent_page_inputs, hosted_vlm_region_parent_page_shards,
    prepare_hosted_vlm_recovery_region_inputs,
};
#[cfg(any(
    feature = "document-extract-pdf-source-range",
    feature = "document-extract-pdf-render",
    test
))]
use xiuxian_wendao_attachments::pdf::profile::{
    PdfSourcePageProfile, pdf_source_page_is_backend_text_topup_profile,
    pdf_source_page_is_fast_profile_risk, pdf_source_page_requires_structure_authority,
    pdf_source_page_structure_cost, source_pdf_page_profiles_cached,
};
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageRegionRenderRequest, PdfPageRenderProfile, PdfRegionShardRenderRequest,
    page_region_render_request_chunks_all, page_region_render_request_chunks_by_page,
    page_region_render_request_chunks_by_page_area_desc,
    page_region_render_request_chunks_by_page_max_area_desc,
    page_region_render_request_chunks_by_region,
    page_region_render_request_chunks_by_region_seed_page,
    render_pdf_region_shards_with_source_hash,
};
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageRenderShardReport, PdfRenderRoutingDecision, PdfRenderStatus, source_pdf_page_count,
};
use xiuxian_wendao_server::transport::{
    DOCUMENT_EXTRACT_FULL_PROFILE, DocumentExtractFlightRequest, DocumentExtractFlightRouteResponse,
};

#[cfg(feature = "document-extract-pdf-render")]
use super::hybrid_page_ocr_render_profile_with_lookup;
use super::{
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, HybridDocumentResourceBatch,
    HybridPdfOcr2RegionPlanner, HybridPdfOcrProfilePlanner, PageRangeDoclingFallbackChunkTiming,
    PageRangeDoclingFallbackPlanRange, PageRangeDoclingFallbackPlanSummary,
    PageRangeDoclingFallbackSourceProfileSummary, apply_hybrid_page_ocr_profile_plan,
    hybrid_page_ocr_input_arrow_path, hybrid_page_ocr_profile_planner,
    hybrid_page_ocr_profile_planner_with_lookup, hybrid_page_ocr_request_paths,
    hybrid_page_ocr2_region_planner_with_lookup, render_hybrid_page_ocr_shards,
    validate_hybrid_page_coverage, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results_for_inputs,
};
use crate::studio::PdfOcrShardSchedulerTrace;
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, write_arrow_file,
};
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    read_arrow_file, read_cached_document_batches,
};
use crate::studio::router::handlers::analysis::document_extract::order_ocr_results_by_inputs;
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::{
    PdfOcrWorkerScheduler, pdf_ocr_endpoint_urls,
};
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::write_hybrid_document_resource_artifacts;
use crate::studio::router::handlers::analysis::document_extract::provider::{
    DEFAULT_DOCUMENT_EXTRACT_ENDPOINT, StudioDocumentExtractFlightRouteProvider,
};

#[path = "route_parts/artifact_cache.rs"]
mod artifact_cache;
#[path = "route_parts/batch.rs"]
mod batch;
#[path = "route_parts/docling_range.rs"]
mod docling_range;
#[path = "route_parts/docling_structure_budget.rs"]
mod docling_structure_budget;
#[path = "route_parts/failed_page.rs"]
mod failed_page;
#[path = "route_parts/pipeline.rs"]
mod pipeline;
#[path = "route_parts/regions/mod.rs"]
mod regions;
#[path = "route_parts/reports.rs"]
mod reports;
#[path = "route_parts/resource_rows.rs"]
mod resource_rows;
#[path = "route_parts/source_inputs.rs"]
mod source_inputs;
#[path = "route_parts/support.rs"]
mod support;

#[cfg(all(test, feature = "document-extract-pdf-render"))]
use artifact_cache::{
    hybrid_page_ocr_artifact_cache_key_for_test, hybrid_page_ocr_artifact_cache_response_for_test,
    store_hybrid_page_ocr_artifact_cache_for_test,
};
use artifact_cache::{
    hybrid_page_ocr_artifact_cache_response, store_hybrid_page_ocr_artifact_cache,
};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(crate) use batch::docling_page_range_document_extract_endpoint_count_with_lookup;
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use batch::{
    DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE_ENV,
    docling_page_range_fallback_profile_with_lookup,
};
use batch::{
    materialize_docling_page_range_resource_batch, materialize_hybrid_page_ocr_resource_batch,
    materialize_hybrid_page_ocr_resource_batch_with_eager_docling_fallback,
};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use docling_range::{
    contiguous_page_ranges, docling_page_range_chunk_concurrency_with_lookup,
    docling_page_range_chunk_plan_with_lookup, docling_page_range_chunk_size_for_pages_with_lookup,
    docling_page_range_chunk_size_for_planner_with_lookup,
    docling_page_range_chunk_size_with_lookup, docling_page_range_fallback_ranges,
    docling_page_range_fallback_ranges_with_lookup, weighted_docling_page_range_fallback_ranges,
};
use docling_range::{
    docling_page_range_chunk_concurrency_limit_with_lookup,
    docling_page_range_fallback_page_indices,
    docling_page_range_fallback_plan_for_source_with_lookup,
    docling_page_range_hedge_delay_ms_with_lookup, docling_page_range_target_chunk_count,
    docling_structure_recovery_page_range_fallback_pages, failed_backend_text_page_indices,
    has_region_shard_on_pages, has_unhandled_non_success_result,
    kept_results_without_docling_page_range_fallback_pages,
    scheduled_inputs_without_docling_page_range_fallback_pages,
};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use docling_structure_budget::{
    structure_cost_budgeted_docling_page_range_fallback_ranges,
    structure_cost_budgeted_docling_page_range_fallback_ranges_with_limit,
};
use failed_page::recover_failed_page_ocr_results;
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use failed_page::{failed_page_recovery_candidates, failed_page_recovery_input};
#[cfg(feature = "document-extract-pdf-render")]
use pipeline::materialize_hybrid_page_ocr_resource_batch_with_region_pipeline;
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use pipeline::{Ocr2RegionPipelineBatchKind, record_ocr2_region_pipeline_batch_result};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(crate) use regions::OCR2_REGION_SCAFFOLD_FILE_NAME;
#[cfg(feature = "document-extract-pdf-render")]
use regions::{
    cached_ocr2_region_render_report, ocr2_recovery_region_requests_for_inputs,
    ocr2_region_render_cache_dir_with_source_hash, sha256_file_hex,
    write_ocr2_region_scaffold_sidecar_with_lookup,
};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(crate) use regions::{has_ocr2_recovery_page_candidates, merge_ocr2_recovery_page_inputs};
use regions::{materialize_ocr2_recovery_page_images, materialize_ocr2_recovery_region_images};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use regions::{
    ocr2_region_render_cache_key, ocr2_region_render_cache_key_with_source_hash,
    ocr2_region_scaffold_payload,
};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use reports::{
    docling_centered_structure_authority_page_count, page_range_docling_fallback_chunk_summary,
};
use reports::{write_hybrid_page_ocr_fallback_report, write_hybrid_page_ocr_timing_report};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use resource_rows::normalize_docling_page_range_wrapper_rows;
use resource_rows::{
    concat_document_resource_batches, materialize_hybrid_page_ocr_resource_batch_from_results,
};
use source_inputs::direct_docling_structure_recovery_source_inputs;
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use source_inputs::direct_docling_structure_recovery_source_inputs_for_profiles;
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use source_inputs::direct_docling_structure_recovery_source_inputs_for_profiles_with_lookup;
use support::{
    Ocr2RegionMaterialization, Ocr2RegionMaterializationStats,
    direct_docling_structure_recovery_page_range_enabled,
    direct_docling_structure_recovery_render_report, failed_page_recovery_mode,
    failed_page_recovery_mode_label, ocr2_region_pipeline_enabled, ocr2_region_pipeline_mode_label,
    ocr2_region_render_chunk_mode_label, record_ocr_scheduler_or_docling_fallback_phase,
    record_phase_elapsed,
};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use support::{
    direct_docling_structure_recovery_page_range_enabled_with_lookup,
    failed_page_recovery_mode_with_lookup,
};
#[cfg(feature = "document-extract-pdf-render")]
use support::{
    ocr2_region_render_ahead_limit_for_capacity_with_lookup,
    ocr2_region_render_request_chunks_with_lookup,
};

const HYBRID_PAGE_OCR_FALLBACK_REPORT_NAME: &str = "_hybrid_page_ocr_fallback.json";
const HYBRID_PAGE_OCR_TIMING_REPORT_NAME: &str = "_hybrid_page_ocr_timing.json";
#[cfg(feature = "document-extract-pdf-render")]
const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD";
const DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY";
const DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_SIZE_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_SIZE";
const DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN";
const DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_CONCURRENCY_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_CONCURRENCY";
const DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_STRUCTURE_COST_BUDGET_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_STRUCTURE_COST_BUDGET";
const DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_HEDGE_DELAY_MS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_HEDGE_DELAY_MS";
const DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_THRESHOLD: usize = 4;
const DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_CHUNK_SIZE: u32 = 1;
const DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE: u32 = 3;
const FAILED_PAGE_RECOVERY_HOSTED_VLM_PAGE_MODE: &str = "hosted-vlm-page";
const HOSTED_VLM_DIRECT_OCR_ENGINE: &str = "hosted-vlm-direct-ocr";
const PDF_RENDER_REQUIRE_PDFIUM_ENV: &str = "WENDAO_PDF_RENDER_REQUIRE_PDFIUM";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HybridPdfFailedPageRecoveryMode {
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

struct HybridPageOcrFinishRequest<'a> {
    request: &'a DocumentExtractFlightRequest,
    source: &'a Path,
    output: &'a Path,
    resource_batch: HybridDocumentResourceBatch,
    ocr2_region_materialization_stats: Ocr2RegionMaterializationStats,
    phase_elapsed_ms: BTreeMap<String, f64>,
    ocr_scheduler_trace: &'a [PdfOcrShardSchedulerTrace],
    total_started: Instant,
}

struct RenderedHybridPageOcrBatch {
    resource_batch: HybridDocumentResourceBatch,
    ocr2_region_materialization_stats: Ocr2RegionMaterializationStats,
    ocr_scheduler_trace: Vec<PdfOcrShardSchedulerTrace>,
}

impl StudioDocumentExtractFlightRouteProvider {
    pub(crate) async fn hybrid_page_ocr_document_extract_batch(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let total_started = Instant::now();
        let mut phase_elapsed_ms = BTreeMap::new();
        let (source, output) = hybrid_page_ocr_request_paths(request);
        if let Some(response) = Self::cached_hybrid_page_ocr_response(request, &source, &output)? {
            return Ok(response);
        }

        tokio::fs::create_dir_all(output.as_path())
            .await
            .map_err(|error| {
                format!(
                    "create hybrid PDF OCR output directory `{}`: {error}",
                    output.display()
                )
            })?;

        if direct_docling_structure_recovery_page_range_enabled() {
            let resource_batch = match self
                .direct_docling_structure_recovery_batch(
                    source.as_path(),
                    output.as_path(),
                    &mut phase_elapsed_ms,
                )
                .await
            {
                Ok(batch) => batch,
                Err(reason) => {
                    return self
                        .fallback_python_document_extract(request, output.as_path(), &reason)
                        .await;
                }
            };
            return self
                .finish_hybrid_page_ocr_document_extract(HybridPageOcrFinishRequest {
                    request,
                    source: source.as_path(),
                    output: output.as_path(),
                    resource_batch,
                    ocr2_region_materialization_stats: Ocr2RegionMaterializationStats::default(),
                    phase_elapsed_ms,
                    ocr_scheduler_trace: &[],
                    total_started,
                })
                .await;
        }

        let rendered_batch = match self
            .rendered_hybrid_page_ocr_batch(
                source.as_path(),
                output.as_path(),
                &mut phase_elapsed_ms,
            )
            .await
        {
            Err(reason) => {
                if hybrid_page_ocr_render_failure_requires_fail_fast(reason.as_str()) {
                    return Err(reason);
                }
                return self
                    .fallback_python_document_extract(request, output.as_path(), &reason)
                    .await;
            }
            Ok(batch) => batch,
        };
        self.finish_hybrid_page_ocr_document_extract(HybridPageOcrFinishRequest {
            request,
            source: source.as_path(),
            output: output.as_path(),
            resource_batch: rendered_batch.resource_batch,
            ocr2_region_materialization_stats: rendered_batch.ocr2_region_materialization_stats,
            phase_elapsed_ms,
            ocr_scheduler_trace: rendered_batch.ocr_scheduler_trace.as_slice(),
            total_started,
        })
        .await
    }

    fn cached_hybrid_page_ocr_response(
        request: &DocumentExtractFlightRequest,
        source: &Path,
        output: &Path,
    ) -> Result<Option<DocumentExtractFlightRouteResponse>, String> {
        if source.exists()
            && !request.force
            && let Some(batches) = read_cached_document_batches(source, output)?
        {
            return Ok(Some(DocumentExtractFlightRouteResponse::from_batches(
                batches,
            )));
        }
        if source.exists() && request.force {
            match hybrid_page_ocr_artifact_cache_response(source, output) {
                Ok(Some(response)) => return Ok(Some(response)),
                Ok(None) => {}
                Err(reason) => {
                    log::warn!(
                        "hybrid PDF OCR full artifact cache lookup failed for `{}`: {reason}",
                        source.display()
                    );
                }
            }
        }
        Ok(None)
    }

    async fn direct_docling_structure_recovery_batch(
        &self,
        source: &Path,
        output: &Path,
        phase_elapsed_ms: &mut BTreeMap<String, f64>,
    ) -> Result<HybridDocumentResourceBatch, String> {
        let phase_started = Instant::now();
        let page_count = source_pdf_page_count(source)?;
        record_phase_elapsed(phase_elapsed_ms, "sourcePageCount", phase_started);
        if page_count == 0 {
            return Err("source PDF page tree is empty".to_string());
        }
        let render_report =
            direct_docling_structure_recovery_render_report(source, output, page_count);
        let direct_inputs = direct_docling_structure_recovery_source_inputs(source, page_count)?;
        let fallback_pages =
            docling_structure_recovery_page_range_fallback_pages(&direct_inputs, true);
        let phase_started = Instant::now();
        let resource_batch = if fallback_pages.len() == direct_inputs.len() {
            materialize_docling_page_range_resource_batch(
                self,
                output,
                &render_report,
                &fallback_pages,
                Vec::new(),
                Vec::new(),
                0.0,
            )
            .await
        } else {
            materialize_hybrid_page_ocr_resource_batch_with_eager_docling_fallback(
                &render_report,
                direct_inputs,
                &self.runtime.pdf_ocr_scheduler,
                self,
                output,
                fallback_pages,
            )
            .await
        }?;
        record_ocr_scheduler_or_docling_fallback_phase(
            phase_elapsed_ms,
            &resource_batch,
            phase_started,
        );
        Ok(resource_batch)
    }

    async fn rendered_hybrid_page_ocr_batch(
        &self,
        source: &Path,
        output: &Path,
        phase_elapsed_ms: &mut BTreeMap<String, f64>,
    ) -> Result<RenderedHybridPageOcrBatch, String> {
        let phase_started = Instant::now();
        let render_report = render_hybrid_page_ocr_shards(source, output).await?;
        record_phase_elapsed(phase_elapsed_ms, "renderShardInputs", phase_started);

        let phase_started = Instant::now();
        let ocr_input_path = hybrid_page_ocr_input_arrow_path(&render_report)?;
        let input_batches = read_arrow_file(ocr_input_path.as_path())?;
        let inputs = decode_ocr_shard_input_batches(&input_batches)?;
        if inputs.is_empty() {
            return Err("hybrid PDF OCR route found no OCR shard inputs".to_string());
        }
        record_phase_elapsed(phase_elapsed_ms, "decodeOcrInputs", phase_started);

        let phase_started = Instant::now();
        let inputs = apply_hybrid_page_ocr_profile_plan(inputs);
        record_phase_elapsed(phase_elapsed_ms, "profilePlan", phase_started);

        self.materialize_rendered_hybrid_page_ocr_batch(
            output,
            &render_report,
            inputs,
            phase_elapsed_ms,
        )
        .await
    }

    async fn materialize_rendered_hybrid_page_ocr_batch(
        &self,
        output: &Path,
        render_report: &PdfPageRenderShardReport,
        inputs: Vec<PdfOcrShardInput>,
        phase_elapsed_ms: &mut BTreeMap<String, f64>,
    ) -> Result<RenderedHybridPageOcrBatch, String> {
        if ocr2_region_pipeline_enabled() {
            #[cfg(feature = "document-extract-pdf-render")]
            {
                return self
                    .materialize_rendered_hybrid_page_ocr_region_pipeline(
                        output,
                        render_report,
                        inputs,
                        phase_elapsed_ms,
                    )
                    .await;
            }
            #[cfg(not(feature = "document-extract-pdf-render"))]
            {
                return Err(
                    "hosted VLM/OCR region pipeline requires the `document-extract-pdf-render` feature"
                        .to_string(),
                );
            }
        }
        self.materialize_rendered_hybrid_page_ocr_legacy(
            output,
            render_report,
            inputs,
            phase_elapsed_ms,
        )
        .await
    }

    #[cfg(feature = "document-extract-pdf-render")]
    async fn materialize_rendered_hybrid_page_ocr_region_pipeline(
        &self,
        output: &Path,
        render_report: &PdfPageRenderShardReport,
        inputs: Vec<PdfOcrShardInput>,
        phase_elapsed_ms: &mut BTreeMap<String, f64>,
    ) -> Result<RenderedHybridPageOcrBatch, String> {
        let phase_started = Instant::now();
        let pipeline = materialize_hybrid_page_ocr_resource_batch_with_region_pipeline(
            render_report,
            inputs,
            &self.runtime.pdf_ocr_scheduler,
            self,
            output,
        )
        .await?;
        phase_elapsed_ms.extend(pipeline.phase_elapsed_ms);
        record_phase_elapsed(phase_elapsed_ms, "regionPipeline", phase_started);
        Ok(RenderedHybridPageOcrBatch {
            resource_batch: pipeline.resource_batch,
            ocr2_region_materialization_stats: pipeline.stats,
            ocr_scheduler_trace: pipeline.scheduler_trace,
        })
    }

    async fn materialize_rendered_hybrid_page_ocr_legacy(
        &self,
        output: &Path,
        render_report: &PdfPageRenderShardReport,
        inputs: Vec<PdfOcrShardInput>,
        phase_elapsed_ms: &mut BTreeMap<String, f64>,
    ) -> Result<RenderedHybridPageOcrBatch, String> {
        let phase_started = Instant::now();
        let region_materialization =
            materialize_ocr2_recovery_region_images(render_report, inputs).await?;
        let Ocr2RegionMaterialization {
            inputs,
            stats,
            phase_elapsed_ms: region_phase_elapsed_ms,
        } = region_materialization;
        phase_elapsed_ms.extend(region_phase_elapsed_ms);
        record_phase_elapsed(phase_elapsed_ms, "regionMaterialize", phase_started);

        let phase_started = Instant::now();
        let inputs = materialize_ocr2_recovery_page_images(render_report, inputs).await?;
        record_phase_elapsed(phase_elapsed_ms, "pageMaterialize", phase_started);

        let phase_started = Instant::now();
        let resource_batch = materialize_hybrid_page_ocr_resource_batch(
            render_report,
            inputs,
            &self.runtime.pdf_ocr_scheduler,
            self,
            output,
        )
        .await?;
        record_ocr_scheduler_or_docling_fallback_phase(
            phase_elapsed_ms,
            &resource_batch,
            phase_started,
        );
        Ok(RenderedHybridPageOcrBatch {
            resource_batch,
            ocr2_region_materialization_stats: stats,
            ocr_scheduler_trace: Vec::new(),
        })
    }

    async fn finish_hybrid_page_ocr_document_extract(
        &self,
        finish: HybridPageOcrFinishRequest<'_>,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let phase_started = Instant::now();
        if let Err(reason) = write_hybrid_document_resource_artifacts(
            finish.output,
            finish.source,
            &finish.resource_batch,
        ) {
            return self
                .fallback_python_document_extract(finish.request, finish.output, reason.as_str())
                .await;
        }
        let mut phase_elapsed_ms = finish.phase_elapsed_ms;
        record_phase_elapsed(&mut phase_elapsed_ms, "writeArtifacts", phase_started);
        let total_elapsed_ms = finish.total_started.elapsed().as_secs_f64() * 1000.0;
        phase_elapsed_ms.insert("total".to_string(), total_elapsed_ms);
        write_hybrid_page_ocr_timing_report(
            finish.source,
            finish.output,
            &finish.resource_batch,
            &finish.ocr2_region_materialization_stats,
            &phase_elapsed_ms,
            finish.ocr_scheduler_trace,
            total_elapsed_ms,
        )
        .await;
        tokio::fs::File::create(finish.output.join("_complete.marker"))
            .await
            .map_err(|error| format!("touch hybrid PDF OCR complete marker: {error}"))?;
        if finish.request.force
            && finish.source.exists()
            && let Err(reason) = store_hybrid_page_ocr_artifact_cache(finish.source, finish.output)
        {
            log::warn!(
                "hybrid PDF OCR full artifact cache store failed for `{}`: {reason}",
                finish.source.display()
            );
        }

        Ok(DocumentExtractFlightRouteResponse::new(
            finish.resource_batch.batch,
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
        self.sync_document_extract_batch(
            request.source_path.as_str(),
            output_string.as_str(),
            request.force,
            request.error_row,
            request.profile.as_str(),
        )
        .await
    }
}

fn hybrid_page_ocr_render_failure_requires_fail_fast(reason: &str) -> bool {
    hybrid_page_ocr_render_failure_requires_fail_fast_with_lookup(reason, &|key| {
        std::env::var(key).ok()
    })
}

pub(crate) fn hybrid_page_ocr_render_failure_requires_fail_fast_with_lookup(
    reason: &str,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    env_flag_enabled(lookup(PDF_RENDER_REQUIRE_PDFIUM_ENV).as_deref())
        && is_pdfium_render_failure(reason)
}

fn env_flag_enabled(value: Option<&str>) -> bool {
    matches!(
        value.map(|value| value.trim().to_ascii_lowercase()),
        Some(value)
            if !value.is_empty()
                && value != "0"
                && value != "false"
                && value != "off"
                && value != "disabled"
    )
}

fn is_pdfium_render_failure(reason: &str) -> bool {
    reason.contains("render status `fallback`")
        || reason.contains("Pdfium")
        || reason.contains("PDFium")
        || reason.contains("document-extract-pdf-render")
}

#[cfg(feature = "document-extract-pdf-render")]
#[cfg(any(feature = "document-extract-pdf-render", test))]
#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/provider/hybrid/route/mod.rs"]
mod tests;
