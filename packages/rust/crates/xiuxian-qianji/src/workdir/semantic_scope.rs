use std::fmt::Write as _;

use xiuxian_wendao_parsers::semantic_ssot::{
    SemanticObjectKind, SemanticProjectionFreshnessPolicyReport, SemanticProjectionStaleness,
    SemanticScopeBundle, SemanticStatus, parse_semantic_scope_metadata_envelope_json,
};

use crate::error::QianjiError;

/// Advisory status derived from a Wendao semantic-scope bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkdirSemanticScopeGuardStatus {
    /// The bundle is usable as advisory execution context.
    Ready,
    /// The bundle can be read, but a human or refresh step should review it.
    ReviewRequired,
    /// The bundle has unresolved required semantic context.
    Blocked,
}

impl WorkdirSemanticScopeGuardStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::ReviewRequired => "review_required",
            Self::Blocked => "blocked",
        }
    }
}

/// Compact object row carried in a Qianji semantic-scope guard trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirSemanticScopeObjectSummary {
    /// Stable semantic object id.
    pub id: String,
    /// Stable object kind token.
    pub kind: String,
    /// Semantic lifecycle status token.
    pub status: String,
    /// Human-readable object title.
    pub title: String,
}

/// Compact semantic SQL guard evidence consumed by Qianji as advisory context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirSemanticSqlGuardSummary {
    /// Stable guard identifier.
    pub guard_id: String,
    /// Advisory guard status token.
    pub status: String,
    /// Count of rows that caused the guard to request review.
    pub failing_row_count: usize,
    /// Human-readable guard message.
    pub message: String,
}

/// Compact projection freshness policy evidence consumed by Qianji as advisory context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirSemanticProjectionPolicySummary {
    /// Stable policy identifier.
    pub policy_id: String,
    /// Advisory policy status token.
    pub status: String,
    /// Count of projections that caused the policy to request review.
    pub failing_projection_count: usize,
    /// Human-readable policy message.
    pub message: String,
}

/// Qianji-owned advisory trace produced from a Wendao semantic-scope bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirSemanticScopeGuardTrace {
    /// Advisory trace status.
    pub status: WorkdirSemanticScopeGuardStatus,
    /// Optional task object id anchoring the scope.
    pub task_id: Option<String>,
    /// Object ids explicitly requested by the caller.
    pub requested_object_ids: Vec<String>,
    /// Included semantic objects.
    pub objects: Vec<WorkdirSemanticScopeObjectSummary>,
    /// Number of included relation edges.
    pub relation_count: usize,
    /// Included semantic change-intent ids.
    pub change_intent_ids: Vec<String>,
    /// Included affected invariant ids.
    pub affected_invariants: Vec<String>,
    /// Required validation commands from the bundle.
    pub required_validations: Vec<String>,
    /// Projection revision selected by Wendao.
    pub projection_revision: String,
    /// Optional projection source revision.
    pub projection_source_revision: Option<String>,
    /// Optional projection freshness token.
    pub projection_staleness: Option<String>,
    /// Requested object ids that Wendao could not resolve.
    pub unresolved_ids: Vec<String>,
    /// Semantic SQL guard evidence supplied by Wendao or Studio metadata.
    pub sql_guard_evidence: Vec<WorkdirSemanticSqlGuardSummary>,
    /// Semantic projection freshness policy evidence supplied by Wendao or Studio metadata.
    pub projection_policy_evidence: Vec<WorkdirSemanticProjectionPolicySummary>,
    /// Advisory guard issues Qianji should surface before execution.
    pub issues: Vec<String>,
}

/// Build a Qianji advisory guard trace from one validated semantic-scope bundle.
#[must_use]
pub fn trace_workdir_semantic_scope_bundle(
    bundle: &SemanticScopeBundle,
) -> WorkdirSemanticScopeGuardTrace {
    trace_workdir_semantic_scope_bundle_with_sql_guard_evidence(bundle, Vec::new())
}

/// Build a Qianji advisory guard trace from a bundle and semantic SQL guard evidence.
#[must_use]
pub fn trace_workdir_semantic_scope_bundle_with_sql_guard_evidence(
    bundle: &SemanticScopeBundle,
    sql_guard_evidence: Vec<WorkdirSemanticSqlGuardSummary>,
) -> WorkdirSemanticScopeGuardTrace {
    trace_workdir_semantic_scope_bundle_with_evidence(bundle, sql_guard_evidence, Vec::new())
}

/// Build a Qianji advisory guard trace from a bundle and external semantic evidence.
#[must_use]
pub fn trace_workdir_semantic_scope_bundle_with_evidence(
    bundle: &SemanticScopeBundle,
    sql_guard_evidence: Vec<WorkdirSemanticSqlGuardSummary>,
    projection_policy_evidence: Vec<WorkdirSemanticProjectionPolicySummary>,
) -> WorkdirSemanticScopeGuardTrace {
    let mut issues = Vec::new();
    if bundle.objects.is_empty() {
        issues.push("semantic scope contains no objects".to_string());
    }
    if !bundle.unresolved_ids.is_empty() {
        issues.push(format!(
            "semantic scope contains unresolved ids: {}",
            bundle.unresolved_ids.join(", ")
        ));
    }
    if bundle.projection_staleness == Some(SemanticProjectionStaleness::Stale) {
        issues.push(
            "semantic projection is stale and must be refreshed or reviewed before relying on it"
                .to_string(),
        );
    }
    for guard in &sql_guard_evidence {
        if guard.status != "passed" {
            issues.push(format!(
                "semantic SQL guard `{}` reported `{}`: {}",
                guard.guard_id, guard.status, guard.message
            ));
        }
    }
    for policy in &projection_policy_evidence {
        if policy.status != "passed" {
            issues.push(format!(
                "semantic projection policy `{}` reported `{}`: {}",
                policy.policy_id, policy.status, policy.message
            ));
        }
    }

    let status = if bundle.objects.is_empty() || !bundle.unresolved_ids.is_empty() {
        WorkdirSemanticScopeGuardStatus::Blocked
    } else if bundle.projection_staleness == Some(SemanticProjectionStaleness::Stale)
        || sql_guard_evidence
            .iter()
            .any(|guard| guard.status != "passed")
        || projection_policy_evidence
            .iter()
            .any(|policy| policy.status != "passed")
    {
        WorkdirSemanticScopeGuardStatus::ReviewRequired
    } else {
        WorkdirSemanticScopeGuardStatus::Ready
    };

    WorkdirSemanticScopeGuardTrace {
        status,
        task_id: bundle.task_id.clone(),
        requested_object_ids: bundle.requested_object_ids.clone(),
        objects: bundle
            .objects
            .iter()
            .map(|object| WorkdirSemanticScopeObjectSummary {
                id: object.id.clone(),
                kind: semantic_kind_token(&object.kind).to_string(),
                status: semantic_status_token(&object.status).to_string(),
                title: object.title.clone(),
            })
            .collect(),
        relation_count: bundle.relations.len(),
        change_intent_ids: bundle
            .change_intents
            .iter()
            .map(|intent| intent.id.clone())
            .collect(),
        affected_invariants: bundle.affected_invariants.clone(),
        required_validations: bundle.required_validations.clone(),
        projection_revision: bundle.projection_revision.clone(),
        projection_source_revision: bundle.projection_source_revision.clone(),
        projection_staleness: bundle
            .projection_staleness
            .as_ref()
            .map(semantic_projection_staleness_token)
            .map(str::to_string),
        unresolved_ids: bundle.unresolved_ids.clone(),
        sql_guard_evidence,
        projection_policy_evidence,
        issues,
    }
}

/// Decode Wendao semantic-scope app metadata JSON and build a Qianji trace.
///
/// Accepts either a raw `SemanticScopeBundle` JSON object or the Studio Flight
/// metadata envelope containing `semanticScopeBundle`.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the JSON cannot be parsed or decoded
/// as a semantic-scope bundle.
pub fn trace_workdir_semantic_scope_json(
    raw_metadata_json: &str,
) -> Result<WorkdirSemanticScopeGuardTrace, QianjiError> {
    let envelope =
        parse_semantic_scope_metadata_envelope_json(raw_metadata_json).map_err(|error| {
            QianjiError::Topology(format!(
                "failed to decode semantic-scope metadata envelope: {error}"
            ))
        })?;
    let sql_guard_evidence =
        semantic_sql_guard_summaries_from_optional_metadata(envelope.sql_guard_evidence.as_ref())?;
    let projection_policy_evidence = semantic_projection_policy_summaries_from_report(
        envelope.projection_policy_evidence.as_ref(),
    );
    Ok(trace_workdir_semantic_scope_bundle_with_evidence(
        &envelope.bundle,
        sql_guard_evidence,
        projection_policy_evidence,
    ))
}

/// Render one semantic-scope guard trace as compact JSON for workflow context.
#[must_use]
pub fn workdir_semantic_scope_guard_trace_json(
    trace: &WorkdirSemanticScopeGuardTrace,
) -> serde_json::Value {
    serde_json::json!({
        "status": trace.status.as_str(),
        "taskId": &trace.task_id,
        "requestedObjectIds": &trace.requested_object_ids,
        "objects": trace.objects.iter().map(workdir_semantic_scope_object_json).collect::<Vec<_>>(),
        "relationCount": trace.relation_count,
        "changeIntentIds": &trace.change_intent_ids,
        "affectedInvariants": &trace.affected_invariants,
        "requiredValidations": &trace.required_validations,
        "projectionRevision": &trace.projection_revision,
        "projectionSourceRevision": &trace.projection_source_revision,
        "projectionStaleness": &trace.projection_staleness,
        "unresolvedIds": &trace.unresolved_ids,
        "sqlGuardEvidence": trace.sql_guard_evidence.iter().map(workdir_semantic_sql_guard_json).collect::<Vec<_>>(),
        "projectionPolicyEvidence": trace.projection_policy_evidence.iter().map(workdir_semantic_projection_policy_json).collect::<Vec<_>>(),
        "issues": &trace.issues,
    })
}

/// Render one semantic-scope guard trace as compact Markdown.
#[must_use]
pub fn render_workdir_semantic_scope_guard_trace(trace: &WorkdirSemanticScopeGuardTrace) -> String {
    let mut rendered = String::new();
    rendered.push_str("# Semantic Scope Guard Trace\n\n");
    let _ = writeln!(rendered, "Status: {}", trace.status.as_str());
    if let Some(task_id) = &trace.task_id {
        let _ = writeln!(rendered, "Task: {task_id}");
    }
    let _ = write!(rendered, "Projection: {}", trace.projection_revision);
    if let Some(staleness) = &trace.projection_staleness {
        let _ = write!(rendered, " ({staleness})");
    }
    rendered.push('\n');
    let _ = writeln!(rendered, "Relations: {}", trace.relation_count);

    if !trace.issues.is_empty() {
        rendered.push_str("\n## Issues\n\n");
        for issue in &trace.issues {
            let _ = writeln!(rendered, "- {issue}");
        }
    }

    rendered.push_str("\n## Objects\n\n");
    for object in &trace.objects {
        let _ = writeln!(
            rendered,
            "- {} [{} / {}] - {}",
            object.id, object.kind, object.status, object.title
        );
    }

    if !trace.change_intent_ids.is_empty() {
        rendered.push_str("\n## Change Intents\n\n");
        for change_intent_id in &trace.change_intent_ids {
            let _ = writeln!(rendered, "- {change_intent_id}");
        }
    }

    if !trace.sql_guard_evidence.is_empty() {
        rendered.push_str("\n## SQL Guard Evidence\n\n");
        for guard in &trace.sql_guard_evidence {
            let _ = write!(
                rendered,
                "- {}: {} ({} failing row(s))",
                guard.guard_id, guard.status, guard.failing_row_count
            );
            if !guard.message.is_empty() {
                let _ = write!(rendered, " - {}", guard.message);
            }
            rendered.push('\n');
        }
    }

    if !trace.projection_policy_evidence.is_empty() {
        rendered.push_str("\n## Projection Policy Evidence\n\n");
        for policy in &trace.projection_policy_evidence {
            let _ = write!(
                rendered,
                "- {}: {} ({} failing projection(s))",
                policy.policy_id, policy.status, policy.failing_projection_count
            );
            if !policy.message.is_empty() {
                let _ = write!(rendered, " - {}", policy.message);
            }
            rendered.push('\n');
        }
    }

    if !trace.required_validations.is_empty() {
        rendered.push_str("\n## Required Validations\n\n");
        for validation in &trace.required_validations {
            let _ = writeln!(rendered, "- {validation}");
        }
    }

    rendered
}

fn workdir_semantic_scope_object_json(
    object: &WorkdirSemanticScopeObjectSummary,
) -> serde_json::Value {
    serde_json::json!({
        "id": &object.id,
        "kind": &object.kind,
        "status": &object.status,
        "title": &object.title,
    })
}

fn workdir_semantic_sql_guard_json(guard: &WorkdirSemanticSqlGuardSummary) -> serde_json::Value {
    serde_json::json!({
        "guardId": &guard.guard_id,
        "status": &guard.status,
        "failingRowCount": guard.failing_row_count,
        "message": &guard.message,
    })
}

fn workdir_semantic_projection_policy_json(
    policy: &WorkdirSemanticProjectionPolicySummary,
) -> serde_json::Value {
    serde_json::json!({
        "policyId": &policy.policy_id,
        "status": &policy.status,
        "failingProjectionCount": policy.failing_projection_count,
        "message": &policy.message,
    })
}

fn semantic_sql_guard_summaries_from_optional_metadata(
    evidence_value: Option<&serde_json::Value>,
) -> Result<Vec<WorkdirSemanticSqlGuardSummary>, QianjiError> {
    let Some(evidence_value) = evidence_value else {
        return Ok(Vec::new());
    };

    match evidence_value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(values) => values
            .iter()
            .map(semantic_sql_guard_summary_from_value)
            .collect(),
        serde_json::Value::Object(_) => {
            Ok(vec![semantic_sql_guard_summary_from_value(evidence_value)?])
        }
        _ => Err(QianjiError::Topology(
            "`semanticSqlGuardEvidence` must be an object or array".to_string(),
        )),
    }
}

fn semantic_sql_guard_summary_from_value(
    value: &serde_json::Value,
) -> Result<WorkdirSemanticSqlGuardSummary, QianjiError> {
    Ok(WorkdirSemanticSqlGuardSummary {
        guard_id: semantic_sql_guard_string(value, "guardId", "guard_id")?,
        status: semantic_sql_guard_string(value, "status", "status")?,
        failing_row_count: semantic_sql_guard_usize(value, "failingRowCount", "failing_row_count")?,
        message: semantic_sql_guard_string(value, "message", "message")?,
    })
}

fn semantic_sql_guard_string(
    value: &serde_json::Value,
    camel_key: &str,
    snake_key: &str,
) -> Result<String, QianjiError> {
    value
        .get(camel_key)
        .or_else(|| value.get(snake_key))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            QianjiError::Topology(format!(
                "semantic SQL guard evidence is missing string field `{camel_key}`"
            ))
        })
}

fn semantic_sql_guard_usize(
    value: &serde_json::Value,
    camel_key: &str,
    snake_key: &str,
) -> Result<usize, QianjiError> {
    value
        .get(camel_key)
        .or_else(|| value.get(snake_key))
        .and_then(serde_json::Value::as_u64)
        .and_then(|raw| usize::try_from(raw).ok())
        .ok_or_else(|| {
            QianjiError::Topology(format!(
                "semantic SQL guard evidence is missing integer field `{camel_key}`"
            ))
        })
}

fn semantic_projection_policy_summaries_from_report(
    report: Option<&SemanticProjectionFreshnessPolicyReport>,
) -> Vec<WorkdirSemanticProjectionPolicySummary> {
    report
        .map(|report| {
            vec![WorkdirSemanticProjectionPolicySummary {
                policy_id: report.policy_id.clone(),
                status: report.status.clone(),
                failing_projection_count: report.failing_projection_count,
                message: report.message.clone(),
            }]
        })
        .unwrap_or_default()
}

fn semantic_kind_token(kind: &SemanticObjectKind) -> &'static str {
    match kind {
        SemanticObjectKind::Component => "component",
        SemanticObjectKind::Decision => "decision",
        SemanticObjectKind::Invariant => "invariant",
        SemanticObjectKind::Task => "task",
    }
}

fn semantic_status_token(status: &SemanticStatus) -> &'static str {
    match status {
        SemanticStatus::Draft => "draft",
        SemanticStatus::Candidate => "candidate",
        SemanticStatus::Active => "active",
        SemanticStatus::Superseded => "superseded",
        SemanticStatus::Deprecated => "deprecated",
        SemanticStatus::Retired => "retired",
    }
}

fn semantic_projection_staleness_token(staleness: &SemanticProjectionStaleness) -> &'static str {
    match staleness {
        SemanticProjectionStaleness::Fresh => "fresh",
        SemanticProjectionStaleness::Stale => "stale",
    }
}
