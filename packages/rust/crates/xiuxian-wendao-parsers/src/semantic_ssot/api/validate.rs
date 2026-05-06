//! Validation rules for semantic `SSOT` repositories.

use super::hash::semantic_projection_source_revision_from_map;
use crate::semantic_ssot::types::{
    SemanticChangeIntent, SemanticConfidenceSource, SemanticObject, SemanticObjectKind,
    SemanticProjection, SemanticProjectionStaleness, SemanticRepository, SemanticStatus,
    SemanticStatusTransition, SemanticValidationReport,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub(super) fn validate_repository(repository: &mut SemanticRepository) {
    let mut seen_ids = BTreeSet::new();
    let object_ids = repository
        .objects
        .iter()
        .map(|object| object.id.clone())
        .collect::<BTreeSet<_>>();

    for object in &repository.objects {
        validate_object(object, &mut seen_ids, &object_ids, &mut repository.report);
    }

    let object_by_id = repository
        .objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<BTreeMap<_, _>>();
    let projection_names = repository
        .projections
        .iter()
        .map(|projection| projection.projection.as_str())
        .collect::<BTreeSet<_>>();

    for projection in &repository.projections {
        validate_projection(
            projection,
            &object_ids,
            &object_by_id,
            &mut repository.report,
        );
    }

    let mut seen_change_ids = BTreeSet::new();
    for intent in &repository.change_intents {
        validate_change_intent(
            intent,
            &mut seen_change_ids,
            &object_ids,
            &object_by_id,
            &projection_names,
            &mut repository.report,
        );
    }
    validate_candidate_object_lifecycle(
        &repository.objects,
        &repository.change_intents,
        &mut repository.report,
    );
}

fn validate_object(
    object: &SemanticObject,
    seen_ids: &mut BTreeSet<String>,
    object_ids: &BTreeSet<String>,
    report: &mut SemanticValidationReport,
) {
    let path = Some(object.source_path.clone());
    validate_non_empty(
        &object.id,
        "semantic object `id` must be non-empty",
        path.clone(),
        report,
    );
    validate_non_empty(
        &object.title,
        "semantic object `title` must be non-empty",
        path.clone(),
        report,
    );
    if !object.id.starts_with(object.kind.id_prefix())
        || object
            .id
            .as_bytes()
            .get(object.kind.id_prefix().len())
            .is_none_or(|value| *value != b'.')
    {
        report.push(
            path.clone(),
            format!(
                "semantic object id `{}` must start with `{}.`",
                object.id,
                object.kind.id_prefix()
            ),
        );
    }
    if !seen_ids.insert(object.id.clone()) {
        report.push(
            path.clone(),
            format!("duplicate semantic object id `{}`", object.id),
        );
    }
    if !(0.0..=1.0).contains(&object.confidence.score) {
        report.push(
            path.clone(),
            "semantic object confidence score must be between 0.0 and 1.0",
        );
    }
    if object.status == SemanticStatus::Active
        && matches!(
            object.confidence.source,
            SemanticConfidenceSource::LlmSuggested
        )
    {
        report.push(
            path.clone(),
            "active semantic objects cannot use `llm_suggested` confidence source",
        );
    }
    validate_owners(object, path.as_ref(), report);
    validate_provenance(object, path.as_ref(), report);
    validate_verification(object, path.as_ref(), report);
    for relation in &object.relations {
        if relation.target.trim().is_empty() {
            report.push(path.clone(), "semantic relation target must be non-empty");
        } else if !object_ids.contains(&relation.target) {
            report.push(
                path.clone(),
                format!(
                    "semantic relation target `{}` does not resolve to a known object",
                    relation.target
                ),
            );
        }
    }
}

fn clone_path(path: Option<&PathBuf>) -> Option<PathBuf> {
    path.cloned()
}

fn validate_owners(
    object: &SemanticObject,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if object.owners.is_empty() {
        report.push(
            clone_path(path),
            "semantic object `owners` must be non-empty",
        );
    }
    for owner in &object.owners {
        validate_non_empty(
            &owner.scope,
            "semantic owner `scope` must be non-empty",
            clone_path(path),
            report,
        );
        validate_non_empty(
            &owner.role,
            "semantic owner `role` must be non-empty",
            clone_path(path),
            report,
        );
    }
}

fn validate_provenance(
    object: &SemanticObject,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    validate_non_empty(
        &object.provenance.source,
        "semantic provenance `source` must be non-empty",
        clone_path(path),
        report,
    );
    validate_non_empty(
        &object.provenance.recorded_by,
        "semantic provenance `recorded_by` must be non-empty",
        clone_path(path),
        report,
    );
    validate_non_empty(
        &object.provenance.recorded_at,
        "semantic provenance `recorded_at` must be non-empty",
        clone_path(path),
        report,
    );
}

fn validate_verification(
    object: &SemanticObject,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if object.verification.required.is_empty() {
        report.push(
            clone_path(path),
            "semantic verification `required` must be non-empty",
        );
    }
    for command in &object.verification.required {
        validate_non_empty(
            command,
            "semantic verification command must be non-empty",
            clone_path(path),
            report,
        );
    }
}

fn validate_projection(
    projection: &SemanticProjection,
    object_ids: &BTreeSet<String>,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
    report: &mut SemanticValidationReport,
) {
    let path = Some(projection.source_path.clone());
    if projection.projection_type != "semantic_projection" {
        report.push(
            path.clone(),
            "semantic projection `type` must be `semantic_projection`",
        );
    }
    validate_non_empty(
        &projection.projection,
        "semantic projection name must be non-empty",
        path.clone(),
        report,
    );
    validate_non_empty(
        &projection.source_revision,
        "semantic projection source revision must be non-empty",
        path.clone(),
        report,
    );
    validate_non_empty(
        &projection.projection_revision,
        "semantic projection revision must be non-empty",
        path.clone(),
        report,
    );
    if projection.source_objects.is_empty() {
        report.push(
            path.clone(),
            "semantic projection `source_objects` must be non-empty",
        );
    }
    for source_object in &projection.source_objects {
        if !object_ids.contains(source_object) {
            report.push(
                path.clone(),
                format!("semantic projection source object `{source_object}` does not resolve"),
            );
        }
    }
    if let Some(current_revision) =
        semantic_projection_source_revision_from_map(projection, object_by_id)
    {
        let declared_revision = projection.source_revision.trim();
        if declared_revision != current_revision
            && projection.staleness != SemanticProjectionStaleness::Stale
        {
            report.push(
                path.clone(),
                format!(
                    "semantic projection source revision is stale: declared `{declared_revision}`, current `{current_revision}`"
                ),
            );
        }
    }
}

fn validate_change_intent(
    intent: &SemanticChangeIntent,
    seen_ids: &mut BTreeSet<String>,
    object_ids: &BTreeSet<String>,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
    projection_names: &BTreeSet<&str>,
    report: &mut SemanticValidationReport,
) {
    let path = Some(intent.source_path.clone());
    validate_change_intent_metadata(intent, seen_ids, path.as_ref(), report);
    validate_change_touched_objects(intent, object_ids, path.as_ref(), report);
    validate_changed_relations(intent, object_ids, path.as_ref(), report);
    validate_status_transitions(intent, object_ids, object_by_id, path.as_ref(), report);
    validate_lifecycle_outcome_targets(intent, object_ids, path.as_ref(), report);
    validate_affected_invariants(intent, object_ids, object_by_id, path.as_ref(), report);
    validate_change_required_validations(intent, path.as_ref(), report);
    validate_projection_refresh_targets(intent, projection_names, path.as_ref(), report);
    validate_candidate_suggestions(intent, object_ids, object_by_id, path.as_ref(), report);
}

fn validate_change_intent_metadata(
    intent: &SemanticChangeIntent,
    seen_ids: &mut BTreeSet<String>,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if intent.intent_type != "semantic_change_intent" {
        report.push(
            clone_path(path),
            "semantic change intent `type` must be `semantic_change_intent`",
        );
    }
    validate_non_empty(
        &intent.id,
        "semantic change intent `id` must be non-empty",
        clone_path(path),
        report,
    );
    if !intent.id.starts_with("change.") {
        report.push(
            clone_path(path),
            format!(
                "semantic change intent id `{}` must start with `change.`",
                intent.id
            ),
        );
    }
    if !seen_ids.insert(intent.id.clone()) {
        report.push(
            clone_path(path),
            format!("duplicate semantic change intent id `{}`", intent.id),
        );
    }
    validate_non_empty(
        &intent.title,
        "semantic change intent `title` must be non-empty",
        clone_path(path),
        report,
    );
}

fn validate_change_touched_objects(
    intent: &SemanticChangeIntent,
    object_ids: &BTreeSet<String>,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if intent.touched_objects.is_empty() {
        report.push(
            clone_path(path),
            "semantic change intent `touched_objects` must be non-empty",
        );
    }
    for object_id in &intent.touched_objects {
        validate_object_reference(
            object_id,
            object_ids,
            "semantic change intent touched object",
            clone_path(path),
            report,
        );
    }
}

fn validate_changed_relations(
    intent: &SemanticChangeIntent,
    object_ids: &BTreeSet<String>,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    for relation in &intent.changed_relations {
        validate_object_reference(
            &relation.source,
            object_ids,
            "semantic change relation source",
            clone_path(path),
            report,
        );
        validate_object_reference(
            &relation.target,
            object_ids,
            "semantic change relation target",
            clone_path(path),
            report,
        );
    }
}

fn validate_status_transitions(
    intent: &SemanticChangeIntent,
    object_ids: &BTreeSet<String>,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    let touched_objects = intent
        .touched_objects
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    for transition in &intent.status_transitions {
        validate_object_reference(
            &transition.object_id,
            object_ids,
            "semantic status transition object",
            clone_path(path),
            report,
        );
        if transition.from == transition.to {
            report.push(
                clone_path(path),
                format!(
                    "semantic status transition `{}` must change status",
                    transition.object_id
                ),
            );
        }
        if !semantic_status_transition_allowed(&transition.from, &transition.to) {
            report.push(
                clone_path(path),
                format!(
                    "semantic status transition `{}` from `{}` to `{}` is not allowed",
                    transition.object_id,
                    semantic_status_label(&transition.from),
                    semantic_status_label(&transition.to)
                ),
            );
        }
        if !touched_objects.contains(transition.object_id.as_str()) {
            report.push(
                clone_path(path),
                format!(
                    "semantic status transition `{}` must also be listed in touched_objects",
                    transition.object_id
                ),
            );
        }
        if let Some(object) = object_by_id.get(transition.object_id.as_str())
            && object.status != transition.to
        {
            report.push(
                clone_path(path),
                format!(
                    "semantic status transition `{}` current status must match transition target",
                    transition.object_id
                ),
            );
        }
    }
}

fn semantic_status_transition_allowed(from: &SemanticStatus, to: &SemanticStatus) -> bool {
    matches!(
        (from, to),
        (
            SemanticStatus::Draft,
            SemanticStatus::Candidate | SemanticStatus::Active
        ) | (
            SemanticStatus::Candidate | SemanticStatus::Deprecated,
            SemanticStatus::Active
        ) | (
            SemanticStatus::Candidate
                | SemanticStatus::Active
                | SemanticStatus::Deprecated
                | SemanticStatus::Superseded,
            SemanticStatus::Retired,
        ) | (
            SemanticStatus::Active,
            SemanticStatus::Deprecated | SemanticStatus::Superseded
        )
    )
}

fn semantic_status_label(status: &SemanticStatus) -> &'static str {
    match status {
        SemanticStatus::Draft => "draft",
        SemanticStatus::Candidate => "candidate",
        SemanticStatus::Active => "active",
        SemanticStatus::Superseded => "superseded",
        SemanticStatus::Deprecated => "deprecated",
        SemanticStatus::Retired => "retired",
    }
}

fn validate_lifecycle_outcome_targets(
    intent: &SemanticChangeIntent,
    object_ids: &BTreeSet<String>,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    let touched_objects = intent
        .touched_objects
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let promotion_transitions = intent
        .status_transitions
        .iter()
        .filter(|transition| semantic_status_transition_is_promotion(transition))
        .map(|transition| transition.object_id.as_str())
        .collect::<BTreeSet<_>>();
    let demotion_transitions = intent
        .status_transitions
        .iter()
        .filter(|transition| semantic_status_transition_is_demotion(transition))
        .map(|transition| transition.object_id.as_str())
        .collect::<BTreeSet<_>>();

    validate_lifecycle_target_set(
        &intent.promotion_targets,
        &promotion_transitions,
        object_ids,
        &touched_objects,
        LifecycleTargetRule {
            outcome: "promotion",
            target_field: "promotion_targets",
            required_transition: "candidate to active status transition",
        },
        path,
        report,
    );
    validate_lifecycle_target_set(
        &intent.demotion_targets,
        &demotion_transitions,
        object_ids,
        &touched_objects,
        LifecycleTargetRule {
            outcome: "demotion",
            target_field: "demotion_targets",
            required_transition: "status transition to deprecated, superseded, or retired",
        },
        path,
        report,
    );

    require_transition_targets(
        &promotion_transitions,
        &intent.promotion_targets,
        LifecycleTargetRule {
            outcome: "promotion",
            target_field: "promotion_targets",
            required_transition: "candidate to active status transition",
        },
        path,
        report,
    );
    require_transition_targets(
        &demotion_transitions,
        &intent.demotion_targets,
        LifecycleTargetRule {
            outcome: "demotion",
            target_field: "demotion_targets",
            required_transition: "status transition to deprecated, superseded, or retired",
        },
        path,
        report,
    );
}

#[derive(Clone, Copy)]
struct LifecycleTargetRule<'a> {
    outcome: &'a str,
    target_field: &'a str,
    required_transition: &'a str,
}

fn validate_lifecycle_target_set(
    targets: &[String],
    matching_transitions: &BTreeSet<&str>,
    object_ids: &BTreeSet<String>,
    touched_objects: &BTreeSet<&str>,
    rule: LifecycleTargetRule<'_>,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    let mut seen_targets = BTreeSet::new();
    for object_id in targets {
        validate_object_reference(
            object_id,
            object_ids,
            &format!("semantic change {} target", rule.outcome),
            clone_path(path),
            report,
        );
        if !seen_targets.insert(object_id.as_str()) {
            report.push(
                clone_path(path),
                format!(
                    "semantic {} target `{object_id}` is duplicated in {}",
                    rule.outcome, rule.target_field
                ),
            );
        }
        if !touched_objects.contains(object_id.as_str()) {
            report.push(
                clone_path(path),
                format!(
                    "semantic {} target `{object_id}` must also be listed in touched_objects",
                    rule.outcome
                ),
            );
        }
        if !matching_transitions.contains(object_id.as_str()) {
            report.push(
                clone_path(path),
                format!(
                    "semantic {} target `{object_id}` must match a {}",
                    rule.outcome, rule.required_transition
                ),
            );
        }
    }
}

fn require_transition_targets(
    transitions: &BTreeSet<&str>,
    targets: &[String],
    rule: LifecycleTargetRule<'_>,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    let target_set = targets.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for object_id in transitions {
        if !target_set.contains(object_id) {
            report.push(
                clone_path(path),
                format!(
                    "semantic {} status transition `{object_id}` must be listed in {}",
                    rule.outcome, rule.target_field
                ),
            );
        }
    }
}

fn semantic_status_transition_is_promotion(transition: &SemanticStatusTransition) -> bool {
    transition.from == SemanticStatus::Candidate && transition.to == SemanticStatus::Active
}

fn semantic_status_transition_is_demotion(transition: &SemanticStatusTransition) -> bool {
    matches!(
        transition.to,
        SemanticStatus::Deprecated | SemanticStatus::Superseded | SemanticStatus::Retired
    )
}

fn validate_affected_invariants(
    intent: &SemanticChangeIntent,
    object_ids: &BTreeSet<String>,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if intent.affected_invariants.is_empty() {
        report.push(
            clone_path(path),
            "semantic change intent `affected_invariants` must be non-empty",
        );
    }
    for object_id in &intent.affected_invariants {
        validate_object_reference(
            object_id,
            object_ids,
            "semantic change affected invariant",
            clone_path(path),
            report,
        );
        if let Some(object) = object_by_id.get(object_id.as_str())
            && object.kind != SemanticObjectKind::Invariant
        {
            report.push(
                clone_path(path),
                format!(
                    "semantic change affected invariant `{object_id}` must reference an invariant object"
                ),
            );
        }
    }
}

fn validate_change_required_validations(
    intent: &SemanticChangeIntent,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if intent.required_validations.is_empty() {
        report.push(
            clone_path(path),
            "semantic change intent `required_validations` must be non-empty",
        );
    }
    for command in &intent.required_validations {
        validate_non_empty(
            command,
            "semantic change required validation must be non-empty",
            clone_path(path),
            report,
        );
    }
}

fn validate_projection_refresh_targets(
    intent: &SemanticChangeIntent,
    projection_names: &BTreeSet<&str>,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if intent.projections_to_refresh.is_empty() {
        report.push(
            clone_path(path),
            "semantic change intent `projections_to_refresh` must be non-empty",
        );
    }
    for projection in &intent.projections_to_refresh {
        if !projection_names.contains(projection.as_str()) {
            report.push(
                clone_path(path),
                format!("semantic change projection `{projection}` does not resolve"),
            );
        }
    }
}

fn validate_candidate_suggestions(
    intent: &SemanticChangeIntent,
    object_ids: &BTreeSet<String>,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
    path: Option<&PathBuf>,
    report: &mut SemanticValidationReport,
) {
    for object_id in &intent.candidate_suggestions {
        validate_object_reference(
            object_id,
            object_ids,
            "semantic change candidate suggestion",
            clone_path(path),
            report,
        );
        if let Some(object) = object_by_id.get(object_id.as_str())
            && object.status != SemanticStatus::Candidate
        {
            report.push(
                clone_path(path),
                format!(
                    "semantic change candidate suggestion `{object_id}` must reference a candidate object"
                ),
            );
        }
    }
}

fn validate_candidate_object_lifecycle(
    objects: &[SemanticObject],
    change_intents: &[SemanticChangeIntent],
    report: &mut SemanticValidationReport,
) {
    let governed_candidates = change_intents
        .iter()
        .filter(|intent| intent.status == SemanticStatus::Active)
        .flat_map(|intent| intent.candidate_suggestions.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();

    for object in objects {
        if object.status != SemanticStatus::Candidate {
            continue;
        }
        let path = Some(object.source_path.clone());
        if object.confidence.source != SemanticConfidenceSource::LlmSuggested {
            report.push(
                path.clone(),
                format!(
                    "candidate semantic object `{}` must use `llm_suggested` confidence source until accepted",
                    object.id
                ),
            );
        }
        if !governed_candidates.contains(object.id.as_str()) {
            report.push(
                path,
                format!(
                    "candidate semantic object `{}` must be referenced by an active change intent candidate_suggestions entry",
                    object.id
                ),
            );
        }
    }
}

fn validate_object_reference(
    object_id: &str,
    object_ids: &BTreeSet<String>,
    label: &str,
    path: Option<PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if object_id.trim().is_empty() {
        report.push(path, format!("{label} must be non-empty"));
    } else if !object_ids.contains(object_id) {
        report.push(
            path,
            format!("{label} `{object_id}` does not resolve to a known object"),
        );
    }
}

fn validate_non_empty(
    value: &str,
    message: &str,
    path: Option<PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if value.trim().is_empty() {
        report.push(path, message);
    }
}
