use crate::analyzers::projection::ProjectionPageKind;
use crate::analyzers::{
    DocsPlannerWorksetBalance, DocsPlannerWorksetFamilyBalanceEntry,
    DocsPlannerWorksetGapKindBalanceEntry, DocsPlannerWorksetStrategy,
    DocsPlannerWorksetStrategyCode, DocsPlannerWorksetStrategyReasonCode, ProjectedGapKind,
};

use super::scoring::{match_field_score, normalize_planner_search_text};
use super::workset::{build_docs_planner_workset_strategy, quota_band};

#[test]
fn normalize_planner_search_text_trims_and_lowercases() {
    assert_eq!(
        normalize_planner_search_text("  Planner Search  "),
        "planner search"
    );
}

#[test]
fn match_field_score_distinguishes_exact_contains_and_miss() {
    assert_eq!(
        match_field_score("Planner Search", "planner search", 91, 71),
        91
    );
    assert_eq!(match_field_score("Planner Search", "search", 91, 71), 71);
    assert_eq!(match_field_score("Planner Search", "missing", 91, 71), 0);
}

#[test]
fn quota_band_handles_empty_and_evenly_split_groups() {
    assert_eq!(quota_band(0, 0), (0, 0));
    assert_eq!(quota_band(4, 2), (2, 2));
    assert_eq!(quota_band(5, 2), (2, 3));
}

#[test]
fn build_docs_planner_workset_strategy_marks_balanced_multi_lane() {
    let balance = DocsPlannerWorksetBalance {
        selection_count: 4,
        gap_kind_group_count: 2,
        family_group_count: 2,
        gap_kind_target_floor_count: 2,
        gap_kind_target_ceiling_count: 2,
        family_target_floor_count: 2,
        family_target_ceiling_count: 2,
        gap_kind_distribution: vec![
            DocsPlannerWorksetGapKindBalanceEntry {
                kind: ProjectedGapKind::ModuleReferenceWithoutDocumentation,
                selected_count: 2,
                within_target_band: true,
            },
            DocsPlannerWorksetGapKindBalanceEntry {
                kind: ProjectedGapKind::SymbolReferenceWithoutDocumentation,
                selected_count: 2,
                within_target_band: true,
            },
        ],
        family_distribution: vec![
            DocsPlannerWorksetFamilyBalanceEntry {
                kind: ProjectionPageKind::Reference,
                selected_count: 2,
                within_target_band: true,
            },
            DocsPlannerWorksetFamilyBalanceEntry {
                kind: ProjectionPageKind::Explanation,
                selected_count: 2,
                within_target_band: true,
            },
        ],
        gap_kind_spread: 0,
        family_spread: 0,
        gap_kind_balanced: true,
        family_balanced: true,
    };

    let strategy = build_docs_planner_workset_strategy(&balance);
    assert_eq!(
        strategy.code,
        DocsPlannerWorksetStrategyCode::BalancedMultiLane
    );
    assert_eq!(strategy.gap_kind_group_count, 2);
    assert_eq!(strategy.family_group_count, 2);
    assert!(
        strategy
            .reasons
            .iter()
            .any(|reason| reason.code == DocsPlannerWorksetStrategyReasonCode::GapKindBalanced)
    );
    assert!(
        strategy
            .reasons
            .iter()
            .any(|reason| reason.code == DocsPlannerWorksetStrategyReasonCode::FamilyBalanced)
    );
}

#[test]
fn build_docs_planner_workset_strategy_handles_empty_selection() {
    let balance = DocsPlannerWorksetBalance {
        selection_count: 0,
        gap_kind_group_count: 0,
        family_group_count: 0,
        gap_kind_target_floor_count: 0,
        gap_kind_target_ceiling_count: 0,
        family_target_floor_count: 0,
        family_target_ceiling_count: 0,
        gap_kind_distribution: Vec::new(),
        family_distribution: Vec::new(),
        gap_kind_spread: 0,
        family_spread: 0,
        gap_kind_balanced: true,
        family_balanced: true,
    };

    let strategy: DocsPlannerWorksetStrategy = build_docs_planner_workset_strategy(&balance);
    assert_eq!(
        strategy.code,
        DocsPlannerWorksetStrategyCode::EmptySelection
    );
    assert_eq!(strategy.reasons.len(), 1);
    assert_eq!(
        strategy.reasons[0].code,
        DocsPlannerWorksetStrategyReasonCode::EmptySelection
    );
}
