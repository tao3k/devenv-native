use axum::http::StatusCode;

use crate::studio::StudioApiError;
use crate::studio::types::{SearchHit, SourceSymbolHit};

pub(super) fn local_symbol_hit_to_search_hit(hit: SourceSymbolHit, code_biased: bool) -> SearchHit {
    let mut tags = vec![
        hit.crate_name.clone(),
        "code".to_string(),
        "symbol".to_string(),
        hit.language.clone(),
        format!("lang:{}", hit.language),
    ];
    if let Some(node_kind) = hit
        .node_kind
        .as_ref()
        .map(crate::contracts::StudioContractNodeKind::as_str)
    {
        tags.push(node_kind.to_string());
        tags.push(format!("kind:{node_kind}"));
    } else {
        tags.push("kind:symbol".to_string());
    }
    if let Some(project_name) = hit.project_name.as_deref() {
        tags.push(project_name.to_string());
    }

    let best_section = if hit.signature.trim().is_empty() {
        hit.owner_title.clone()
    } else {
        Some(hit.signature.clone())
    };

    SearchHit {
        stem: hit.name.clone(),
        title: Some(hit.name),
        path: hit.path.clone().into(),
        doc_type: Some("symbol".into()),
        tags: tags.into_iter().map(Into::into).collect(),
        score: normalize_local_symbol_score(hit.score, code_biased),
        best_section,
        match_reason: Some("local_symbol_search".to_string()),
        hierarchical_uri: None,
        hierarchy: Some(hit.path.split('/').map(str::to_string).collect::<Vec<_>>()),
        saliency_score: None,
        audit_status: None,
        verification_state: None,
        implicit_backlinks: None,
        implicit_backlink_items: None,
        navigation_target: Some(hit.navigation_target),
    }
}

pub(super) fn repo_content_hit_to_intent_hit(mut hit: SearchHit, code_biased: bool) -> SearchHit {
    if code_biased {
        hit.score = (hit.score + 0.04).min(0.9);
    }
    hit
}

fn normalize_local_symbol_score(score: f64, code_biased: bool) -> f64 {
    if code_biased {
        (score + 0.02).min(1.0)
    } else {
        score
    }
}

pub(super) fn compare_intent_hits(left: &SearchHit, right: &SearchHit) -> std::cmp::Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| intent_hit_priority(right).cmp(&intent_hit_priority(left)))
        .then_with(|| left.path.cmp(&right.path))
        .then_with(|| left.stem.cmp(&right.stem))
}

fn intent_hit_priority(hit: &SearchHit) -> u8 {
    match hit.doc_type.as_ref().map(AsRef::as_ref) {
        Some("symbol") => 3,
        Some("file") => 2,
        _ => 1,
    }
}

pub(super) fn intent_candidate_limit(limit: usize) -> usize {
    limit.saturating_mul(2).max(8)
}

pub(super) fn is_code_biased_intent(
    intent: Option<&str>,
    query_text: &str,
    repo_hint: Option<&str>,
) -> bool {
    if repo_hint.is_some() {
        return true;
    }

    let normalized = intent.unwrap_or_default().to_ascii_lowercase();
    if [
        "code",
        "debug",
        "symbol",
        "definition",
        "reference",
        "implement",
        "trace",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
    {
        return true;
    }

    query_text.contains("lang:")
        || query_text.contains("kind:")
        || query_text.contains("repo:")
        || query_text
            .chars()
            .any(|ch| matches!(ch, '_' | ':' | '(' | ')' | '/' | '@'))
}

pub(super) fn is_index_not_ready(error: &StudioApiError) -> bool {
    error.status() == StatusCode::CONFLICT && error.code() == "INDEX_NOT_READY"
}
