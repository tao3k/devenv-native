//! Repo-native semantic SSOT loading, validation, and scope APIs.

use super::types::{
    SemanticBundleProvenance, SemanticChangeIntent, SemanticConfidenceSource, SemanticObject,
    SemanticObjectKind, SemanticProjection, SemanticProjectionStaleness, SemanticRelationEdge,
    SemanticRelationKind, SemanticRepository, SemanticScopeBundle, SemanticScopeRequest,
    SemanticStatus, SemanticStatusTransition, SemanticValidationReport,
};
use crate::frontmatter::split_frontmatter_raw;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Error returned when parsing one semantic artifact fails.
#[derive(Debug)]
pub enum SemanticArtifactParseError {
    /// The Markdown document does not start with YAML frontmatter.
    MissingFrontmatter,
    /// The YAML frontmatter cannot be deserialized into the expected schema.
    InvalidYaml(serde_yaml::Error),
}

impl fmt::Display for SemanticArtifactParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFrontmatter => {
                write!(
                    formatter,
                    "document must start with a YAML frontmatter block"
                )
            }
            Self::InvalidYaml(error) => write!(formatter, "invalid semantic frontmatter: {error}"),
        }
    }
}

impl std::error::Error for SemanticArtifactParseError {}

/// Parse one semantic object artifact from Markdown content.
///
/// # Errors
///
/// Returns [`SemanticArtifactParseError`] when the document has no leading
/// YAML frontmatter or the frontmatter does not match the semantic object
/// schema.
pub fn parse_semantic_object(
    path: impl AsRef<Path>,
    content: &str,
) -> Result<SemanticObject, SemanticArtifactParseError> {
    let Some(frontmatter) = split_frontmatter_raw(content) else {
        return Err(SemanticArtifactParseError::MissingFrontmatter);
    };
    let mut object = serde_yaml::from_str::<SemanticObject>(frontmatter.yaml)
        .map_err(SemanticArtifactParseError::InvalidYaml)?;
    object.body = frontmatter.body.trim().to_string();
    object.source_path = path.as_ref().to_path_buf();
    Ok(object)
}

/// Parse one semantic projection artifact from Markdown content.
///
/// # Errors
///
/// Returns [`SemanticArtifactParseError`] when the document has no leading
/// YAML frontmatter or the frontmatter does not match the semantic projection
/// schema.
pub fn parse_semantic_projection(
    path: impl AsRef<Path>,
    content: &str,
) -> Result<SemanticProjection, SemanticArtifactParseError> {
    let Some(frontmatter) = split_frontmatter_raw(content) else {
        return Err(SemanticArtifactParseError::MissingFrontmatter);
    };
    let mut projection = serde_yaml::from_str::<SemanticProjection>(frontmatter.yaml)
        .map_err(SemanticArtifactParseError::InvalidYaml)?;
    projection.body = frontmatter.body.trim().to_string();
    projection.source_path = path.as_ref().to_path_buf();
    Ok(projection)
}

/// Parse one semantic change-intent artifact from Markdown content.
///
/// # Errors
///
/// Returns [`SemanticArtifactParseError`] when the document has no leading
/// YAML frontmatter or the frontmatter does not match the semantic
/// change-intent schema.
pub fn parse_semantic_change_intent(
    path: impl AsRef<Path>,
    content: &str,
) -> Result<SemanticChangeIntent, SemanticArtifactParseError> {
    let Some(frontmatter) = split_frontmatter_raw(content) else {
        return Err(SemanticArtifactParseError::MissingFrontmatter);
    };
    let mut intent = serde_yaml::from_str::<SemanticChangeIntent>(frontmatter.yaml)
        .map_err(SemanticArtifactParseError::InvalidYaml)?;
    intent.body = frontmatter.body.trim().to_string();
    intent.source_path = path.as_ref().to_path_buf();
    Ok(intent)
}

/// Load and validate one semantic repository root.
#[must_use]
pub fn load_semantic_repository(root: impl AsRef<Path>) -> SemanticRepository {
    let root = root.as_ref().to_path_buf();
    let mut repository = SemanticRepository {
        root: root.clone(),
        objects: Vec::new(),
        projections: Vec::new(),
        change_intents: Vec::new(),
        report: SemanticValidationReport::default(),
    };

    if !root.exists() {
        repository.report.push(
            None,
            format!("semantic root `{}` does not exist", root.display()),
        );
        return repository;
    }

    load_objects(&root, &mut repository);
    load_projections(&root, &mut repository);
    load_change_intents(&root, &mut repository);
    validate_repository(&mut repository);
    repository
}

/// Compute the source revision for one projection from its referenced objects.
#[must_use]
pub fn semantic_projection_source_revision(
    repository: &SemanticRepository,
    projection: &SemanticProjection,
) -> Option<String> {
    let object_by_id = semantic_object_by_id(repository);
    semantic_projection_source_revision_from_map(projection, &object_by_id)
}

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

fn semantic_object_by_id(repository: &SemanticRepository) -> BTreeMap<&str, &SemanticObject> {
    repository
        .objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<BTreeMap<_, _>>()
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

fn load_objects(root: &Path, repository: &mut SemanticRepository) {
    let objects_root = root.join("objects");
    if !objects_root.exists() {
        repository.report.push(
            Some(PathBuf::from("objects")),
            "semantic objects directory is missing",
        );
        return;
    }

    for entry in WalkDir::new(&objects_root) {
        let Ok(entry) = entry else {
            repository.report.push(
                Some(PathBuf::from("objects")),
                "failed to read semantic object entry",
            );
            continue;
        };
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("md")
        {
            continue;
        }
        let relative_path = relative_path(root, entry.path());
        match fs::read_to_string(entry.path()) {
            Ok(content) => match parse_semantic_object(&relative_path, &content) {
                Ok(object) => repository.objects.push(object),
                Err(error) => repository.report.push(
                    Some(relative_path),
                    format!("failed to parse semantic object: {error}"),
                ),
            },
            Err(error) => repository.report.push(
                Some(relative_path),
                format!("failed to read semantic object: {error}"),
            ),
        }
    }
}

fn load_projections(root: &Path, repository: &mut SemanticRepository) {
    let projections_root = root.join("projections");
    if !projections_root.exists() {
        repository.report.push(
            Some(PathBuf::from("projections")),
            "semantic projections directory is missing",
        );
        return;
    }

    for entry in WalkDir::new(&projections_root) {
        let Ok(entry) = entry else {
            repository.report.push(
                Some(PathBuf::from("projections")),
                "failed to read semantic projection entry",
            );
            continue;
        };
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("md")
        {
            continue;
        }
        let relative_path = relative_path(root, entry.path());
        match fs::read_to_string(entry.path()) {
            Ok(content) => match parse_semantic_projection(&relative_path, &content) {
                Ok(projection) => repository.projections.push(projection),
                Err(error) => repository.report.push(
                    Some(relative_path),
                    format!("failed to parse semantic projection: {error}"),
                ),
            },
            Err(error) => repository.report.push(
                Some(relative_path),
                format!("failed to read semantic projection: {error}"),
            ),
        }
    }
}

fn load_change_intents(root: &Path, repository: &mut SemanticRepository) {
    let intents_root = root.join("change-intents");
    if !intents_root.exists() {
        return;
    }

    for entry in WalkDir::new(&intents_root) {
        let Ok(entry) = entry else {
            repository.report.push(
                Some(PathBuf::from("change-intents")),
                "failed to read semantic change-intent entry",
            );
            continue;
        };
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("md")
        {
            continue;
        }
        let relative_path = relative_path(root, entry.path());
        match fs::read_to_string(entry.path()) {
            Ok(content) => match parse_semantic_change_intent(&relative_path, &content) {
                Ok(intent) => repository.change_intents.push(intent),
                Err(error) => repository.report.push(
                    Some(relative_path),
                    format!("failed to parse semantic change intent: {error}"),
                ),
            },
            Err(error) => repository.report.push(
                Some(relative_path),
                format!("failed to read semantic change intent: {error}"),
            ),
        }
    }
}

fn validate_repository(repository: &mut SemanticRepository) {
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
            super::types::SemanticConfidenceSource::LlmSuggested
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

fn semantic_projection_source_revision_from_map(
    projection: &SemanticProjection,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
) -> Option<String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"semantic-ssot-source-revision-v1\0");
    update_hash_field(&mut hasher, "projection", projection.projection.as_str());
    for object_id in &projection.source_objects {
        let object = object_by_id.get(object_id.as_str())?;
        update_hash_field(&mut hasher, "object.id", object.id.as_str());
        update_hash_field(&mut hasher, "object.kind", object_kind_token(&object.kind));
        update_hash_field(&mut hasher, "object.title", object.title.as_str());
        update_hash_field(&mut hasher, "object.status", status_token(&object.status));
        update_hash_field(
            &mut hasher,
            "object.confidence.score",
            &format!("{:.12}", object.confidence.score),
        );
        update_hash_field(
            &mut hasher,
            "object.confidence.source",
            confidence_source_token(&object.confidence.source),
        );
        for owner in &object.owners {
            update_hash_field(&mut hasher, "object.owner.scope", owner.scope.as_str());
            update_hash_field(&mut hasher, "object.owner.role", owner.role.as_str());
        }
        update_hash_field(
            &mut hasher,
            "object.provenance.source",
            object.provenance.source.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "object.provenance.recorded_by",
            object.provenance.recorded_by.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "object.provenance.recorded_at",
            object.provenance.recorded_at.as_str(),
        );
        for command in &object.verification.required {
            update_hash_field(
                &mut hasher,
                "object.verification.required",
                command.as_str(),
            );
        }
        for evidence in &object.verification.evidence {
            update_hash_field(
                &mut hasher,
                "object.verification.evidence",
                evidence.as_str(),
            );
        }
        for relation in &object.relations {
            update_hash_field(
                &mut hasher,
                "object.relation.kind",
                relation_kind_token(&relation.kind),
            );
            update_hash_field(
                &mut hasher,
                "object.relation.target",
                relation.target.as_str(),
            );
        }
        update_hash_field(
            &mut hasher,
            "object.source_path",
            object.source_path.to_string_lossy().as_ref(),
        );
        update_hash_field(&mut hasher, "object.body", object.body.as_str());
    }
    Some(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn update_hash_field(hasher: &mut blake3::Hasher, name: &str, value: &str) {
    hasher.update(name.as_bytes());
    hasher.update(b"\0");
    hasher.update(value.as_bytes());
    hasher.update(b"\0");
}

fn object_kind_token(kind: &SemanticObjectKind) -> &'static str {
    kind.id_prefix()
}

fn status_token(status: &SemanticStatus) -> &'static str {
    match status {
        SemanticStatus::Draft => "draft",
        SemanticStatus::Candidate => "candidate",
        SemanticStatus::Active => "active",
        SemanticStatus::Superseded => "superseded",
        SemanticStatus::Deprecated => "deprecated",
        SemanticStatus::Retired => "retired",
    }
}

fn confidence_source_token(source: &SemanticConfidenceSource) -> &'static str {
    match source {
        SemanticConfidenceSource::HumanSigned => "human_signed",
        SemanticConfidenceSource::Verified => "verified",
        SemanticConfidenceSource::LlmSuggested => "llm_suggested",
    }
}

fn relation_kind_token(kind: &SemanticRelationKind) -> &'static str {
    match kind {
        SemanticRelationKind::Contains => "contains",
        SemanticRelationKind::DependsOn => "depends_on",
        SemanticRelationKind::Constrains => "constrains",
        SemanticRelationKind::Implements => "implements",
        SemanticRelationKind::Governs => "governs",
        SemanticRelationKind::Affects => "affects",
        SemanticRelationKind::Validates => "validates",
        SemanticRelationKind::Supersedes => "supersedes",
        SemanticRelationKind::ProjectsTo => "projects_to",
        SemanticRelationKind::ConsumedBy => "consumed_by",
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

fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map_or_else(|_| path.to_path_buf(), std::path::Path::to_path_buf)
}
