use std::sync::Arc;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, build_ocr_result_resource_batch};

use crate::gateway::studio::document_extract_pdf_ocr_client::{
    PdfOcrShardFlightClient, PdfOcrShardFlightResponse,
};

pub(super) const DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS";

#[derive(Debug)]
pub(super) struct PdfOcrWorkerScheduler {
    permits: Arc<Semaphore>,
    worker_limit: usize,
}

impl PdfOcrWorkerScheduler {
    pub(super) fn from_environment() -> Self {
        Self::with_limit(pdf_ocr_worker_limit())
    }

    pub(super) fn with_limit(worker_limit: usize) -> Self {
        let worker_limit = worker_limit.max(1);
        Self {
            permits: Arc::new(Semaphore::new(worker_limit)),
            worker_limit,
        }
    }

    pub(super) fn worker_limit(&self) -> usize {
        self.worker_limit
    }

    pub(super) fn available_permits(&self) -> usize {
        self.permits.available_permits()
    }

    #[cfg(test)]
    pub(super) fn permits_for_tests(&self) -> Arc<Semaphore> {
        Arc::clone(&self.permits)
    }

    pub(super) async fn request_shards(
        &self,
        endpoint_url: String,
        inputs: &[PdfOcrShardInput],
    ) -> Result<PdfOcrShardFlightResponse, String> {
        if inputs.is_empty() {
            return Err("PDF OCR shard request inputs cannot be empty".to_string());
        }
        let client = PdfOcrShardFlightClient::connect(endpoint_url).await?;
        if is_contiguous_source_pdf_page_range(inputs) {
            let permits = self.acquire_worker_permits(1).await?;
            let response = client.request_with_worker_budget(inputs, Some(1)).await;
            drop(permits);
            return response;
        }
        let mut results = Vec::with_capacity(inputs.len());
        let mut offset = 0;
        while offset < inputs.len() {
            let remaining = inputs.len() - offset;
            let permits = self.acquire_worker_permits(remaining).await?;
            let worker_budget = permits.len().clamp(1, remaining);
            let end = offset + worker_budget;
            let response = client
                .request_with_worker_budget(&inputs[offset..end], Some(worker_budget))
                .await;
            drop(permits);
            let response = response?;
            results.extend(response.results);
            offset = end;
        }
        let resource_batch = build_ocr_result_resource_batch(results.as_slice())?;
        Ok(PdfOcrShardFlightResponse {
            results,
            resource_batch,
        })
    }

    async fn acquire_worker_permits(
        &self,
        remaining_shards: usize,
    ) -> Result<Vec<OwnedSemaphorePermit>, String> {
        let target = pdf_ocr_chunk_target(remaining_shards, self.worker_limit);
        let first = Arc::clone(&self.permits)
            .acquire_owned()
            .await
            .map_err(|error| format!("acquire PDF OCR worker permit: {error}"))?;
        let mut permits = vec![first];
        while permits.len() < target {
            match Arc::clone(&self.permits).try_acquire_owned() {
                Ok(permit) => permits.push(permit),
                Err(_) => break,
            }
        }
        Ok(permits)
    }
}

fn pdf_ocr_worker_limit() -> usize {
    pdf_ocr_worker_limit_with_lookup(
        &|key| std::env::var(key).ok(),
        std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .ok(),
    )
}

#[cfg(test)]
fn pdf_ocr_worker_budget_with_lookup(
    shard_count: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
    available_parallelism: Option<usize>,
) -> usize {
    pdf_ocr_chunk_target(
        shard_count,
        pdf_ocr_worker_limit_with_lookup(lookup, available_parallelism),
    )
}

pub(super) fn pdf_ocr_worker_limit_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
    available_parallelism: Option<usize>,
) -> usize {
    let machine_budget = available_parallelism.unwrap_or(1).max(1);
    lookup(DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|budget| *budget > 0)
        .unwrap_or(machine_budget)
        .max(1)
}

fn pdf_ocr_chunk_target(shard_count: usize, worker_limit: usize) -> usize {
    shard_count.max(1).min(worker_limit.max(1))
}

fn is_contiguous_source_pdf_page_range(inputs: &[PdfOcrShardInput]) -> bool {
    let Some(first) = inputs.first() else {
        return false;
    };
    if !first.source_path.to_ascii_lowercase().ends_with(".pdf") {
        return false;
    }
    inputs.iter().enumerate().all(|(offset, input)| {
        input.source_path == first.source_path
            && input.shard_type == "page"
            && input.page_index == first.page_index + u32::try_from(offset).unwrap_or(u32::MAX)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use xiuxian_wendao_attachments::pdf::ocr::PDF_OCR_SHARD_INPUT_SCHEMA_VERSION;

    #[test]
    fn pdf_ocr_worker_budget_defaults_to_available_parallelism() {
        let budget = pdf_ocr_worker_budget_with_lookup(21, &|_| None, Some(12));

        assert_eq!(budget, 12);
    }

    #[test]
    fn pdf_ocr_worker_budget_caps_to_shard_count() {
        let budget = pdf_ocr_worker_budget_with_lookup(2, &|_| None, Some(12));

        assert_eq!(budget, 2);
    }

    #[test]
    fn pdf_ocr_worker_budget_accepts_deployment_override() {
        let budget = pdf_ocr_worker_budget_with_lookup(
            21,
            &|key| (key == DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV).then(|| "6".to_string()),
            Some(12),
        );

        assert_eq!(budget, 6);
    }

    #[tokio::test]
    async fn pdf_ocr_worker_permits_share_global_runtime_budget() -> Result<(), String> {
        let scheduler = PdfOcrWorkerScheduler::with_limit(3);
        let held_permits = Arc::clone(&scheduler.permits)
            .acquire_many_owned(2)
            .await
            .map_err(|error| error.to_string())?;

        let permits = scheduler.acquire_worker_permits(10).await?;

        assert_eq!(permits.len(), 1);
        assert_eq!(scheduler.available_permits(), 0);
        drop(permits);
        drop(held_permits);
        assert_eq!(scheduler.available_permits(), 3);
        Ok(())
    }

    #[tokio::test]
    async fn pdf_ocr_worker_permits_take_idle_machine_capacity() -> Result<(), String> {
        let scheduler = PdfOcrWorkerScheduler::with_limit(4);

        let permits = scheduler.acquire_worker_permits(10).await?;

        assert_eq!(permits.len(), 4);
        assert_eq!(scheduler.available_permits(), 0);
        Ok(())
    }

    #[test]
    fn contiguous_source_pdf_page_range_detects_batchable_inputs() {
        let inputs = vec![
            sample_ocr_input("/tmp/source.pdf", 0, "page"),
            sample_ocr_input("/tmp/source.pdf", 1, "page"),
            sample_ocr_input("/tmp/source.pdf", 2, "page"),
        ];

        assert!(is_contiguous_source_pdf_page_range(inputs.as_slice()));
    }

    #[test]
    fn contiguous_source_pdf_page_range_rejects_regions_and_gaps() {
        let region_inputs = vec![
            sample_ocr_input("/tmp/source.pdf", 0, "page"),
            sample_ocr_input("/tmp/source.pdf", 1, "region"),
        ];
        let gap_inputs = vec![
            sample_ocr_input("/tmp/source.pdf", 0, "page"),
            sample_ocr_input("/tmp/source.pdf", 2, "page"),
        ];

        assert!(!is_contiguous_source_pdf_page_range(
            region_inputs.as_slice()
        ));
        assert!(!is_contiguous_source_pdf_page_range(gap_inputs.as_slice()));
    }

    fn sample_ocr_input(source_path: &str, page_index: u32, shard_type: &str) -> PdfOcrShardInput {
        PdfOcrShardInput {
            contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
            source_path: source_path.to_string(),
            source_content_hash: "sourcehash".to_string(),
            page_index,
            image_path: format!("/tmp/page-{page_index:05}.png"),
            image_mime_type: "image/png".to_string(),
            raster_sha256: format!("rasterhash-{page_index}"),
            render_profile: "pdfium-render-page-shards-v1".to_string(),
            ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
            ocr_engine: "docling-compatible-ocr".to_string(),
            preferred_languages: vec!["auto".to_string()],
            min_confidence: 0.0,
            preserve_layout: true,
            raster_width_px: 2400,
            raster_height_px: 3100,
            render_dpi: 300,
            rotation_degrees: 0,
            crop_left: 0.0,
            crop_bottom: 0.0,
            crop_right: 612.0,
            crop_top: 792.0,
            point_to_pixel_scale_x: 3.921_568_627,
            point_to_pixel_scale_y: 3.914_141_414,
            shard_element_id: format!("shard-{page_index}"),
            shard_type: shard_type.to_string(),
            region_index: 0,
            parent_shard_element_id: String::new(),
            reading_order_key: format!("{page_index:06}.000000"),
            source_page_pixel_left: 0,
            source_page_pixel_top: 0,
            source_page_pixel_right: 2400,
            source_page_pixel_bottom: 3100,
        }
    }
}
