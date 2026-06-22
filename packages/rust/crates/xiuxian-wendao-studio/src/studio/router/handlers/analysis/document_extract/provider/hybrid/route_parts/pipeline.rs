#[cfg(feature = "document-extract-pdf-render")]
use super::{
    BTreeMap, BTreeSet, DEFAULT_DOCUMENT_EXTRACT_ENDPOINT, HybridDocumentResourceBatch, Instant,
    Ocr2RegionMaterializationStats, Path, PathBuf, PdfOcrShardInput, PdfOcrShardResult,
    PdfOcrShardSchedulerTrace, PdfOcrWorkerScheduler, PdfPageRegionRenderRequest,
    PdfPageRenderProfile, PdfPageRenderShardReport, PdfRegionShardRenderRequest,
    StudioDocumentExtractFlightRouteProvider, cached_ocr2_region_render_report,
    decode_ocr_shard_input_batches, downgrade_hosted_vlm_region_parent_page_inputs,
    hosted_vlm_region_parent_page_shards, hybrid_page_ocr_input_arrow_path,
    hybrid_page_ocr_render_profile_with_lookup, materialize_hybrid_page_ocr_resource_batch,
    materialize_hybrid_page_ocr_resource_batch_from_results, materialize_ocr2_recovery_page_images,
    ocr2_recovery_region_requests_for_inputs,
    ocr2_region_render_ahead_limit_for_capacity_with_lookup,
    ocr2_region_render_cache_dir_with_source_hash, ocr2_region_render_request_chunks_with_lookup,
    order_ocr_results_by_inputs, pdf_ocr_endpoint_urls, prepare_hosted_vlm_recovery_region_inputs,
    read_arrow_file, record_phase_elapsed, render_pdf_region_shards_with_source_hash,
    sha256_file_hex, write_ocr2_region_scaffold_sidecar_with_lookup,
};
#[cfg(feature = "document-extract-pdf-render")]
use futures::future::{BoxFuture, FutureExt};
#[cfg(feature = "document-extract-pdf-render")]
use futures::stream::{FuturesUnordered, StreamExt};

#[cfg(feature = "document-extract-pdf-render")]
pub(super) struct Ocr2RegionPipelineBatch {
    pub(super) resource_batch: HybridDocumentResourceBatch,
    pub(super) stats: Ocr2RegionMaterializationStats,
    pub(super) phase_elapsed_ms: BTreeMap<String, f64>,
    pub(super) scheduler_trace: Vec<PdfOcrShardSchedulerTrace>,
}

#[cfg(feature = "document-extract-pdf-render")]
#[derive(Debug)]
pub(super) struct Ocr2RegionRenderChunk {
    output_dir: PathBuf,
    render_cache_hit: bool,
    report: PdfPageRenderShardReport,
}

#[cfg(feature = "document-extract-pdf-render")]
#[derive(Debug)]
pub(super) struct ScheduledOcrBatch {
    kind: Ocr2RegionPipelineBatchKind,
    inputs: Vec<PdfOcrShardInput>,
    results: Vec<PdfOcrShardResult>,
    scheduler_trace: Vec<PdfOcrShardSchedulerTrace>,
}

#[cfg(feature = "document-extract-pdf-render")]
struct Ocr2RegionPipelineDrainRequest<'a> {
    source_path: &'a Path,
    source_content_hash: &'a str,
    page_count: u32,
    render_profile: &'a PdfPageRenderProfile,
    region_chunks: &'a [Vec<PdfPageRegionRenderRequest>],
    parent_page_shards: &'a BTreeMap<u32, String>,
    explicit_regions: bool,
    pdf_ocr_scheduler: &'a PdfOcrWorkerScheduler,
    endpoint_urls: &'a [String],
    render_ahead_limit: usize,
}

#[cfg(feature = "document-extract-pdf-render")]
struct Ocr2RegionPipelineDrainState<'a> {
    phase_elapsed_ms: &'a mut BTreeMap<String, f64>,
    stats: &'a mut Ocr2RegionMaterializationStats,
    all_inputs: Vec<PdfOcrShardInput>,
    all_results: Vec<PdfOcrShardResult>,
    scheduler_trace: Vec<PdfOcrShardSchedulerTrace>,
    pending_ocr: FuturesUnordered<BoxFuture<'a, Result<ScheduledOcrBatch, String>>>,
    active_renders:
        FuturesUnordered<tokio::task::JoinHandle<Result<Ocr2RegionRenderChunk, String>>>,
    chunk_index: usize,
}

#[cfg(feature = "document-extract-pdf-render")]
struct Ocr2RegionPipelineDrainOutput {
    all_inputs: Vec<PdfOcrShardInput>,
    all_results: Vec<PdfOcrShardResult>,
    scheduler_trace: Vec<PdfOcrShardSchedulerTrace>,
}

#[cfg(feature = "document-extract-pdf-render")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Ocr2RegionPipelineBatchKind {
    Base,
    Region,
}

#[cfg(feature = "document-extract-pdf-render")]
pub(super) async fn materialize_hybrid_page_ocr_resource_batch_with_region_pipeline(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
    pdf_ocr_scheduler: &PdfOcrWorkerScheduler,
    provider: &StudioDocumentExtractFlightRouteProvider,
    output: &Path,
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
        return materialize_regionless_ocr2_pipeline_batch(
            render_report,
            inputs,
            pdf_ocr_scheduler,
            provider,
            output,
            phase_elapsed_ms,
            stats,
        )
        .await;
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
    let render_ahead_limit = ocr2_region_render_ahead_limit_for_capacity_with_lookup(
        region_chunks.len(),
        endpoint_urls.len(),
        &|key| std::env::var(key).ok(),
    );
    let source_content_hash = sha256_file_hex(source_path.as_path())?;
    stats.pipeline_planned_render_chunk_count = region_chunks.len();
    stats.pipeline_endpoint_count = endpoint_urls.len();
    stats.pipeline_render_ahead_limit = render_ahead_limit;
    let scheduler_started = Instant::now();
    let drain_output = drain_ocr2_region_pipeline(
        Ocr2RegionPipelineDrainRequest {
            source_path: source_path.as_path(),
            source_content_hash: source_content_hash.as_str(),
            page_count: render_report.page_count,
            render_profile: &render_profile,
            region_chunks: region_chunks.as_slice(),
            parent_page_shards: &parent_page_shards,
            explicit_regions,
            pdf_ocr_scheduler,
            endpoint_urls: endpoint_urls.as_slice(),
            render_ahead_limit,
        },
        base_inputs,
        &mut phase_elapsed_ms,
        &mut stats,
        scheduler_started,
    )
    .await?;
    let scheduler_elapsed_ms = scheduler_started.elapsed().as_secs_f64() * 1000.0;
    phase_elapsed_ms.insert("ocrScheduler".to_string(), scheduler_elapsed_ms);
    let mut all_inputs = drain_output.all_inputs;
    all_inputs.sort_by(|left, right| left.reading_order_key.cmp(&right.reading_order_key));
    let resource_batch = materialize_hybrid_page_ocr_resource_batch_from_results(
        render_report,
        all_inputs,
        drain_output.all_results,
        scheduler_elapsed_ms,
    )?;
    Ok(Ocr2RegionPipelineBatch {
        resource_batch,
        stats,
        phase_elapsed_ms,
        scheduler_trace: drain_output.scheduler_trace,
    })
}

#[cfg(feature = "document-extract-pdf-render")]
async fn materialize_regionless_ocr2_pipeline_batch(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
    pdf_ocr_scheduler: &PdfOcrWorkerScheduler,
    provider: &StudioDocumentExtractFlightRouteProvider,
    output: &Path,
    mut phase_elapsed_ms: BTreeMap<String, f64>,
    stats: Ocr2RegionMaterializationStats,
) -> Result<Ocr2RegionPipelineBatch, String> {
    let phase_started = Instant::now();
    let inputs = materialize_ocr2_recovery_page_images(render_report, inputs).await?;
    record_phase_elapsed(&mut phase_elapsed_ms, "pageMaterialize", phase_started);

    let phase_started = Instant::now();
    let resource_batch = materialize_hybrid_page_ocr_resource_batch(
        render_report,
        inputs,
        pdf_ocr_scheduler,
        provider,
        output,
    )
    .await?;
    record_phase_elapsed(&mut phase_elapsed_ms, "ocrScheduler", phase_started);
    Ok(Ocr2RegionPipelineBatch {
        resource_batch,
        stats,
        phase_elapsed_ms,
        scheduler_trace: Vec::new(),
    })
}

#[cfg(feature = "document-extract-pdf-render")]
async fn drain_ocr2_region_pipeline<'a>(
    request: Ocr2RegionPipelineDrainRequest<'a>,
    base_inputs: Vec<PdfOcrShardInput>,
    phase_elapsed_ms: &'a mut BTreeMap<String, f64>,
    materialization_stats: &'a mut Ocr2RegionMaterializationStats,
    scheduler_started: Instant,
) -> Result<Ocr2RegionPipelineDrainOutput, String> {
    let mut drain_state = Ocr2RegionPipelineDrainState {
        phase_elapsed_ms,
        stats: materialization_stats,
        all_inputs: base_inputs.clone(),
        all_results: Vec::new(),
        scheduler_trace: Vec::new(),
        pending_ocr: FuturesUnordered::new(),
        active_renders: FuturesUnordered::new(),
        chunk_index: 0,
    };
    dispatch_base_ocr_batch(&request, &mut drain_state, base_inputs, scheduler_started);
    fill_region_render_queue(&request, &mut drain_state, scheduler_started);
    while !drain_state.active_renders.is_empty() || !drain_state.pending_ocr.is_empty() {
        tokio::select! {
            render_join = drain_state.active_renders.next(), if !drain_state.active_renders.is_empty() => {
                let render_join = render_join
                    .ok_or_else(|| "hosted VLM/OCR region pipeline render queue ended unexpectedly".to_string())?;
                let render_chunk = render_join
                    .map_err(|error| format!("join hosted VLM/OCR region pipeline render task: {error}"))??;
                handle_ready_region_render(&request, &mut drain_state, &render_chunk, scheduler_started)?;
            }
            scheduled = drain_state.pending_ocr.next(), if !drain_state.pending_ocr.is_empty() => {
                let scheduled = scheduled
                    .ok_or_else(|| "hosted VLM/OCR region pipeline request queue ended unexpectedly".to_string())??;
                handle_completed_ocr_batch(&mut drain_state, scheduled, scheduler_started)?;
            }
        }
    }
    Ok(Ocr2RegionPipelineDrainOutput {
        all_inputs: drain_state.all_inputs,
        all_results: drain_state.all_results,
        scheduler_trace: drain_state.scheduler_trace,
    })
}

#[cfg(feature = "document-extract-pdf-render")]
fn dispatch_base_ocr_batch<'a>(
    request: &Ocr2RegionPipelineDrainRequest<'a>,
    state: &mut Ocr2RegionPipelineDrainState<'a>,
    base_inputs: Vec<PdfOcrShardInput>,
    scheduler_started: Instant,
) {
    if base_inputs.is_empty() {
        return;
    }
    state.phase_elapsed_ms.insert(
        "regionPipelineBaseDispatch".to_string(),
        scheduler_started.elapsed().as_secs_f64() * 1000.0,
    );
    state.pending_ocr.push(schedule_ocr_input_batch(
        request.pdf_ocr_scheduler,
        request.endpoint_urls,
        Ocr2RegionPipelineBatchKind::Base,
        base_inputs,
    ));
}

#[cfg(feature = "document-extract-pdf-render")]
fn fill_region_render_queue<'a>(
    request: &Ocr2RegionPipelineDrainRequest<'a>,
    state: &mut Ocr2RegionPipelineDrainState<'a>,
    scheduler_started: Instant,
) {
    let chunk_index_before = state.chunk_index;
    fill_ocr2_region_render_ahead(request, state);
    let spawned_count = state.chunk_index.saturating_sub(chunk_index_before);
    if spawned_count > 0 {
        let spawn_elapsed_ms = scheduler_started.elapsed().as_secs_f64() * 1000.0;
        state.stats.pipeline_render_spawn_count = state
            .stats
            .pipeline_render_spawn_count
            .saturating_add(spawned_count);
        if state.stats.pipeline_render_spawn_count == spawned_count {
            state.phase_elapsed_ms.insert(
                "regionPipelineFirstRenderSpawn".to_string(),
                spawn_elapsed_ms,
            );
        }
        state.phase_elapsed_ms.insert(
            "regionPipelineLastRenderSpawn".to_string(),
            spawn_elapsed_ms,
        );
    }
}

#[cfg(feature = "document-extract-pdf-render")]
fn handle_ready_region_render<'a>(
    request: &Ocr2RegionPipelineDrainRequest<'a>,
    state: &mut Ocr2RegionPipelineDrainState<'a>,
    render_chunk: &Ocr2RegionRenderChunk,
    scheduler_started: Instant,
) -> Result<(), String> {
    let render_ready_elapsed_ms = scheduler_started.elapsed().as_secs_f64() * 1000.0;
    state.stats.pipeline_render_chunk_count =
        state.stats.pipeline_render_chunk_count.saturating_add(1);
    if state.stats.pipeline_render_chunk_count == 1 {
        state.phase_elapsed_ms.insert(
            "regionPipelineFirstRegionReady".to_string(),
            render_ready_elapsed_ms,
        );
    }
    state.phase_elapsed_ms.insert(
        "regionPipelineLastRegionReady".to_string(),
        render_ready_elapsed_ms,
    );
    let region_inputs = decode_ocr2_region_render_chunk(
        request.source_path,
        render_chunk,
        request.parent_page_shards,
        request.explicit_regions,
        state.stats,
    )?;
    dispatch_region_ocr_batch(request, state, region_inputs, scheduler_started);
    fill_region_render_queue(request, state, scheduler_started);
    Ok(())
}

#[cfg(feature = "document-extract-pdf-render")]
fn dispatch_region_ocr_batch<'a>(
    request: &Ocr2RegionPipelineDrainRequest<'a>,
    state: &mut Ocr2RegionPipelineDrainState<'a>,
    region_inputs: Vec<PdfOcrShardInput>,
    scheduler_started: Instant,
) {
    if region_inputs.is_empty() {
        return;
    }
    state.all_inputs.extend(region_inputs.clone());
    let dispatch_elapsed_ms = scheduler_started.elapsed().as_secs_f64() * 1000.0;
    state.stats.pipeline_region_dispatch_count =
        state.stats.pipeline_region_dispatch_count.saturating_add(1);
    if state.stats.pipeline_region_dispatch_count == 1 {
        state.phase_elapsed_ms.insert(
            "regionPipelineFirstRegionDispatch".to_string(),
            dispatch_elapsed_ms,
        );
    }
    state.phase_elapsed_ms.insert(
        "regionPipelineLastRegionDispatch".to_string(),
        dispatch_elapsed_ms,
    );
    state.pending_ocr.push(schedule_ocr_input_batch(
        request.pdf_ocr_scheduler,
        request.endpoint_urls,
        Ocr2RegionPipelineBatchKind::Region,
        region_inputs,
    ));
}

#[cfg(feature = "document-extract-pdf-render")]
fn handle_completed_ocr_batch(
    state: &mut Ocr2RegionPipelineDrainState<'_>,
    scheduled: ScheduledOcrBatch,
    scheduler_started: Instant,
) -> Result<(), String> {
    let completed_elapsed_ms = scheduler_started.elapsed().as_secs_f64() * 1000.0;
    record_ocr2_region_pipeline_batch_result(
        state.phase_elapsed_ms,
        state.stats,
        scheduled.kind,
        scheduled.inputs.len(),
        completed_elapsed_ms,
    );
    collect_scheduled_ocr_batch(
        &mut state.all_results,
        &mut state.scheduler_trace,
        scheduled,
    )
}

#[cfg(feature = "document-extract-pdf-render")]
fn fill_ocr2_region_render_ahead<'a>(
    request: &Ocr2RegionPipelineDrainRequest<'a>,
    state: &mut Ocr2RegionPipelineDrainState<'a>,
) {
    while state.active_renders.len() < request.render_ahead_limit {
        let Some(render) = spawn_next_ocr2_region_render_chunk(
            request.source_path,
            request.source_content_hash,
            request.page_count,
            request.render_profile,
            request.region_chunks,
            &mut state.chunk_index,
        ) else {
            break;
        };
        state.active_renders.push(render);
    }
}

#[cfg(feature = "document-extract-pdf-render")]
pub(super) fn schedule_ocr_input_batch<'a>(
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
pub(super) fn record_ocr2_region_pipeline_batch_result(
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
pub(super) fn collect_scheduled_ocr_batch(
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
pub(super) fn spawn_next_ocr2_region_render_chunk(
    source_path: &Path,
    source_content_hash: &str,
    page_count: u32,
    render_profile: &PdfPageRenderProfile,
    region_chunks: &[Vec<PdfPageRegionRenderRequest>],
    chunk_index: &mut usize,
) -> Option<tokio::task::JoinHandle<Result<Ocr2RegionRenderChunk, String>>> {
    let regions = region_chunks.get(*chunk_index)?.clone();
    *chunk_index = (*chunk_index).saturating_add(1);
    let source_path = source_path.to_path_buf();
    let source_content_hash = source_content_hash.to_string();
    let render_profile = render_profile.clone();
    Some(tokio::spawn(async move {
        let output_dir = ocr2_region_render_cache_dir_with_source_hash(
            source_content_hash.as_str(),
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
            render_pdf_region_shards_with_source_hash(PdfRegionShardRenderRequest {
                path: source_for_render.as_path(),
                output_dir: output_for_render.as_path(),
                profile: &render_profile_for_render,
                regions: regions_for_render.as_slice(),
                source_hash: source_content_hash.as_str(),
            })
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
pub(super) fn decode_ocr2_region_render_chunk(
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
    stats.record_render_artifact_cache_report(&render_chunk.report);
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
