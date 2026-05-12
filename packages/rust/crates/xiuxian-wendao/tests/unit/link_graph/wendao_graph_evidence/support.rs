use std::fs;
use std::path::{Path, PathBuf};

use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;

use crate::link_graph::LinkGraphIndex;

use crate::link_graph::WendaoGraphSemanticOverlayEdge;

pub(super) fn write_note(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|error| panic!("create note parent: {error}"));
    }
    fs::write(path, content).unwrap_or_else(|error| panic!("write note: {error}"));
}

pub(super) fn fixture_index() -> LinkGraphIndex {
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

pub(super) fn string_column(batch: &RecordBatch, index: usize) -> Vec<String> {
    batch
        .column(index)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap_or_else(|| panic!("column {index} should be StringArray"))
        .iter()
        .map(|value| value.unwrap_or("").to_string())
        .collect()
}

pub(super) fn float_column(batch: &RecordBatch, index: usize) -> Vec<f64> {
    batch
        .column(index)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap_or_else(|| panic!("column {index} should be Float64Array"))
        .iter()
        .map(|value| value.unwrap_or(f64::NAN))
        .collect()
}

pub(super) fn int64_column(batch: &RecordBatch, index: usize) -> Vec<i64> {
    batch
        .column(index)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap_or_else(|| panic!("column {index} should be Int64Array"))
        .iter()
        .map(|value| value.unwrap_or(i64::MIN))
        .collect()
}

pub(super) fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wendaograph_page_index_reasoning_host")
}

pub(super) fn read_tsv_rows(relative: &str) -> Vec<Vec<String>> {
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

pub(super) fn page_index_node_rows(batch: &RecordBatch) -> Vec<Vec<String>> {
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

pub(super) fn page_index_edge_rows(batch: &RecordBatch) -> Vec<Vec<String>> {
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

pub(super) fn page_index_seed_rows(batch: &RecordBatch) -> Vec<Vec<String>> {
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

pub(super) fn inject_page_index_edge(index: &mut LinkGraphIndex) -> (String, String) {
    let page_root_id = "docs/alpha#page-root".to_string();
    let page_child_id = "docs/alpha#page-child".to_string();
    index.node_parent_map.insert(page_root_id.clone(), None);
    index
        .node_parent_map
        .insert(page_child_id.clone(), Some(page_root_id.clone()));
    (page_root_id, page_child_id)
}

pub(super) fn semantic_overlay_edge(
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
