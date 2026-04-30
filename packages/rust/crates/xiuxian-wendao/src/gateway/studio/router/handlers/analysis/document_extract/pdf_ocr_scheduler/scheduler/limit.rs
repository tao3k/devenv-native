use std::time::Duration;

use xiuxian_wendao_attachments::pdf::ocr::PdfOcrShardInput;

pub(in super::super) const DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS";
pub(in super::super) const DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS";

pub(super) fn pdf_ocr_worker_limit() -> usize {
    pdf_ocr_worker_limit_with_lookup(
        &|key| std::env::var(key).ok(),
        std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .ok(),
    )
}

pub(in super::super) fn pdf_ocr_worker_limit_with_lookup(
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

pub(super) fn duration_to_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

pub(in super::super) fn source_pdf_page_range_chunks(
    inputs: &[PdfOcrShardInput],
    chunk_count: usize,
) -> Vec<&[PdfOcrShardInput]> {
    if inputs.is_empty() {
        return Vec::new();
    }
    let chunk_count = chunk_count.clamp(1, inputs.len());
    let base = inputs.len() / chunk_count;
    let extra = inputs.len() % chunk_count;
    let mut start = 0;
    let mut chunks = Vec::with_capacity(chunk_count);
    for chunk_index in 0..chunk_count {
        let size = base + usize::from(chunk_index < extra);
        let end = start + size;
        chunks.push(&inputs[start..end]);
        start = end;
    }
    chunks
}
