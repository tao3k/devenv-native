//! Scope selection for semantic `SSOT` bundles.

use super::hash::semantic_object_by_id;
use super::projection::semantic_projection_source_revision;
use crate::semantic_ssot::types::{
    SemanticBundleProvenance, SemanticChangeIntent, SemanticObject, SemanticObjectKind,
    SemanticProjection, SemanticRelationEdge, SemanticRepository, SemanticScopeBundle,
    SemanticScopeRequest, SemanticStatus,
};
use std::collections::{BTreeMap, BTreeSet};

/// Build a deterministic semantic scope bundle from a loaded repository.
#[must_use]
pub fn semantic_scope_bundle(
    repository: &SemanticRepository,
    request: &SemanticScopeRequest,
) -> SemanticScopeBundle {
    let object_by_id = semantic_object_by_id(repository);
    let (requested, selected, unresolved) = selected_scope_ids(repository, request, &object_by_id);
    let objects = selected_scope_objects(&selected, &object_by_id);
    let selected_ids = selected_object_ids(&objects);
    let relations = selected_scope_relations(&objects, &selected_ids);
    let change_intents = selected_change_intents(repository, &selected_ids);
    let active_projection = active_projection(repository);

    SemanticScopeBundle {
        task_id: request.task_id.clone(),
        requested_object_ids: requested,
        affected_invariants: selected_scope_invariants(&objects),
        required_validations: selected_required_validations(&objects, &change_intents),
        provenance: selected_scope_provenance(&objects),
        objects,
        relations,
        change_intents,
        projection_revision: active_projection.map_or_else(
            || "semantic-ssot-unprojected".to_string(),
            |projection| projection.projection_revision.clone(),
        ),
        projection_source_revision: active_projection.map(|projection| {
            semantic_projection_source_revision(repository, projection)
                .unwrap_or_else(|| projection.source_revision.clone())
        }),
        projection_staleness: active_projection.map(|projection| projection.staleness.clone()),
        unresolved_ids: unresolved.into_iter().collect(),
    }
}

fn selected_scope_ids(
    repository: &SemanticRepository,
    request: &SemanticScopeRequest,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
) -> (Vec<String>, BTreeSet<String>, BTreeSet<String>) {
    let mut selected = BTreeSet::new();
    let mut unresolved = BTreeSet::new();
    let mut requested = Vec::new();

    if request.task_id.is_none() && request.object_ids.is_empty() {
        selected.extend(
            repository
                .objects
                .iter()
                .filter(|object| object.status == SemanticStatus::Active)
                .map(|object| object.id.clone()),
        );
    }

    if let Some(task_id) = request.task_id.as_deref() {
        requested.push(task_id.to_string());
        select_requested_id(task_id, object_by_id, &mut selected, &mut unresolved);
    }

    for object_id in &request.object_ids {
        requested.push(object_id.clone());
        select_requested_id(object_id, object_by_id, &mut selected, &mut unresolved);
    }

    let anchors = selected.clone();
    for object_id in anchors {
        let Some(object) = object_by_id.get(object_id.as_str()) else {
            continue;
        };
        for relation in &object.relations {
            if let Some(target) = object_by_id.get(relation.target.as_str())
                && target.status == SemanticStatus::Active
            {
                selected.insert(target.id.clone());
            }
        }
    }

    (requested, selected, unresolved)
}

fn selected_scope_objects(
    selected: &BTreeSet<String>,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
) -> Vec<SemanticObject> {
    let mut objects = selected
        .iter()
        .filter_map(|object_id| object_by_id.get(object_id.as_str()).copied())
        .cloned()
        .collect::<Vec<_>>();
    objects.sort_by(|left, right| left.id.cmp(&right.id));
    objects
}

fn selected_object_ids(objects: &[SemanticObject]) -> BTreeSet<&str> {
    objects
        .iter()
        .map(|object| object.id.as_str())
        .collect::<BTreeSet<_>>()
}

fn selected_scope_relations(
    objects: &[SemanticObject],
    selected_ids: &BTreeSet<&str>,
) -> Vec<SemanticRelationEdge> {
    let mut relations = objects
        .iter()
        .flat_map(|object| {
            object
                .relations
                .iter()
                .filter(|relation| selected_ids.contains(relation.target.as_str()))
                .map(|relation| SemanticRelationEdge {
                    source: object.id.clone(),
                    kind: relation.kind.clone(),
                    target: relation.target.clone(),
                })
        })
        .collect::<Vec<_>>();
    relations.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then_with(|| left.target.cmp(&right.target))
    });
    relations
}

fn selected_scope_invariants(objects: &[SemanticObject]) -> Vec<String> {
    objects
        .iter()
        .filter(|object| object.kind == SemanticObjectKind::Invariant)
        .map(|object| object.id.clone())
        .collect::<Vec<_>>()
}

fn selected_change_intents(
    repository: &SemanticRepository,
    selected_ids: &BTreeSet<&str>,
) -> Vec<SemanticChangeIntent> {
    let mut change_intents = repository
        .change_intents
        .iter()
        .filter(|intent| intent.status == SemanticStatus::Active)
        .filter(|intent| change_intent_intersects_scope(intent, selected_ids))
        .cloned()
        .collect::<Vec<_>>();
    change_intents.sort_by(|left, right| left.id.cmp(&right.id));
    change_intents
}

fn selected_required_validations(
    objects: &[SemanticObject],
    change_intents: &[SemanticChangeIntent],
) -> Vec<String> {
    objects
        .iter()
        .flat_map(|object| object.verification.required.iter().cloned())
        .chain(
            change_intents
                .iter()
                .flat_map(|intent| intent.required_validations.iter().cloned()),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
}

fn selected_scope_provenance(objects: &[SemanticObject]) -> Vec<SemanticBundleProvenance> {
    objects
        .iter()
        .map(|object| SemanticBundleProvenance {
            object_id: object.id.clone(),
            source_path: object.source_path.clone(),
            source: object.provenance.source.clone(),
        })
        .collect::<Vec<_>>()
}

fn change_intent_intersects_scope(
    intent: &SemanticChangeIntent,
    selected_ids: &BTreeSet<&str>,
) -> bool {
    intent
        .touched_objects
        .iter()
        .any(|object_id| selected_ids.contains(object_id.as_str()))
        || intent
            .affected_invariants
            .iter()
            .any(|object_id| selected_ids.contains(object_id.as_str()))
        || intent.changed_relations.iter().any(|relation| {
            selected_ids.contains(relation.source.as_str())
                || selected_ids.contains(relation.target.as_str())
        })
        || intent
            .status_transitions
            .iter()
            .any(|transition| selected_ids.contains(transition.object_id.as_str()))
        || intent
            .promotion_targets
            .iter()
            .any(|object_id| selected_ids.contains(object_id.as_str()))
        || intent
            .demotion_targets
            .iter()
            .any(|object_id| selected_ids.contains(object_id.as_str()))
        || intent
            .candidate_suggestions
            .iter()
            .any(|object_id| selected_ids.contains(object_id.as_str()))
}

fn select_requested_id(
    object_id: &str,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
    selected: &mut BTreeSet<String>,
    unresolved: &mut BTreeSet<String>,
) {
    if let Some(object) = object_by_id.get(object_id) {
        if matches!(
            object.status,
            SemanticStatus::Active | SemanticStatus::Candidate
        ) {
            selected.insert((*object_id).to_string());
        }
    } else {
        unresolved.insert(object_id.to_string());
    }
}

fn active_projection(repository: &SemanticRepository) -> Option<&SemanticProjection> {
    repository
        .projections
        .iter()
        .find(|projection| projection.status == SemanticStatus::Active)
}
