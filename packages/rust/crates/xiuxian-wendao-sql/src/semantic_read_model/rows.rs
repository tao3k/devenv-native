use xiuxian_wendao_parsers::semantic_ssot::{
    SemanticConfidenceSource, SemanticObject, SemanticObjectKind, SemanticProjection,
    SemanticProjectionStaleness, SemanticRelationKind, SemanticRepository, SemanticStatus,
    semantic_projection_source_revision,
};

/// Complete semantic read-model row set.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticReadModelRows {
    /// Rows for the `semantic_objects` table.
    pub objects: Vec<SemanticObjectReadModelRow>,
    /// Rows for the `semantic_relations` table.
    pub relations: Vec<SemanticRelationReadModelRow>,
    /// Rows for the `semantic_projection_state` table.
    pub projection_state: Vec<SemanticProjectionStateReadModelRow>,
}

/// One row in the provisional `semantic_objects` table.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticObjectReadModelRow {
    /// Semantic object id.
    pub id: String,
    /// Semantic object kind token.
    pub kind: String,
    /// Semantic object title.
    pub title: String,
    /// Semantic lifecycle status token.
    pub status: String,
    /// Normalized confidence score.
    pub confidence_score: f64,
    /// Confidence source token.
    pub confidence_source: String,
    /// Number of declared owners.
    pub owner_count: i64,
    /// JSON-encoded owner declarations.
    pub owners_json: String,
    /// Source artifact recorded in object provenance.
    pub provenance_source: String,
    /// Actor that recorded the object.
    pub provenance_recorded_by: String,
    /// Date or timestamp when the object was recorded.
    pub provenance_recorded_at: String,
    /// JSON-encoded required verification commands.
    pub verification_required_json: String,
    /// JSON-encoded verification evidence references.
    pub verification_evidence_json: String,
    /// Number of outgoing semantic relations.
    pub relation_count: i64,
    /// Path relative to the semantic root.
    pub source_path: String,
    /// Source revision carried by the selected read-model projection.
    pub read_model_source_revision: String,
    /// Projection revision carried by the selected read-model projection.
    pub read_model_projection_revision: String,
    /// Projection staleness carried by the selected read-model projection.
    pub read_model_projection_staleness: String,
}

/// One row in the provisional `semantic_relations` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticRelationReadModelRow {
    /// Source semantic object id.
    pub source: String,
    /// Relation kind token.
    pub kind: String,
    /// Target semantic object id.
    pub target: String,
    /// Source object path relative to the semantic root.
    pub source_path: String,
    /// Source revision carried by the selected read-model projection.
    pub read_model_source_revision: String,
    /// Projection revision carried by the selected read-model projection.
    pub read_model_projection_revision: String,
    /// Projection staleness carried by the selected read-model projection.
    pub read_model_projection_staleness: String,
}

/// One row in the provisional `semantic_projection_state` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticProjectionStateReadModelRow {
    /// Projection name.
    pub projection: String,
    /// Projection lifecycle status token.
    pub status: String,
    /// Declared source revision stored on the projection artifact.
    pub source_revision: String,
    /// Current source revision computed from source objects.
    pub current_source_revision: String,
    /// Projection revision identifier.
    pub projection_revision: String,
    /// Projection staleness token.
    pub staleness: String,
    /// Number of source object ids declared by the projection.
    pub source_object_count: i64,
    /// JSON-encoded source object ids.
    pub source_objects_json: String,
    /// Projection artifact path relative to the semantic root.
    pub source_path: String,
}

pub(super) fn build_rows(repository: &SemanticRepository) -> Result<SemanticReadModelRows, String> {
    if !repository.report.is_success() {
        return Err(format!(
            "semantic repository validation failed: {} issue(s)",
            repository.report.issues.len()
        ));
    }

    let read_model_projection = repository
        .projections
        .iter()
        .find(|projection| projection.status == SemanticStatus::Active);
    let read_model_meta = ReadModelProjectionMeta::from_projection(read_model_projection);

    let accepted_ids = repository
        .objects
        .iter()
        .filter(|object| object_is_accepted_for_read_model(object))
        .map(|object| object.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    let objects = repository
        .objects
        .iter()
        .filter(|object| accepted_ids.contains(object.id.as_str()))
        .map(|object| object_row(object, &read_model_meta))
        .collect::<Result<Vec<_>, _>>()?;

    let mut relations = Vec::new();
    for object in repository
        .objects
        .iter()
        .filter(|object| accepted_ids.contains(object.id.as_str()))
    {
        for relation in &object.relations {
            if accepted_ids.contains(relation.target.as_str()) {
                relations.push(SemanticRelationReadModelRow {
                    source: object.id.clone(),
                    kind: relation_kind_token(&relation.kind).to_string(),
                    target: relation.target.clone(),
                    source_path: source_path_string(object),
                    read_model_source_revision: read_model_meta.source_revision.clone(),
                    read_model_projection_revision: read_model_meta.projection_revision.clone(),
                    read_model_projection_staleness: read_model_meta.staleness.clone(),
                });
            }
        }
    }

    let projection_state = repository
        .projections
        .iter()
        .filter(|projection| projection_is_accepted_for_read_model(projection))
        .map(|projection| projection_state_row(repository, projection))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SemanticReadModelRows {
        objects,
        relations,
        projection_state,
    })
}

struct ReadModelProjectionMeta {
    source_revision: String,
    projection_revision: String,
    staleness: String,
}

impl ReadModelProjectionMeta {
    fn from_projection(projection: Option<&SemanticProjection>) -> Self {
        projection.map_or_else(
            || Self {
                source_revision: String::new(),
                projection_revision: String::new(),
                staleness: "unprojected".to_string(),
            },
            |projection| Self {
                source_revision: projection.source_revision.clone(),
                projection_revision: projection.projection_revision.clone(),
                staleness: projection_staleness_token(&projection.staleness).to_string(),
            },
        )
    }
}

fn object_row(
    object: &SemanticObject,
    read_model_meta: &ReadModelProjectionMeta,
) -> Result<SemanticObjectReadModelRow, String> {
    Ok(SemanticObjectReadModelRow {
        id: object.id.clone(),
        kind: object_kind_token(&object.kind).to_string(),
        title: object.title.clone(),
        status: status_token(&object.status).to_string(),
        confidence_score: object.confidence.score,
        confidence_source: confidence_source_token(&object.confidence.source).to_string(),
        owner_count: i64::try_from(object.owners.len()).unwrap_or(i64::MAX),
        owners_json: serde_json::to_string(&object.owners)
            .map_err(|error| format!("failed to encode semantic owners JSON: {error}"))?,
        provenance_source: object.provenance.source.clone(),
        provenance_recorded_by: object.provenance.recorded_by.clone(),
        provenance_recorded_at: object.provenance.recorded_at.clone(),
        verification_required_json: serde_json::to_string(&object.verification.required).map_err(
            |error| format!("failed to encode semantic verification requirements JSON: {error}"),
        )?,
        verification_evidence_json: serde_json::to_string(&object.verification.evidence).map_err(
            |error| format!("failed to encode semantic verification evidence JSON: {error}"),
        )?,
        relation_count: i64::try_from(object.relations.len()).unwrap_or(i64::MAX),
        source_path: source_path_string(object),
        read_model_source_revision: read_model_meta.source_revision.clone(),
        read_model_projection_revision: read_model_meta.projection_revision.clone(),
        read_model_projection_staleness: read_model_meta.staleness.clone(),
    })
}

fn projection_state_row(
    repository: &SemanticRepository,
    projection: &SemanticProjection,
) -> Result<SemanticProjectionStateReadModelRow, String> {
    Ok(SemanticProjectionStateReadModelRow {
        projection: projection.projection.clone(),
        status: status_token(&projection.status).to_string(),
        source_revision: projection.source_revision.clone(),
        current_source_revision: semantic_projection_source_revision(repository, projection)
            .unwrap_or_default(),
        projection_revision: projection.projection_revision.clone(),
        staleness: projection_staleness_token(&projection.staleness).to_string(),
        source_object_count: i64::try_from(projection.source_objects.len()).unwrap_or(i64::MAX),
        source_objects_json: serde_json::to_string(&projection.source_objects).map_err(
            |error| format!("failed to encode semantic projection source objects JSON: {error}"),
        )?,
        source_path: projection.source_path.to_string_lossy().to_string(),
    })
}

fn object_is_accepted_for_read_model(object: &SemanticObject) -> bool {
    matches!(
        object.status,
        SemanticStatus::Active | SemanticStatus::Deprecated | SemanticStatus::Superseded
    )
}

fn projection_is_accepted_for_read_model(projection: &SemanticProjection) -> bool {
    matches!(
        projection.status,
        SemanticStatus::Active | SemanticStatus::Deprecated | SemanticStatus::Superseded
    )
}

fn source_path_string(object: &SemanticObject) -> String {
    object.source_path.to_string_lossy().to_string()
}

fn object_kind_token(kind: &SemanticObjectKind) -> &'static str {
    match kind {
        SemanticObjectKind::Component => "component",
        SemanticObjectKind::Decision => "decision",
        SemanticObjectKind::Invariant => "invariant",
        SemanticObjectKind::Task => "task",
    }
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

fn projection_staleness_token(staleness: &SemanticProjectionStaleness) -> &'static str {
    match staleness {
        SemanticProjectionStaleness::Fresh => "fresh",
        SemanticProjectionStaleness::Stale => "stale",
    }
}
