//! Hybrid PDF OCR document extraction route.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use arrow::array::{Array, ArrayRef, Int32Array, StringArray};
use arrow::compute::concat_batches;
use arrow::record_batch::RecordBatch as EngineRecordBatch;
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
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_DEFAULT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE,
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE, PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput,
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
    PdfSourcePageProfile, pdf_source_page_is_backend_text_topup_profile,
    pdf_source_page_is_fast_profile_risk, pdf_source_page_requires_structure_authority,
    pdf_source_page_structure_cost, source_pdf_page_profiles_cached,
};
#[cfg(any(feature = "document-extract-pdf-render", test))]
use xiuxian_wendao_attachments::pdf::render::{PdfPageRegionRenderRequest, PdfPageRenderProfile};
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageRenderShardReport, PdfRenderRoutingDecision, PdfRenderStatus, source_pdf_page_count,
};
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::render::{
    page_region_render_request_chunks_all, page_region_render_request_chunks_by_page,
    page_region_render_request_chunks_by_page_area_desc,
    page_region_render_request_chunks_by_page_max_area_desc,
    page_region_render_request_chunks_by_region,
    page_region_render_request_chunks_by_region_seed_page, render_pdf_page_shards_for_page_indices,
    render_pdf_region_shards,
};
use xiuxian_wendao_server::transport::{
    DOCUMENT_EXTRACT_FULL_PROFILE, DocumentExtractFlightRequest,
    DocumentExtractFlightRouteProvider, DocumentExtractFlightRouteResponse,
};

use super::precision_gate::{
    validate_hybrid_page_coverage, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results_for_inputs,
};
use super::profile::{
    HybridPdfOcrProfilePlanner, apply_hybrid_page_ocr_profile_plan,
    hybrid_page_ocr_profile_planner, hybrid_page_ocr_profile_planner_with_lookup,
};
#[cfg(feature = "document-extract-pdf-render")]
use super::render::hybrid_page_ocr_render_profile_with_lookup;
use super::render::{HybridPdfOcr2RegionPlanner, hybrid_page_ocr2_region_planner_with_lookup};
#[cfg(any(feature = "document-extract-pdf-render", test))]
use super::render::{
    automatic_ocr2_recovery_region_requests_for_source_with_lookup,
    hybrid_page_ocr_region_requests_for_source_with_lookup,
};
use super::render::{
    hybrid_page_ocr_input_arrow_path, hybrid_page_ocr_request_paths, render_hybrid_page_ocr_shards,
};
use super::structure::write_hybrid_document_resource_artifacts;
use super::types::{
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, HybridDocumentResourceBatch,
    PageRangeDoclingFallbackChunkTiming, PageRangeDoclingFallbackPlanRange,
    PageRangeDoclingFallbackPlanSummary, PageRangeDoclingFallbackSourceProfileSummary,
};
#[cfg(any(feature = "document-extract-pdf-render", test))]
use super::types::{
    HybridPdfOcr2RegionPipelineMode, HybridPdfOcr2RegionRenderChunkMode, HybridPdfOcr2ScaffoldMode,
    hybrid_page_ocr2_region_pipeline_mode_with_lookup,
    hybrid_page_ocr2_region_render_chunk_mode_with_lookup,
    hybrid_page_ocr2_scaffold_mode_with_lookup,
};
use crate::studio::document_extract_pdf_ocr_client::PdfOcrShardSchedulerTrace;
#[cfg(test)]
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, write_arrow_file,
};
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
#[path = "route_parts/regions.rs"]
mod regions;
#[path = "route_parts/reports.rs"]
mod reports;
#[path = "route_parts/resource_rows.rs"]
mod resource_rows;
#[path = "route_parts/source_inputs.rs"]
mod source_inputs;
#[path = "route_parts/support.rs"]
mod support;

#[cfg(test)]
use artifact_cache::{
    hybrid_page_ocr_artifact_cache_key_for_test, hybrid_page_ocr_artifact_cache_response_for_test,
    store_hybrid_page_ocr_artifact_cache_for_test,
};
use artifact_cache::{
    hybrid_page_ocr_artifact_cache_response, store_hybrid_page_ocr_artifact_cache,
};
#[cfg(test)]
pub(crate) use batch::docling_page_range_document_extract_endpoint_count_with_lookup;
#[cfg(test)]
use batch::{
    DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE_ENV,
    docling_page_range_fallback_profile_with_lookup,
};
use batch::{
    materialize_docling_page_range_resource_batch, materialize_hybrid_page_ocr_resource_batch,
    materialize_hybrid_page_ocr_resource_batch_with_eager_docling_fallback,
};
#[cfg(test)]
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
#[cfg(test)]
use docling_structure_budget::{
    structure_cost_budgeted_docling_page_range_fallback_ranges,
    structure_cost_budgeted_docling_page_range_fallback_ranges_with_limit,
};
use failed_page::recover_failed_page_ocr_results;
#[cfg(test)]
use failed_page::{failed_page_recovery_candidates, failed_page_recovery_input};
#[cfg(feature = "document-extract-pdf-render")]
use pipeline::materialize_hybrid_page_ocr_resource_batch_with_region_pipeline;
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use pipeline::{Ocr2RegionPipelineBatchKind, record_ocr2_region_pipeline_batch_result};
#[cfg(any(feature = "document-extract-pdf-render", test))]
use regions::{
    cached_ocr2_region_render_report, materialize_ocr2_recovery_page_images,
    materialize_ocr2_recovery_region_images, ocr2_recovery_region_requests_for_inputs,
    ocr2_region_render_cache_dir, write_ocr2_region_scaffold_sidecar_with_lookup,
};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(crate) use regions::{has_ocr2_recovery_page_candidates, merge_ocr2_recovery_page_inputs};
#[cfg(test)]
use regions::{ocr2_region_render_cache_key, ocr2_region_scaffold_payload};
#[cfg(test)]
use reports::{
    docling_centered_structure_authority_page_count, page_range_docling_fallback_chunk_summary,
};
use reports::{write_hybrid_page_ocr_fallback_report, write_hybrid_page_ocr_timing_report};
#[cfg(test)]
use resource_rows::normalize_docling_page_range_wrapper_rows;
use resource_rows::{
    concat_document_resource_batches, materialize_hybrid_page_ocr_resource_batch_from_results,
};
use source_inputs::direct_docling_structure_recovery_source_inputs;
#[cfg(test)]
use source_inputs::direct_docling_structure_recovery_source_inputs_for_profiles;
#[cfg(test)]
use source_inputs::direct_docling_structure_recovery_source_inputs_for_profiles_with_lookup;
use support::{
    Ocr2RegionMaterialization, Ocr2RegionMaterializationStats,
    direct_docling_structure_recovery_page_range_enabled,
    direct_docling_structure_recovery_render_report, failed_page_recovery_mode,
    failed_page_recovery_mode_label, ocr2_region_pipeline_mode_label,
    ocr2_region_render_chunk_mode_label, record_ocr_scheduler_or_docling_fallback_phase,
    record_phase_elapsed,
};
#[cfg(test)]
use support::{
    direct_docling_structure_recovery_page_range_enabled_with_lookup,
    failed_page_recovery_mode_with_lookup,
};
#[cfg(feature = "document-extract-pdf-render")]
use support::{
    ocr2_region_pipeline_enabled, ocr2_region_render_ahead_limit_with_lookup,
    ocr2_region_render_request_chunks_with_lookup,
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
        if source.exists() && request.force {
            match hybrid_page_ocr_artifact_cache_response(source.as_path(), output.as_path()) {
                Ok(Some(response)) => return Ok(response),
                Ok(None) => {}
                Err(reason) => {
                    log::warn!(
                        "hybrid PDF OCR full artifact cache lookup failed for `{}`: {reason}",
                        source.display()
                    );
                }
            }
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
            let phase_started = Instant::now();
            let page_count = match source_pdf_page_count(source.as_path()) {
                Ok(page_count) => page_count,
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
            record_phase_elapsed(&mut phase_elapsed_ms, "sourcePageCount", phase_started);
            if page_count == 0 {
                return self
                    .fallback_python_document_extract(
                        request,
                        output.as_path(),
                        "source PDF page tree is empty",
                    )
                    .await;
            }
            let render_report = direct_docling_structure_recovery_render_report(
                source.as_path(),
                output.as_path(),
                page_count,
            );
            let direct_inputs =
                match direct_docling_structure_recovery_source_inputs(source.as_path(), page_count)
                {
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
            let fallback_pages =
                docling_structure_recovery_page_range_fallback_pages(&direct_inputs, true);
            let phase_started = Instant::now();
            let resource_batch = match if fallback_pages.len() == direct_inputs.len() {
                materialize_docling_page_range_resource_batch(
                    self,
                    output.as_path(),
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
                    output.as_path(),
                    fallback_pages,
                )
                .await
            } {
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
            record_ocr_scheduler_or_docling_fallback_phase(
                &mut phase_elapsed_ms,
                &resource_batch,
                phase_started,
            );
            return self
                .finish_hybrid_page_ocr_document_extract(
                    request,
                    source.as_path(),
                    output.as_path(),
                    resource_batch,
                    Ocr2RegionMaterializationStats::default(),
                    phase_elapsed_ms,
                    &[],
                    total_started,
                )
                .await;
        }

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
                            self,
                            output.as_path(),
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
                    self,
                    output.as_path(),
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
                record_ocr_scheduler_or_docling_fallback_phase(
                    &mut phase_elapsed_ms,
                    &batch,
                    phase_started,
                );
                (batch, Vec::new())
            }
        };
        self.finish_hybrid_page_ocr_document_extract(
            request,
            source.as_path(),
            output.as_path(),
            resource_batch,
            ocr2_region_materialization_stats,
            phase_elapsed_ms,
            ocr_scheduler_trace.as_slice(),
            total_started,
        )
        .await
    }

    async fn finish_hybrid_page_ocr_document_extract(
        &self,
        request: &DocumentExtractFlightRequest,
        source: &Path,
        output: &Path,
        resource_batch: HybridDocumentResourceBatch,
        ocr2_region_materialization_stats: Ocr2RegionMaterializationStats,
        mut phase_elapsed_ms: BTreeMap<String, f64>,
        ocr_scheduler_trace: &[PdfOcrShardSchedulerTrace],
        total_started: Instant,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let phase_started = Instant::now();
        if let Err(reason) =
            write_hybrid_document_resource_artifacts(output, source, &resource_batch)
        {
            return self
                .fallback_python_document_extract(request, output, reason.as_str())
                .await;
        }
        record_phase_elapsed(&mut phase_elapsed_ms, "writeArtifacts", phase_started);
        let total_elapsed_ms = total_started.elapsed().as_secs_f64() * 1000.0;
        phase_elapsed_ms.insert("total".to_string(), total_elapsed_ms);
        write_hybrid_page_ocr_timing_report(
            source,
            output,
            &resource_batch,
            &ocr2_region_materialization_stats,
            &phase_elapsed_ms,
            ocr_scheduler_trace,
            total_elapsed_ms,
        )
        .await;
        tokio::fs::File::create(output.join("_complete.marker"))
            .await
            .map_err(|error| format!("touch hybrid PDF OCR complete marker: {error}"))?;
        if request.force
            && source.exists()
            && let Err(reason) = store_hybrid_page_ocr_artifact_cache(source, output)
        {
            log::warn!(
                "hybrid PDF OCR full artifact cache store failed for `{}`: {reason}",
                source.display()
            );
        }

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

#[cfg(feature = "document-extract-pdf-render")]
#[cfg(any(feature = "document-extract-pdf-render", test))]
#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/provider/hybrid/route/mod.rs"]
mod tests;
