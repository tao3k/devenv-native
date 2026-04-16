use std::collections::BTreeMap;

use crate::analyzers::projection::ProjectionPageKind;
use crate::analyzers::service::projection::planner::workset::math::{
    empty_quota_hint, quota_hint_for_selection,
};
use crate::analyzers::{
    DocsPlannerItemResult, DocsPlannerRankHit, DocsPlannerWorksetFamilyGroup,
    DocsPlannerWorksetGroup,
};

pub(super) fn build_planner_workset_groups(
    ranked_hits: &[DocsPlannerRankHit],
    items: &[DocsPlannerItemResult],
) -> Vec<DocsPlannerWorksetGroup> {
    let mut groups = initial_planner_workset_groups(ranked_hits, items);
    populate_family_groups(&mut groups);
    apply_gap_kind_quota_hints(&mut groups);
    groups
}

fn initial_planner_workset_groups(
    ranked_hits: &[DocsPlannerRankHit],
    items: &[DocsPlannerItemResult],
) -> Vec<DocsPlannerWorksetGroup> {
    let mut grouped =
        BTreeMap::<crate::analyzers::ProjectedGapKind, DocsPlannerWorksetGroup>::new();

    for (ranked_hit, item) in ranked_hits.iter().cloned().zip(items.iter().cloned()) {
        let entry = grouped
            .entry(ranked_hit.gap.kind)
            .or_insert_with(|| DocsPlannerWorksetGroup {
                kind: ranked_hit.gap.kind,
                selected_count: 0,
                quota: empty_quota_hint(),
                families: Vec::new(),
                ranked_hits: Vec::new(),
                items: Vec::new(),
            });
        entry.selected_count += 1;
        entry.ranked_hits.push(ranked_hit);
        entry.items.push(item);
    }

    let mut groups = grouped.into_values().collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .selected_count
            .cmp(&left.selected_count)
            .then_with(|| left.kind.cmp(&right.kind))
    });
    groups
}

fn populate_family_groups(groups: &mut [DocsPlannerWorksetGroup]) {
    for group in groups {
        group.families = family_groups_for_workset_group(group);
    }
}

fn family_groups_for_workset_group(
    group: &DocsPlannerWorksetGroup,
) -> Vec<DocsPlannerWorksetFamilyGroup> {
    let mut families = BTreeMap::<ProjectionPageKind, DocsPlannerWorksetFamilyGroup>::new();

    for (ranked_hit, item) in group
        .ranked_hits
        .iter()
        .cloned()
        .zip(group.items.iter().cloned())
    {
        let entry = families.entry(ranked_hit.gap.page_kind).or_insert_with(|| {
            DocsPlannerWorksetFamilyGroup {
                kind: ranked_hit.gap.page_kind,
                selected_count: 0,
                quota: empty_quota_hint(),
                ranked_hits: Vec::new(),
                items: Vec::new(),
            }
        });
        entry.selected_count += 1;
        entry.ranked_hits.push(ranked_hit);
        entry.items.push(item);
    }

    let mut family_groups = families.into_values().collect::<Vec<_>>();
    apply_family_quota_hints(group.selected_count, &mut family_groups);
    family_groups.sort_by(|left, right| {
        right
            .selected_count
            .cmp(&left.selected_count)
            .then_with(|| left.kind.cmp(&right.kind))
    });
    family_groups
}

fn apply_family_quota_hints(
    selection_count: usize,
    families: &mut [DocsPlannerWorksetFamilyGroup],
) {
    let (target_floor_count, target_ceiling_count) =
        crate::analyzers::service::projection::planner::workset::math::quota_band(
            selection_count,
            families.len(),
        );
    for family in families {
        family.quota = quota_hint_for_selection(
            family.selected_count,
            target_floor_count,
            target_ceiling_count,
        );
    }
}

fn apply_gap_kind_quota_hints(groups: &mut [DocsPlannerWorksetGroup]) {
    let selection_count = groups
        .iter()
        .map(|group| group.selected_count)
        .sum::<usize>();
    let (target_floor_count, target_ceiling_count) =
        crate::analyzers::service::projection::planner::workset::math::quota_band(
            selection_count,
            groups.len(),
        );
    for group in groups {
        group.quota = quota_hint_for_selection(
            group.selected_count,
            target_floor_count,
            target_ceiling_count,
        );
    }
}
