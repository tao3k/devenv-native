//! Repo-native semantic SSOT loading, validation, and scope APIs.

use super::types::{
    SemanticBundleProvenance, SemanticConfidenceSource, SemanticObject, SemanticObjectKind,
    SemanticProjection, SemanticProjectionStaleness, SemanticRelationEdge, SemanticRelationKind,
    SemanticRepository, SemanticScopeBundle, SemanticScopeRequest, SemanticStatus,
    SemanticValidationReport,
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

/// Load and validate one semantic repository root.
#[must_use]
pub fn load_semantic_repository(root: impl AsRef<Path>) -> SemanticRepository {
    let root = root.as_ref().to_path_buf();
    let mut repository = SemanticRepository {
        root: root.clone(),
        objects: Vec::new(),
        projections: Vec::new(),
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
    validate_repository(&mut repository);
    repository
}

/// Compute the source revision for one projection from its referenced objects.
#[must_use]
pub fn semantic_projection_source_revision(
    repository: &SemanticRepository,
    projection: &SemanticProjection,
) -> Option<String> {
    let object_by_id = repository
        .objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<BTreeMap<_, _>>();
    semantic_projection_source_revision_from_map(projection, &object_by_id)
}

/// Build a deterministic semantic scope bundle from a loaded repository.
#[must_use]
pub fn semantic_scope_bundle(
    repository: &SemanticRepository,
    request: &SemanticScopeRequest,
) -> SemanticScopeBundle {
    let object_by_id = repository
        .objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<BTreeMap<_, _>>();
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
        select_requested_id(task_id, &object_by_id, &mut selected, &mut unresolved);
    }

    for object_id in &request.object_ids {
        requested.push(object_id.clone());
        select_requested_id(object_id, &object_by_id, &mut selected, &mut unresolved);
    }

    let anchors = selected.clone();
    for object_id in anchors {
        let Some(object) = object_by_id.get(object_id.as_str()) else {
            continue;
        };
        for relation in &object.relations {
            if let Some(target) = object_by_id.get(relation.target.as_str()) {
                if target.status == SemanticStatus::Active {
                    selected.insert(target.id.clone());
                }
            }
        }
    }

    let mut objects = selected
        .iter()
        .filter_map(|object_id| object_by_id.get(object_id.as_str()).copied())
        .cloned()
        .collect::<Vec<_>>();
    objects.sort_by(|left, right| left.id.cmp(&right.id));

    let selected_ids = objects
        .iter()
        .map(|object| object.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut relations = objects
        .iter()
        .flat_map(|object| {
            object.relations.iter().filter_map(|relation| {
                selected_ids
                    .contains(relation.target.as_str())
                    .then(|| SemanticRelationEdge {
                        source: object.id.clone(),
                        kind: relation.kind.clone(),
                        target: relation.target.clone(),
                    })
            })
        })
        .collect::<Vec<_>>();
    relations.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then_with(|| left.target.cmp(&right.target))
    });

    let affected_invariants = objects
        .iter()
        .filter(|object| object.kind == SemanticObjectKind::Invariant)
        .map(|object| object.id.clone())
        .collect::<Vec<_>>();

    let required_validations = objects
        .iter()
        .flat_map(|object| object.verification.required.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let provenance = objects
        .iter()
        .map(|object| SemanticBundleProvenance {
            object_id: object.id.clone(),
            source_path: object.source_path.clone(),
            source: object.provenance.source.clone(),
        })
        .collect::<Vec<_>>();

    let active_projection = active_projection(repository);

    SemanticScopeBundle {
        task_id: request.task_id.clone(),
        requested_object_ids: requested,
        objects,
        relations,
        affected_invariants,
        required_validations,
        projection_revision: active_projection
            .map(|projection| projection.projection_revision.clone())
            .unwrap_or_else(|| "semantic-ssot-unprojected".to_string()),
        projection_source_revision: active_projection.map(|projection| {
            semantic_projection_source_revision(repository, projection)
                .unwrap_or_else(|| projection.source_revision.clone())
        }),
        projection_staleness: active_projection.map(|projection| projection.staleness.clone()),
        provenance,
        unresolved_ids: unresolved.into_iter().collect(),
    }
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

    for projection in &repository.projections {
        validate_projection(
            projection,
            &object_ids,
            &object_by_id,
            &mut repository.report,
        );
    }
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
        || !object
            .id
            .as_bytes()
            .get(object.kind.id_prefix().len())
            .is_some_and(|value| *value == b'.')
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
    validate_owners(object, path.clone(), report);
    validate_provenance(object, path.clone(), report);
    validate_verification(object, path.clone(), report);
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

fn validate_owners(
    object: &SemanticObject,
    path: Option<PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if object.owners.is_empty() {
        report.push(path.clone(), "semantic object `owners` must be non-empty");
    }
    for owner in &object.owners {
        validate_non_empty(
            &owner.scope,
            "semantic owner `scope` must be non-empty",
            path.clone(),
            report,
        );
        validate_non_empty(
            &owner.role,
            "semantic owner `role` must be non-empty",
            path.clone(),
            report,
        );
    }
}

fn validate_provenance(
    object: &SemanticObject,
    path: Option<PathBuf>,
    report: &mut SemanticValidationReport,
) {
    validate_non_empty(
        &object.provenance.source,
        "semantic provenance `source` must be non-empty",
        path.clone(),
        report,
    );
    validate_non_empty(
        &object.provenance.recorded_by,
        "semantic provenance `recorded_by` must be non-empty",
        path.clone(),
        report,
    );
    validate_non_empty(
        &object.provenance.recorded_at,
        "semantic provenance `recorded_at` must be non-empty",
        path,
        report,
    );
}

fn validate_verification(
    object: &SemanticObject,
    path: Option<PathBuf>,
    report: &mut SemanticValidationReport,
) {
    if object.verification.required.is_empty() {
        report.push(
            path.clone(),
            "semantic verification `required` must be non-empty",
        );
    }
    for command in &object.verification.required {
        validate_non_empty(
            command,
            "semantic verification command must be non-empty",
            path.clone(),
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
        .filter(|projection| projection.status == SemanticStatus::Active)
        .next()
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
