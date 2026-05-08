use std::fs;
use std::path::{Path, PathBuf};

use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_julia::{
    validate_wendao_graph_evidence_request_schema,
    validate_wendao_graph_page_index_reasoning_request_schema,
};

use super::{
    LinkGraphWendaoGraphEvidenceError, WendaoGraphEvidenceRequestOptions,
    WendaoGraphPageIndexReasoningRequestOptions, WendaoGraphSemanticOverlayEdge,
    build_wendao_graph_evidence_request_bundle,
    build_wendao_graph_evidence_request_bundle_with_options,
    build_wendao_graph_page_index_reasoning_request_bundle,
    build_wendao_graph_page_index_reasoning_request_bundle_with_options,
};
use crate::link_graph::LinkGraphIndex;

mod semantic_reasoning;

fn write_note(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|error| panic!("create note parent: {error}"));
    }
    fs::write(path, content).unwrap_or_else(|error| panic!("write note: {error}"));
}

fn fixture_index() -> LinkGraphIndex {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let root = temp.path();
    write_note(
        root,
        "docs/alpha.md",
        "# Alpha\n\nLinks to [[Beta]].\n\n## Alpha Detail\n\nDetail body.\n",
    );
    write_note(root, "docs/beta.md", "# Beta\n\nBack to [[Alpha]].\n");
    LinkGraphIndex::build_with_filters(root, &["docs".to_string()], &[])
        .unwrap_or_else(|error| panic!("build fixture index: {error}"))
}

fn string_column(batch: &RecordBatch, index: usize) -> Vec<String> {
    batch
        .column(index)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap_or_else(|| panic!("column {index} should be StringArray"))
        .iter()
        .map(|value| value.unwrap_or("").to_string())
        .collect()
}

fn float_column(batch: &RecordBatch, index: usize) -> Vec<f64> {
    batch
        .column(index)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap_or_else(|| panic!("column {index} should be Float64Array"))
        .iter()
        .map(|value| value.unwrap_or(f64::NAN))
        .collect()
}

fn int64_column(batch: &RecordBatch, index: usize) -> Vec<i64> {
    batch
        .column(index)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap_or_else(|| panic!("column {index} should be Int64Array"))
        .iter()
        .map(|value| value.unwrap_or(i64::MIN))
        .collect()
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wendaograph_page_index_reasoning_host")
}

fn read_tsv_rows(relative: &str) -> Vec<Vec<String>> {
    let path = fixture_dir().join(relative);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read fixture {}: {error}", path.display()));
    content
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split('\t').map(ToString::to_string).collect())
        .collect()
}

fn page_index_node_rows(batch: &RecordBatch) -> Vec<Vec<String>> {
    let node_ids = string_column(batch, 0);
    let page_ids = string_column(batch, 1);
    let parent_ids = string_column(batch, 2);
    let depths = int64_column(batch, 3);
    let ranks = int64_column(batch, 4);
    let titles = string_column(batch, 5);
    let summaries = string_column(batch, 6);
    let line_starts = int64_column(batch, 7);
    let line_ends = int64_column(batch, 8);
    let token_counts = int64_column(batch, 9);

    node_ids
        .into_iter()
        .zip(page_ids)
        .zip(parent_ids)
        .zip(depths)
        .zip(ranks)
        .zip(titles)
        .zip(summaries)
        .zip(line_starts)
        .zip(line_ends)
        .zip(token_counts)
        .map(
            |(
                (
                    (
                        ((((((node_id, page_id), parent_id), depth), rank), title), summary),
                        line_start,
                    ),
                    line_end,
                ),
                token_count,
            )| {
                vec![
                    node_id,
                    page_id,
                    parent_id,
                    depth.to_string(),
                    rank.to_string(),
                    title,
                    summary,
                    line_start.to_string(),
                    line_end.to_string(),
                    token_count.to_string(),
                ]
            },
        )
        .collect()
}

fn page_index_edge_rows(batch: &RecordBatch) -> Vec<Vec<String>> {
    let source_ids = string_column(batch, 0);
    let target_ids = string_column(batch, 1);
    let edge_kinds = string_column(batch, 2);
    let weights = float_column(batch, 3);

    source_ids
        .into_iter()
        .zip(target_ids)
        .zip(edge_kinds)
        .zip(weights)
        .map(|(((source_id, target_id), edge_kind), weight)| {
            vec![source_id, target_id, edge_kind, format!("{weight:.1}")]
        })
        .collect()
}

fn page_index_seed_rows(batch: &RecordBatch) -> Vec<Vec<String>> {
    let node_ids = string_column(batch, 0);
    let weights = float_column(batch, 1);
    let seed_kinds = string_column(batch, 2);

    node_ids
        .into_iter()
        .zip(weights)
        .zip(seed_kinds)
        .map(|((node_id, weight), seed_kind)| vec![node_id, format!("{weight:.1}"), seed_kind])
        .collect()
}

fn inject_page_index_edge(index: &mut LinkGraphIndex) -> (String, String) {
    let page_root_id = "docs/alpha#page-root".to_string();
    let page_child_id = "docs/alpha#page-child".to_string();
    index.node_parent_map.insert(page_root_id.clone(), None);
    index
        .node_parent_map
        .insert(page_child_id.clone(), Some(page_root_id.clone()));
    (page_root_id, page_child_id)
}

#[test]
fn page_index_reasoning_host_fixture_matches_builder_output() {
    let index = fixture_index();
    let options = WendaoGraphPageIndexReasoningRequestOptions::default().with_seed(
        "docs/alpha#alpha",
        1.0,
        "query_match",
    );
    let bundle =
        build_wendao_graph_page_index_reasoning_request_bundle_with_options(&index, &options)
            .unwrap_or_else(|error| panic!("build seeded PageIndex bundle: {error}"));

    assert_eq!(
        page_index_node_rows(&bundle.nodes),
        read_tsv_rows("page_index_nodes.tsv")
    );
    assert_eq!(
        page_index_edge_rows(&bundle.edges),
        read_tsv_rows("page_index_edges.tsv")
    );
    assert_eq!(
        page_index_seed_rows(&bundle.seeds),
        read_tsv_rows("page_index_seeds.tsv")
    );
}

#[test]
fn request_bundle_projects_document_and_page_index_edges() {
    let mut index = fixture_index();
    let (page_root_id, page_child_id) = inject_page_index_edge(&mut index);
    let bundle = build_wendao_graph_evidence_request_bundle(&index)
        .unwrap_or_else(|error| panic!("build WendaoGraph request bundle: {error}"));

    validate_wendao_graph_evidence_request_schema("nodes", bundle.nodes.schema().as_ref())
        .unwrap_or_else(|error| panic!("validate nodes schema: {error}"));
    validate_wendao_graph_evidence_request_schema("edges", bundle.edges.schema().as_ref())
        .unwrap_or_else(|error| panic!("validate edges schema: {error}"));

    let node_ids = string_column(&bundle.nodes, 0);
    assert!(node_ids.contains(&"docs/alpha".to_string()));
    assert!(node_ids.contains(&"docs/beta".to_string()));
    assert!(node_ids.contains(&page_root_id));
    assert!(node_ids.contains(&page_child_id));

    let sources = string_column(&bundle.edges, 0);
    let targets = string_column(&bundle.edges, 1);
    let edges = sources
        .into_iter()
        .zip(targets)
        .collect::<Vec<(String, String)>>();
    assert!(edges.contains(&("docs/alpha".to_string(), "docs/beta".to_string())));
    assert!(edges.contains(&(page_root_id, page_child_id)));
    assert!(bundle.semantic_neighbors.is_none());
    assert!(bundle.semantic_overlay.is_none());
}

#[test]
fn page_index_reasoning_bundle_projects_wendaograph_sidecar_tables() {
    let index = fixture_index();
    let bundle = build_wendao_graph_page_index_reasoning_request_bundle(&index)
        .unwrap_or_else(|error| panic!("build WendaoGraph PageIndex request bundle: {error}"));

    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_nodes",
        bundle.nodes.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_nodes schema: {error}"));
    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_edges",
        bundle.edges.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_edges schema: {error}"));
    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_seeds",
        bundle.seeds.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_seeds schema: {error}"));

    let table_names = bundle
        .record_batches()
        .into_iter()
        .map(|(table_name, _)| table_name)
        .collect::<Vec<_>>();
    assert_eq!(
        table_names,
        vec!["page_index_nodes", "page_index_edges", "page_index_seeds"]
    );

    let node_ids = string_column(&bundle.nodes, 0);
    let page_ids = string_column(&bundle.nodes, 1);
    let parent_ids = string_column(&bundle.nodes, 2);
    let depths = int64_column(&bundle.nodes, 3);
    let line_starts = int64_column(&bundle.nodes, 7);
    let token_counts = int64_column(&bundle.nodes, 9);
    assert!(!node_ids.is_empty());
    assert!(page_ids.iter().any(|page_id| page_id == "docs/alpha"));
    assert!(parent_ids.iter().any(String::is_empty));
    assert!(depths.iter().all(|depth| *depth >= 0));
    assert!(line_starts.iter().all(|line_start| *line_start > 0));
    assert!(token_counts.iter().all(|token_count| *token_count > 0));

    let edge_kinds = string_column(&bundle.edges, 2);
    assert!(edge_kinds.iter().all(|edge_kind| edge_kind == "hierarchy"));
    assert_eq!(bundle.seeds.num_rows(), 0);

    let link_bundle = build_wendao_graph_evidence_request_bundle(&index)
        .unwrap_or_else(|error| panic!("build link evidence bundle: {error}"));
    assert!(link_bundle.table("page_index_nodes").is_none());
}

#[test]
fn page_index_reasoning_bundle_projects_valid_seed_rows() {
    let index = fixture_index();
    let seed_source = build_wendao_graph_page_index_reasoning_request_bundle(&index)
        .unwrap_or_else(|error| panic!("build PageIndex source bundle: {error}"));
    let seed_node = string_column(&seed_source.nodes, 0)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("fixture should produce at least one PageIndex node"));
    let options = WendaoGraphPageIndexReasoningRequestOptions::default().with_seed(
        seed_node.clone(),
        0.75,
        "query_match",
    );
    let seeded =
        build_wendao_graph_page_index_reasoning_request_bundle_with_options(&index, &options)
            .unwrap_or_else(|error| panic!("build seeded PageIndex bundle: {error}"));

    assert_eq!(string_column(&seeded.seeds, 0), vec![seed_node]);
    assert_eq!(float_column(&seeded.seeds, 1), vec![0.75]);
    assert_eq!(
        string_column(&seeded.seeds, 2),
        vec!["query_match".to_string()]
    );
}

#[test]
fn request_bundle_can_disable_page_index_projection() {
    let mut index = fixture_index();
    let (page_root_id, page_child_id) = inject_page_index_edge(&mut index);
    let options = WendaoGraphEvidenceRequestOptions::default().without_page_index();
    let bundle = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
        .unwrap_or_else(|error| panic!("build WendaoGraph request bundle: {error}"));

    let node_ids = string_column(&bundle.nodes, 0);
    assert!(node_ids.contains(&"docs/alpha".to_string()));
    assert!(!node_ids.contains(&page_child_id));

    let edges = string_column(&bundle.edges, 0)
        .into_iter()
        .zip(string_column(&bundle.edges, 1))
        .collect::<Vec<(String, String)>>();
    assert!(!edges.contains(&(page_root_id, page_child_id)));
}

#[test]
fn request_bundle_projects_valid_seeds_in_canonical_order() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default().with_seed("docs/alpha", 2.5);
    let bundle = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
        .unwrap_or_else(|error| panic!("build WendaoGraph request bundle: {error}"));

    let Some(seeds) = bundle.table("seeds") else {
        panic!("seeds batch should be present");
    };
    validate_wendao_graph_evidence_request_schema("seeds", seeds.schema().as_ref())
        .unwrap_or_else(|error| panic!("validate seeds schema: {error}"));
    assert_eq!(string_column(seeds, 0), vec!["docs/alpha".to_string()]);
    assert_eq!(float_column(seeds, 1), vec![2.5]);

    let table_names = bundle
        .record_batches()
        .into_iter()
        .map(|(table_name, _)| table_name)
        .collect::<Vec<_>>();
    assert_eq!(table_names, vec!["nodes", "edges", "seeds"]);
}

#[test]
fn request_bundle_projects_semantic_neighbors_in_canonical_order() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default().with_semantic_neighbor(
        "docs/alpha",
        "docs/beta",
        1,
        2,
        1,
        0.25,
    );
    let bundle = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
        .unwrap_or_else(|error| panic!("build WendaoGraph request bundle: {error}"));

    let Some(semantic_neighbors) = bundle.table("semantic_neighbors") else {
        panic!("semantic_neighbors batch should be present");
    };
    validate_wendao_graph_evidence_request_schema(
        "semantic_neighbors",
        semantic_neighbors.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate semantic_neighbors schema: {error}"));
    assert_eq!(
        string_column(semantic_neighbors, 0),
        vec!["docs/alpha".to_string()]
    );
    assert_eq!(
        string_column(semantic_neighbors, 1),
        vec!["docs/beta".to_string()]
    );
    assert_eq!(int64_column(semantic_neighbors, 2), vec![1]);
    assert_eq!(int64_column(semantic_neighbors, 3), vec![2]);
    assert_eq!(int64_column(semantic_neighbors, 4), vec![1]);
    assert_eq!(float_column(semantic_neighbors, 5), vec![0.25]);

    let table_names = bundle
        .record_batches()
        .into_iter()
        .map(|(table_name, _)| table_name)
        .collect::<Vec<_>>();
    assert_eq!(table_names, vec!["nodes", "edges", "semantic_neighbors"]);
}

fn semantic_overlay_edge(
    source_id: &str,
    target_id: &str,
    source_index: i64,
    target_index: i64,
) -> WendaoGraphSemanticOverlayEdge {
    WendaoGraphSemanticOverlayEdge {
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        source_index,
        target_index,
        rank: 1,
        distance: 0.25,
        weight: 0.8,
        edge_kind: "semantic".to_string(),
    }
}

#[test]
fn request_bundle_projects_semantic_overlay_in_canonical_order() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default()
        .with_semantic_overlay_edge(semantic_overlay_edge("docs/alpha", "docs/beta", 1, 2));
    let bundle = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
        .unwrap_or_else(|error| panic!("build WendaoGraph request bundle: {error}"));

    let Some(semantic_overlay) = bundle.table("semantic_overlay") else {
        panic!("semantic_overlay batch should be present");
    };
    validate_wendao_graph_evidence_request_schema(
        "semantic_overlay",
        semantic_overlay.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate semantic_overlay schema: {error}"));
    assert_eq!(
        string_column(semantic_overlay, 0),
        vec!["docs/alpha".to_string()]
    );
    assert_eq!(
        string_column(semantic_overlay, 1),
        vec!["docs/beta".to_string()]
    );
    assert_eq!(int64_column(semantic_overlay, 2), vec![1]);
    assert_eq!(int64_column(semantic_overlay, 3), vec![2]);
    assert_eq!(int64_column(semantic_overlay, 4), vec![1]);
    assert_eq!(float_column(semantic_overlay, 5), vec![0.25]);
    assert_eq!(float_column(semantic_overlay, 6), vec![0.8]);
    assert_eq!(
        string_column(semantic_overlay, 7),
        vec!["semantic".to_string()]
    );

    let table_names = bundle
        .record_batches()
        .into_iter()
        .map(|(table_name, _)| table_name)
        .collect::<Vec<_>>();
    assert_eq!(table_names, vec!["nodes", "edges", "semantic_overlay"]);
}

#[test]
fn request_bundle_rejects_conflicting_semantic_inputs() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default()
        .with_semantic_neighbor("docs/alpha", "docs/beta", 1, 2, 1, 0.25)
        .with_semantic_overlay_edge(semantic_overlay_edge("docs/alpha", "docs/beta", 1, 2));
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("conflicting semantic input variants should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::ConflictingSemanticEvidence
    ));
}

#[test]
fn request_bundle_rejects_seed_outside_projected_nodes() {
    let index = fixture_index();
    let options =
        WendaoGraphEvidenceRequestOptions::default().with_seed("docs/missing#anchor", 1.0);
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("unknown seed node should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::UnknownSeedNode { .. }
    ));
}

#[test]
fn request_bundle_rejects_invalid_seed_weight() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default().with_seed("docs/alpha", -1.0);
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("negative seed weight should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::InvalidSeedWeight { .. }
    ));
}

#[test]
fn request_bundle_rejects_semantic_neighbor_outside_projected_nodes() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default().with_semantic_neighbor(
        "docs/alpha",
        "docs/missing",
        1,
        2,
        1,
        0.25,
    );
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("unknown semantic neighbor node should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::UnknownSemanticNeighborNode { .. }
    ));
}

#[test]
fn request_bundle_rejects_invalid_semantic_neighbor_distance() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default().with_semantic_neighbor(
        "docs/alpha",
        "docs/beta",
        1,
        2,
        1,
        f64::NAN,
    );
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("non-finite semantic neighbor distance should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::InvalidSemanticNeighbor {
            field: "distance",
            ..
        }
    ));
}

#[test]
fn request_bundle_rejects_semantic_overlay_outside_projected_nodes() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default()
        .with_semantic_overlay_edge(semantic_overlay_edge("docs/alpha", "docs/missing", 1, 2));
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("unknown semantic overlay node should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::UnknownSemanticOverlayNode { .. }
    ));
}

#[test]
fn request_bundle_rejects_invalid_semantic_overlay_weight() {
    let index = fixture_index();
    let mut edge = semantic_overlay_edge("docs/alpha", "docs/beta", 1, 2);
    edge.weight = f64::INFINITY;
    let options = WendaoGraphEvidenceRequestOptions::default().with_semantic_overlay_edge(edge);
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("non-finite semantic overlay weight should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::InvalidSemanticOverlay {
            field: "weight",
            ..
        }
    ));
}
