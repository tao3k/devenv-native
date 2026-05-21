//! Project semantic SSOT scopes into `WendaoGraph.jl` page-index reasoning tables.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use xiuxian_wendao_parsers::semantic_ssot::{
    SemanticObject, SemanticObjectKind, SemanticRelationEdge, SemanticRelationKind,
    SemanticScopeBundle, SemanticStatus,
};

use super::page_index::{
    PageIndexReasoningEdgeRow, PageIndexReasoningNodeRow,
    build_page_index_reasoning_request_bundle_from_rows, semantic_usize_to_i64,
};
use super::types::{
    LinkGraphWendaoGraphEvidenceError, WendaoGraphPageIndexReasoningRequestBundle,
    WendaoGraphPageIndexReasoningRequestOptions, WendaoGraphPageIndexReasoningSeed,
};

/// Build a `WendaoGraph` `PageIndex` reasoning request bundle from semantic SSOT scope facts.
///
/// The semantic scope remains the source of truth. This function only projects
/// those facts into existing `PageIndex` request tables so Julia can compute
/// derived reasoning evidence.
///
/// # Errors
///
/// Returns an error when semantic relations reference objects outside the
/// projected scope, semantic containment cycles prevent deterministic depth
/// assignment, seed rows are invalid, or Arrow/schema construction fails.
pub fn build_semantic_scope_page_index_reasoning_request_bundle(
    scope: &SemanticScopeBundle,
) -> Result<WendaoGraphPageIndexReasoningRequestBundle, LinkGraphWendaoGraphEvidenceError> {
    let options = semantic_scope_page_index_reasoning_default_options(scope);
    build_semantic_scope_page_index_reasoning_request_bundle_with_options(scope, &options)
}

/// Build a `WendaoGraph` `PageIndex` reasoning request bundle from semantic SSOT
/// scope facts and explicit seed options.
///
/// # Errors
///
/// Returns an error when semantic relations reference objects outside the
/// projected scope, semantic containment cycles prevent deterministic depth
/// assignment, seed rows are invalid, or Arrow/schema construction fails.
pub fn build_semantic_scope_page_index_reasoning_request_bundle_with_options(
    scope: &SemanticScopeBundle,
    options: &WendaoGraphPageIndexReasoningRequestOptions,
) -> Result<WendaoGraphPageIndexReasoningRequestBundle, LinkGraphWendaoGraphEvidenceError> {
    let projection = SemanticPageIndexProjection::from_scope(scope)?;
    build_page_index_reasoning_request_bundle_from_rows(
        &projection.nodes,
        &projection.edges,
        &projection.node_ids,
        &options.seeds,
    )
}

/// Build default `PageIndex` reasoning seeds from semantic scope request anchors.
#[must_use]
pub fn semantic_scope_page_index_reasoning_default_options(
    scope: &SemanticScopeBundle,
) -> WendaoGraphPageIndexReasoningRequestOptions {
    let object_ids = scope
        .objects
        .iter()
        .map(|object| object.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut options = WendaoGraphPageIndexReasoningRequestOptions::default();

    if let Some(task_id) = scope.task_id.as_deref()
        && object_ids.contains(task_id)
        && seen.insert(task_id.to_string())
    {
        options.seeds.push(WendaoGraphPageIndexReasoningSeed::new(
            task_id,
            1.0,
            "semantic_task_anchor",
        ));
    }

    for object_id in &scope.requested_object_ids {
        if object_ids.contains(object_id.as_str()) && seen.insert(object_id.clone()) {
            options.seeds.push(WendaoGraphPageIndexReasoningSeed::new(
                object_id,
                0.8,
                "semantic_requested_object",
            ));
        }
    }

    options
}

struct SemanticPageIndexProjection {
    nodes: Vec<PageIndexReasoningNodeRow>,
    edges: Vec<PageIndexReasoningEdgeRow>,
    node_ids: BTreeSet<String>,
}

impl SemanticPageIndexProjection {
    fn from_scope(scope: &SemanticScopeBundle) -> Result<Self, LinkGraphWendaoGraphEvidenceError> {
        let node_ids = scope
            .objects
            .iter()
            .map(|object| object.id.clone())
            .collect::<BTreeSet<_>>();
        validate_relation_nodes(&scope.relations, &node_ids)?;

        let parent_by_child = semantic_parent_by_child(&scope.relations);
        let mut objects = scope.objects.iter().collect::<Vec<_>>();
        objects.sort_by(|left, right| left.id.cmp(&right.id));

        let nodes = objects
            .into_iter()
            .enumerate()
            .map(|(rank, object)| semantic_object_node_row(object, rank, &parent_by_child))
            .collect::<Result<Vec<_>, _>>()?;
        let edges = scope
            .relations
            .iter()
            .map(semantic_relation_edge_row)
            .collect::<Vec<_>>();

        Ok(Self {
            nodes,
            edges,
            node_ids,
        })
    }
}

fn semantic_object_node_row(
    object: &SemanticObject,
    rank: usize,
    parent_by_child: &BTreeMap<String, String>,
) -> Result<PageIndexReasoningNodeRow, LinkGraphWendaoGraphEvidenceError> {
    let depth = semantic_depth(&object.id, parent_by_child)?;
    Ok(PageIndexReasoningNodeRow {
        node_id: object.id.clone(),
        page_id: semantic_page_id(object),
        parent_id: parent_by_child.get(&object.id).cloned().unwrap_or_default(),
        depth: semantic_usize_to_i64("depth", depth)?,
        rank: semantic_usize_to_i64("rank", rank)?,
        title: object.title.clone(),
        summary: semantic_summary(object),
        line_start: 1,
        line_end: semantic_usize_to_i64("line_end", semantic_line_count(object))?,
        token_count: semantic_usize_to_i64("token_count", semantic_token_count(object))?,
    })
}

fn semantic_relation_edge_row(relation: &SemanticRelationEdge) -> PageIndexReasoningEdgeRow {
    PageIndexReasoningEdgeRow {
        source_id: relation.source.clone(),
        target_id: relation.target.clone(),
        edge_kind: semantic_relation_kind_token(&relation.kind).to_string(),
        weight: semantic_relation_weight(&relation.kind),
    }
}

fn validate_relation_nodes(
    relations: &[SemanticRelationEdge],
    node_ids: &BTreeSet<String>,
) -> Result<(), LinkGraphWendaoGraphEvidenceError> {
    for relation in relations {
        if !node_ids.contains(&relation.source) {
            return Err(
                LinkGraphWendaoGraphEvidenceError::SemanticRelationMissingNode {
                    node_id: relation.source.clone(),
                },
            );
        }
        if !node_ids.contains(&relation.target) {
            return Err(
                LinkGraphWendaoGraphEvidenceError::SemanticRelationMissingNode {
                    node_id: relation.target.clone(),
                },
            );
        }
    }
    Ok(())
}

fn semantic_parent_by_child(relations: &[SemanticRelationEdge]) -> BTreeMap<String, String> {
    relations
        .iter()
        .filter(|relation| relation.kind == SemanticRelationKind::Contains)
        .map(|relation| (relation.target.clone(), relation.source.clone()))
        .collect()
}

fn semantic_depth(
    object_id: &str,
    parent_by_child: &BTreeMap<String, String>,
) -> Result<usize, LinkGraphWendaoGraphEvidenceError> {
    let mut depth = 0usize;
    let mut current = object_id;
    let mut seen = BTreeSet::new();

    while let Some(parent) = parent_by_child.get(current) {
        if !seen.insert(current.to_string()) {
            return Err(
                LinkGraphWendaoGraphEvidenceError::SemanticContainmentCycle {
                    node_id: object_id.to_string(),
                },
            );
        }
        depth += 1;
        current = parent;
    }

    Ok(depth)
}

fn semantic_page_id(object: &SemanticObject) -> String {
    if !path_is_empty(&object.source_path) {
        return object.source_path.to_string_lossy().into_owned();
    }
    if !object.provenance.source.trim().is_empty() {
        return object.provenance.source.clone();
    }
    "semantic:ssot".to_string()
}

fn path_is_empty(path: &Path) -> bool {
    path.as_os_str().is_empty()
}

fn semantic_summary(object: &SemanticObject) -> String {
    let owner_refs = object
        .owners
        .iter()
        .map(|owner| format!("{}:{}", owner.scope, owner.role))
        .collect::<Vec<_>>()
        .join(",");
    let source_path = if path_is_empty(&object.source_path) {
        ""
    } else {
        object.source_path.to_str().unwrap_or_default()
    };
    format!(
        "semantic_kind={}; semantic_status={}; confidence={:.3}; confidence_source={}; owners={}; provenance_source={}; source_path={}; required_validations={}",
        semantic_object_kind_token(&object.kind),
        semantic_status_token(&object.status),
        object.confidence.score,
        semantic_confidence_source_token(&object.confidence.source),
        owner_refs,
        object.provenance.source,
        source_path,
        object.verification.required.join(",")
    )
}

fn semantic_line_count(object: &SemanticObject) -> usize {
    object.body.lines().count().max(1)
}

fn semantic_token_count(object: &SemanticObject) -> usize {
    object
        .body
        .split_whitespace()
        .count()
        .max(object.title.split_whitespace().count())
        .max(1)
}

fn semantic_object_kind_token(kind: &SemanticObjectKind) -> &'static str {
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

fn semantic_confidence_source_token(
    source: &xiuxian_wendao_parsers::semantic_ssot::SemanticConfidenceSource,
) -> &'static str {
    match source {
        xiuxian_wendao_parsers::semantic_ssot::SemanticConfidenceSource::HumanSigned => {
            "human_signed"
        }
        xiuxian_wendao_parsers::semantic_ssot::SemanticConfidenceSource::Verified => "verified",
        xiuxian_wendao_parsers::semantic_ssot::SemanticConfidenceSource::LlmSuggested => {
            "llm_suggested"
        }
    }
}

fn semantic_relation_kind_token(kind: &SemanticRelationKind) -> &'static str {
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

fn semantic_relation_weight(kind: &SemanticRelationKind) -> f64 {
    match kind {
        SemanticRelationKind::Contains => 1.0,
        SemanticRelationKind::Governs
        | SemanticRelationKind::Constrains
        | SemanticRelationKind::Validates => 0.95,
        SemanticRelationKind::Implements | SemanticRelationKind::DependsOn => 0.9,
        SemanticRelationKind::ProjectsTo | SemanticRelationKind::ConsumedBy => 0.85,
        SemanticRelationKind::Affects | SemanticRelationKind::Supersedes => 0.8,
    }
}
