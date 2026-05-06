use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[cfg(any(feature = "document-extract-pdf-render", test))]
use serde_json::Value;
use serde_json::json;
#[cfg(any(feature = "document-extract-pdf-render", test))]
use sha2::{Digest, Sha256};
use xiuxian_wendao_attachments::pdf::metrics::PdfOcrShardMetric;
#[cfg(any(feature = "document-extract-pdf-render", test))]
use xiuxian_wendao_attachments::pdf::ocr::PDF_OCR_FAST_TEXT_PROFILE;
use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE, PdfOcrShardInput, decode_ocr_shard_input_batches,
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
    render_pdf_page_shards_for_page_indices, render_pdf_region_shards,
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
use super::render::{
    automatic_ocr2_recovery_region_requests_for_source_with_lookup,
    hybrid_page_ocr_input_arrow_path, hybrid_page_ocr_region_requests_for_source_with_lookup,
    hybrid_page_ocr_request_paths, render_hybrid_page_ocr_shards,
};
use super::structure::write_hybrid_document_resource_artifacts;
use super::types::DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV;
use super::types::HybridDocumentResourceBatch;
#[cfg(any(feature = "document-extract-pdf-render", test))]
use super::types::{HybridPdfOcr2ScaffoldMode, hybrid_page_ocr2_scaffold_mode_with_lookup};
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    read_arrow_file, read_cached_document_batches,
};
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
const OCR2_REGION_RENDER_CACHE_DIR_NAME: &str = "ocr2-region-renders";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR_SHARD_MANIFEST_ARROW_NAME: &str = "_ocr_shards.arrow";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR_SHARD_INPUT_ARROW_NAME: &str = "_ocr_input.arrow";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR_PENDING_RESOURCE_ARROW_NAME: &str = "_ocr_pending.arrow";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_REGION_SCAFFOLD_FILE_NAME: &str = "_ocr2_region_scaffolds.json";

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
        let resource_batch = {
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
            let inputs = match materialize_ocr2_recovery_page_images(&render_report, inputs).await {
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
            batch
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

#[derive(Debug, Clone, Copy, Default)]
struct Ocr2RegionMaterializationStats {
    requested_region_count: usize,
    rendered_region_count: usize,
    render_cache_hit_count: usize,
    render_cache_miss_count: usize,
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
                && input.ocr_profile == PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE)
            .count(),
        "ocr2RegionRequestCount": region_materialization_stats.requested_region_count,
        "ocr2RegionRenderedShardCount": region_materialization_stats.rendered_region_count,
        "ocr2RegionRenderCacheHitCount": region_materialization_stats.render_cache_hit_count,
        "ocr2RegionRenderCacheMissCount": region_materialization_stats.render_cache_miss_count,
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
                && input.ocr_profile == PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE
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
        .map_err(|error| format!("join OCR2 recovery page render task: {error}"))??;
        let ocr_input_path = hybrid_page_ocr_input_arrow_path(&page_render_report)?;
        let input_batches = read_arrow_file(ocr_input_path.as_path())?;
        let rendered_inputs = decode_ocr_shard_input_batches(&input_batches)?;
        return merge_ocr2_recovery_page_inputs(inputs, rendered_inputs);
    }

    #[cfg(not(feature = "document-extract-pdf-render"))]
    {
        let _ = render_report;
        Err("OCR2 recovery pages require the `document-extract-pdf-render` feature".to_string())
    }
}

async fn materialize_ocr2_recovery_region_images(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
) -> Result<Ocr2RegionMaterialization, String> {
    let source_path = Path::new(render_report.source_path.as_str()).to_path_buf();
    let mut materialization = Ocr2RegionMaterialization::new(inputs);
    let phase_started = Instant::now();
    let explicit_regions = std::env::var(DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).is_ok();
    let regions = if explicit_regions {
        hybrid_page_ocr_region_requests_for_source_with_lookup(source_path.as_path(), &|key| {
            std::env::var(key).ok()
        })?
    } else {
        if !has_ocr2_recovery_page_candidates(materialization.inputs.as_slice()) {
            materialization.record_phase_elapsed("regionMaterializePlan", phase_started);
            return Ok(materialization);
        }
        automatic_ocr2_recovery_region_requests_for_source_with_lookup(
            source_path.as_path(),
            materialization.inputs.as_slice(),
            &|key| std::env::var(key).ok(),
        )
    };
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
        )?;
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
            .map_err(|error| format!("join OCR2 recovery region render task: {error}"))??
        };
        materialization.record_phase_elapsed("regionMaterializeRender", phase_started);

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
        let merged_inputs =
            merge_ocr2_recovery_region_inputs(existing_inputs, rendered_inputs, &region_pages)?;
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

    #[cfg(not(feature = "document-extract-pdf-render"))]
    {
        let _ = render_report;
        Err("OCR2 recovery regions require the `document-extract-pdf-render` feature".to_string())
    }
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
    .map_err(|error| format!("serialize OCR2 region render cache key: {error}"))?;
    Ok(sha256_hex(payload.as_slice()))
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn cached_ocr2_region_render_report(
    source: &Path,
    output_dir: &Path,
    page_count: u32,
    profile: &PdfPageRenderProfile,
    request_count: usize,
) -> Result<Option<PdfPageRenderShardReport>, String> {
    let manifest_arrow_path = output_dir.join(OCR_SHARD_MANIFEST_ARROW_NAME);
    let ocr_input_arrow_path = output_dir.join(OCR_SHARD_INPUT_ARROW_NAME);
    let pending_resource_arrow_path = output_dir.join(OCR_PENDING_RESOURCE_ARROW_NAME);
    if !manifest_arrow_path.is_file()
        || !ocr_input_arrow_path.is_file()
        || !pending_resource_arrow_path.is_file()
    {
        return Ok(None);
    }

    let input_batches = match read_arrow_file(ocr_input_arrow_path.as_path()) {
        Ok(batches) => batches,
        Err(_) => return Ok(None),
    };
    let inputs = match decode_ocr_shard_input_batches(&input_batches) {
        Ok(inputs) => inputs,
        Err(_) => return Ok(None),
    };
    if inputs.len() != request_count
        || inputs.iter().any(|input| {
            input.shard_type != "region" || !Path::new(input.image_path.as_str()).is_file()
        })
    {
        return Ok(None);
    }

    Ok(Some(PdfPageRenderShardReport {
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
    }))
}

pub(crate) fn has_ocr2_recovery_page_candidates(inputs: &[PdfOcrShardInput]) -> bool {
    inputs.iter().any(|input| {
        input.shard_type == "page" && input.ocr_profile == PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE
    })
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(crate) fn merge_ocr2_recovery_region_inputs(
    mut inputs: Vec<PdfOcrShardInput>,
    rendered_inputs: Vec<PdfOcrShardInput>,
    region_pages: &BTreeSet<u32>,
) -> Result<Vec<PdfOcrShardInput>, String> {
    for input in &mut inputs {
        if input.shard_type == "page"
            && input.ocr_profile == PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE
            && region_pages.contains(&input.page_index)
        {
            input.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
            input.ocr_engine = "docling-fast-text-ocr".to_string();
        }
    }

    let parent_page_shards = inputs
        .iter()
        .filter(|input| input.shard_type == "page")
        .map(|input| (input.page_index, input.shard_element_id.clone()))
        .collect::<BTreeMap<_, _>>();
    for mut input in rendered_inputs {
        if input.shard_type != "region" {
            return Err(format!(
                "OCR2 recovery region render produced non-region shard `{}`",
                input.shard_element_id
            ));
        }
        let parent_shard_element_id = parent_page_shards
            .get(&input.page_index)
            .ok_or_else(|| {
                format!(
                    "OCR2 recovery region `{}` has no parent page shard for page {}",
                    input.shard_element_id, input.page_index
                )
            })?
            .clone();
        input.parent_shard_element_id = parent_shard_element_id;
        input.ocr_profile = PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE.to_string();
        input.ocr_engine = "deepseek-ocr2-direct-vlm".to_string();
        inputs.push(input);
    }
    Ok(inputs)
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn write_ocr2_region_scaffold_sidecar_with_lookup(
    source: &Path,
    output_dir: &Path,
    inputs: &[PdfOcrShardInput],
    explicit_regions: bool,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<(), String> {
    let Some(payload) = ocr2_region_scaffold_payload(source, inputs, explicit_regions, lookup)?
    else {
        return Ok(());
    };
    std::fs::create_dir_all(output_dir)
        .map_err(|error| format!("create OCR2 scaffold output directory: {error}"))?;
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|error| format!("serialize OCR2 region scaffold sidecar: {error}"))?;
    std::fs::write(output_dir.join(OCR2_REGION_SCAFFOLD_FILE_NAME), bytes)
        .map_err(|error| format!("write OCR2 region scaffold sidecar: {error}"))
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
fn sha256_file_hex(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| {
        format!(
            "read OCR2 region render cache source `{}`: {error}",
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
) -> Result<Option<Value>, String> {
    if hybrid_page_ocr2_scaffold_mode_with_lookup(lookup)
        != HybridPdfOcr2ScaffoldMode::RegionTableJson
    {
        return Ok(None);
    }
    let region_inputs = inputs
        .iter()
        .filter(|input| {
            input.shard_type == "region"
                && input.ocr_profile == PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE
        })
        .collect::<Vec<_>>();
    if region_inputs.is_empty() {
        return Ok(None);
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
    Ok(Some(json!({
        "schema": "xiuxian_wendao.ocr2_region_scaffold.v1",
        "mode": "region-table-json",
        "sourcePath": source.to_string_lossy(),
        "items": items,
    })))
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
        if input.shard_type != "page"
            || input.ocr_profile != PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE
        {
            continue;
        }
        let Some(rendered) = rendered_by_page.get(&input.page_index) else {
            return Err(format!(
                "OCR2 recovery render did not produce page {}",
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
    let scheduler_elapsed_ms = scheduler_started.elapsed().as_secs_f64() * 1000.0;
    validate_successful_ocr_results(
        response.results.as_slice(),
        render_report.page_count,
        u32::try_from(inputs.len()).unwrap_or(u32::MAX),
    )?;
    validate_ocr_results_match_inputs(inputs.as_slice(), response.results.as_slice())?;
    let has_region_shards = inputs.iter().any(|input| input.shard_type == "region");

    if render_report.shard_count == render_report.page_count && !has_region_shards {
        validate_hybrid_page_coverage(render_report.page_count, &[], response.results.as_slice())?;
        let metrics = response
            .results
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
            response.resource_batch,
            inputs,
            response.results,
            metrics,
            render_report.page_count,
            Vec::new(),
        ));
    }

    validate_hybrid_shard_coverage(
        render_report.page_count,
        &[],
        inputs.as_slice(),
        response.results.as_slice(),
    )?;
    let metrics = response
        .results
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
        response.resource_batch,
        inputs,
        response.results,
        metrics,
        render_report.page_count,
        Vec::new(),
    ))
}
