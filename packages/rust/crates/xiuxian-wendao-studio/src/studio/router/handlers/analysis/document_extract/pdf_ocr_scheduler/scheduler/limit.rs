use std::collections::{BTreeMap, HashSet};
use std::time::Duration;

use xiuxian_wendao_attachments::pdf::ocr::PdfOcrShardInput;

pub(crate) const DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS";
pub(crate) const DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS";
const SOURCE_PDF_PAGE_RANGE_BRIDGE_GAP_LIMIT: u32 = 2;

pub(super) fn pdf_ocr_worker_limit() -> usize {
    pdf_ocr_worker_limit_with_lookup(
        &|key| std::env::var(key).ok(),
        std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .ok(),
    )
}

pub(crate) fn pdf_ocr_worker_limit_with_lookup(
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

pub(crate) fn source_pdf_page_range_chunks(
    inputs: &[PdfOcrShardInput],
    chunk_count: usize,
) -> Vec<&[PdfOcrShardInput]> {
    if inputs.is_empty() {
        return Vec::new();
    }

    let runs = source_pdf_page_range_runs(inputs);
    if runs.len() <= 1 {
        return balanced_chunks(inputs, chunk_count);
    }

    let requested_chunks = chunk_count.max(1).max(runs.len()).min(inputs.len());
    let mut remaining_chunks = requested_chunks;
    let mut chunks = Vec::with_capacity(requested_chunks);
    for (run_index, run) in runs.iter().enumerate() {
        let runs_remaining = runs.len() - run_index;
        let reserved_for_later_runs = runs_remaining.saturating_sub(1);
        let available_for_run = remaining_chunks
            .saturating_sub(reserved_for_later_runs)
            .max(1);
        let proportional_for_run = run
            .len()
            .saturating_mul(requested_chunks)
            .div_ceil(inputs.len())
            .max(1);
        let run_chunk_count = proportional_for_run
            .min(available_for_run)
            .min(run.len())
            .max(1);
        chunks.extend(balanced_chunks(run, run_chunk_count));
        remaining_chunks = remaining_chunks.saturating_sub(run_chunk_count);
    }
    chunks
}

pub(crate) fn source_pdf_page_range_bridge_inputs(
    all_inputs: &[PdfOcrShardInput],
    missing_inputs: &[PdfOcrShardInput],
    required_inputs: &[PdfOcrShardInput],
) -> Vec<PdfOcrShardInput> {
    let Some(first) = required_inputs.first() else {
        return Vec::new();
    };
    if required_inputs.len() == 1 {
        return required_inputs.to_vec();
    }

    let source_path = first.source_path.as_str();
    if !is_source_pdf_page_input(first, source_path) {
        return required_inputs.to_vec();
    }

    let mut page_inputs = BTreeMap::new();
    for input in all_inputs {
        if !is_source_pdf_page_input(input, source_path) {
            return required_inputs.to_vec();
        }
        if page_inputs.insert(input.page_index, input).is_some() {
            return required_inputs.to_vec();
        }
    }

    let missing_shard_ids = missing_inputs
        .iter()
        .map(|input| input.shard_element_id.as_str())
        .collect::<HashSet<_>>();
    let mut required_pages = Vec::with_capacity(required_inputs.len());
    for input in required_inputs {
        if !is_source_pdf_page_input(input, source_path) {
            return required_inputs.to_vec();
        }
        if !page_inputs.contains_key(&input.page_index) {
            return required_inputs.to_vec();
        }
        required_pages.push(input.page_index);
    }
    required_pages.sort_unstable();
    required_pages.dedup();
    if required_pages.len() != required_inputs.len() {
        return required_inputs.to_vec();
    }

    let mut planned_pages = Vec::with_capacity(required_pages.len());
    for (index, page) in required_pages.iter().copied().enumerate() {
        if index == 0 {
            planned_pages.push(page);
            continue;
        }
        let previous_page = required_pages[index - 1];
        if source_pdf_page_range_bridge_gap_available(
            &page_inputs,
            &missing_shard_ids,
            previous_page,
            page,
        ) {
            for bridge_page in previous_page.saturating_add(1)..page {
                planned_pages.push(bridge_page);
            }
        }
        planned_pages.push(page);
    }

    planned_pages
        .into_iter()
        .filter_map(|page| page_inputs.get(&page).map(|input| (*input).to_owned()))
        .collect()
}

fn balanced_chunks(inputs: &[PdfOcrShardInput], chunk_count: usize) -> Vec<&[PdfOcrShardInput]> {
    if inputs.is_empty() {
        return Vec::new();
    }
    let chunk_count = chunk_count.max(1).min(inputs.len());
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

fn source_pdf_page_range_runs(inputs: &[PdfOcrShardInput]) -> Vec<&[PdfOcrShardInput]> {
    if inputs.is_empty() {
        return Vec::new();
    }

    let mut runs = Vec::new();
    let mut run_start = 0usize;
    for index in 1..inputs.len() {
        if !extends_source_pdf_page_range_run(&inputs[index - 1], &inputs[index]) {
            runs.push(&inputs[run_start..index]);
            run_start = index;
        }
    }
    runs.push(&inputs[run_start..]);
    runs
}

fn extends_source_pdf_page_range_run(
    previous: &PdfOcrShardInput,
    current: &PdfOcrShardInput,
) -> bool {
    current.source_path == previous.source_path
        && current.shard_type == "page"
        && previous.shard_type == "page"
        && current.page_index == previous.page_index.saturating_add(1)
}

fn source_pdf_page_range_bridge_gap_available(
    page_inputs: &BTreeMap<u32, &PdfOcrShardInput>,
    missing_shard_ids: &HashSet<&str>,
    previous_page: u32,
    current_page: u32,
) -> bool {
    let gap_pages = current_page.saturating_sub(previous_page).saturating_sub(1);
    if gap_pages == 0 {
        return true;
    }
    if gap_pages > SOURCE_PDF_PAGE_RANGE_BRIDGE_GAP_LIMIT {
        return false;
    }
    let Some(gap_start) = previous_page.checked_add(1) else {
        return false;
    };
    for page in gap_start..current_page {
        let Some(input) = page_inputs.get(&page) else {
            return false;
        };
        if missing_shard_ids.contains(input.shard_element_id.as_str()) {
            return false;
        }
    }
    true
}

fn is_source_pdf_page_input(input: &PdfOcrShardInput, source_path: &str) -> bool {
    input.source_path == source_path
        && input.shard_type == "page"
        && input.source_path.to_ascii_lowercase().ends_with(".pdf")
}
