use std::time::Duration;

use xiuxian_wendao_attachments::pdf::ocr::PdfOcrShardInput;
use xiuxian_wendao_attachments::pdf::profile::source_pdf_page_profiles_cached;

pub(crate) const DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS";
pub(crate) const DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS";
const DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV: &str = "WENDAO_DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE";

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
    if let Some(weights) = source_pdf_page_range_weights(inputs) {
        return source_pdf_page_range_chunks_with_weights(inputs, chunk_count, weights.as_slice());
    }
    source_pdf_page_range_chunks_without_weights(inputs, chunk_count)
}

pub(crate) fn rendered_region_shard_chunks(
    inputs: &[PdfOcrShardInput],
) -> Vec<&[PdfOcrShardInput]> {
    rendered_region_shard_chunks_with_composite_size(
        inputs,
        rendered_region_composite_size_from_environment(),
    )
}

pub(crate) fn rendered_region_shard_chunks_with_composite_size(
    inputs: &[PdfOcrShardInput],
    composite_size: usize,
) -> Vec<&[PdfOcrShardInput]> {
    if inputs.is_empty() {
        return Vec::new();
    }
    if composite_size > 1 {
        return rendered_region_composite_chunks(inputs, composite_size);
    }
    let mut chunks = inputs.chunks(1).collect::<Vec<_>>();
    chunks.sort_by(|left, right| {
        rendered_region_shard_weight(&right[0])
            .cmp(&rendered_region_shard_weight(&left[0]))
            .then_with(|| left[0].page_index.cmp(&right[0].page_index))
            .then_with(|| left[0].region_index.cmp(&right[0].region_index))
            .then_with(|| left[0].reading_order_key.cmp(&right[0].reading_order_key))
    });
    chunks
}

fn rendered_region_composite_size_from_environment() -> usize {
    std::env::var(DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 1)
        .unwrap_or(1)
}

fn rendered_region_composite_chunks(
    inputs: &[PdfOcrShardInput],
    composite_size: usize,
) -> Vec<&[PdfOcrShardInput]> {
    let composite_size = composite_size.max(1);
    let mut chunks = Vec::new();
    let mut start = 0usize;
    for index in 1..inputs.len() {
        let previous = &inputs[index - 1];
        let current = &inputs[index];
        if index - start >= composite_size
            || !can_extend_rendered_region_composite(previous, current)
        {
            chunks.push(&inputs[start..index]);
            start = index;
        }
    }
    chunks.push(&inputs[start..]);
    chunks.sort_by(|left, right| {
        rendered_region_chunk_weight(right)
            .cmp(&rendered_region_chunk_weight(left))
            .then_with(|| left[0].page_index.cmp(&right[0].page_index))
            .then_with(|| left[0].region_index.cmp(&right[0].region_index))
            .then_with(|| left[0].reading_order_key.cmp(&right[0].reading_order_key))
    });
    chunks
}

fn can_extend_rendered_region_composite(
    previous: &PdfOcrShardInput,
    current: &PdfOcrShardInput,
) -> bool {
    previous.source_path == current.source_path
        && previous.source_content_hash == current.source_content_hash
        && previous.page_index == current.page_index
        && previous.parent_shard_element_id == current.parent_shard_element_id
}

pub(crate) fn source_pdf_page_range_chunks_with_weights<'a>(
    inputs: &'a [PdfOcrShardInput],
    chunk_count: usize,
    weights: &[u32],
) -> Vec<&'a [PdfOcrShardInput]> {
    if inputs.is_empty() {
        return Vec::new();
    }
    if weights.len() != inputs.len() {
        return source_pdf_page_range_chunks_without_weights(inputs, chunk_count);
    }

    let runs = source_pdf_page_range_runs(inputs);
    if runs.len() <= 1 {
        return weighted_chunks(inputs, weights, chunk_count);
    }

    let requested_chunks = chunk_count.max(1).max(runs.len()).min(inputs.len());
    let mut remaining_chunks = requested_chunks;
    let mut run_start = 0usize;
    let mut chunks = Vec::with_capacity(requested_chunks);
    for (run_index, run) in runs.iter().enumerate() {
        let runs_remaining = runs.len() - run_index;
        let reserved_for_later_runs = runs_remaining.saturating_sub(1);
        let available_for_run = remaining_chunks
            .saturating_sub(reserved_for_later_runs)
            .max(1);
        let run_end = run_start + run.len();
        let run_weight = total_weight(&weights[run_start..run_end]);
        let all_weight = total_weight(weights);
        let proportional_for_run = usize::try_from(run_weight)
            .unwrap_or(usize::MAX)
            .saturating_mul(requested_chunks)
            .div_ceil(usize::try_from(all_weight).unwrap_or(usize::MAX).max(1))
            .max(1);
        let run_chunk_count = proportional_for_run
            .min(available_for_run)
            .min(run.len())
            .max(1);
        chunks.extend(weighted_chunks(
            run,
            &weights[run_start..run_end],
            run_chunk_count,
        ));
        remaining_chunks = remaining_chunks.saturating_sub(run_chunk_count);
        run_start = run_end;
    }
    chunks
}

fn source_pdf_page_range_chunks_without_weights(
    inputs: &[PdfOcrShardInput],
    chunk_count: usize,
) -> Vec<&[PdfOcrShardInput]> {
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

fn weighted_chunks<'a>(
    inputs: &'a [PdfOcrShardInput],
    weights: &[u32],
    chunk_count: usize,
) -> Vec<&'a [PdfOcrShardInput]> {
    if inputs.is_empty() {
        return Vec::new();
    }
    if weights.len() != inputs.len() {
        return balanced_chunks(inputs, chunk_count);
    }
    let chunk_count = chunk_count.max(1).min(inputs.len());
    let total = total_weight(weights);
    if total == 0 {
        return balanced_chunks(inputs, chunk_count);
    }
    let target = total.div_ceil(u32::try_from(chunk_count).unwrap_or(u32::MAX).max(1));
    let mut chunks = Vec::with_capacity(chunk_count);
    let mut start = 0usize;
    let mut accumulated = 0u32;
    for (index, weight) in weights.iter().copied().enumerate() {
        let weight = weight.max(1);
        let remaining_items = inputs.len().saturating_sub(index + 1);
        if weight >= target
            && accumulated > 0
            && start < index
            && chunks.len() + 2 <= chunk_count
            && remaining_items >= chunk_count.saturating_sub(chunks.len() + 2)
        {
            chunks.push(&inputs[start..index]);
            start = index;
            accumulated = 0;
        }
        accumulated = accumulated.saturating_add(weight);
        let remaining_items = inputs.len().saturating_sub(index + 1);
        let remaining_chunks = chunk_count.saturating_sub(chunks.len() + 1);
        if (accumulated >= target || remaining_items == remaining_chunks)
            && remaining_items >= remaining_chunks
        {
            chunks.push(&inputs[start..=index]);
            start = index + 1;
            accumulated = 0;
            if chunks.len() + 1 == chunk_count {
                break;
            }
        }
    }
    if start < inputs.len() {
        chunks.push(&inputs[start..]);
    }
    chunks
}

fn total_weight(weights: &[u32]) -> u32 {
    weights
        .iter()
        .copied()
        .map(|weight| weight.max(1))
        .fold(0_u32, u32::saturating_add)
}

fn rendered_region_shard_weight(input: &PdfOcrShardInput) -> u64 {
    let source_width = input
        .source_page_pixel_right
        .saturating_sub(input.source_page_pixel_left);
    let source_height = input
        .source_page_pixel_bottom
        .saturating_sub(input.source_page_pixel_top);
    let source_area = u64::from(source_width).saturating_mul(u64::from(source_height));
    if source_area > 0 {
        return source_area;
    }
    u64::from(input.raster_width_px).saturating_mul(u64::from(input.raster_height_px))
}

fn rendered_region_chunk_weight(inputs: &[PdfOcrShardInput]) -> u64 {
    inputs
        .iter()
        .map(rendered_region_shard_weight)
        .fold(0_u64, u64::saturating_add)
}

fn source_pdf_page_range_weights(inputs: &[PdfOcrShardInput]) -> Option<Vec<u32>> {
    let first = inputs.first()?;
    if inputs
        .iter()
        .any(|input| input.source_path != first.source_path)
    {
        return None;
    }
    let profiles =
        source_pdf_page_profiles_cached(std::path::Path::new(first.source_path.as_str()))
            .map_err(|error| {
                log::debug!("source PDF page profile unavailable for OCR scheduler: {error}");
                error
            })
            .ok()?;
    Some(
        inputs
            .iter()
            .map(|input| {
                let Some(profile_index) = usize::try_from(input.page_index).ok() else {
                    return 1;
                };
                profiles
                    .get(profile_index)
                    .filter(|profile| profile.page_index == input.page_index)
                    .map_or(1, |profile| profile.estimated_weight)
            })
            .collect(),
    )
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
        && current.ocr_profile == previous.ocr_profile
        && current.shard_type == "page"
        && previous.shard_type == "page"
        && current.page_index == previous.page_index.saturating_add(1)
}
