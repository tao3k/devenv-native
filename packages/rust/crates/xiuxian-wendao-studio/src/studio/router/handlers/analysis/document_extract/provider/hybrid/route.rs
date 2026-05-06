use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Instant;

use serde_json::json;
use xiuxian_wendao_attachments::pdf::metrics::PdfOcrShardMetric;
use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE, PDF_OCR_FAST_TEXT_PROFILE, PdfOcrShardInput,
    decode_ocr_shard_input_batches,
};
use xiuxian_wendao_attachments::pdf::render::PdfPageRenderShardReport;
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
use super::render::{
    automatic_ocr2_recovery_region_requests_for_source_with_lookup,
    hybrid_page_ocr_input_arrow_path, hybrid_page_ocr_region_requests_for_source_with_lookup,
    hybrid_page_ocr_render_profile_with_lookup, hybrid_page_ocr_request_paths,
    render_hybrid_page_ocr_shards,
};
use super::structure::write_hybrid_document_resource_artifacts;
use super::types::DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV;
use super::types::HybridDocumentResourceBatch;
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
            let inputs = match materialize_ocr2_recovery_region_images(&render_report, inputs).await
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
) -> Result<Vec<PdfOcrShardInput>, String> {
    let source_path = Path::new(render_report.source_path.as_str()).to_path_buf();
    let regions = if std::env::var(DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).is_ok() {
        hybrid_page_ocr_region_requests_for_source_with_lookup(source_path.as_path(), &|key| {
            std::env::var(key).ok()
        })?
    } else {
        automatic_ocr2_recovery_region_requests_for_source_with_lookup(
            source_path.as_path(),
            inputs.as_slice(),
            &|key| std::env::var(key).ok(),
        )
    };
    if regions.is_empty() {
        return Ok(inputs);
    }

    #[cfg(feature = "document-extract-pdf-render")]
    {
        let output_dir = Path::new(render_report.output_dir.as_str()).join("_ocr2-region-renders");
        let region_pages = regions
            .iter()
            .map(|region| region.page_index)
            .collect::<BTreeSet<_>>();
        let render_profile =
            hybrid_page_ocr_render_profile_with_lookup(true, &|key| std::env::var(key).ok());
        let region_render_report = tokio::task::spawn_blocking(move || {
            render_pdf_region_shards(
                source_path.as_path(),
                output_dir.as_path(),
                &render_profile,
                regions.as_slice(),
            )
        })
        .await
        .map_err(|error| format!("join OCR2 recovery region render task: {error}"))??;
        let ocr_input_path = hybrid_page_ocr_input_arrow_path(&region_render_report)?;
        let input_batches = read_arrow_file(ocr_input_path.as_path())?;
        let rendered_inputs = decode_ocr_shard_input_batches(&input_batches)?;
        return merge_ocr2_recovery_region_inputs(inputs, rendered_inputs, &region_pages);
    }

    #[cfg(not(feature = "document-extract-pdf-render"))]
    {
        let _ = render_report;
        Err("OCR2 recovery regions require the `document-extract-pdf-render` feature".to_string())
    }
}

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
