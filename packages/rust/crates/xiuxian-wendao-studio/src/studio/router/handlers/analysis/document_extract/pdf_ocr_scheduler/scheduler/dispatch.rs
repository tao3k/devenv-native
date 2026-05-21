use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::future::try_join_all;
use tokio::sync::OwnedSemaphorePermit;
use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_FAST_TEXT_PROFILE, PdfOcrShardInput, PdfOcrShardResult, build_ocr_result_resource_batch,
};

use super::core::PdfOcrWorkerScheduler;
use super::limit::{
    duration_to_ms, rendered_region_shard_chunks, source_pdf_page_range_dispatch_budget,
    source_pdf_page_range_dispatch_chunks,
};
use super::local_text::local_backend_text_results;
use crate::studio::document_extract_pdf_ocr_client::{
    PdfOcrShardFlightClient, PdfOcrShardFlightResponse, PdfOcrShardSchedulerTrace,
};
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_cache::ocr_shard_cache_key;
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_order::order_ocr_results_by_inputs;
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::capacity::{
    OcrSchedulerLane, classify_ocr_lane, source_range_override_from_environment,
};
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::inflight::{
    InFlightShardEntry, InFlightShardReservation,
};

#[derive(Debug)]
struct OwnedShardRequest {
    position: usize,
    key: String,
    entry: Arc<InFlightShardEntry>,
    input: PdfOcrShardInput,
}

#[derive(Debug)]
struct SchedulerLiveShardResponse {
    results: Vec<PdfOcrShardResult>,
    trace: Vec<PdfOcrShardSchedulerTrace>,
}

const FAST_TEXT_ENDPOINT_AFFINITY_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY";
const FAST_TEXT_ENDPOINT_AFFINITY_SINGLE_PAGE_FIRST: &str = "single-page-first";

#[derive(Debug)]
pub(crate) struct SchedulerShardGroup {
    pub(super) positions: Vec<usize>,
    pub(super) inputs: Vec<PdfOcrShardInput>,
}

impl PdfOcrWorkerScheduler {
    pub(crate) async fn request_shards_with_endpoints(
        &self,
        endpoint_urls: &[String],
        inputs: &[PdfOcrShardInput],
    ) -> Result<PdfOcrShardFlightResponse, String> {
        if inputs.is_empty() {
            return Err("PDF OCR shard request inputs cannot be empty".to_string());
        }
        if endpoint_urls.is_empty() {
            return Err("PDF OCR shard endpoint pool cannot be empty".to_string());
        }
        let cache_resolution = self.cache.resolve(inputs);
        let cache_hit_count = cache_resolution.hit_count();
        let misses = cache_resolution.misses().to_vec();
        self.metrics
            .record_cache_resolution(cache_hit_count, misses.len());
        if cache_hit_count > 0 {
            log::debug!(
                "PDF OCR shard cache hits: {cache_hit_count}, misses: {}",
                misses.len()
            );
        }
        let live_response = if misses.is_empty() {
            SchedulerLiveShardResponse {
                results: Vec::new(),
                trace: Vec::new(),
            }
        } else {
            self.request_missed_shards(endpoint_urls, misses.as_slice())
                .await?
        };

        let results = cache_resolution.merge(live_response.results)?;
        let resource_batch = build_ocr_result_resource_batch(results.as_slice())?;
        Ok(PdfOcrShardFlightResponse {
            results,
            resource_batch,
            scheduler_trace: live_response.trace,
        })
    }

    async fn request_missed_shards(
        &self,
        endpoint_urls: &[String],
        inputs: &[PdfOcrShardInput],
    ) -> Result<SchedulerLiveShardResponse, String> {
        let local_started = Instant::now();
        let mut slots = local_backend_text_results(inputs);
        let local_latency = local_started.elapsed();
        let mut trace = scheduler_trace_for_local_results(inputs, slots.as_slice(), local_latency);
        self.store_local_backend_text_results(inputs, slots.as_slice());
        let (owner_requests, waiters) = self.reserve_unresolved_shards(inputs, slots.as_slice());

        if !owner_requests.is_empty() {
            trace.extend(
                self.handle_owned_shard_requests(endpoint_urls, &mut slots, owner_requests)
                    .await?,
            );
            await_waiter_shards(&mut slots, waiters).await?;
            let results = resolve_ocr_shard_slots(slots)?;
            return Ok(SchedulerLiveShardResponse { results, trace });
        }

        await_waiter_shards(&mut slots, waiters).await?;
        let results = resolve_ocr_shard_slots(slots)?;
        Ok(SchedulerLiveShardResponse { results, trace })
    }

    fn store_local_backend_text_results(
        &self,
        inputs: &[PdfOcrShardInput],
        slots: &[Option<PdfOcrShardResult>],
    ) {
        for (position, result) in slots.iter().enumerate() {
            let Some(result) = result else {
                continue;
            };
            if let Err(error) = self.cache.store_successful(&inputs[position], result) {
                log::warn!("failed to store local backend-text OCR shard cache row: {error}");
            }
        }
    }

    fn reserve_unresolved_shards(
        &self,
        inputs: &[PdfOcrShardInput],
        slots: &[Option<PdfOcrShardResult>],
    ) -> (
        Vec<OwnedShardRequest>,
        Vec<(usize, Arc<InFlightShardEntry>)>,
    ) {
        let mut owner_requests = Vec::new();
        let mut waiters = Vec::new();
        for (position, input) in unresolved_shard_inputs(inputs, slots) {
            match self.inflight.reserve(ocr_shard_cache_key(input)) {
                InFlightShardReservation::Owner { key, entry } => {
                    owner_requests.push(OwnedShardRequest {
                        position,
                        key,
                        entry,
                        input: input.clone(),
                    });
                }
                InFlightShardReservation::Waiter { entry } => {
                    waiters.push((position, entry));
                }
            }
        }
        (owner_requests, waiters)
    }

    async fn handle_owned_shard_requests(
        &self,
        endpoint_urls: &[String],
        slots: &mut [Option<PdfOcrShardResult>],
        owner_requests: Vec<OwnedShardRequest>,
    ) -> Result<Vec<PdfOcrShardSchedulerTrace>, String> {
        let clients = connect_pdf_ocr_clients(endpoint_urls).await?;
        let owner_inputs = owner_requests
            .iter()
            .map(|request| request.input.clone())
            .collect::<Vec<_>>();
        let live_response = match self
            .request_uncached_shards(clients.as_slice(), owner_inputs.as_slice())
            .await
        {
            Ok(response) => {
                let results =
                    order_ocr_results_by_inputs(owner_inputs.as_slice(), response.results)?;
                Ok::<_, String>(SchedulerLiveShardResponse {
                    results,
                    trace: response.trace,
                })
            }
            Err(error) => {
                for request in owner_requests {
                    self.inflight
                        .publish(request.key.as_str(), &request.entry, Err(error.clone()));
                }
                return Err(error);
            }
        }?;

        for (request, result) in owner_requests.iter().zip(live_response.results) {
            if let Err(error) = self.cache.store_successful(&request.input, &result) {
                log::warn!("failed to store PDF OCR shard cache row: {error}");
            }
            self.inflight
                .publish(request.key.as_str(), &request.entry, Ok(result.clone()));
            slots[request.position] = Some(result);
        }
        Ok(live_response.trace)
    }

    async fn request_uncached_shards(
        &self,
        clients: &[PdfOcrShardFlightClient],
        inputs: &[PdfOcrShardInput],
    ) -> Result<SchedulerLiveShardResponse, String> {
        let groups = scheduler_shard_groups(inputs);
        if groups.len() > 1 {
            return self
                .request_partitioned_uncached_shards(clients, groups)
                .await;
        }
        self.request_uncached_shard_group(clients, inputs).await
    }

    async fn request_partitioned_uncached_shards(
        &self,
        clients: &[PdfOcrShardFlightClient],
        groups: Vec<SchedulerShardGroup>,
    ) -> Result<SchedulerLiveShardResponse, String> {
        let input_count = groups
            .iter()
            .map(|group| group.positions.len())
            .sum::<usize>();
        let mut requests = Vec::with_capacity(groups.len());
        for group in groups {
            let clients = clients.to_vec();
            requests.push(async move {
                let response = self
                    .request_uncached_shard_group(clients.as_slice(), group.inputs.as_slice())
                    .await?;
                Ok::<_, String>((group.positions, response.results, response.trace))
            });
        }
        let group_results = try_join_all(requests).await?;
        let mut slots = vec![None; input_count];
        let mut trace = Vec::new();
        for (positions, results, group_trace) in group_results {
            if positions.len() != results.len() {
                return Err(format!(
                    "PDF OCR scheduler group returned {} rows for {} inputs",
                    results.len(),
                    positions.len()
                ));
            }
            for (position, result) in positions.into_iter().zip(results) {
                let Some(slot) = slots.get_mut(position) else {
                    return Err(format!(
                        "PDF OCR scheduler group result position {position} exceeded input count {input_count}"
                    ));
                };
                *slot = Some(result);
            }
            trace.extend(group_trace);
        }
        let results = slots
            .into_iter()
            .enumerate()
            .map(|(position, result)| {
                result.ok_or_else(|| {
                    format!(
                        "PDF OCR scheduler group merge left input position {position} unresolved"
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SchedulerLiveShardResponse { results, trace })
    }

    async fn request_uncached_shard_group(
        &self,
        clients: &[PdfOcrShardFlightClient],
        inputs: &[PdfOcrShardInput],
    ) -> Result<SchedulerLiveShardResponse, String> {
        if clients.is_empty() {
            return Err("PDF OCR shard endpoint pool cannot be empty".to_string());
        }
        let lane = classify_ocr_lane(inputs);
        if lane == OcrSchedulerLane::SourcePdfPageRange {
            return self
                .request_source_pdf_page_range_shards(clients, inputs)
                .await;
        }
        if lane == OcrSchedulerLane::RenderedRegion {
            return self.request_rendered_region_shards(clients, inputs).await;
        }
        let mut results = Vec::with_capacity(inputs.len());
        let mut trace = Vec::new();
        let mut offset = 0;
        while offset < inputs.len() {
            let remaining = inputs.len() - offset;
            let target = self.capacity.budget_for_lane(
                remaining,
                lane,
                source_range_override_from_environment(),
            );
            let (permits, queue_wait) = self.acquire_worker_permits(target).await?;
            self.metrics.record_queue_wait(queue_wait);
            let worker_budget = permits.len().clamp(1, remaining);
            let end = offset + worker_budget;
            let latency_start = Instant::now();
            self.metrics
                .record_live_request(lane, inputs[offset..end].len());
            let client = self.endpoint_client_for_next_request(clients)?;
            let response = client
                .request_with_worker_budget(&inputs[offset..end], Some(worker_budget))
                .await;
            let latency = latency_start.elapsed();
            self.metrics.record_ocr_latency(latency);
            drop(permits);

            let response = match response {
                Ok(response) => {
                    self.capacity
                        .record_success(duration_to_ms(queue_wait), duration_to_ms(latency));
                    response
                }
                Err(error) => {
                    self.capacity.record_failure();
                    return Err(error);
                }
            };
            let ordered = order_ocr_results_by_inputs(&inputs[offset..end], response.results)?;
            trace.push(scheduler_trace_for_chunk_with_timing(
                lane,
                &inputs[offset..end],
                ordered.as_slice(),
                latency,
                Some(duration_to_trace_ms(queue_wait)),
                Some(0.0),
                Some(duration_to_trace_ms(latency)),
            ));
            results.extend(ordered);
            offset = end;
        }
        Ok(SchedulerLiveShardResponse { results, trace })
    }

    async fn request_rendered_region_shards(
        &self,
        clients: &[PdfOcrShardFlightClient],
        inputs: &[PdfOcrShardInput],
    ) -> Result<SchedulerLiveShardResponse, String> {
        let lane = OcrSchedulerLane::RenderedRegion;
        let target = self.capacity.budget_for_lane(
            inputs.len(),
            lane,
            source_range_override_from_environment(),
        );
        let (permits, queue_wait) = self.acquire_worker_permits(target).await?;
        self.metrics.record_queue_wait(queue_wait);
        let max_parallel_chunks = permits.len().clamp(1, inputs.len());
        let chunks = rendered_region_shard_chunks(inputs);
        self.metrics.record_live_request(lane, inputs.len());
        let latency_start = Instant::now();
        let mut chunk_results = Vec::with_capacity(chunks.len());
        let mut trace = Vec::new();
        let mut chunk_error = None;
        for wave in chunks.chunks(max_parallel_chunks) {
            let mut wave_requests = Vec::with_capacity(wave.len());
            for chunk in wave {
                let chunk = *chunk;
                let client = self.endpoint_client_for_next_request(clients).cloned();
                wave_requests.push(async move {
                    let client = client?;
                    let dispatch_start_ms = duration_to_trace_ms(latency_start.elapsed());
                    let chunk_started = Instant::now();
                    let response = client.request_with_worker_budget(chunk, Some(1)).await?;
                    let latency = chunk_started.elapsed();
                    let ordered = order_ocr_results_by_inputs(chunk, response.results)?;
                    let dispatch_end_ms = dispatch_start_ms + duration_to_trace_ms(latency);
                    Ok::<_, String>((
                        ordered.clone(),
                        scheduler_trace_for_chunk_with_timing(
                            lane,
                            chunk,
                            ordered.as_slice(),
                            latency,
                            Some(duration_to_trace_ms(queue_wait)),
                            Some(dispatch_start_ms),
                            Some(dispatch_end_ms),
                        ),
                    ))
                });
            }
            match try_join_all(wave_requests).await {
                Ok(wave_results) => {
                    for (results, chunk_trace) in wave_results {
                        chunk_results.push(results);
                        trace.push(chunk_trace);
                    }
                }
                Err(error) => {
                    chunk_error = Some(error);
                    break;
                }
            }
        }
        let latency = latency_start.elapsed();
        self.metrics.record_ocr_latency(latency);
        drop(permits);

        if let Some(error) = chunk_error {
            self.capacity.record_failure();
            return Err(error);
        }
        self.capacity
            .record_success(duration_to_ms(queue_wait), duration_to_ms(latency));

        let mut results = Vec::with_capacity(inputs.len());
        for chunk in chunk_results {
            results.extend(chunk);
        }
        let results = order_ocr_results_by_inputs(inputs, results)?;
        Ok(SchedulerLiveShardResponse { results, trace })
    }

    async fn request_source_pdf_page_range_shards(
        &self,
        clients: &[PdfOcrShardFlightClient],
        inputs: &[PdfOcrShardInput],
    ) -> Result<SchedulerLiveShardResponse, String> {
        let lane = OcrSchedulerLane::SourcePdfPageRange;
        let target = self.capacity.budget_for_lane(
            inputs.len(),
            lane,
            source_range_override_from_environment(),
        );
        let target = source_pdf_page_range_dispatch_budget(inputs, target);
        let (permits, queue_wait) = self.acquire_worker_permits(target).await?;
        self.metrics.record_queue_wait(queue_wait);
        let max_parallel_chunks = permits.len().clamp(1, inputs.len());
        let chunks = source_pdf_page_range_dispatch_chunks(inputs, max_parallel_chunks);
        self.metrics.record_live_request(lane, inputs.len());
        let latency_start = Instant::now();
        let mut chunk_results = Vec::with_capacity(chunks.len());
        let mut trace = Vec::new();
        let mut chunk_error = None;
        for wave in chunks.chunks(max_parallel_chunks) {
            let mut wave_requests = Vec::with_capacity(wave.len());
            for chunk in wave {
                let chunk = *chunk;
                let client = self
                    .endpoint_client_for_source_pdf_page_range_chunk(clients, chunk)
                    .cloned();
                wave_requests.push(async move {
                    let client = client?;
                    let dispatch_start_ms = duration_to_trace_ms(latency_start.elapsed());
                    let chunk_started = Instant::now();
                    let response = client.request_with_worker_budget(chunk, Some(1)).await?;
                    let latency = chunk_started.elapsed();
                    let ordered = order_ocr_results_by_inputs(chunk, response.results)?;
                    let dispatch_end_ms = dispatch_start_ms + duration_to_trace_ms(latency);
                    Ok::<_, String>((
                        ordered.clone(),
                        scheduler_trace_for_chunk_with_timing(
                            lane,
                            chunk,
                            ordered.as_slice(),
                            latency,
                            Some(duration_to_trace_ms(queue_wait)),
                            Some(dispatch_start_ms),
                            Some(dispatch_end_ms),
                        ),
                    ))
                });
            }
            match try_join_all(wave_requests).await {
                Ok(wave_results) => {
                    for (results, chunk_trace) in wave_results {
                        chunk_results.push(results);
                        trace.push(chunk_trace);
                    }
                }
                Err(error) => {
                    chunk_error = Some(error);
                    break;
                }
            }
        }
        let latency = latency_start.elapsed();
        self.metrics.record_ocr_latency(latency);
        drop(permits);

        if let Some(error) = chunk_error {
            self.capacity.record_failure();
            return Err(error);
        }
        self.capacity
            .record_success(duration_to_ms(queue_wait), duration_to_ms(latency));

        let mut results = Vec::with_capacity(inputs.len());
        for chunk in chunk_results {
            results.extend(chunk);
        }
        let results = order_ocr_results_by_inputs(inputs, results)?;
        Ok(SchedulerLiveShardResponse { results, trace })
    }

    pub(super) async fn acquire_worker_permits(
        &self,
        target: usize,
    ) -> Result<(Vec<OwnedSemaphorePermit>, Duration), String> {
        let target = target.max(1).min(self.worker_limit);
        let wait_started = Instant::now();
        let first = Arc::clone(&self.permits)
            .acquire_owned()
            .await
            .map_err(|error| format!("acquire PDF OCR worker permit: {error}"))?;
        let queue_wait = wait_started.elapsed();
        let mut permits = vec![first];
        while permits.len() < target {
            match Arc::clone(&self.permits).try_acquire_owned() {
                Ok(permit) => permits.push(permit),
                Err(_) => break,
            }
        }
        Ok((permits, queue_wait))
    }

    pub(super) fn endpoint_client_for_next_request<'a>(
        &self,
        clients: &'a [PdfOcrShardFlightClient],
    ) -> Result<&'a PdfOcrShardFlightClient, String> {
        let endpoint_index = self.endpoint_index_for_next_request(clients.len())?;
        clients
            .get(endpoint_index)
            .ok_or_else(|| "PDF OCR shard endpoint pool cannot be empty".to_string())
    }

    fn endpoint_client_for_source_pdf_page_range_chunk<'a>(
        &self,
        clients: &'a [PdfOcrShardFlightClient],
        chunk: &[PdfOcrShardInput],
    ) -> Result<&'a PdfOcrShardFlightClient, String> {
        let endpoint_index =
            self.endpoint_index_for_source_pdf_page_range_chunk(clients.len(), chunk)?;
        clients
            .get(endpoint_index)
            .ok_or_else(|| "PDF OCR shard endpoint pool cannot be empty".to_string())
    }

    pub(super) fn endpoint_index_for_source_pdf_page_range_chunk(
        &self,
        endpoint_count: usize,
        chunk: &[PdfOcrShardInput],
    ) -> Result<usize, String> {
        source_pdf_page_range_chunk_endpoint_index_with_lookup(
            endpoint_count,
            chunk,
            &|key| std::env::var(key).ok(),
            || self.endpoint_index_for_next_request(endpoint_count),
        )
    }
}

fn unresolved_shard_inputs<'a>(
    inputs: &'a [PdfOcrShardInput],
    slots: &'a [Option<PdfOcrShardResult>],
) -> impl Iterator<Item = (usize, &'a PdfOcrShardInput)> {
    inputs
        .iter()
        .enumerate()
        .filter(|(position, _)| slots[*position].is_none())
}

async fn await_waiter_shards(
    slots: &mut [Option<PdfOcrShardResult>],
    waiters: Vec<(usize, Arc<InFlightShardEntry>)>,
) -> Result<(), String> {
    for (position, entry) in waiters {
        slots[position] = Some(entry.wait().await?);
    }
    Ok(())
}

fn resolve_ocr_shard_slots(
    slots: Vec<Option<PdfOcrShardResult>>,
) -> Result<Vec<PdfOcrShardResult>, String> {
    slots
        .into_iter()
        .enumerate()
        .map(|(position, result)| {
            result.ok_or_else(|| {
                format!("PDF OCR in-flight merge left input position {position} unresolved")
            })
        })
        .collect()
}

pub(crate) fn source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(
    chunk: &[PdfOcrShardInput],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    fast_text_endpoint_affinity_single_page_first_enabled(lookup)
        && is_single_fast_text_source_pdf_page_range_chunk(chunk)
}

pub(crate) fn source_pdf_page_range_chunk_endpoint_index_with_lookup(
    endpoint_count: usize,
    chunk: &[PdfOcrShardInput],
    lookup: &dyn Fn(&str) -> Option<String>,
    next_endpoint_index: impl FnOnce() -> Result<usize, String>,
) -> Result<usize, String> {
    if source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(chunk, lookup) {
        if endpoint_count == 0 {
            return Err("PDF OCR shard endpoint pool cannot be empty".to_string());
        }
        return Ok(0);
    }
    next_endpoint_index()
}

fn fast_text_endpoint_affinity_single_page_first_enabled(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    lookup(FAST_TEXT_ENDPOINT_AFFINITY_ENV).is_some_and(|value| {
        value.trim().replace('_', "-").to_ascii_lowercase()
            == FAST_TEXT_ENDPOINT_AFFINITY_SINGLE_PAGE_FIRST
    })
}

fn is_single_fast_text_source_pdf_page_range_chunk(chunk: &[PdfOcrShardInput]) -> bool {
    let [input] = chunk else {
        return false;
    };
    input.ocr_profile == PDF_OCR_FAST_TEXT_PROFILE
        && input.shard_type == "page"
        && input.source_path.to_ascii_lowercase().ends_with(".pdf")
}

pub(crate) fn scheduler_shard_groups(inputs: &[PdfOcrShardInput]) -> Vec<SchedulerShardGroup> {
    inputs
        .iter()
        .enumerate()
        .collect::<Vec<_>>()
        .chunk_by(|(_, previous), (_, current)| {
            classify_ocr_lane(std::slice::from_ref(previous))
                == classify_ocr_lane(std::slice::from_ref(current))
        })
        .map(|chunk| SchedulerShardGroup {
            positions: chunk.iter().map(|(position, _)| *position).collect(),
            inputs: chunk.iter().map(|(_, input)| (*input).clone()).collect(),
        })
        .collect()
}

fn scheduler_trace_for_local_results(
    inputs: &[PdfOcrShardInput],
    results: &[Option<PdfOcrShardResult>],
    latency: Duration,
) -> Vec<PdfOcrShardSchedulerTrace> {
    let mut traces = Vec::new();
    let mut run_start = None;
    for index in 0..results.len() {
        if results[index].is_none() {
            if let Some(start) = run_start.take() {
                push_local_trace_run(&mut traces, inputs, results, start, index, latency);
            }
            continue;
        }
        let Some(start) = run_start else {
            run_start = Some(index);
            continue;
        };
        let previous = index.saturating_sub(1);
        if inputs[index].ocr_profile != inputs[previous].ocr_profile {
            push_local_trace_run(&mut traces, inputs, results, start, index, latency);
            run_start = Some(index);
        }
    }
    if let Some(start) = run_start {
        push_local_trace_run(&mut traces, inputs, results, start, results.len(), latency);
    }
    traces
}

fn push_local_trace_run(
    traces: &mut Vec<PdfOcrShardSchedulerTrace>,
    inputs: &[PdfOcrShardInput],
    results: &[Option<PdfOcrShardResult>],
    start: usize,
    end: usize,
    latency: Duration,
) {
    let chunk_results = results[start..end]
        .iter()
        .filter_map(Clone::clone)
        .collect::<Vec<_>>();
    if chunk_results.len() != end.saturating_sub(start) {
        return;
    }
    traces.push(scheduler_trace_for_chunk(
        OcrSchedulerLane::SourcePdfPageRange,
        &inputs[start..end],
        chunk_results.as_slice(),
        latency,
    ));
}

pub(super) fn scheduler_trace_for_chunk(
    lane: OcrSchedulerLane,
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
    latency: Duration,
) -> PdfOcrShardSchedulerTrace {
    scheduler_trace_for_chunk_with_timing(lane, inputs, results, latency, None, None, None)
}

fn scheduler_trace_for_chunk_with_timing(
    lane: OcrSchedulerLane,
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
    latency: Duration,
    queue_wait_ms: Option<f64>,
    dispatch_start_ms: Option<f64>,
    dispatch_end_ms: Option<f64>,
) -> PdfOcrShardSchedulerTrace {
    PdfOcrShardSchedulerTrace {
        lane: scheduler_lane_label(lane),
        shard_count: inputs.len(),
        page_start: inputs.iter().map(|input| input.page_index).min(),
        page_end: inputs.iter().map(|input| input.page_index).max(),
        shard_type: inputs.first().map(|input| input.shard_type.clone().into()),
        ocr_profile: inputs.first().map(|input| input.ocr_profile.clone()),
        queue_wait_ms,
        dispatch_start_ms,
        dispatch_end_ms,
        latency_ms: latency.as_secs_f64() * 1000.0,
        text_char_count: results
            .iter()
            .filter_map(|result| result.text.as_deref())
            .map(|text| text.chars().count())
            .sum(),
    }
}

fn duration_to_trace_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn scheduler_lane_label(lane: OcrSchedulerLane) -> &'static str {
    match lane {
        OcrSchedulerLane::SourcePdfPageRange => "source-pdf-page-range",
        OcrSchedulerLane::RenderedPage => "rendered-page",
        OcrSchedulerLane::RenderedRegion => "rendered-region",
    }
}

async fn connect_pdf_ocr_clients(
    endpoint_urls: &[String],
) -> Result<Vec<PdfOcrShardFlightClient>, String> {
    if endpoint_urls.is_empty() {
        return Err("PDF OCR shard endpoint pool cannot be empty".to_string());
    }
    try_join_all(endpoint_urls.iter().map(|endpoint_url| async move {
        PdfOcrShardFlightClient::connect(endpoint_url.clone()).await
    }))
    .await
}

pub(crate) fn endpoint_index_for_request(
    request_index: usize,
    endpoint_count: usize,
) -> Result<usize, String> {
    if endpoint_count == 0 {
        return Err("PDF OCR shard endpoint pool cannot be empty".to_string());
    }
    Ok(request_index % endpoint_count)
}
