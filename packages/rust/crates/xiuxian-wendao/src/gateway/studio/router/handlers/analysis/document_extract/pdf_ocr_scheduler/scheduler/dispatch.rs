use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::future::try_join_all;
use tokio::sync::OwnedSemaphorePermit;
use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, build_ocr_result_resource_batch,
};

use super::core::PdfOcrWorkerScheduler;
use super::limit::{duration_to_ms, source_pdf_page_range_chunks};
use crate::gateway::studio::document_extract_pdf_ocr_client::{
    PdfOcrShardFlightClient, PdfOcrShardFlightResponse,
};
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_cache::ocr_shard_cache_key;
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_order::order_ocr_results_by_inputs;
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::capacity::{
    OcrSchedulerLane, classify_ocr_lane, source_range_override_from_environment,
};
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::inflight::{
    InFlightShardEntry, InFlightShardReservation,
};

#[derive(Debug)]
struct OwnedShardRequest {
    position: usize,
    key: String,
    entry: Arc<InFlightShardEntry>,
    input: PdfOcrShardInput,
}

impl PdfOcrWorkerScheduler {
    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) async fn request_shards(
        &self,
        endpoint_url: String,
        inputs: &[PdfOcrShardInput],
    ) -> Result<PdfOcrShardFlightResponse, String> {
        if inputs.is_empty() {
            return Err("PDF OCR shard request inputs cannot be empty".to_string());
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
        let live_results = if misses.is_empty() {
            Vec::new()
        } else {
            self.request_missed_shards(endpoint_url, misses.as_slice())
                .await?
        };

        let results = cache_resolution.merge(live_results)?;
        let resource_batch = build_ocr_result_resource_batch(results.as_slice())?;
        Ok(PdfOcrShardFlightResponse {
            results,
            resource_batch,
        })
    }

    async fn request_missed_shards(
        &self,
        endpoint_url: String,
        inputs: &[PdfOcrShardInput],
    ) -> Result<Vec<PdfOcrShardResult>, String> {
        let mut slots = vec![None; inputs.len()];
        let mut owner_requests = Vec::new();
        let mut waiters = Vec::new();

        for (position, input) in inputs.iter().enumerate() {
            let key = ocr_shard_cache_key(input);
            match self.inflight.reserve(key) {
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

        if !owner_requests.is_empty() {
            self.handle_owned_shard_requests(endpoint_url, &mut slots, owner_requests)
                .await?;
        }

        for (position, entry) in waiters {
            slots[position] = Some(entry.wait().await?);
        }

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

    async fn handle_owned_shard_requests(
        &self,
        endpoint_url: String,
        slots: &mut [Option<PdfOcrShardResult>],
        owner_requests: Vec<OwnedShardRequest>,
    ) -> Result<(), String> {
        let client = PdfOcrShardFlightClient::connect(endpoint_url).await?;
        let owner_inputs = owner_requests
            .iter()
            .map(|request| request.input.clone())
            .collect::<Vec<_>>();
        let live_results = match self
            .request_uncached_shards(&client, owner_inputs.as_slice())
            .await
        {
            Ok(results) => order_ocr_results_by_inputs(owner_inputs.as_slice(), results),
            Err(error) => {
                for request in owner_requests {
                    self.inflight
                        .publish(request.key.as_str(), &request.entry, Err(error.clone()));
                }
                return Err(error);
            }
        }?;

        for (request, result) in owner_requests.iter().zip(live_results) {
            if let Err(error) = self.cache.store_successful(&request.input, &result) {
                log::warn!("failed to store PDF OCR shard cache row: {error}");
            }
            self.inflight
                .publish(request.key.as_str(), &request.entry, Ok(result.clone()));
            slots[request.position] = Some(result);
        }
        Ok(())
    }

    async fn request_uncached_shards(
        &self,
        client: &PdfOcrShardFlightClient,
        inputs: &[PdfOcrShardInput],
    ) -> Result<Vec<PdfOcrShardResult>, String> {
        if classify_ocr_lane(inputs) == OcrSchedulerLane::SourcePdfPageRange {
            return self
                .request_source_pdf_page_range_shards(client, inputs)
                .await;
        }
        let lane = classify_ocr_lane(inputs);
        let mut results = Vec::with_capacity(inputs.len());
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
            results.extend(ordered);
            offset = end;
        }
        Ok(results)
    }

    async fn request_source_pdf_page_range_shards(
        &self,
        client: &PdfOcrShardFlightClient,
        inputs: &[PdfOcrShardInput],
    ) -> Result<Vec<PdfOcrShardResult>, String> {
        let lane = OcrSchedulerLane::SourcePdfPageRange;
        let target = self.capacity.budget_for_lane(
            inputs.len(),
            lane,
            source_range_override_from_environment(),
        );
        let (permits, queue_wait) = self.acquire_worker_permits(target).await?;
        self.metrics.record_queue_wait(queue_wait);
        let chunk_count = permits.len().clamp(1, inputs.len());
        let chunks = source_pdf_page_range_chunks(inputs, chunk_count);
        self.metrics.record_live_request(lane, inputs.len());
        let latency_start = Instant::now();
        let chunk_results = try_join_all(chunks.iter().map(|chunk| {
            let client = client.clone();
            async move {
                let response = client.request_with_worker_budget(chunk, Some(1)).await?;
                order_ocr_results_by_inputs(chunk, response.results)
            }
        }))
        .await;
        let latency = latency_start.elapsed();
        self.metrics.record_ocr_latency(latency);
        drop(permits);

        let chunk_results = match chunk_results {
            Ok(chunk_results) => {
                self.capacity
                    .record_success(duration_to_ms(queue_wait), duration_to_ms(latency));
                chunk_results
            }
            Err(error) => {
                self.capacity.record_failure();
                return Err(error);
            }
        };

        let mut results = Vec::with_capacity(inputs.len());
        for chunk in chunk_results {
            results.extend(chunk);
        }
        order_ocr_results_by_inputs(inputs, results)
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
}
