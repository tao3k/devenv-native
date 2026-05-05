//! Synthetic performance fixtures for local `wendao get` projections.

use std::fs;
use std::path::Path;

use anyhow::Result;

use super::run::{
    build_local_page_index_trees_with_ignore, canonical_scope_target, default_ignore_dir_names,
};
use super::types::ProjectedPageIndexNode;

/// Number of Markdown documents in the default local get benchmark fixture.
pub const GET_BENCH_DOC_COUNT: usize = 512;

/// Number of second-level sections per benchmark document.
pub const GET_BENCH_SECTIONS_PER_DOC: usize = 6;

/// Summary returned by the local page-index benchmark path.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LocalPageIndexBenchmarkSummary {
    /// Number of materialized page-index trees.
    pub tree_count: usize,
    /// Number of materialized page-index nodes.
    pub node_count: usize,
}

/// Write a deterministic local Markdown corpus for `wendao get` benchmarks.
///
/// # Panics
///
/// Panics when fixture directories or files cannot be written.
pub fn write_local_get_benchmark_fixture(root: &Path) {
    let docs = root.join("docs");
    fs::create_dir_all(docs.as_path())
        .unwrap_or_else(|error| panic!("create benchmark docs directory: {error}"));

    for index in 0..GET_BENCH_DOC_COUNT {
        let mut body = format!("# Document {index}\n\n");
        for section in 0..GET_BENCH_SECTIONS_PER_DOC {
            body.push_str(
                format!(
                    "## Section {section}\n\nSee [next](doc_{next:04}.md#section-{section}) and ![diagram](assets/diagram_{section}.png).\n\n",
                    next = (index + 1) % GET_BENCH_DOC_COUNT,
                )
                .as_str(),
            );
        }
        fs::write(docs.join(format!("doc_{index:04}.md")), body)
            .unwrap_or_else(|error| panic!("write benchmark markdown document: {error}"));
    }
}

/// Materialize local page-index trees for the benchmark fixture.
///
/// # Errors
///
/// Returns an error when the fixture root cannot be resolved or parsed.
pub fn benchmark_local_page_index(root: &Path) -> Result<LocalPageIndexBenchmarkSummary> {
    let scope = canonical_scope_target(root, Path::new("."))?;
    let ignored_dirs = default_ignore_dir_names();
    let result = build_local_page_index_trees_with_ignore(&scope, root, ignored_dirs.as_slice())?;
    let node_count = result
        .trees
        .iter()
        .map(|tree| count_page_index_nodes(tree.roots.as_slice()))
        .sum::<usize>();

    Ok(LocalPageIndexBenchmarkSummary {
        tree_count: result.trees.len(),
        node_count,
    })
}

fn count_page_index_nodes(nodes: &[ProjectedPageIndexNode]) -> usize {
    nodes
        .iter()
        .map(|node| 1 + count_page_index_nodes(node.children.as_slice()))
        .sum()
}
