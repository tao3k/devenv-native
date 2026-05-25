#[cfg(feature = "julia")]
use arrow::{
    array::{Array, ListArray, StringArray},
    record_batch::RecordBatch,
};
#[cfg(feature = "julia")]
use std::collections::BTreeMap;
#[cfg(feature = "julia")]
use std::collections::HashSet;
#[cfg(feature = "julia")]
use xiuxian_julia_core::{
    GraphStructuralKeywordOverlapCandidateMetadataInput,
    GraphStructuralKeywordOverlapCandidateMetadataInputs, GraphStructuralKeywordOverlapQueryInput,
    GraphStructuralKeywordOverlapQueryInputs, GraphStructuralKeywordOverlapRawCandidateInput,
    GraphStructuralKeywordOverlapRawCandidateInputs, GraphStructuralKeywordTagQueryContextInput,
    GraphStructuralQueryContext, GraphStructuralRawConnectedPairCollectionRawTupleInput,
    JuliaContractKind,
};
#[cfg(feature = "julia")]
use xiuxian_wendao::{
    LinkGraphAgenticCandidatePair, LinkGraphAgenticExpansionPlan, LinkGraphAgenticWorkerPlan,
    LinkGraphIndex, RegisteredRepository, RepoIntelligenceError, RepositoryPluginConfig,
    RepositoryRefreshPolicy,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_builtin::{
    GraphStructuralRawConnectedPairCollectionCandidateInputs, GraphStructuralRerankScoreRow,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates,
    fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections,
};

#[cfg(feature = "julia")]
pub(super) struct GenericTopologyCandidateFixture {
    pub candidate_id: String,
    pub expected_nodes: usize,
    pub expected_edges: usize,
    pub candidate: GraphStructuralRawConnectedPairCollectionCandidateInputs,
}

#[cfg(feature = "julia")]
#[derive(Debug, Clone, Copy)]
pub(super) struct GenericTopologyCandidateScores {
    pub dependency: f64,
    pub keyword: f64,
    pub tag: f64,
}

#[cfg(feature = "julia")]
impl GenericTopologyCandidateScores {
    pub(super) const fn new(dependency: f64, keyword: f64, tag: f64) -> Self {
        Self {
            dependency,
            keyword,
            tag,
        }
    }
}

#[cfg(feature = "julia")]
#[derive(Debug, Clone, Copy)]
pub(super) struct GenericTopologyCandidateBuildOptions<'a> {
    pub candidate_id_prefix: &'a str,
    pub fallback_edge_kind: &'a str,
    pub scores: GenericTopologyCandidateScores,
}

#[cfg(feature = "julia")]
impl<'a> GenericTopologyCandidateBuildOptions<'a> {
    pub(super) const fn new(
        candidate_id_prefix: &'a str,
        fallback_edge_kind: &'a str,
        scores: GenericTopologyCandidateScores,
    ) -> Self {
        Self {
            candidate_id_prefix,
            fallback_edge_kind,
            scores,
        }
    }
}

#[cfg(feature = "julia")]
pub(super) fn first_worker_pair(
    plan: &LinkGraphAgenticExpansionPlan,
) -> &LinkGraphAgenticCandidatePair {
    let Some(worker) = plan.workers.first() else {
        panic!("agentic expansion plan should include at least one worker");
    };
    let Some(pair) = worker.pairs.first() else {
        panic!("agentic expansion plan should include at least one pair");
    };
    pair
}

#[cfg(feature = "julia")]
pub(super) fn first_connected_pair_collection(
    plan: &LinkGraphAgenticExpansionPlan,
) -> Vec<LinkGraphAgenticCandidatePair> {
    connected_pair_collections(plan, 1)
        .into_iter()
        .next()
        .unwrap_or_else(|| {
            panic!("agentic expansion plan should include one connected pair collection")
        })
}

#[cfg(feature = "julia")]
pub(super) fn connected_pair_collections(
    plan: &LinkGraphAgenticExpansionPlan,
    max_collections: usize,
) -> Vec<Vec<LinkGraphAgenticCandidatePair>> {
    let mut collections = Vec::new();
    let mut seen_collections = HashSet::<Vec<(String, String)>>::new();

    for worker in &plan.workers {
        for (left_index, left_pair) in worker.pairs.iter().enumerate() {
            let left_ids = HashSet::from([left_pair.left_id.as_str(), left_pair.right_id.as_str()]);
            for right_pair in worker.pairs.iter().skip(left_index + 1) {
                let right_ids =
                    HashSet::from([right_pair.left_id.as_str(), right_pair.right_id.as_str()]);
                let shared_node_count = left_ids.intersection(&right_ids).count();
                let unique_node_count = left_ids.union(&right_ids).count();
                if shared_node_count >= 1 && unique_node_count == 3 {
                    let mut normalized_pairs = vec![
                        normalize_pair_endpoints(&left_pair.left_id, &left_pair.right_id),
                        normalize_pair_endpoints(&right_pair.left_id, &right_pair.right_id),
                    ];
                    normalized_pairs.sort();
                    if seen_collections.insert(normalized_pairs) {
                        collections.push(vec![left_pair.clone(), right_pair.clone()]);
                        if collections.len() >= max_collections {
                            return collections;
                        }
                    }
                }
            }
        }
    }

    collections
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
pub(super) fn build_pair_rerank_request_batch(
    index: &LinkGraphIndex,
    pair: &LinkGraphAgenticCandidatePair,
) -> Result<RecordBatch, Box<dyn std::error::Error>> {
    let left = index
        .metadata(&pair.left_id)
        .ok_or_else(|| format!("missing metadata for `{}`", pair.left_id))?;
    let right = index
        .metadata(&pair.right_id)
        .ok_or_else(|| format!("missing metadata for `{}`", pair.right_id))?;
    let batch =
        build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates(
            &build_graph_structural_keyword_overlap_query_inputs(
                "agentic-query-alpha",
                0,
                1,
                vec!["alpha".to_string()],
                Vec::new(),
            ),
            &[build_graph_structural_keyword_overlap_raw_candidate_inputs(
                build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                    pair.left_id.clone(),
                    pair.right_id.clone(),
                    Vec::new(),
                    left.tags.clone(),
                    right.tags.clone(),
                ),
                pair.priority,
                0.0,
                true,
            )],
        )?;
    Ok(batch)
}

#[cfg(feature = "julia")]
pub(super) fn build_graph_structural_keyword_overlap_query_inputs(
    query_id: impl Into<String>,
    retrieval_layer: i32,
    query_max_layers: i32,
    keyword_anchors: Vec<String>,
    edge_constraint_kinds: Vec<String>,
) -> GraphStructuralKeywordOverlapQueryInputs {
    xiuxian_wendao_builtin::build_graph_structural_keyword_overlap_query_inputs(
        GraphStructuralKeywordOverlapQueryInput {
            query_id: query_id.into(),
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            edge_constraint_kinds,
        },
    )
}

#[cfg(feature = "julia")]
pub(super) fn build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
    left_tags: Vec<String>,
    right_tags: Vec<String>,
) -> GraphStructuralKeywordOverlapCandidateMetadataInputs {
    xiuxian_wendao_builtin::build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
        GraphStructuralKeywordOverlapCandidateMetadataInput {
            left_id: left_id.into(),
            right_id: right_id.into(),
            edge_kinds,
            left_tags,
            right_tags,
        },
    )
}

#[cfg(feature = "julia")]
pub(super) fn build_graph_structural_keyword_overlap_raw_candidate_inputs(
    metadata_inputs: GraphStructuralKeywordOverlapCandidateMetadataInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> GraphStructuralKeywordOverlapRawCandidateInputs {
    xiuxian_wendao_builtin::build_graph_structural_keyword_overlap_raw_candidate_inputs(
        GraphStructuralKeywordOverlapRawCandidateInput {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        },
    )
}

#[cfg(feature = "julia")]
pub(super) fn build_graph_structural_keyword_tag_query_context(
    query_id: impl Into<String>,
    retrieval_layer: i32,
    query_max_layers: i32,
    keyword_anchors: Vec<String>,
    tag_anchors: Vec<String>,
    edge_constraint_kinds: Vec<String>,
) -> Result<GraphStructuralQueryContext, RepoIntelligenceError> {
    xiuxian_wendao_builtin::build_graph_structural_keyword_tag_query_context(
        GraphStructuralKeywordTagQueryContextInput {
            query_id: query_id.into(),
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            tag_anchors,
            edge_constraint_kinds,
        },
    )
}

#[cfg(feature = "julia")]
fn pair_collection_shape(pair_collection: &[LinkGraphAgenticCandidatePair]) -> (usize, usize) {
    let mut node_ids = HashSet::<String>::new();
    let mut normalized_edges = HashSet::<(String, String)>::new();

    for pair in pair_collection {
        node_ids.insert(pair.left_id.clone());
        node_ids.insert(pair.right_id.clone());
        normalized_edges.insert(normalize_pair_endpoints(&pair.left_id, &pair.right_id));
    }

    (node_ids.len(), normalized_edges.len())
}

#[cfg(feature = "julia")]
pub(super) fn build_raw_connected_pair_collection_candidate_from_pairs(
    candidate_id: impl Into<String>,
    pair_collection: &[LinkGraphAgenticCandidatePair],
    fallback_edge_kind: impl Into<String>,
    dependency_score: f64,
    keyword_score: f64,
    tag_score: f64,
) -> Result<GraphStructuralRawConnectedPairCollectionCandidateInputs, Box<dyn std::error::Error>> {
    Ok(
        xiuxian_wendao_builtin::build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
            GraphStructuralRawConnectedPairCollectionRawTupleInput {
                candidate_id: candidate_id.into(),
                pair_candidates: pair_collection
                    .iter()
                    .map(|pair| (pair.left_id.clone(), pair.right_id.clone(), pair.priority))
                    .collect::<Vec<_>>(),
                fallback_edge_kind: JuliaContractKind::from(fallback_edge_kind.into()),
                dependency_score,
                keyword_score,
                tag_score,
            },
        )?,
    )
}

#[cfg(feature = "julia")]
pub(super) fn build_raw_connected_pair_collection_candidates_from_plan(
    plan: &LinkGraphAgenticExpansionPlan,
    max_collections: usize,
    options: &GenericTopologyCandidateBuildOptions<'_>,
) -> Result<Vec<GraphStructuralRawConnectedPairCollectionCandidateInputs>, Box<dyn std::error::Error>>
{
    connected_pair_collections(plan, max_collections)
        .iter()
        .enumerate()
        .map(|(collection_index, pair_collection)| {
            build_raw_connected_pair_collection_candidate_from_pairs(
                format!("{}-{collection_index}", options.candidate_id_prefix),
                pair_collection,
                options.fallback_edge_kind,
                options.scores.dependency,
                options.scores.keyword,
                options.scores.tag,
            )
        })
        .collect()
}

#[cfg(feature = "julia")]
pub(super) fn worker_partition_pair_groups(
    plan: &LinkGraphAgenticExpansionPlan,
    max_workers: usize,
    min_pairs: usize,
) -> Vec<&LinkGraphAgenticWorkerPlan> {
    plan.workers
        .iter()
        .filter(|worker| worker.pairs.len() >= min_pairs)
        .take(max_workers)
        .collect()
}

#[cfg(feature = "julia")]
pub(super) fn build_worker_partition_generic_topology_candidate_fixtures_from_plan(
    plan: &LinkGraphAgenticExpansionPlan,
    max_workers: usize,
    min_pairs: usize,
    options: &GenericTopologyCandidateBuildOptions<'_>,
) -> Result<Vec<GenericTopologyCandidateFixture>, Box<dyn std::error::Error>> {
    worker_partition_pair_groups(plan, max_workers, min_pairs)
        .into_iter()
        .map(|worker| {
            let candidate_id = format!("{}-{}", options.candidate_id_prefix, worker.worker_id);
            let (expected_nodes, expected_edges) = pair_collection_shape(&worker.pairs);
            let candidate = build_raw_connected_pair_collection_candidate_from_pairs(
                candidate_id.clone(),
                &worker.pairs,
                options.fallback_edge_kind,
                options.scores.dependency,
                options.scores.keyword,
                options.scores.tag,
            )?;
            Ok(GenericTopologyCandidateFixture {
                candidate_id,
                expected_nodes,
                expected_edges,
                candidate,
            })
        })
        .collect()
}

#[cfg(feature = "julia")]
pub(super) fn default_agentic_execution_relation_edge_kind() -> String {
    LinkGraphIndex::resolve_agentic_execution_config().relation
}

#[cfg(feature = "julia")]
pub(super) fn seed_centered_pair_collections(
    plan: &LinkGraphAgenticExpansionPlan,
    max_collections: usize,
    min_pairs: usize,
) -> Vec<(String, Vec<LinkGraphAgenticCandidatePair>)> {
    let mut collections = Vec::new();
    let mut grouped_pairs = BTreeMap::<String, Vec<LinkGraphAgenticCandidatePair>>::new();

    for worker in &plan.workers {
        for pair in &worker.pairs {
            grouped_pairs
                .entry(pair.left_id.clone())
                .or_default()
                .push(pair.clone());
            grouped_pairs
                .entry(pair.right_id.clone())
                .or_default()
                .push(pair.clone());
        }
    }

    for (seed_id, mut pairs) in grouped_pairs {
        if pairs.len() < min_pairs {
            continue;
        }
        pairs.sort_by(|left, right| {
            normalize_pair_endpoints(&left.left_id, &left.right_id)
                .cmp(&normalize_pair_endpoints(&right.left_id, &right.right_id))
        });
        pairs.dedup_by(|left, right| {
            normalize_pair_endpoints(&left.left_id, &left.right_id)
                == normalize_pair_endpoints(&right.left_id, &right.right_id)
        });
        if pairs.len() < min_pairs {
            continue;
        }
        collections.push((seed_id, pairs));
        if collections.len() >= max_collections {
            break;
        }
    }

    collections
}

#[cfg(feature = "julia")]
pub(super) fn build_raw_seed_centered_pair_collection_candidates_from_plan(
    plan: &LinkGraphAgenticExpansionPlan,
    max_collections: usize,
    min_pairs: usize,
    options: &GenericTopologyCandidateBuildOptions<'_>,
) -> Result<Vec<GraphStructuralRawConnectedPairCollectionCandidateInputs>, Box<dyn std::error::Error>>
{
    seed_centered_pair_collections(plan, max_collections, min_pairs)
        .iter()
        .enumerate()
        .map(|(collection_index, (_, pair_collection))| {
            build_raw_connected_pair_collection_candidate_from_pairs(
                format!("{}-{collection_index}", options.candidate_id_prefix),
                pair_collection,
                options.fallback_edge_kind,
                options.scores.dependency,
                options.scores.keyword,
                options.scores.tag,
            )
        })
        .collect()
}

#[cfg(feature = "julia")]
pub(super) async fn fetch_generic_topology_rows_via_manifest_discovery(
    base_url: impl Into<String>,
    query_id: &str,
    candidates: &[GraphStructuralRawConnectedPairCollectionCandidateInputs],
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

    fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections(
        &repository,
        &build_graph_structural_keyword_tag_query_context(
            query_id,
            0,
            2,
            vec!["alpha".to_string()],
            Vec::new(),
            vec![default_agentic_execution_relation_edge_kind()],
        )?,
        candidates,
    )
    .await
    .map_err(Box::<dyn std::error::Error>::from)
}

#[cfg(feature = "julia")]
pub(super) fn assert_solver_demo_generic_topology_row_basics(
    row: &GraphStructuralRerankScoreRow,
    label: &str,
) {
    assert!(
        row.feasible,
        "unexpected {label} generic solver_demo row: candidate_id={} structural_score={} final_score={} explanation={} pin_assignment={:?}",
        row.candidate_id,
        row.structural_score,
        row.final_score,
        row.explanation,
        row.pin_assignment
    );
    assert!(
        row.structural_score > 0.0,
        "unexpected {label} generic solver_demo structural_score: {}",
        row.structural_score
    );
    assert!(
        row.final_score > row.structural_score,
        "unexpected {label} generic solver_demo final_score={} structural_score={}",
        row.final_score,
        row.structural_score
    );
    assert!(
        row.explanation.contains("explicit edges"),
        "unexpected {label} explanation: {}",
        row.explanation
    );
}

#[cfg(feature = "julia")]
pub(super) fn assert_solver_demo_generic_topology_row_shape(
    row: &GraphStructuralRerankScoreRow,
    label: &str,
    expected_nodes: usize,
    expected_edges: usize,
) {
    assert!(
        row.explanation.contains(&format!(
            "with {expected_nodes} nodes, {expected_edges} explicit edges"
        )),
        "unexpected {label} explanation: {}",
        row.explanation
    );
}

#[cfg(feature = "julia")]
pub(super) fn assert_solver_demo_generic_topology_row_infeasible(
    row: &GraphStructuralRerankScoreRow,
    label: &str,
) {
    assert!(
        !row.feasible,
        "unexpected {label} generic solver_demo feasibility: candidate_id={} structural_score={} final_score={} explanation={} pin_assignment={:?}",
        row.candidate_id,
        row.structural_score,
        row.final_score,
        row.explanation,
        row.pin_assignment
    );
    assert_zero_score(row.structural_score, label, "structural_score");
    assert_zero_score(row.final_score, label, "final_score");
    assert!(
        row.pin_assignment.is_empty(),
        "unexpected {label} infeasible pin_assignment: {:?}",
        row.pin_assignment
    );
    assert!(
        row.explanation.contains("found no feasible gadget"),
        "unexpected {label} infeasible explanation: {}",
        row.explanation
    );
}

#[cfg(feature = "julia")]
fn assert_zero_score(score: f64, label: &str, score_name: &str) {
    assert!(
        score.abs() < f64::EPSILON,
        "unexpected {label} infeasible {score_name}: {score}"
    );
}

#[cfg(feature = "julia")]
pub(super) fn required_column<'a, T: Array + 'static>(
    batch: &'a RecordBatch,
    column_name: &str,
    expected_type: &str,
) -> &'a T {
    let Some(column) = batch.column_by_name(column_name) else {
        panic!("`{column_name}` column should exist");
    };
    let Some(column) = column.as_any().downcast_ref::<T>() else {
        panic!("`{column_name}` column should be {expected_type}");
    };
    column
}

#[cfg(feature = "julia")]
pub(super) fn required_utf8_list_row_values(
    batch: &RecordBatch,
    column_name: &str,
    row_index: usize,
) -> Vec<String> {
    let column = required_column::<ListArray>(batch, column_name, "list");
    let values = column.value(row_index);
    let Some(values) = values.as_any().downcast_ref::<StringArray>() else {
        panic!("`{column_name}` values should be utf8");
    };
    (0..values.len())
        .map(|index| values.value(index).to_string())
        .collect()
}
