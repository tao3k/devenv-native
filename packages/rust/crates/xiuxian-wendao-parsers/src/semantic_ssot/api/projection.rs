//! Projection freshness and refresh-plan APIs for semantic `SSOT` read models.

use super::hash::{semantic_object_by_id, semantic_projection_source_revision_from_map};
use crate::semantic_ssot::types::{
    SemanticProjection, SemanticProjectionFreshnessPolicyEntry,
    SemanticProjectionFreshnessPolicyReport, SemanticProjectionRefreshPlanEntry,
    SemanticProjectionRefreshPlanReport, SemanticProjectionStaleness, SemanticRepository,
    SemanticStatus,
};
use std::collections::BTreeSet;

/// Stable semantic projection freshness policy identifier.
pub const SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID: &str =
    "semantic_projection.required_refresh_targets";

/// Compute the source revision for one projection from its referenced objects.
#[must_use]
pub fn semantic_projection_source_revision(
    repository: &SemanticRepository,
    projection: &SemanticProjection,
) -> Option<String> {
    let object_by_id = semantic_object_by_id(repository);
    semantic_projection_source_revision_from_map(projection, &object_by_id)
}

/// Build the shared projection freshness policy report for one semantic repository.
#[must_use]
pub fn semantic_projection_freshness_policy_report(
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
            let reason = semantic_projection_policy_failure_reason(
                projection.source_revision.as_str(),
                &projection.staleness,
                current_source_revision.as_deref(),
            )?;
            Some(SemanticProjectionFreshnessPolicyEntry {
                projection: projection.projection.clone(),
                source_revision: projection.source_revision.clone(),
                current_source_revision,
                staleness: semantic_projection_staleness_token(&projection.staleness).to_string(),
                reason: reason.to_string(),
                source_path: semantic_projection_policy_source_path(projection),
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
        policy_id: SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID.to_string(),
        status: status.to_string(),
        failing_projection_count,
        message: message.to_string(),
        projections,
    }
}

/// Build a read-only projection metadata refresh plan for one semantic repository.
#[must_use]
pub fn semantic_projection_refresh_plan_report(
    repository: &SemanticRepository,
) -> SemanticProjectionRefreshPlanReport {
    let mut projections = repository
        .projections
        .iter()
        .filter_map(|projection| {
            let current_source_revision =
                semantic_projection_source_revision(repository, projection)?;
            let reason = semantic_projection_policy_failure_reason(
                projection.source_revision.as_str(),
                &projection.staleness,
                Some(current_source_revision.as_str()),
            )?;
            Some(SemanticProjectionRefreshPlanEntry {
                projection: projection.projection.clone(),
                source_revision: projection.source_revision.clone(),
                current_source_revision,
                staleness: semantic_projection_staleness_token(&projection.staleness).to_string(),
                action: "refresh_source_revision".to_string(),
                reason: reason.to_string(),
                source_path: semantic_projection_policy_source_path(projection),
            })
        })
        .collect::<Vec<_>>();
    projections.sort_by(|left, right| left.projection.cmp(&right.projection));
    let refreshable_projection_count = projections.len();
    let (status, message) = if refreshable_projection_count == 0 {
        ("up_to_date", "all semantic projections are fresh")
    } else {
        (
            "refresh_required",
            "semantic projection metadata refresh is required; run `wendao-client lint semantic --refresh-projections`",
        )
    };
    SemanticProjectionRefreshPlanReport {
        status: status.to_string(),
        refreshable_projection_count,
        message: message.to_string(),
        projections,
    }
}

fn semantic_projection_policy_failure_reason(
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

fn semantic_projection_policy_source_path(projection: &SemanticProjection) -> Option<String> {
    (!projection.source_path.as_os_str().is_empty())
        .then(|| projection.source_path.display().to_string())
}

fn semantic_projection_staleness_token(staleness: &SemanticProjectionStaleness) -> &'static str {
    match staleness {
        SemanticProjectionStaleness::Fresh => "fresh",
        SemanticProjectionStaleness::Stale => "stale",
    }
}
