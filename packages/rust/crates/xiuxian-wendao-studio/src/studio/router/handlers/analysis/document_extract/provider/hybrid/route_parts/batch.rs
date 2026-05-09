use super::{
    BTreeMap, BTreeSet, DEFAULT_DOCUMENT_EXTRACT_ENDPOINT, DOCUMENT_EXTRACT_FULL_PROFILE, Duration,
    EngineRecordBatch, HybridDocumentResourceBatch, HybridPdfOcrProfilePlanner, Instant,
    PageRangeDoclingFallbackChunkTiming, PageRangeDoclingFallbackSourceProfileSummary, Path,
    PdfOcrShardInput, PdfOcrShardMetric, PdfOcrShardResult, PdfOcrWorkerScheduler,
    PdfPageRenderShardReport, PdfSourcePageProfile, StudioDocumentExtractFlightRouteProvider,
    build_ocr_result_resource_batch, concat_document_resource_batches,
    docling_page_range_chunk_concurrency_limit_with_lookup,
    docling_page_range_fallback_page_indices,
    docling_page_range_fallback_plan_for_source_with_lookup,
    docling_page_range_hedge_delay_ms_with_lookup, docling_page_range_target_chunk_count,
    docling_structure_recovery_page_range_fallback_pages, failed_backend_text_page_indices,
    has_region_shard_on_pages, has_unhandled_non_success_result, hybrid_page_ocr_profile_planner,
    kept_results_without_docling_page_range_fallback_pages,
    materialize_hybrid_page_ocr_resource_batch_from_results, order_ocr_results_by_inputs,
    pdf_ocr_endpoint_urls, pdf_source_page_is_backend_text_topup_profile,
    pdf_source_page_is_fast_profile_risk, pdf_source_page_requires_structure_authority,
    pdf_source_page_structure_cost, read_arrow_file, recover_failed_page_ocr_results,
    scheduled_inputs_without_docling_page_range_fallback_pages, source_pdf_page_profiles_cached,
};
use crate::studio::router::handlers::analysis::document_extract::provider::transport::{
    document_extract_default_endpoint_with_lookup, document_extract_endpoint_urls_with_lookup,
};
use arrow::array::{Array, Float64Array, StringArray};

const DOCUMENT_TIMING_ARROW_CACHE_NAME: &str = "_document_metrics.arrow";
pub(super) const DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE";
const DOCUMENT_EXTRACT_STRUCTURE_TEXT_PROFILE: &str = "structure-text";

#[derive(Debug, Default)]
struct PageRangeDocumentTiming {
    total_elapsed_ms: Option<f64>,
    phase_elapsed_ms: BTreeMap<String, f64>,
}

impl PageRangeDocumentTiming {
    fn is_empty(&self) -> bool {
        self.total_elapsed_ms.is_none() && self.phase_elapsed_ms.is_empty()
    }
}

pub(super) async fn materialize_hybrid_page_ocr_resource_batch(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
    pdf_ocr_scheduler: &PdfOcrWorkerScheduler,
    provider: &StudioDocumentExtractFlightRouteProvider,
    output: &Path,
) -> Result<HybridDocumentResourceBatch, String> {
    let docling_structure_recovery =
        hybrid_page_ocr_profile_planner() == HybridPdfOcrProfilePlanner::DoclingStructureRecovery;
    let eager_fallback_pages = docling_structure_recovery_page_range_fallback_pages(
        inputs.as_slice(),
        docling_structure_recovery,
    );
    if !eager_fallback_pages.is_empty()
        && !has_region_shard_on_pages(inputs.as_slice(), &eager_fallback_pages)
    {
        return materialize_hybrid_page_ocr_resource_batch_with_eager_docling_fallback(
            render_report,
            inputs,
            pdf_ocr_scheduler,
            provider,
            output,
            eager_fallback_pages,
        )
        .await;
    }

    let (inputs, results, scheduler_elapsed_ms) =
        request_scheduled_ocr_shards(render_report, pdf_ocr_scheduler, inputs).await?;
    if let Some(resource_batch) = materialize_docling_page_range_fallback_batch(
        provider,
        output,
        render_report,
        inputs.as_slice(),
        results.as_slice(),
        scheduler_elapsed_ms,
    )
    .await?
    {
        return Ok(resource_batch);
    }
    materialize_hybrid_page_ocr_resource_batch_from_results(
        render_report,
        inputs,
        results,
        scheduler_elapsed_ms,
    )
}

pub(super) async fn materialize_hybrid_page_ocr_resource_batch_with_eager_docling_fallback(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
    pdf_ocr_scheduler: &PdfOcrWorkerScheduler,
    provider: &StudioDocumentExtractFlightRouteProvider,
    output: &Path,
    fallback_pages: BTreeSet<u32>,
) -> Result<HybridDocumentResourceBatch, String> {
    let scheduled_inputs =
        scheduled_inputs_without_docling_page_range_fallback_pages(inputs, &fallback_pages);
    if scheduled_inputs.is_empty() {
        return materialize_docling_page_range_resource_batch(
            provider,
            output,
            render_report,
            &fallback_pages,
            Vec::new(),
            Vec::new(),
            0.0,
        )
        .await;
    }

    let initial_fallback_pages = fallback_pages.clone();
    let scheduler_future =
        request_scheduled_ocr_shards(render_report, pdf_ocr_scheduler, scheduled_inputs);
    let docling_future = materialize_docling_page_range_resource_batch(
        provider,
        output,
        render_report,
        &initial_fallback_pages,
        Vec::new(),
        Vec::new(),
        0.0,
    );
    let (docling_batch, (scheduled_inputs, scheduled_results, scheduler_elapsed_ms)) =
        tokio::try_join!(docling_future, scheduler_future)?;
    let mut fallback_pages = fallback_pages;
    fallback_pages.extend(failed_backend_text_page_indices(
        scheduled_inputs.as_slice(),
        scheduled_results.as_slice(),
    ));
    let (kept_inputs, kept_results) = kept_results_without_docling_page_range_fallback_pages(
        scheduled_inputs.as_slice(),
        scheduled_results.as_slice(),
        &fallback_pages,
    );
    if has_unhandled_non_success_result(
        scheduled_inputs.as_slice(),
        scheduled_results.as_slice(),
        &fallback_pages,
        true,
    ) {
        return materialize_hybrid_page_ocr_resource_batch_from_results(
            render_report,
            scheduled_inputs,
            scheduled_results,
            scheduler_elapsed_ms,
        );
    }
    if fallback_pages != initial_fallback_pages {
        return materialize_docling_page_range_resource_batch(
            provider,
            output,
            render_report,
            &fallback_pages,
            kept_inputs,
            kept_results,
            scheduler_elapsed_ms,
        )
        .await;
    }
    merge_docling_page_range_batch_with_kept_results(
        docling_batch,
        render_report,
        kept_inputs,
        kept_results,
        scheduler_elapsed_ms,
    )
}

async fn request_scheduled_ocr_shards(
    render_report: &PdfPageRenderShardReport,
    pdf_ocr_scheduler: &PdfOcrWorkerScheduler,
    inputs: Vec<PdfOcrShardInput>,
) -> Result<(Vec<PdfOcrShardInput>, Vec<PdfOcrShardResult>, f64), String> {
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
    Ok((
        inputs,
        results,
        scheduler_started.elapsed().as_secs_f64() * 1000.0,
    ))
}

fn merge_docling_page_range_batch_with_kept_results(
    docling_batch: HybridDocumentResourceBatch,
    render_report: &PdfPageRenderShardReport,
    kept_inputs: Vec<PdfOcrShardInput>,
    kept_results: Vec<PdfOcrShardResult>,
    scheduler_elapsed_ms: f64,
) -> Result<HybridDocumentResourceBatch, String> {
    let HybridDocumentResourceBatch {
        batch: docling_resource_batch,
        page_range_docling_fallback_pages,
        page_range_docling_fallback_chunks,
        page_range_docling_fallback_plan,
        ..
    } = docling_batch;
    let mut resource_batches = Vec::new();
    if !kept_results.is_empty() {
        resource_batches.push(build_ocr_result_resource_batch(kept_results.as_slice())?);
    }
    resource_batches.push(docling_resource_batch);
    let metrics = kept_results
        .iter()
        .zip(kept_inputs.iter())
        .map(|(result, input)| {
            PdfOcrShardMetric::from_ocr_result(
                input,
                result,
                render_report.page_count,
                Some(scheduler_elapsed_ms),
            )
        })
        .collect::<Vec<_>>();
    let resource_batch = concat_document_resource_batches(resource_batches.as_slice())?;
    let mut batch = HybridDocumentResourceBatch::new(
        resource_batch,
        kept_inputs,
        kept_results,
        metrics,
        render_report.page_count,
        page_range_docling_fallback_pages.clone(),
    )
    .with_page_range_docling_fallback_pages(page_range_docling_fallback_pages)
    .with_page_range_docling_fallback_chunks(page_range_docling_fallback_chunks);
    if let Some(plan) = page_range_docling_fallback_plan {
        batch = batch.with_page_range_docling_fallback_plan(plan);
    }
    Ok(batch)
}

pub(super) async fn materialize_docling_page_range_fallback_batch(
    provider: &StudioDocumentExtractFlightRouteProvider,
    output: &Path,
    render_report: &PdfPageRenderShardReport,
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
    scheduler_elapsed_ms: f64,
) -> Result<Option<HybridDocumentResourceBatch>, String> {
    if inputs.len() != results.len() {
        return Ok(None);
    }
    let docling_structure_recovery =
        hybrid_page_ocr_profile_planner() == HybridPdfOcrProfilePlanner::DoclingStructureRecovery;
    let fallback_pages =
        docling_page_range_fallback_page_indices(inputs, results, docling_structure_recovery);
    if fallback_pages.is_empty() {
        return Ok(None);
    }
    if has_unhandled_non_success_result(
        inputs,
        results,
        &fallback_pages,
        docling_structure_recovery,
    ) || has_region_shard_on_pages(inputs, &fallback_pages)
    {
        return Ok(None);
    }

    let (kept_inputs, kept_results) =
        kept_results_without_docling_page_range_fallback_pages(inputs, results, &fallback_pages);

    materialize_docling_page_range_resource_batch(
        provider,
        output,
        render_report,
        &fallback_pages,
        kept_inputs,
        kept_results,
        scheduler_elapsed_ms,
    )
    .await
    .map(Some)
}

pub(super) async fn materialize_docling_page_range_resource_batch(
    provider: &StudioDocumentExtractFlightRouteProvider,
    output: &Path,
    render_report: &PdfPageRenderShardReport,
    fallback_pages: &BTreeSet<u32>,
    kept_inputs: Vec<PdfOcrShardInput>,
    kept_results: Vec<PdfOcrShardResult>,
    scheduler_elapsed_ms: f64,
) -> Result<HybridDocumentResourceBatch, String> {
    let mut resource_batches = Vec::new();
    if !kept_results.is_empty() {
        resource_batches.push(build_ocr_result_resource_batch(kept_results.as_slice())?);
    }
    let endpoint_count = docling_page_range_document_extract_endpoint_count_with_lookup(
        provider.configured_default_endpoint.as_deref(),
        &|key| std::env::var(key).ok(),
    );
    let target_chunk_count = docling_page_range_target_chunk_count(
        hybrid_page_ocr_profile_planner(),
        endpoint_count,
        fallback_pages.len(),
    );
    let (page_ranges, page_range_plan) = docling_page_range_fallback_plan_for_source_with_lookup(
        fallback_pages,
        hybrid_page_ocr_profile_planner(),
        Path::new(render_report.source_path.as_str()),
        target_chunk_count,
        endpoint_count,
        &|key| std::env::var(key).ok(),
    )?;
    let source_profiles =
        source_pdf_page_profiles_cached(Path::new(render_report.source_path.as_str()))
            .unwrap_or_default();
    let mut chunk_timings = Vec::with_capacity(page_ranges.len());
    let chunk_concurrency = docling_page_range_chunk_concurrency_limit_with_lookup(
        page_ranges.len(),
        endpoint_count,
        &|key| std::env::var(key).ok(),
    );
    let hedge_delay_ms =
        docling_page_range_hedge_delay_ms_with_lookup(&|key| std::env::var(key).ok());
    let page_range_profile =
        docling_page_range_fallback_profile_with_lookup(&|key| std::env::var(key).ok());
    for page_range_wave in page_ranges.chunks(chunk_concurrency) {
        let page_range_futures = page_range_wave
            .iter()
            .copied()
            .map(|(start_page, end_page)| {
                let page_range_profile = page_range_profile.clone();
                let source_profile = page_range_source_profile_summary(
                    source_profiles.as_slice(),
                    start_page,
                    end_page,
                );
                async move {
                    let one_based_start = start_page.saturating_add(1);
                    let one_based_end = end_page.saturating_add(1);
                    let page_output = output
                        .join("_docling_page_fallback")
                        .join(format!("pages-{start_page:05}-{end_page:05}"));
                    let page_output_string = page_output.to_string_lossy().to_string();
                    let chunk_started = Instant::now();
                    let (page_batches, hedged, attempt_count) = request_docling_page_range_chunk(
                        provider,
                        render_report.source_path.as_str(),
                        page_output_string.as_str(),
                        one_based_start,
                        one_based_end,
                        page_range_profile.as_str(),
                        hedge_delay_ms,
                    )
                    .await?;
                    let document_timing =
                        page_range_document_timing_for_output(page_output.as_path(), hedged);
                    let timing = PageRangeDoclingFallbackChunkTiming {
                        page_start: start_page,
                        page_end: end_page,
                        one_based_start,
                        one_based_end,
                        elapsed_ms: chunk_started.elapsed().as_secs_f64() * 1000.0,
                        resource_rows: page_batches.iter().map(|batch| batch.num_rows()).sum(),
                        document_extract_profile: page_range_profile,
                        hedged,
                        attempt_count,
                        hedge_delay_ms,
                        document_timing_total_elapsed_ms: document_timing.total_elapsed_ms,
                        document_timing_phase_elapsed_ms: document_timing.phase_elapsed_ms,
                        source_profile,
                    };
                    Ok::<_, String>((page_batches, timing))
                }
            });
        for (mut page_batches, timing) in futures::future::try_join_all(page_range_futures).await? {
            resource_batches.append(&mut page_batches);
            chunk_timings.push(timing);
        }
    }

    let metrics = kept_results
        .iter()
        .zip(kept_inputs.iter())
        .map(|(result, input)| {
            PdfOcrShardMetric::from_ocr_result(
                input,
                result,
                render_report.page_count,
                Some(scheduler_elapsed_ms),
            )
        })
        .collect::<Vec<_>>();
    let resource_batch = concat_document_resource_batches(resource_batches.as_slice())?;
    let fallback_page_indices = fallback_pages.iter().copied().collect::<Vec<_>>();
    Ok(HybridDocumentResourceBatch::new(
        resource_batch,
        kept_inputs,
        kept_results,
        metrics,
        render_report.page_count,
        fallback_page_indices.clone(),
    )
    .with_page_range_docling_fallback_pages(fallback_page_indices)
    .with_page_range_docling_fallback_chunks(chunk_timings)
    .with_page_range_docling_fallback_plan(page_range_plan))
}

fn page_range_source_profile_summary(
    profiles: &[PdfSourcePageProfile],
    start_page: u32,
    end_page: u32,
) -> Option<PageRangeDoclingFallbackSourceProfileSummary> {
    let page_profiles = profiles
        .iter()
        .filter(|profile| profile.page_index >= start_page && profile.page_index <= end_page)
        .collect::<Vec<_>>();
    if page_profiles.is_empty() {
        return None;
    }

    Some(PageRangeDoclingFallbackSourceProfileSummary {
        page_count: page_profiles.len(),
        estimated_weight_total: page_profiles
            .iter()
            .map(|profile| u64::from(profile.estimated_weight))
            .sum(),
        estimated_weight_max: page_profiles
            .iter()
            .map(|profile| profile.estimated_weight)
            .max()
            .unwrap_or(0),
        estimated_structure_cost_total: page_profiles
            .iter()
            .map(|profile| u64::from(pdf_source_page_structure_cost(profile)))
            .sum(),
        estimated_structure_cost_max: page_profiles
            .iter()
            .map(|profile| pdf_source_page_structure_cost(profile))
            .max()
            .unwrap_or(0),
        content_bytes_total: page_profiles
            .iter()
            .map(|profile| u64::from(profile.content_bytes))
            .sum(),
        operation_count_total: page_profiles
            .iter()
            .map(|profile| u64::from(profile.operation_count))
            .sum(),
        text_show_ops_total: page_profiles
            .iter()
            .map(|profile| u64::from(profile.text_show_ops))
            .sum(),
        path_ops_total: page_profiles
            .iter()
            .map(|profile| u64::from(profile.path_ops))
            .sum(),
        rectangle_ops_total: page_profiles
            .iter()
            .map(|profile| u64::from(profile.rectangle_ops))
            .sum(),
        draw_object_ops_total: page_profiles
            .iter()
            .map(|profile| u64::from(profile.draw_object_ops))
            .sum(),
        structure_authority_required_count: page_profiles
            .iter()
            .filter(|profile| pdf_source_page_requires_structure_authority(profile))
            .count(),
        fast_profile_risk_count: page_profiles
            .iter()
            .filter(|profile| pdf_source_page_is_fast_profile_risk(profile))
            .count(),
        backend_text_topup_count: page_profiles
            .iter()
            .filter(|profile| pdf_source_page_is_backend_text_topup_profile(profile))
            .count(),
    })
}

fn page_range_document_timing_for_output(
    page_output: &Path,
    hedged: bool,
) -> PageRangeDocumentTiming {
    if hedged {
        let hedge_output = format!("{}-hedge", page_output.to_string_lossy());
        let hedge_timing = read_page_range_document_timing(Path::new(hedge_output.as_str()));
        if !hedge_timing.is_empty() {
            return hedge_timing;
        }
    }
    read_page_range_document_timing(page_output)
}

pub(super) fn docling_page_range_fallback_profile_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> String {
    match lookup(DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE_ENV)
        .unwrap_or_default()
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .as_str()
    {
        "docling-structure-text" | DOCUMENT_EXTRACT_STRUCTURE_TEXT_PROFILE => {
            DOCUMENT_EXTRACT_STRUCTURE_TEXT_PROFILE.to_string()
        }
        _ => DOCUMENT_EXTRACT_FULL_PROFILE.to_string(),
    }
}

pub(crate) fn docling_page_range_document_extract_endpoint_count_with_lookup(
    configured_default_endpoint: Option<&str>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> usize {
    let default_endpoint =
        document_extract_default_endpoint_with_lookup(configured_default_endpoint, lookup);
    document_extract_endpoint_urls_with_lookup(default_endpoint.as_str(), lookup)
        .len()
        .max(1)
}

fn read_page_range_document_timing(page_output: &Path) -> PageRangeDocumentTiming {
    let timing_path = page_output.join(DOCUMENT_TIMING_ARROW_CACHE_NAME);
    if !timing_path.exists() {
        return PageRangeDocumentTiming::default();
    }
    match read_arrow_file(timing_path.as_path())
        .and_then(|batches| document_timing_from_batches(batches.as_slice()))
    {
        Ok(timing) => timing,
        Err(error) => {
            log::warn!(
                "failed to read Docling page-range timing sidecar `{}`: {error}",
                timing_path.display()
            );
            PageRangeDocumentTiming::default()
        }
    }
}

fn document_timing_from_batches(
    batches: &[EngineRecordBatch],
) -> Result<PageRangeDocumentTiming, String> {
    let mut phase_elapsed_ms = BTreeMap::new();
    for batch in batches {
        let Some(phase_column) = batch.column_by_name("phase") else {
            continue;
        };
        let Some(elapsed_column) = batch.column_by_name("elapsedMs") else {
            continue;
        };
        let Some(phases) = phase_column.as_any().downcast_ref::<StringArray>() else {
            return Err("document timing `phase` column is not utf8".to_string());
        };
        let Some(elapsed_values) = elapsed_column.as_any().downcast_ref::<Float64Array>() else {
            return Err("document timing `elapsedMs` column is not float64".to_string());
        };
        for row in 0..batch.num_rows() {
            if phases.is_null(row) || elapsed_values.is_null(row) {
                continue;
            }
            *phase_elapsed_ms
                .entry(phases.value(row).to_string())
                .or_insert(0.0) += elapsed_values.value(row);
        }
    }
    Ok(PageRangeDocumentTiming {
        total_elapsed_ms: phase_elapsed_ms.get("total").copied(),
        phase_elapsed_ms,
    })
}

async fn request_docling_page_range_chunk(
    provider: &StudioDocumentExtractFlightRouteProvider,
    source_path: &str,
    output_dir: &str,
    one_based_start: u32,
    one_based_end: u32,
    profile: &str,
    hedge_delay_ms: Option<u64>,
) -> Result<(Vec<EngineRecordBatch>, bool, usize), String> {
    let page_range = Some((one_based_start, one_based_end));
    let Some(hedge_delay_ms) = hedge_delay_ms else {
        return provider
            .request_python_document_extract_with_page_range(
                source_path,
                output_dir,
                true,
                true,
                profile,
                page_range,
            )
            .await
            .map(|batches| (batches, false, 1));
    };

    let primary = provider.request_python_document_extract_with_page_range(
        source_path,
        output_dir,
        true,
        true,
        profile,
        page_range,
    );
    tokio::pin!(primary);

    tokio::select! {
        result = &mut primary => return result.map(|batches| (batches, false, 1)),
        () = tokio::time::sleep(Duration::from_millis(hedge_delay_ms)) => {}
    }

    let hedge_output_dir = format!("{output_dir}-hedge");
    let hedge = provider.request_python_document_extract_with_page_range(
        source_path,
        hedge_output_dir.as_str(),
        true,
        true,
        profile,
        page_range,
    );
    tokio::pin!(hedge);

    tokio::select! {
        primary_result = &mut primary => match primary_result {
            Ok(batches) => Ok((batches, false, 2)),
            Err(primary_error) => match hedge.await {
                Ok(batches) => Ok((batches, true, 2)),
                Err(hedge_error) => Err(format!(
                    "Docling page-range primary and hedge requests failed; primary: {primary_error}; hedge: {hedge_error}"
                )),
            },
        },
        hedge_result = &mut hedge => match hedge_result {
            Ok(batches) => Ok((batches, true, 2)),
            Err(hedge_error) => match primary.await {
                Ok(batches) => Ok((batches, false, 2)),
                Err(primary_error) => Err(format!(
                    "Docling page-range hedge and primary requests failed; hedge: {hedge_error}; primary: {primary_error}"
                )),
            },
        },
    }
}
