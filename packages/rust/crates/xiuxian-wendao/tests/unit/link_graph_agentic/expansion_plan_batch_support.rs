#[cfg(feature = "julia")]
use arrow::record_batch::RecordBatch;
#[cfg(feature = "julia")]
use std::collections::{BTreeMap, BTreeSet, HashSet};
#[cfg(feature = "julia")]
use xiuxian_wendao::{
    LinkGraphAgenticExpansionPlan, LinkGraphAgenticWorkerPlan, LinkGraphIndex,
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_builtin::{
    GraphStructuralFilterConstraint, GraphStructuralFilterScoreRow,
    GraphStructuralRawConnectedPairCollectionCandidateInputs, GraphStructuralRerankScoreRow,
    build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_keyword_tag_query_context,
    fetch_graph_structural_generic_topology_filter_rows_for_repository_from_raw_connected_pair_collections,
    fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections,
};

#[cfg(feature = "julia")]
use super::expansion_support::{
    build_raw_connected_pair_collection_candidate_from_pairs, worker_partition_pair_groups,
};

#[cfg(feature = "julia")]
pub(super) struct PlanAwareGenericTopologyBatchFixture {
    pub query_id: String,
    pub expected_retrieval_layer: i32,
    pub expected_query_max_layers: i32,
    pub expected_constraint_kind: String,
    pub expected_required_boundary_size: i32,
    pub keyword_anchors: Vec<String>,
    pub tag_anchors: Vec<String>,
    pub anchor_planes: Vec<String>,
    pub anchor_values: Vec<String>,
    pub edge_constraint_kinds: Vec<String>,
    pub candidates: Vec<PlanAwareGenericTopologyCandidateFixture>,
}

#[cfg(feature = "julia")]
pub(super) struct PlanAwareGenericTopologyCandidateFixture {
    pub candidate_id: String,
    pub expected_nodes: usize,
    pub expected_edges: usize,
    pub expected_candidate_node_ids: Vec<String>,
    pub expected_candidate_edge_sources: Vec<String>,
    pub expected_candidate_edge_destinations: Vec<String>,
    pub expected_candidate_edge_kinds: Vec<String>,
    pub expected_semantic_score: f64,
    pub expected_dependency_score: f64,
    pub expected_keyword_score: f64,
    pub expected_tag_score: f64,
    pub candidate: GraphStructuralRawConnectedPairCollectionCandidateInputs,
}

#[cfg(feature = "julia")]
fn normalized_query_keywords(plan: &LinkGraphAgenticExpansionPlan) -> Vec<String> {
    plan.query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| vec![value.to_string()])
        .unwrap_or_default()
}

#[cfg(feature = "julia")]
fn shared_worker_seed_tags(
    index: &LinkGraphIndex,
    plan: &LinkGraphAgenticExpansionPlan,
    max_workers: usize,
    min_pairs: usize,
) -> Vec<String> {
    let seed_ids = worker_partition_pair_groups(plan, max_workers, min_pairs)
        .into_iter()
        .flat_map(|worker| worker.seed_ids.iter().cloned())
        .collect::<BTreeSet<_>>();

    let mut tag_sets = seed_ids
        .into_iter()
        .filter_map(|seed_id| {
            index.metadata(&seed_id).map(|metadata| {
                metadata
                    .tags
                    .iter()
                    .map(|tag| tag.trim().to_string())
                    .filter(|tag| !tag.is_empty())
                    .collect::<HashSet<_>>()
            })
        })
        .filter(|tags| !tags.is_empty())
        .collect::<Vec<_>>();

    let Some(first) = tag_sets.pop() else {
        return Vec::new();
    };

    let mut shared = first;
    for tags in tag_sets {
        shared.retain(|tag| tags.contains(tag));
    }

    let mut shared = shared.into_iter().collect::<Vec<_>>();
    shared.sort();
    shared
}

#[cfg(feature = "julia")]
fn worker_dependency_score(worker: &LinkGraphAgenticWorkerPlan) -> f64 {
    if worker.pairs.is_empty() {
        return 0.0;
    }
    worker.pairs.iter().map(|pair| pair.priority).sum::<f64>()
        / pair_count_as_f64(worker.pairs.len())
}

#[cfg(feature = "julia")]
fn worker_semantic_score(worker: &LinkGraphAgenticWorkerPlan) -> f64 {
    if worker.pairs.is_empty() {
        return 0.0;
    }
    worker.pairs.iter().map(|pair| pair.priority).sum::<f64>()
        / pair_count_as_f64(worker.pairs.len())
}

#[cfg(feature = "julia")]
fn binary_plane_score(matched: bool) -> f64 {
    if matched { 1.0 } else { 0.0 }
}

#[cfg(feature = "julia")]
fn pair_count_as_f64(pair_count: usize) -> f64 {
    let pair_count = u32::try_from(pair_count)
        .unwrap_or_else(|error| panic!("worker pair count should fit within u32: {error}"));
    f64::from(pair_count)
}

#[cfg(feature = "julia")]
fn plan_aware_required_boundary_size(
    anchor_values: &[String],
    candidates: &[PlanAwareGenericTopologyCandidateFixture],
) -> Result<i32, Box<dyn std::error::Error>> {
    let requested_boundary_size = anchor_values.len().clamp(1, 2);
    let min_candidate_nodes = candidates
        .iter()
        .map(|candidate| candidate.expected_nodes)
        .min()
        .unwrap_or(1);
    i32::try_from(std::cmp::min(requested_boundary_size, min_candidate_nodes))
        .map_err(Box::<dyn std::error::Error>::from)
}

#[cfg(feature = "julia")]
fn plan_aware_constraint_kind(anchor_values: &[String], required_boundary_size: i32) -> String {
    if required_boundary_size > 1 && anchor_values.len() > 1 {
        "boundary_match".to_string()
    } else {
        "pin_assignment".to_string()
    }
}

#[cfg(feature = "julia")]
fn normalize_pair_endpoints(left_id: &str, right_id: &str) -> (String, String) {
    if left_id <= right_id {
        (left_id.to_string(), right_id.to_string())
    } else {
        (right_id.to_string(), left_id.to_string())
    }
}

#[cfg(feature = "julia")]
fn worker_expected_topology(
    worker: &LinkGraphAgenticWorkerPlan,
    fallback_edge_kind: &str,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut node_ids = Vec::new();
    let mut seen_nodes = HashSet::new();
    let mut edge_sources = Vec::new();
    let mut edge_destinations = Vec::new();
    let mut edge_kinds = Vec::new();
    let mut seen_edges = HashSet::new();

    for pair in &worker.pairs {
        let (left_id, right_id) = normalize_pair_endpoints(&pair.left_id, &pair.right_id);
        if seen_nodes.insert(left_id.clone()) {
            node_ids.push(left_id.clone());
        }
        if seen_nodes.insert(right_id.clone()) {
            node_ids.push(right_id.clone());
        }
        let edge_key = (
            left_id.clone(),
            right_id.clone(),
            fallback_edge_kind.to_string(),
        );
        if seen_edges.insert(edge_key) {
            edge_sources.push(left_id.clone());
            edge_destinations.push(right_id.clone());
            edge_kinds.push(fallback_edge_kind.to_string());
        }
    }

    (node_ids, edge_sources, edge_destinations, edge_kinds)
}

#[cfg(feature = "julia")]
pub(super) fn build_worker_partition_plan_aware_generic_topology_batch_fixture(
    index: &LinkGraphIndex,
    plan: &LinkGraphAgenticExpansionPlan,
    max_workers: usize,
    min_pairs: usize,
    query_id: &str,
    candidate_id_prefix: &str,
    relation_edge_kind: &str,
) -> Result<PlanAwareGenericTopologyBatchFixture, Box<dyn std::error::Error>> {
    let expected_retrieval_layer = 0;
    let expected_query_max_layers = 2;
    let keyword_anchors = normalized_query_keywords(plan);
    let tag_anchors = shared_worker_seed_tags(index, plan, max_workers, min_pairs);
    let anchor_planes = keyword_anchors
        .iter()
        .map(|_| "keyword".to_string())
        .chain(tag_anchors.iter().map(|_| "tag".to_string()))
        .collect::<Vec<_>>();
    let anchor_values = keyword_anchors
        .iter()
        .cloned()
        .chain(tag_anchors.iter().cloned())
        .collect::<Vec<_>>();

    if keyword_anchors.is_empty() && tag_anchors.is_empty() {
        return Err("plan-aware generic-topology batch requires at least one query anchor".into());
    }

    let worker_groups = worker_partition_pair_groups(plan, max_workers, min_pairs);
    let expected_keyword_score = binary_plane_score(!keyword_anchors.is_empty());
    let expected_tag_score = binary_plane_score(!tag_anchors.is_empty());
    let candidates = worker_groups
        .into_iter()
        .map(|worker| {
            let candidate_id = format!("{candidate_id_prefix}-{}", worker.worker_id);
            let (
                expected_candidate_node_ids,
                expected_candidate_edge_sources,
                expected_candidate_edge_destinations,
                expected_candidate_edge_kinds,
            ) = worker_expected_topology(worker, relation_edge_kind);
            let (expected_nodes, expected_edges) =
                worker
                    .pairs
                    .iter()
                    .fold((BTreeSet::new(), BTreeSet::new()), |mut acc, pair| {
                        acc.0.insert(pair.left_id.clone());
                        acc.0.insert(pair.right_id.clone());
                        let (left_id, right_id) = if pair.left_id <= pair.right_id {
                            (pair.left_id.clone(), pair.right_id.clone())
                        } else {
                            (pair.right_id.clone(), pair.left_id.clone())
                        };
                        acc.1.insert((left_id, right_id));
                        acc
                    });
            let expected_semantic_score = worker_semantic_score(worker);
            let expected_dependency_score = worker_dependency_score(worker);
            let candidate = build_raw_connected_pair_collection_candidate_from_pairs(
                candidate_id.clone(),
                &worker.pairs,
                relation_edge_kind,
                expected_dependency_score,
                expected_keyword_score,
                expected_tag_score,
            )?;
            Ok(PlanAwareGenericTopologyCandidateFixture {
                candidate_id,
                expected_nodes: expected_nodes.len(),
                expected_edges: expected_edges.len(),
                expected_candidate_node_ids,
                expected_candidate_edge_sources,
                expected_candidate_edge_destinations,
                expected_candidate_edge_kinds,
                expected_semantic_score,
                expected_dependency_score,
                expected_keyword_score,
                expected_tag_score,
                candidate,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let expected_required_boundary_size =
        plan_aware_required_boundary_size(&anchor_values, &candidates)?;
    let expected_constraint_kind =
        plan_aware_constraint_kind(&anchor_values, expected_required_boundary_size);

    Ok(PlanAwareGenericTopologyBatchFixture {
        query_id: query_id.to_string(),
        expected_retrieval_layer,
        expected_query_max_layers,
        expected_constraint_kind,
        expected_required_boundary_size,
        keyword_anchors,
        tag_anchors,
        anchor_planes,
        anchor_values,
        edge_constraint_kinds: vec![relation_edge_kind.to_string()],
        candidates,
    })
}

#[cfg(feature = "julia")]
pub(super) fn build_plan_aware_generic_topology_rerank_request_batch(
    fixture: &PlanAwareGenericTopologyBatchFixture,
) -> Result<RecordBatch, Box<dyn std::error::Error>> {
    let query = build_graph_structural_keyword_tag_query_context(
        &fixture.query_id,
        fixture.expected_retrieval_layer,
        fixture.expected_query_max_layers,
        fixture.keyword_anchors.clone(),
        fixture.tag_anchors.clone(),
        fixture.edge_constraint_kinds.clone(),
    )?;
    let candidates = fixture
        .candidates
        .iter()
        .map(|candidate| candidate.candidate.clone())
        .collect::<Vec<_>>();
    build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections(
        &query,
        &candidates,
    )
    .map_err(Box::<dyn std::error::Error>::from)
}

#[cfg(feature = "julia")]
pub(super) fn build_plan_aware_generic_topology_filter_request_batch(
    fixture: &PlanAwareGenericTopologyBatchFixture,
    constraint_kind: &str,
    required_boundary_size: i32,
) -> Result<RecordBatch, Box<dyn std::error::Error>> {
    let query = build_graph_structural_keyword_tag_query_context(
        &fixture.query_id,
        fixture.expected_retrieval_layer,
        fixture.expected_query_max_layers,
        fixture.keyword_anchors.clone(),
        fixture.tag_anchors.clone(),
        fixture.edge_constraint_kinds.clone(),
    )?;
    let constraint = GraphStructuralFilterConstraint::new(constraint_kind, required_boundary_size)?;
    let candidates = fixture
        .candidates
        .iter()
        .map(|candidate| candidate.candidate.clone())
        .collect::<Vec<_>>();
    build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections(
        &query,
        &constraint,
        &candidates,
    )
    .map_err(Box::<dyn std::error::Error>::from)
}

#[cfg(feature = "julia")]
pub(super) async fn fetch_plan_aware_generic_topology_rows_via_manifest_discovery(
    base_url: impl Into<String>,
    fixture: &PlanAwareGenericTopologyBatchFixture,
) -> Result<BTreeMap<String, GraphStructuralRerankScoreRow>, Box<dyn std::error::Error>> {
    let repository = RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "capability_manifest_transport": {
                    "base_url": base_url.into(),
                    "route": "/plugin/capabilities",
                    "schema_version": "v0-draft"
                }
            }),
        }],
    };

    let query = build_graph_structural_keyword_tag_query_context(
        &fixture.query_id,
        fixture.expected_retrieval_layer,
        fixture.expected_query_max_layers,
        fixture.keyword_anchors.clone(),
        fixture.tag_anchors.clone(),
        fixture.edge_constraint_kinds.clone(),
    )?;
    let candidates = fixture
        .candidates
        .iter()
        .map(|candidate| candidate.candidate.clone())
        .collect::<Vec<_>>();

    fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections(
        &repository,
        &query,
        &candidates,
    )
    .await
    .map_err(Box::<dyn std::error::Error>::from)
}

#[cfg(feature = "julia")]
pub(super) async fn fetch_plan_aware_generic_topology_filter_rows_via_manifest_discovery(
    base_url: impl Into<String>,
    fixture: &PlanAwareGenericTopologyBatchFixture,
    constraint_kind: &str,
    required_boundary_size: i32,
) -> Result<BTreeMap<String, GraphStructuralFilterScoreRow>, Box<dyn std::error::Error>> {
    let repository = RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "capability_manifest_transport": {
                    "base_url": base_url.into(),
                    "route": "/plugin/capabilities",
                    "schema_version": "v0-draft"
                }
            }),
        }],
    };

    let query = build_graph_structural_keyword_tag_query_context(
        &fixture.query_id,
        fixture.expected_retrieval_layer,
        fixture.expected_query_max_layers,
        fixture.keyword_anchors.clone(),
        fixture.tag_anchors.clone(),
        fixture.edge_constraint_kinds.clone(),
    )?;
    let constraint = GraphStructuralFilterConstraint::new(constraint_kind, required_boundary_size)?;
    let candidates = fixture
        .candidates
        .iter()
        .map(|candidate| candidate.candidate.clone())
        .collect::<Vec<_>>();

    fetch_graph_structural_generic_topology_filter_rows_for_repository_from_raw_connected_pair_collections(
        &repository,
        &query,
        &constraint,
        &candidates,
    )
    .await
    .map_err(Box::<dyn std::error::Error>::from)
}
