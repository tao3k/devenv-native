use super::{
    BTreeMap, BTreeSet, DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE,
    DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_CHUNK_SIZE,
    DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_THRESHOLD,
    DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_CONCURRENCY_ENV,
    DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV,
    DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_SIZE_ENV,
    DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_HEDGE_DELAY_MS_ENV, HybridPdfOcrProfilePlanner,
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_DEFAULT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE,
    PageRangeDoclingFallbackPlanRange, PageRangeDoclingFallbackPlanSummary, Path, PdfOcrShardInput,
    PdfOcrShardResult, PdfOcrShardResultStatus, PdfSourcePageProfile,
    source_pdf_page_profiles_cached,
};

pub(super) fn contiguous_page_ranges(pages: &BTreeSet<u32>) -> Vec<(u32, u32)> {
    pages
        .iter()
        .copied()
        .fold(Vec::new(), push_page_into_contiguous_ranges)
}

fn push_page_into_contiguous_ranges(mut ranges: Vec<(u32, u32)>, page: u32) -> Vec<(u32, u32)> {
    match ranges.last_mut() {
        Some((_, end)) if page == end.saturating_add(1) => *end = page,
        _ => ranges.push((page, page)),
    }
    ranges
}

pub(super) fn docling_page_range_fallback_ranges(
    pages: &BTreeSet<u32>,
    max_chunk_pages: Option<u32>,
) -> Vec<(u32, u32)> {
    let Some(max_chunk_pages) = max_chunk_pages.filter(|value| *value > 0) else {
        return contiguous_page_ranges(pages);
    };
    contiguous_page_ranges(pages)
        .into_iter()
        .flat_map(|(start, end)| split_contiguous_page_range(start, end, max_chunk_pages))
        .collect()
}

#[cfg(test)]
pub(super) fn docling_page_range_fallback_ranges_with_lookup(
    pages: &BTreeSet<u32>,
    planner: HybridPdfOcrProfilePlanner,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<(u32, u32)>, String> {
    if let Some(chunk_plan) = docling_page_range_chunk_plan_with_lookup(pages, lookup)? {
        return Ok(chunk_plan);
    }
    Ok(docling_page_range_fallback_ranges(
        pages,
        docling_page_range_chunk_size_for_pages_with_lookup(pages, planner, lookup),
    ))
}

pub(super) fn docling_page_range_fallback_plan_for_source_with_lookup(
    pages: &BTreeSet<u32>,
    planner: HybridPdfOcrProfilePlanner,
    source_path: &Path,
    target_chunk_count: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<(Vec<(u32, u32)>, PageRangeDoclingFallbackPlanSummary), String> {
    let fallback_page_count = pages.len();
    if let Some(chunk_plan) = docling_page_range_chunk_plan_with_lookup(pages, lookup)? {
        let plan = page_range_plan_summary(
            "explicit-plan",
            target_chunk_count,
            fallback_page_count,
            None,
            false,
            chunk_plan.as_slice(),
        );
        return Ok((chunk_plan, plan));
    }
    if let Some(explicit_chunk_size) = docling_page_range_chunk_size_with_lookup(lookup) {
        let ranges = docling_page_range_fallback_ranges(pages, Some(explicit_chunk_size));
        let plan = page_range_plan_summary(
            "explicit-chunk-size",
            target_chunk_count,
            fallback_page_count,
            Some(explicit_chunk_size),
            false,
            ranges.as_slice(),
        );
        return Ok((ranges, plan));
    }
    let default_chunk_size =
        usize::try_from(DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE)
            .unwrap_or(1)
            .max(1);
    let fallback_chunk_floor = fallback_page_count.div_ceil(default_chunk_size).max(1);
    let source_profiles = source_pdf_page_profiles_cached(source_path).ok();
    if planner == HybridPdfOcrProfilePlanner::DoclingStructureRecovery
        && pages.len() > DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_THRESHOLD
        && target_chunk_count > 1
        && target_chunk_count > fallback_chunk_floor
        && let Some(profiles) = source_profiles.as_ref()
        && let Some(ranges) = weighted_docling_page_range_fallback_ranges(
            pages,
            profiles.as_slice(),
            target_chunk_count,
        )
    {
        let plan = page_range_plan_summary(
            "source-profile-weighted",
            target_chunk_count,
            fallback_page_count,
            None,
            true,
            ranges.as_slice(),
        );
        return Ok((ranges, plan));
    }
    let chunk_size = docling_page_range_chunk_size_for_pages_with_lookup(pages, planner, lookup);
    let ranges = docling_page_range_fallback_ranges(pages, chunk_size);
    let strategy = if planner == HybridPdfOcrProfilePlanner::DoclingStructureRecovery
        && pages.len() <= DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_THRESHOLD
    {
        "adaptive-small-page"
    } else if chunk_size.is_some() {
        "adaptive-chunk-size"
    } else {
        "contiguous"
    };
    let plan = page_range_plan_summary(
        strategy,
        target_chunk_count,
        fallback_page_count,
        chunk_size,
        false,
        ranges.as_slice(),
    );
    Ok((ranges, plan))
}

pub(super) fn docling_page_range_target_chunk_count(
    planner: HybridPdfOcrProfilePlanner,
    endpoint_count: usize,
    fallback_page_count: usize,
) -> usize {
    let endpoint_count = endpoint_count.max(1);
    let default_chunk_size =
        usize::try_from(DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE)
            .unwrap_or(1)
            .max(1);
    let fallback_chunk_floor = fallback_page_count.div_ceil(default_chunk_size).max(1);
    if planner != HybridPdfOcrProfilePlanner::DoclingStructureRecovery {
        return endpoint_count.max(fallback_chunk_floor);
    }
    if fallback_page_count <= DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_THRESHOLD {
        return fallback_page_count.max(1);
    }
    fallback_chunk_floor
        .saturating_add(1)
        .min(endpoint_count)
        .min(fallback_page_count)
        .max(fallback_chunk_floor)
}

fn page_range_plan_summary(
    strategy: &'static str,
    target_chunk_count: usize,
    fallback_page_count: usize,
    chunk_size: Option<u32>,
    source_profile_used: bool,
    ranges: &[(u32, u32)],
) -> PageRangeDoclingFallbackPlanSummary {
    PageRangeDoclingFallbackPlanSummary {
        strategy,
        target_chunk_count,
        fallback_page_count,
        range_count: ranges.len(),
        chunk_size,
        source_profile_used,
        ranges: ranges
            .iter()
            .copied()
            .map(|(page_start, page_end)| PageRangeDoclingFallbackPlanRange {
                page_start,
                page_end,
                one_based_start: page_start.saturating_add(1),
                one_based_end: page_end.saturating_add(1),
            })
            .collect(),
    }
}

pub(super) fn split_contiguous_page_range(
    start: u32,
    end: u32,
    max_chunk_pages: u32,
) -> Vec<(u32, u32)> {
    let mut ranges = Vec::new();
    let mut chunk_start = start;
    while chunk_start <= end {
        let chunk_end = end.min(chunk_start.saturating_add(max_chunk_pages.saturating_sub(1)));
        ranges.push((chunk_start, chunk_end));
        if chunk_end == u32::MAX {
            break;
        }
        chunk_start = chunk_end.saturating_add(1);
    }
    ranges
}

pub(super) fn weighted_docling_page_range_fallback_ranges(
    pages: &BTreeSet<u32>,
    profiles: &[PdfSourcePageProfile],
    target_chunk_count: usize,
) -> Option<Vec<(u32, u32)>> {
    if pages.is_empty() || target_chunk_count == 0 {
        return None;
    }
    let profile_weights = profiles
        .iter()
        .map(|profile| (profile.page_index, profile.estimated_weight.max(1)))
        .collect::<BTreeMap<_, _>>();
    if profile_weights.is_empty() {
        return None;
    }

    let target_chunk_count = target_chunk_count.min(pages.len()).max(1);
    let contiguous = contiguous_page_ranges(pages);
    let fallback_page_count = pages.len();
    if let Some(ranges) = tail_preserving_source_profile_page_ranges(
        contiguous.as_slice(),
        profiles,
        target_chunk_count,
    ) {
        return Some(ranges);
    }
    let mut ranges = Vec::new();
    let mut assigned_chunks = 0usize;
    for (run_index, (start, end)) in contiguous.iter().copied().enumerate() {
        let run_page_count = page_count_in_range(start, end)?;
        let remaining_runs = contiguous.len().saturating_sub(run_index + 1);
        let remaining_pages = fallback_page_count.saturating_sub(
            contiguous
                .iter()
                .take(run_index + 1)
                .filter_map(|(run_start, run_end)| page_count_in_range(*run_start, *run_end))
                .sum::<usize>(),
        );
        let remaining_chunk_budget = target_chunk_count.saturating_sub(assigned_chunks);
        let reserved_chunks = remaining_runs.min(remaining_pages);
        let run_target_chunks = remaining_chunk_budget
            .saturating_sub(reserved_chunks)
            .min(run_page_count)
            .max(1);
        let page_weights = (start..=end)
            .map(|page| (page, *profile_weights.get(&page).unwrap_or(&1)))
            .collect::<Vec<_>>();
        let mut run_ranges =
            balanced_weighted_contiguous_page_ranges(page_weights.as_slice(), run_target_chunks)?;
        assigned_chunks = assigned_chunks.saturating_add(run_ranges.len());
        ranges.append(&mut run_ranges);
    }
    Some(ranges)
}

fn tail_preserving_source_profile_page_ranges(
    contiguous: &[(u32, u32)],
    profiles: &[PdfSourcePageProfile],
    target_chunk_count: usize,
) -> Option<Vec<(u32, u32)>> {
    let default_chunk_size =
        usize::try_from(DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE).ok()?;
    if default_chunk_size <= 1 || contiguous.len() != 1 {
        return None;
    }

    let (start, end) = *contiguous.first()?;
    let base_ranges = split_contiguous_page_range(
        start,
        end,
        DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE,
    );
    if target_chunk_count != base_ranges.len().saturating_add(1) || base_ranges.len() < 2 {
        return None;
    }

    let profile_by_page = profiles
        .iter()
        .map(|profile| (profile.page_index, profile))
        .collect::<BTreeMap<_, _>>();
    let mut best: Option<(u64, usize, u32)> = None;
    for (range_index, (range_start, range_end)) in base_ranges
        .iter()
        .copied()
        .enumerate()
        .take(base_ranges.len().saturating_sub(1))
    {
        if range_start == range_end {
            continue;
        }
        let split_page = (range_start..=range_end).max_by_key(|page| {
            profile_by_page
                .get(page)
                .map_or(0, |profile| source_profile_split_priority(profile))
        })?;
        let priority = profile_by_page
            .get(&split_page)
            .map_or(0, |profile| source_profile_split_priority(profile));
        if priority == 0 {
            continue;
        }
        match best {
            Some((best_priority, best_range_index, _))
                if priority < best_priority
                    || (priority == best_priority && range_index <= best_range_index) => {}
            _ => best = Some((priority, range_index, split_page)),
        }
    }

    let (_, split_range_index, split_page) = best?;
    let mut ranges = Vec::with_capacity(target_chunk_count);
    for (range_index, (range_start, range_end)) in base_ranges.into_iter().enumerate() {
        if range_index != split_range_index {
            ranges.push((range_start, range_end));
            continue;
        }
        if split_page == range_start {
            ranges.push((range_start, range_start));
            ranges.push((range_start.saturating_add(1), range_end));
        } else {
            ranges.push((range_start, split_page.saturating_sub(1)));
            ranges.push((split_page, range_end));
        }
    }
    Some(ranges)
}

fn source_profile_split_priority(profile: &PdfSourcePageProfile) -> u64 {
    u64::from(profile.estimated_weight.max(1))
        .saturating_add(u64::from(profile.path_ops).saturating_mul(2))
        .saturating_add(u64::from(profile.rectangle_ops))
        .saturating_add(u64::from(profile.draw_object_ops).saturating_mul(32))
}

fn page_count_in_range(start: u32, end: u32) -> Option<usize> {
    usize::try_from(u64::from(end.checked_sub(start)?) + 1).ok()
}

fn balanced_weighted_contiguous_page_ranges(
    page_weights: &[(u32, u32)],
    target_chunk_count: usize,
) -> Option<Vec<(u32, u32)>> {
    if page_weights.is_empty() || target_chunk_count == 0 {
        return None;
    }
    let chunk_count = target_chunk_count.min(page_weights.len()).max(1);
    if chunk_count == 1 {
        return Some(vec![(page_weights.first()?.0, page_weights.last()?.0)]);
    }

    let page_count = page_weights.len();
    let prefix_weights = prefix_weights_for_pages(page_weights);
    let mut costs = vec![vec![u64::MAX; page_count + 1]; chunk_count + 1];
    let mut previous = vec![vec![0usize; page_count + 1]; chunk_count + 1];
    costs[0][0] = 0;
    for chunk_index in 1..=chunk_count {
        record_balanced_chunk_row(
            chunk_index,
            page_count,
            prefix_weights.as_slice(),
            &mut costs,
            &mut previous,
        );
    }
    if costs[chunk_count][page_count] == u64::MAX {
        return None;
    }

    Some(balanced_ranges_from_previous(
        page_weights,
        previous.as_slice(),
        page_count,
        chunk_count,
    ))
}

fn prefix_weights_for_pages(page_weights: &[(u32, u32)]) -> Vec<u64> {
    let mut total = 0_u64;
    std::iter::once(0)
        .chain(page_weights.iter().map(|(_, weight)| {
            total = total.saturating_add(u64::from(*weight));
            total
        }))
        .collect()
}

fn record_balanced_chunk_row(
    chunk_index: usize,
    page_count: usize,
    prefix_weights: &[u64],
    costs: &mut [Vec<u64>],
    previous: &mut [Vec<usize>],
) {
    (chunk_index..=page_count).for_each(|page_end| {
        if let Some((best_cost, best_split)) =
            best_balanced_split(chunk_index, page_end, prefix_weights, costs)
        {
            costs[chunk_index][page_end] = best_cost;
            previous[chunk_index][page_end] = best_split;
        }
    });
}

fn best_balanced_split(
    chunk_index: usize,
    page_end: usize,
    prefix_weights: &[u64],
    costs: &[Vec<u64>],
) -> Option<(u64, usize)> {
    ((chunk_index - 1)..page_end)
        .filter_map(|split| {
            let prior_cost = costs[chunk_index - 1][split];
            (prior_cost != u64::MAX).then(|| {
                let chunk_weight = prefix_weights[page_end].saturating_sub(prefix_weights[split]);
                (prior_cost.max(chunk_weight), split)
            })
        })
        .fold(None, |best, candidate| {
            Some(match best {
                Some(current) if !balanced_candidate_is_better(current, candidate, page_end) => {
                    current
                }
                _ => candidate,
            })
        })
}

fn balanced_candidate_is_better(
    current: (u64, usize),
    candidate: (u64, usize),
    page_end: usize,
) -> bool {
    let (current_cost, current_split) = current;
    let (candidate_cost, candidate_split) = candidate;
    candidate_cost < current_cost
        || (candidate_cost == current_cost
            && candidate_split.saturating_sub(current_split)
                > page_end.saturating_sub(candidate_split))
}

fn balanced_ranges_from_previous(
    page_weights: &[(u32, u32)],
    previous: &[Vec<usize>],
    page_count: usize,
    chunk_count: usize,
) -> Vec<(u32, u32)> {
    let mut page_end = page_count;
    let mut ranges = (1..=chunk_count)
        .rev()
        .map(|chunk_index| {
            let split = previous[chunk_index][page_end];
            let range = (page_weights[split].0, page_weights[page_end - 1].0);
            page_end = split;
            range
        })
        .collect::<Vec<_>>();
    ranges.reverse();
    ranges
}

pub(super) fn docling_page_range_chunk_plan_with_lookup(
    pages: &BTreeSet<u32>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Option<Vec<(u32, u32)>>, String> {
    let Some(raw_plan) = lookup(DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV) else {
        return Ok(None);
    };
    let raw_plan = raw_plan.trim();
    if raw_plan.is_empty() {
        return Ok(None);
    }

    let mut ranges = Vec::new();
    let mut covered_pages = BTreeSet::new();
    for raw_segment in raw_plan.split(',') {
        let segment = raw_segment.trim();
        if segment.is_empty() {
            return Err(format!(
                "{DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV} contains an empty segment"
            ));
        }
        let parts = segment.split(':').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(format!(
                "{DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV} segment `{segment}` must use 1-based inclusive start:end"
            ));
        }
        let start_one_based = parts[0].trim().parse::<u32>().map_err(|_| {
            format!(
                "{DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV} segment `{segment}` has an invalid start page"
            )
        })?;
        let end_one_based = parts[1].trim().parse::<u32>().map_err(|_| {
            format!(
                "{DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV} segment `{segment}` has an invalid end page"
            )
        })?;
        if start_one_based == 0 || end_one_based == 0 || start_one_based > end_one_based {
            return Err(format!(
                "{DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV} segment `{segment}` must be a non-empty 1-based range"
            ));
        }
        let start = start_one_based - 1;
        let end = end_one_based - 1;
        let matching_pages = pages.range(start..=end).copied().collect::<Vec<_>>();
        let expected_page_count = u64::from(end - start) + 1;
        if u64::try_from(matching_pages.len()).unwrap_or(u64::MAX) != expected_page_count {
            return Err(format!(
                "{DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV} segment `{segment}` includes pages outside the Docling fallback set"
            ));
        }
        for page in matching_pages {
            if !covered_pages.insert(page) {
                return Err(format!(
                    "{DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV} segment `{segment}` overlaps another segment at page {}",
                    page.saturating_add(1)
                ));
            }
        }
        ranges.push((start, end));
    }

    if let Some(missing_page) = pages.difference(&covered_pages).next().copied() {
        return Err(format!(
            "{DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN_ENV} does not cover fallback page {}",
            missing_page.saturating_add(1)
        ));
    }
    ranges.sort_unstable();
    Ok(Some(ranges))
}

pub(super) fn docling_page_range_chunk_size_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<u32> {
    lookup(DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_SIZE_ENV)
        .and_then(|value| value.trim().parse::<u32>().ok())
        .filter(|value| *value > 0)
}

#[cfg(test)]
pub(super) fn docling_page_range_chunk_size_for_planner_with_lookup(
    planner: HybridPdfOcrProfilePlanner,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<u32> {
    docling_page_range_chunk_size_with_lookup(lookup).or(match planner {
        HybridPdfOcrProfilePlanner::DoclingStructureRecovery => {
            Some(DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE)
        }
        _ => None,
    })
}

pub(super) fn docling_page_range_chunk_size_for_pages_with_lookup(
    pages: &BTreeSet<u32>,
    planner: HybridPdfOcrProfilePlanner,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<u32> {
    if let Some(explicit_chunk_size) = docling_page_range_chunk_size_with_lookup(lookup) {
        return Some(explicit_chunk_size);
    }
    match planner {
        HybridPdfOcrProfilePlanner::DoclingStructureRecovery
            if pages.len() <= DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_THRESHOLD =>
        {
            Some(DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_CHUNK_SIZE)
        }
        HybridPdfOcrProfilePlanner::DoclingStructureRecovery => {
            Some(DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE)
        }
        _ => None,
    }
}

pub(super) fn docling_page_range_chunk_concurrency_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<usize> {
    lookup(DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_CONCURRENCY_ENV)
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
}

pub(super) fn docling_page_range_chunk_concurrency_limit_with_lookup(
    page_range_count: usize,
    endpoint_count: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> usize {
    let page_range_count = page_range_count.max(1);
    docling_page_range_chunk_concurrency_with_lookup(lookup)
        .unwrap_or_else(|| endpoint_count.max(1).min(page_range_count))
        .min(page_range_count)
        .max(1)
}

pub(super) fn docling_page_range_hedge_delay_ms_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<u64> {
    lookup(DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_HEDGE_DELAY_MS_ENV)
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
}

pub(super) fn docling_structure_recovery_page_range_fallback_pages(
    inputs: &[PdfOcrShardInput],
    docling_structure_recovery: bool,
) -> BTreeSet<u32> {
    if !docling_structure_recovery {
        return BTreeSet::new();
    }
    inputs
        .iter()
        .filter(|input| input.shard_type == "page" && input.ocr_profile == PDF_OCR_DEFAULT_PROFILE)
        .map(|input| input.page_index)
        .collect()
}

pub(super) fn failed_backend_text_page_indices(
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
) -> BTreeSet<u32> {
    inputs
        .iter()
        .zip(results.iter())
        .filter(|(input, result)| {
            input.shard_type == "page"
                && input.ocr_profile == PDF_OCR_BACKEND_TEXT_PROFILE
                && result.status != PdfOcrShardResultStatus::Succeeded
        })
        .map(|(input, _)| input.page_index)
        .collect()
}

pub(super) fn docling_page_range_fallback_page_indices(
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
    docling_structure_recovery: bool,
) -> BTreeSet<u32> {
    let mut pages = failed_backend_text_page_indices(inputs, results);
    if docling_structure_recovery {
        pages.extend(docling_structure_recovery_page_range_fallback_pages(
            inputs,
            docling_structure_recovery,
        ));
    }
    pages
}

pub(super) fn scheduled_inputs_without_docling_page_range_fallback_pages(
    inputs: Vec<PdfOcrShardInput>,
    fallback_pages: &BTreeSet<u32>,
) -> Vec<PdfOcrShardInput> {
    inputs
        .into_iter()
        .filter(|input| !(fallback_pages.contains(&input.page_index) && input.shard_type == "page"))
        .collect()
}

pub(super) fn kept_results_without_docling_page_range_fallback_pages(
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
    fallback_pages: &BTreeSet<u32>,
) -> (Vec<PdfOcrShardInput>, Vec<PdfOcrShardResult>) {
    inputs
        .iter()
        .cloned()
        .zip(results.iter().cloned())
        .filter(|(input, _)| {
            !(fallback_pages.contains(&input.page_index) && input.shard_type == "page")
        })
        .unzip()
}

pub(super) fn docling_page_range_fallback_allows_input(
    input: &PdfOcrShardInput,
    docling_structure_recovery: bool,
) -> bool {
    if input.shard_type != "page" {
        return false;
    }
    if !docling_structure_recovery {
        return input.ocr_profile == PDF_OCR_BACKEND_TEXT_PROFILE;
    }
    matches!(
        input.ocr_profile.as_str(),
        PDF_OCR_DEFAULT_PROFILE | PDF_OCR_BACKEND_TEXT_PROFILE | PDF_OCR_FAST_TEXT_PROFILE
    )
}

pub(super) fn has_unhandled_non_success_result(
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
    fallback_pages: &BTreeSet<u32>,
    docling_structure_recovery: bool,
) -> bool {
    inputs.iter().zip(results.iter()).any(|(input, result)| {
        result.status != PdfOcrShardResultStatus::Succeeded
            && !(fallback_pages.contains(&input.page_index)
                && docling_page_range_fallback_allows_input(input, docling_structure_recovery))
    })
}

pub(super) fn has_region_shard_on_pages(
    inputs: &[PdfOcrShardInput],
    pages: &BTreeSet<u32>,
) -> bool {
    inputs
        .iter()
        .any(|input| input.shard_type == "region" && pages.contains(&input.page_index))
}
