//! Semantic projection freshness policy reports.

use std::collections::BTreeSet;
use xiuxian_wendao_parsers::semantic_ssot::SemanticRepository;
use xiuxian_wendao_parsers::{
    SemanticProjectionStaleness, SemanticStatus, semantic_projection_source_revision,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SemanticProjectionFreshnessPolicyReport {
    pub(crate) policy_id: String,
    pub(crate) status: String,
    pub(crate) failing_projection_count: usize,
    pub(crate) message: String,
    pub(crate) projections: Vec<SemanticProjectionFreshnessPolicyEntry>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SemanticProjectionFreshnessPolicyEntry {
    pub(crate) projection: String,
    pub(crate) source_revision: String,
    pub(crate) current_source_revision: Option<String>,
    pub(crate) staleness: String,
    pub(crate) reason: String,
}

pub(crate) fn semantic_projection_freshness_policy_report(
    repository: &SemanticRepository,
) -> SemanticProjectionFreshnessPolicyReport {
    let required_projection_names = repository
        .change_intents
        .iter()
        .filter(|intent| intent.status == SemanticStatus::Active)
        .flat_map(|intent| intent.projections_to_refresh.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let mut projections = repository
        .projections
        .iter()
        .filter(|projection| required_projection_names.contains(projection.projection.as_str()))
        .filter_map(|projection| {
            let current_source_revision =
                semantic_projection_source_revision(repository, projection);
            projection_policy_failure_reason(
                projection.source_revision.as_str(),
                &projection.staleness,
                current_source_revision.as_deref(),
            )
            .map(|reason| SemanticProjectionFreshnessPolicyEntry {
                projection: projection.projection.clone(),
                source_revision: projection.source_revision.clone(),
                current_source_revision,
                staleness: projection_staleness_token(&projection.staleness).to_string(),
                reason: reason.to_string(),
            })
        })
        .collect::<Vec<_>>();
    projections.sort_by(|left, right| left.projection.cmp(&right.projection));
    let failing_projection_count = projections.len();
    let (status, message) = if failing_projection_count == 0 {
        (
            "passed",
            "all active change-intent projection refresh targets are fresh",
        )
    } else {
        (
            "review_required",
            "active change-intent projection refresh target(s) are stale; run `wendao-client lint semantic --refresh-projections --require-fresh-projections`",
        )
    };
    SemanticProjectionFreshnessPolicyReport {
        policy_id: "semantic_projection.required_refresh_targets".to_string(),
        status: status.to_string(),
        failing_projection_count,
        message: message.to_string(),
        projections,
    }
}

fn projection_policy_failure_reason(
    source_revision: &str,
    staleness: &SemanticProjectionStaleness,
    current_source_revision: Option<&str>,
) -> Option<&'static str> {
    if *staleness != SemanticProjectionStaleness::Fresh {
        return Some("stale");
    }
    match current_source_revision {
        Some(current) if current == source_revision.trim() => None,
        Some(_) => Some("source_revision_mismatch"),
        None => Some("unresolved_source_revision"),
    }
}

fn projection_staleness_token(staleness: &SemanticProjectionStaleness) -> &'static str {
    match staleness {
        SemanticProjectionStaleness::Fresh => "fresh",
        SemanticProjectionStaleness::Stale => "stale",
    }
}
