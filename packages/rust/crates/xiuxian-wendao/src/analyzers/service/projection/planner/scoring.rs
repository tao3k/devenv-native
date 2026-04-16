use crate::analyzers::projection::ProjectionPageKind;
use crate::analyzers::{
    DocsPlannerRankReason, DocsPlannerRankReasonCode, ProjectedGapKind, ProjectedGapRecord,
};

pub(super) fn planner_gap_search_score(gap: &ProjectedGapRecord, normalized_query: &str) -> u8 {
    let mut score = 0_u8;
    score = score.max(match_field_score(
        gap.title.as_str(),
        normalized_query,
        90,
        70,
    ));
    score = score.max(match_field_score(
        gap.path.as_str(),
        normalized_query,
        88,
        68,
    ));
    score = score.max(match_field_score(
        gap.entity_id.as_str(),
        normalized_query,
        86,
        66,
    ));
    score = score.max(match_field_score(
        gap.gap_id.as_str(),
        normalized_query,
        84,
        64,
    ));
    score = score.max(match_field_score(
        gap_kind_token(gap.kind),
        normalized_query,
        82,
        62,
    ));
    score = score.max(match_field_score(
        projection_page_kind_token(gap.page_kind),
        normalized_query,
        80,
        60,
    ));
    for format_hint in &gap.format_hints {
        score = score.max(match_field_score(
            format_hint.as_str(),
            normalized_query,
            72,
            52,
        ));
    }
    score
}

pub(super) fn planner_gap_priority_breakdown(
    gap: &ProjectedGapRecord,
) -> (u8, Vec<DocsPlannerRankReason>) {
    let mut score: u8 = match gap.kind {
        ProjectedGapKind::ModuleReferenceWithoutDocumentation => 96,
        ProjectedGapKind::SymbolReferenceWithoutDocumentation => 92,
        ProjectedGapKind::DocumentationPageWithoutAnchor => 88,
        ProjectedGapKind::ExampleHowToWithoutAnchor => 84,
        ProjectedGapKind::SymbolReferenceUnverified => 72,
    };
    let mut reasons = vec![DocsPlannerRankReason {
        code: DocsPlannerRankReasonCode::GapKindBase,
        points: score,
        detail: format!(
            "base priority from projected gap kind `{}`",
            gap_kind_token(gap.kind)
        ),
    }];

    let page_bonus = match gap.page_kind {
        ProjectionPageKind::Reference => Some((
            DocsPlannerRankReasonCode::ReferencePageBonus,
            2_u8,
            "reference page bonus",
        )),
        ProjectionPageKind::Explanation => Some((
            DocsPlannerRankReasonCode::ExplanationPageBonus,
            1_u8,
            "explanation page bonus",
        )),
        ProjectionPageKind::HowTo | ProjectionPageKind::Tutorial => None,
    };
    if let Some((code, points, detail)) = page_bonus {
        score = score.saturating_add(points);
        reasons.push(DocsPlannerRankReason {
            code,
            points,
            detail: detail.to_string(),
        });
    }

    let module_bonus = u8::try_from(gap.module_ids.len().min(2)).unwrap_or(0);
    if module_bonus > 0 {
        score = score.saturating_add(module_bonus);
        reasons.push(DocsPlannerRankReason {
            code: DocsPlannerRankReasonCode::ModuleAnchorBonus,
            points: module_bonus,
            detail: format!("{} attached module anchor(s)", gap.module_ids.len()),
        });
    }

    let symbol_bonus = u8::try_from(gap.symbol_ids.len().min(2)).unwrap_or(0);
    if symbol_bonus > 0 {
        score = score.saturating_add(symbol_bonus);
        reasons.push(DocsPlannerRankReason {
            code: DocsPlannerRankReasonCode::SymbolAnchorBonus,
            points: symbol_bonus,
            detail: format!("{} attached symbol anchor(s)", gap.symbol_ids.len()),
        });
    }

    let example_bonus = u8::from(!gap.example_ids.is_empty());
    if example_bonus > 0 {
        score = score.saturating_add(example_bonus);
        reasons.push(DocsPlannerRankReason {
            code: DocsPlannerRankReasonCode::ExampleAnchorBonus,
            points: example_bonus,
            detail: format!("{} attached example anchor(s)", gap.example_ids.len()),
        });
    }

    let doc_bonus = u8::from(!gap.doc_ids.is_empty());
    if doc_bonus > 0 {
        score = score.saturating_add(doc_bonus);
        reasons.push(DocsPlannerRankReason {
            code: DocsPlannerRankReasonCode::DocAnchorBonus,
            points: doc_bonus,
            detail: format!("{} attached documentation anchor(s)", gap.doc_ids.len()),
        });
    }

    (score.min(100), reasons)
}

pub(super) fn normalize_planner_search_text(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

pub(super) fn match_field_score(
    field: &str,
    normalized_query: &str,
    exact: u8,
    contains: u8,
) -> u8 {
    let normalized_field = field.to_ascii_lowercase();
    if normalized_field == normalized_query {
        exact
    } else if normalized_field.contains(normalized_query) {
        contains
    } else {
        0
    }
}

pub(super) fn gap_kind_token(kind: ProjectedGapKind) -> &'static str {
    match kind {
        ProjectedGapKind::ModuleReferenceWithoutDocumentation => {
            "module_reference_without_documentation"
        }
        ProjectedGapKind::SymbolReferenceWithoutDocumentation => {
            "symbol_reference_without_documentation"
        }
        ProjectedGapKind::SymbolReferenceUnverified => "symbol_reference_unverified",
        ProjectedGapKind::ExampleHowToWithoutAnchor => "example_how_to_without_anchor",
        ProjectedGapKind::DocumentationPageWithoutAnchor => "documentation_page_without_anchor",
    }
}

pub(super) fn projection_page_kind_token(kind: ProjectionPageKind) -> &'static str {
    match kind {
        ProjectionPageKind::Reference => "reference",
        ProjectionPageKind::HowTo => "how_to",
        ProjectionPageKind::Tutorial => "tutorial",
        ProjectionPageKind::Explanation => "explanation",
    }
}
