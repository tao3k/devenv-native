use std::fs;
use std::path::Path;

use arrow::array::{Float64Array, StringArray};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_julia::validate_wendao_graph_evidence_request_schema;

use super::*;
use crate::link_graph::LinkGraphIndex;

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
