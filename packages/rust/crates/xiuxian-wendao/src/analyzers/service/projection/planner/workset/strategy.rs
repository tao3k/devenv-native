use crate::analyzers::{
    DocsPlannerWorksetBalance, DocsPlannerWorksetStrategy, DocsPlannerWorksetStrategyCode,
    DocsPlannerWorksetStrategyReason, DocsPlannerWorksetStrategyReasonCode,
};

pub(crate) fn build_docs_planner_workset_strategy(
    balance: &DocsPlannerWorksetBalance,
) -> DocsPlannerWorksetStrategy {
    let code = if balance.selection_count == 0 {
        DocsPlannerWorksetStrategyCode::EmptySelection
    } else if balance.gap_kind_group_count == 1 && balance.family_group_count == 1 {
        DocsPlannerWorksetStrategyCode::SingleLaneFocus
    } else if balance.gap_kind_group_count == 1 {
        DocsPlannerWorksetStrategyCode::FamilySplitFocus
    } else if balance.family_group_count == 1 {
        DocsPlannerWorksetStrategyCode::GapKindSplitFocus
    } else if balance.gap_kind_balanced && balance.family_balanced {
        DocsPlannerWorksetStrategyCode::BalancedMultiLane
    } else {
        DocsPlannerWorksetStrategyCode::PriorityStacked
    };

    let mut reasons = Vec::new();
    if balance.selection_count == 0 {
        reasons.push(DocsPlannerWorksetStrategyReason {
            code: DocsPlannerWorksetStrategyReasonCode::EmptySelection,
            detail: "no ranked gaps were selected into the workset".to_string(),
        });
    } else {
        let gap_reason = if balance.gap_kind_group_count == 1 {
            DocsPlannerWorksetStrategyReasonCode::SingleGapKind
        } else {
            DocsPlannerWorksetStrategyReasonCode::MultipleGapKinds
        };
        reasons.push(DocsPlannerWorksetStrategyReason {
            code: gap_reason,
            detail: format!(
                "{} populated gap-kind group(s) contribute to the workset",
                balance.gap_kind_group_count
            ),
        });

        let family_reason = if balance.family_group_count == 1 {
            DocsPlannerWorksetStrategyReasonCode::SingleFamily
        } else {
            DocsPlannerWorksetStrategyReasonCode::MultipleFamilies
        };
        reasons.push(DocsPlannerWorksetStrategyReason {
            code: family_reason,
            detail: format!(
                "{} populated page-family group(s) contribute to the workset",
                balance.family_group_count
            ),
        });

        let gap_balance_reason = if balance.gap_kind_balanced {
            DocsPlannerWorksetStrategyReasonCode::GapKindBalanced
        } else {
            DocsPlannerWorksetStrategyReasonCode::GapKindStacked
        };
        reasons.push(DocsPlannerWorksetStrategyReason {
            code: gap_balance_reason,
            detail: if balance.gap_kind_balanced {
                "gap-kind groups stay within the deterministic balance band".to_string()
            } else {
                "gap-kind groups exceed the deterministic balance band".to_string()
            },
        });

        let family_balance_reason = if balance.family_balanced {
            DocsPlannerWorksetStrategyReasonCode::FamilyBalanced
        } else {
            DocsPlannerWorksetStrategyReasonCode::FamilyStacked
        };
        reasons.push(DocsPlannerWorksetStrategyReason {
            code: family_balance_reason,
            detail: if balance.family_balanced {
                "page-family groups stay within the deterministic balance band".to_string()
            } else {
                "page-family groups exceed the deterministic balance band".to_string()
            },
        });
    }

    DocsPlannerWorksetStrategy {
        code,
        gap_kind_group_count: balance.gap_kind_group_count,
        family_group_count: balance.family_group_count,
        reasons,
    }
}
