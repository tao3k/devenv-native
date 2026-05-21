use std::collections::BTreeMap;

use super::{
    DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_STRUCTURE_COST_BUDGET_ENV, PdfSourcePageProfile,
    pdf_source_page_structure_cost,
};

pub(super) fn docling_page_range_structure_cost_budgeted_ranges_with_lookup(
    ranges: Vec<(u32, u32)>,
    profiles: &[PdfSourcePageProfile],
    max_range_count: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Vec<(u32, u32)> {
    let Some(budget) = docling_page_range_structure_cost_budget_with_lookup(lookup) else {
        return ranges;
    };
    structure_cost_budgeted_docling_page_range_fallback_ranges_with_limit(
        ranges.as_slice(),
        profiles,
        budget,
        max_range_count,
    )
}

fn docling_page_range_structure_cost_budget_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<u32> {
    lookup(DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_STRUCTURE_COST_BUDGET_ENV)
        .and_then(|value| value.trim().parse::<u32>().ok())
        .filter(|value| *value > 0)
}

#[cfg(test)]
pub(super) fn structure_cost_budgeted_docling_page_range_fallback_ranges(
    ranges: &[(u32, u32)],
    profiles: &[PdfSourcePageProfile],
    budget: u32,
) -> Vec<(u32, u32)> {
    structure_cost_budgeted_docling_page_range_fallback_ranges_with_limit(
        ranges,
        profiles,
        budget,
        usize::MAX,
    )
}

pub(super) fn structure_cost_budgeted_docling_page_range_fallback_ranges_with_limit(
    ranges: &[(u32, u32)],
    profiles: &[PdfSourcePageProfile],
    budget: u32,
    max_range_count: usize,
) -> Vec<(u32, u32)> {
    let profile_costs = profiles
        .iter()
        .map(|profile| {
            (
                profile.page_index,
                pdf_source_page_structure_cost(profile).max(1),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if profile_costs.is_empty() {
        return ranges.to_vec();
    }
    let max_range_count = max_range_count.max(ranges.len());
    if ranges.len() >= max_range_count {
        return ranges.to_vec();
    }

    let mut budgeted_ranges = ranges.to_vec();
    while budgeted_ranges.len() < max_range_count {
        let Some(candidate) = best_structure_cost_budget_split_candidate(
            budgeted_ranges.as_slice(),
            &profile_costs,
            budget,
        ) else {
            break;
        };
        let (start, end) = budgeted_ranges[candidate.range_index];
        budgeted_ranges.splice(
            candidate.range_index..=candidate.range_index,
            [
                (start, candidate.split_page),
                (candidate.split_page.saturating_add(1), end),
            ],
        );
    }
    budgeted_ranges
}

#[derive(Debug, Clone, Copy)]
struct StructureCostSplitCandidate {
    range_index: usize,
    split_page: u32,
    original_cost: u64,
    candidate_max_cost: u64,
}

fn best_structure_cost_budget_split_candidate(
    ranges: &[(u32, u32)],
    profile_costs: &BTreeMap<u32, u32>,
    budget: u32,
) -> Option<StructureCostSplitCandidate> {
    ranges
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(range_index, (start, end))| {
            split_candidate_for_range(range_index, start, end, profile_costs, budget)
        })
        .max_by_key(|candidate| {
            (
                candidate
                    .original_cost
                    .saturating_sub(candidate.candidate_max_cost),
                candidate.original_cost,
            )
        })
}

fn split_candidate_for_range(
    range_index: usize,
    start: u32,
    end: u32,
    profile_costs: &BTreeMap<u32, u32>,
    budget: u32,
) -> Option<StructureCostSplitCandidate> {
    if start >= end {
        return None;
    }
    let page_weights = page_weights_for_range(start, end, profile_costs);
    let original_cost = page_weights_total(page_weights.as_slice());
    if original_cost <= u64::from(budget) {
        return None;
    }
    let split_page = best_structure_cost_budget_split_page(page_weights.as_slice())?;
    let left_cost = range_structure_cost(start, split_page, profile_costs);
    let right_cost = range_structure_cost(split_page.saturating_add(1), end, profile_costs);
    Some(StructureCostSplitCandidate {
        range_index,
        split_page,
        original_cost,
        candidate_max_cost: left_cost.max(right_cost),
    })
}

fn page_weights_for_range(
    start: u32,
    end: u32,
    profile_costs: &BTreeMap<u32, u32>,
) -> Vec<(u32, u32)> {
    (start..=end)
        .map(|page| (page, *profile_costs.get(&page).unwrap_or(&1)))
        .collect()
}

fn range_structure_cost(start: u32, end: u32, profile_costs: &BTreeMap<u32, u32>) -> u64 {
    page_weights_total(page_weights_for_range(start, end, profile_costs).as_slice())
}

fn page_weights_total(page_weights: &[(u32, u32)]) -> u64 {
    page_weights.iter().map(|(_, cost)| u64::from(*cost)).sum()
}

fn best_structure_cost_budget_split_page(page_weights: &[(u32, u32)]) -> Option<u32> {
    if page_weights.len() < 2 {
        return None;
    }
    let total = page_weights
        .iter()
        .map(|(_, cost)| u64::from(*cost))
        .sum::<u64>();
    page_weights
        .iter()
        .copied()
        .take(page_weights.len().saturating_sub(1))
        .scan(0_u64, |left, (page, cost)| {
            *left = left.saturating_add(u64::from(cost));
            let right = total.saturating_sub(*left);
            Some(((*left).max(right), page))
        })
        .min_by_key(|(candidate_cost, _)| *candidate_cost)
        .map(|(_, page)| page)
}
