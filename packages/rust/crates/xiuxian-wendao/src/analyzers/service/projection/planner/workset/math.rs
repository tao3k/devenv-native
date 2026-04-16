use crate::analyzers::DocsPlannerWorksetQuotaHint;

pub(crate) fn quota_hint_for_selection(
    selected_count: usize,
    target_floor_count: usize,
    target_ceiling_count: usize,
) -> DocsPlannerWorksetQuotaHint {
    DocsPlannerWorksetQuotaHint {
        target_floor_count,
        target_ceiling_count,
        within_target_band: selected_count >= target_floor_count
            && selected_count <= target_ceiling_count,
    }
}

pub(crate) fn empty_quota_hint() -> DocsPlannerWorksetQuotaHint {
    DocsPlannerWorksetQuotaHint {
        target_floor_count: 0,
        target_ceiling_count: 0,
        within_target_band: true,
    }
}

pub(crate) fn spread_for_counts(counts: &[usize]) -> usize {
    let Some(maximum) = counts.iter().max().copied() else {
        return 0;
    };
    let Some(minimum) = counts.iter().min().copied() else {
        return 0;
    };
    maximum.saturating_sub(minimum)
}

pub(crate) fn quota_band(selection_count: usize, group_count: usize) -> (usize, usize) {
    if group_count == 0 {
        return (0, 0);
    }
    let floor = selection_count / group_count;
    let ceiling = if selection_count.is_multiple_of(group_count) {
        floor
    } else {
        floor + 1
    };
    (floor, ceiling)
}
