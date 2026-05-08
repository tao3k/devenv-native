use std::collections::BTreeMap;
use std::time::Duration;

use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE, PdfOcrShardInput,
};
use xiuxian_wendao_attachments::pdf::profile::source_pdf_page_profiles_cached;

pub(crate) const DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS";
pub(crate) const DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS";
const HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV: &str =
    "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE";
const HOSTED_VLM_REGION_PIPELINE_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE";
const HOSTED_VLM_REGION_PIPELINE_RENDER_DISPATCH: &str = "render-dispatch";
const FAST_TEXT_SOURCE_RANGE_SPLIT_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_SOURCE_RANGE_SPLIT";
const FAST_TEXT_SOURCE_RANGE_SPLIT_SINGLE_PAGE: &str = "single-page";

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
    source_pdf_page_range_chunks_with_fast_text_split(
        inputs,
        chunk_count,
        fast_text_source_range_single_page_split_enabled(),
    )
}

pub(crate) fn source_pdf_page_range_chunks_with_fast_text_split(
    inputs: &[PdfOcrShardInput],
    chunk_count: usize,
    split_fast_text_single_pages: bool,
) -> Vec<&[PdfOcrShardInput]> {
    if inputs.is_empty() {
        return Vec::new();
    }
    if split_fast_text_single_pages && all_fast_text_source_pdf_pages(inputs) {
        return inputs.chunks(1).collect();
    }
    if let Some(weights) = source_pdf_page_range_weights(inputs) {
        return source_pdf_page_range_chunks_with_weights(inputs, chunk_count, weights.as_slice());
    }
    source_pdf_page_range_chunks_without_weights(inputs, chunk_count)
}

pub(crate) fn source_pdf_page_range_dispatch_chunks(
    inputs: &[PdfOcrShardInput],
    chunk_count: usize,
) -> Vec<&[PdfOcrShardInput]> {
    let mut chunks = source_pdf_page_range_chunks(inputs, chunk_count);
    if !inputs
        .iter()
        .any(|input| input.ocr_profile == PDF_OCR_BACKEND_TEXT_PROFILE)
    {
        return chunks;
    }
    chunks.sort_by(|left, right| {
        source_pdf_page_range_dispatch_priority(right)
            .cmp(&source_pdf_page_range_dispatch_priority(left))
            .then_with(|| {
                source_pdf_page_range_chunk_weight(right)
                    .cmp(&source_pdf_page_range_chunk_weight(left))
            })
            .then_with(|| {
                left.first()
                    .map(|input| input.page_index)
                    .cmp(&right.first().map(|input| input.page_index))
            })
    });
    chunks
}

pub(crate) fn source_pdf_page_range_dispatch_budget(
    inputs: &[PdfOcrShardInput],
    requested: usize,
) -> usize {
    source_pdf_page_range_dispatch_budget_with_region_pipeline(
        inputs,
        requested,
        hosted_vlm_region_render_dispatch_enabled(),
    )
}

pub(crate) fn source_pdf_page_range_dispatch_budget_with_region_pipeline(
    inputs: &[PdfOcrShardInput],
    requested: usize,
    hosted_region_render_dispatch: bool,
) -> usize {
    source_pdf_page_range_dispatch_budget_with_region_pipeline_and_fast_text_split(
        inputs,
        requested,
        hosted_region_render_dispatch,
        fast_text_source_range_single_page_split_enabled(),
    )
}

pub(crate) fn source_pdf_page_range_dispatch_budget_with_region_pipeline_and_fast_text_split(
    inputs: &[PdfOcrShardInput],
    requested: usize,
    hosted_region_render_dispatch: bool,
    _split_fast_text_single_pages: bool,
) -> usize {
    if inputs.is_empty() {
        return requested.max(1);
    }
    if hosted_region_render_dispatch {
        return requested
            .max(source_pdf_page_range_runs(inputs).len())
            .min(inputs.len())
            .max(1);
    }
    if !inputs
        .iter()
        .any(|input| input.ocr_profile == PDF_OCR_BACKEND_TEXT_PROFILE)
    {
        return requested.max(1);
    }

    requested
        .max(source_pdf_page_range_runs(inputs).len())
        .min(inputs.len())
        .max(1)
}

fn hosted_vlm_region_render_dispatch_enabled() -> bool {
    std::env::var(HOSTED_VLM_REGION_PIPELINE_ENV)
        .ok()
        .map(|value| {
            value.trim().replace('_', "-").to_ascii_lowercase()
                == HOSTED_VLM_REGION_PIPELINE_RENDER_DISPATCH
        })
        .unwrap_or(false)
}

fn fast_text_source_range_single_page_split_enabled() -> bool {
    std::env::var(FAST_TEXT_SOURCE_RANGE_SPLIT_ENV)
        .ok()
        .map(|value| {
            value.trim().replace('_', "-").to_ascii_lowercase()
                == FAST_TEXT_SOURCE_RANGE_SPLIT_SINGLE_PAGE
        })
        .unwrap_or(false)
}

fn all_fast_text_source_pdf_pages(inputs: &[PdfOcrShardInput]) -> bool {
    let Some(first) = inputs.first() else {
        return false;
    };
    inputs.iter().all(|input| {
        input.source_path == first.source_path
            && input.ocr_profile == PDF_OCR_FAST_TEXT_PROFILE
            && input.shard_type == "page"
            && input.source_path.to_ascii_lowercase().ends_with(".pdf")
    })
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
    let page_counts = rendered_region_page_counts(inputs);
    chunks.sort_by(|left, right| {
        rendered_region_chunk_page_density(right, &page_counts)
            .cmp(&rendered_region_chunk_page_density(left, &page_counts))
            .then_with(|| {
                rendered_region_chunk_weight(right).cmp(&rendered_region_chunk_weight(left))
            })
            .then_with(|| left[0].page_index.cmp(&right[0].page_index))
            .then_with(|| left[0].region_index.cmp(&right[0].region_index))
            .then_with(|| left[0].reading_order_key.cmp(&right[0].reading_order_key))
    });
    chunks
}

fn rendered_region_composite_size_from_environment() -> usize {
    std::env::var(HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV)
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
    let page_counts = rendered_region_page_counts(inputs);
    chunks.sort_by(|left, right| {
        rendered_region_chunk_page_density(right, &page_counts)
            .cmp(&rendered_region_chunk_page_density(left, &page_counts))
            .then_with(|| {
                rendered_region_chunk_weight(right).cmp(&rendered_region_chunk_weight(left))
            })
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

fn source_pdf_page_range_dispatch_priority(inputs: &[PdfOcrShardInput]) -> u8 {
    if inputs
        .iter()
        .any(|input| input.ocr_profile != PDF_OCR_BACKEND_TEXT_PROFILE)
    {
        return 2;
    }
    1
}

fn source_pdf_page_range_chunk_weight(inputs: &[PdfOcrShardInput]) -> u64 {
    inputs
        .iter()
        .map(|input| {
            u64::from(input.raster_width_px)
                .saturating_mul(u64::from(input.raster_height_px))
                .max(1)
        })
        .fold(0_u64, u64::saturating_add)
}

fn rendered_region_chunk_weight(inputs: &[PdfOcrShardInput]) -> u64 {
    inputs
        .iter()
        .map(rendered_region_shard_weight)
        .fold(0_u64, u64::saturating_add)
}

fn rendered_region_chunk_page_density(
    inputs: &[PdfOcrShardInput],
    page_counts: &BTreeMap<RenderedRegionPageKey, usize>,
) -> usize {
    inputs
        .iter()
        .map(|input| {
            page_counts
                .get(&RenderedRegionPageKey::from(input))
                .copied()
                .unwrap_or(1)
        })
        .max()
        .unwrap_or(1)
}

fn rendered_region_page_counts(
    inputs: &[PdfOcrShardInput],
) -> BTreeMap<RenderedRegionPageKey, usize> {
    let mut counts: BTreeMap<RenderedRegionPageKey, usize> = BTreeMap::new();
    for input in inputs {
        counts
            .entry(RenderedRegionPageKey::from(input))
            .and_modify(|count| *count = count.saturating_add(1))
            .or_insert(1);
    }
    counts
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RenderedRegionPageKey {
    source_path: String,
    page_index: u32,
}

impl From<&PdfOcrShardInput> for RenderedRegionPageKey {
    fn from(input: &PdfOcrShardInput) -> Self {
        Self {
            source_path: input.source_path.clone(),
            page_index: input.page_index,
        }
    }
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
