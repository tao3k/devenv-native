use std::collections::BTreeMap;

use crate::analyzers::projection::ProjectionPageKind;
use crate::analyzers::service::projection::planner::workset::math::{
    quota_band, spread_for_counts,
};
use crate::analyzers::{
    DocsPlannerWorksetBalance, DocsPlannerWorksetFamilyBalanceEntry,
    DocsPlannerWorksetGapKindBalanceEntry, DocsPlannerWorksetGroup,
};

pub(super) fn build_docs_planner_workset_balance(
    groups: &[DocsPlannerWorksetGroup],
) -> DocsPlannerWorksetBalance {
    let selection_count = groups
        .iter()
        .map(|group| group.selected_count)
        .sum::<usize>();
    let gap_kind_distribution = groups
        .iter()
        .map(|group| (group.kind, group.selected_count))
        .collect::<Vec<_>>();
    let (gap_kind_target_floor_count, gap_kind_target_ceiling_count) =
        quota_band(selection_count, gap_kind_distribution.len());
    let gap_kind_distribution = gap_kind_distribution
        .into_iter()
        .map(
            |(kind, selected_count)| DocsPlannerWorksetGapKindBalanceEntry {
                kind,
                selected_count,
                within_target_band: selected_count >= gap_kind_target_floor_count
                    && selected_count <= gap_kind_target_ceiling_count,
            },
        )
        .collect::<Vec<_>>();

    let mut family_counts = BTreeMap::<ProjectionPageKind, usize>::new();
    for group in groups {
        for family in &group.families {
            *family_counts.entry(family.kind).or_default() += family.selected_count;
        }
    }
    let family_distribution = family_counts.into_iter().collect::<Vec<_>>();
    let (family_target_floor_count, family_target_ceiling_count) =
        quota_band(selection_count, family_distribution.len());
    let mut family_distribution = family_distribution
        .into_iter()
        .map(
            |(kind, selected_count)| DocsPlannerWorksetFamilyBalanceEntry {
                kind,
                selected_count,
                within_target_band: selected_count >= family_target_floor_count
                    && selected_count <= family_target_ceiling_count,
            },
        )
        .collect::<Vec<_>>();
    family_distribution.sort_by(|left, right| {
        right
            .selected_count
            .cmp(&left.selected_count)
            .then_with(|| left.kind.cmp(&right.kind))
    });

    let gap_kind_counts = gap_kind_distribution
        .iter()
        .map(|entry| entry.selected_count)
        .collect::<Vec<_>>();
    let family_counts = family_distribution
        .iter()
        .map(|entry| entry.selected_count)
        .collect::<Vec<_>>();
    let gap_kind_spread = spread_for_counts(&gap_kind_counts);
    let family_spread = spread_for_counts(&family_counts);

    DocsPlannerWorksetBalance {
        selection_count,
        gap_kind_group_count: gap_kind_distribution.len(),
        family_group_count: family_distribution.len(),
        gap_kind_target_floor_count,
        gap_kind_target_ceiling_count,
        family_target_floor_count,
        family_target_ceiling_count,
        gap_kind_balanced: gap_kind_spread <= 1,
        family_balanced: family_spread <= 1,
        gap_kind_distribution,
        family_distribution,
        gap_kind_spread,
        family_spread,
    }
}
