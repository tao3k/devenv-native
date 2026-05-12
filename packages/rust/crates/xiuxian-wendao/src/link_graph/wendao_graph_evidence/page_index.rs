//! Project `LinkGraph` page-index trees into `WendaoGraph.jl` reasoning tables.

use std::collections::BTreeSet;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_julia::{
    WendaoGraphEvidenceTableKind, validate_wendao_graph_page_index_reasoning_request_schema,
    wendao_graph_page_index_reasoning_table_schema,
};

use super::types::{
    LinkGraphWendaoGraphEvidenceError, WendaoGraphPageIndexReasoningRequestBundle,
    WendaoGraphPageIndexReasoningRequestOptions, WendaoGraphPageIndexReasoningSeed,
};
use crate::link_graph::{LinkGraphIndex, PageIndexNode};

pub(super) const PAGE_INDEX_NODES_TABLE: &str = "page_index_nodes";
pub(super) const PAGE_INDEX_EDGE_KIND_HIERARCHY: &str = "hierarchy";

const PAGE_INDEX_EDGES_TABLE: &str = "page_index_edges";
const PAGE_INDEX_SEEDS_TABLE: &str = "page_index_seeds";

/// Build a `WendaoGraph` `PageIndex` reasoning request bundle for a `LinkGraphIndex`.
///
/// # Errors
///
/// Returns an error when seed rows are invalid, seed nodes are not present in
/// the projected `PageIndex` node table, or Arrow batch construction/request
/// schema validation fails.
pub fn build_wendao_graph_page_index_reasoning_request_bundle(
    index: &LinkGraphIndex,
) -> Result<WendaoGraphPageIndexReasoningRequestBundle, LinkGraphWendaoGraphEvidenceError> {
    build_wendao_graph_page_index_reasoning_request_bundle_with_options(
        index,
        &WendaoGraphPageIndexReasoningRequestOptions::default(),
    )
}

/// Build a `WendaoGraph` `PageIndex` reasoning request bundle for a `LinkGraphIndex`.
///
/// # Errors
///
/// Returns an error when seed rows are invalid, seed nodes are not present in
/// the projected `PageIndex` node table, or Arrow batch construction/request
/// schema validation fails.
pub fn build_wendao_graph_page_index_reasoning_request_bundle_with_options(
    index: &LinkGraphIndex,
    options: &WendaoGraphPageIndexReasoningRequestOptions,
) -> Result<WendaoGraphPageIndexReasoningRequestBundle, LinkGraphWendaoGraphEvidenceError> {
    let projection = ProjectedPageIndexReasoning::from_index(index)?;
    build_page_index_reasoning_request_bundle_from_rows(
        &projection.nodes,
        &projection.edges,
        &projection.node_ids,
        &options.seeds,
    )
}

#[derive(Debug, Clone)]
struct ProjectedPageIndexReasoning {
    nodes: Vec<PageIndexReasoningNodeRow>,
    edges: Vec<PageIndexReasoningEdgeRow>,
    node_ids: BTreeSet<String>,
}

impl ProjectedPageIndexReasoning {
    fn from_index(index: &LinkGraphIndex) -> Result<Self, LinkGraphWendaoGraphEvidenceError> {
        let mut projection = Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_ids: BTreeSet::new(),
        };

        let mut pages = index
            .trees_by_doc
            .iter()
            .collect::<Vec<(&String, &Vec<PageIndexNode>)>>();
        pages.sort_by_key(|(page_id, _)| *page_id);

        for (page_id, roots) in pages {
            for root in roots {
                projection.collect_node(page_id, None, 0, root)?;
            }
        }

        Ok(projection)
    }

    fn collect_node(
        &mut self,
        page_id: &str,
        parent_id: Option<&str>,
        depth: usize,
        node: &PageIndexNode,
    ) -> Result<(), LinkGraphWendaoGraphEvidenceError> {
        let parent_id = node.parent_id.as_deref().or(parent_id).unwrap_or_default();
        let rank = self.nodes.len();
        let (line_start, line_end) = node.metadata.line_range;
        self.node_ids.insert(node.node_id.clone());
        if !parent_id.is_empty() {
            self.edges.push(PageIndexReasoningEdgeRow {
                source_id: parent_id.to_string(),
                target_id: node.node_id.clone(),
                edge_kind: PAGE_INDEX_EDGE_KIND_HIERARCHY.to_string(),
                weight: 1.0,
            });
        }
        self.nodes.push(PageIndexReasoningNodeRow {
            node_id: node.node_id.clone(),
            page_id: page_id.to_string(),
            parent_id: parent_id.to_string(),
            depth: usize_to_i64(PAGE_INDEX_NODES_TABLE, "depth", depth)?,
            rank: usize_to_i64(PAGE_INDEX_NODES_TABLE, "rank", rank)?,
            title: node.title.clone(),
            summary: node.summary.clone().unwrap_or_default(),
            line_start: usize_to_i64(PAGE_INDEX_NODES_TABLE, "line_start", line_start)?,
            line_end: usize_to_i64(PAGE_INDEX_NODES_TABLE, "line_end", line_end)?,
            token_count: usize_to_i64(
                PAGE_INDEX_NODES_TABLE,
                "token_count",
                node.metadata.token_count,
            )?,
        });

        for child in &node.children {
            self.collect_node(page_id, Some(&node.node_id), depth + 1, child)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(super) struct PageIndexReasoningNodeRow {
    pub(super) node_id: String,
    pub(super) page_id: String,
    pub(super) parent_id: String,
    pub(super) depth: i64,
    pub(super) rank: i64,
    pub(super) title: String,
    pub(super) summary: String,
    pub(super) line_start: i64,
    pub(super) line_end: i64,
    pub(super) token_count: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PageIndexReasoningEdgeRow {
    pub(super) source_id: String,
    pub(super) target_id: String,
    pub(super) edge_kind: String,
    pub(super) weight: f64,
}

pub(super) fn build_page_index_reasoning_request_bundle_from_rows(
    nodes: &[PageIndexReasoningNodeRow],
    edges: &[PageIndexReasoningEdgeRow],
    node_ids: &BTreeSet<String>,
    seeds: &[WendaoGraphPageIndexReasoningSeed],
) -> Result<WendaoGraphPageIndexReasoningRequestBundle, LinkGraphWendaoGraphEvidenceError> {
    let nodes = build_page_index_nodes_batch(nodes)?;
    let edges = build_page_index_edges_batch(edges)?;
    let seeds = build_page_index_seeds_batch(node_ids, seeds)?;

    Ok(WendaoGraphPageIndexReasoningRequestBundle {
        nodes,
        edges,
        seeds,
    })
}

fn build_page_index_nodes_batch(
    rows: &[PageIndexReasoningNodeRow],
) -> Result<RecordBatch, LinkGraphWendaoGraphEvidenceError> {
    build_request_batch(
        PAGE_INDEX_NODES_TABLE,
        vec![
            string_array(rows.iter().map(|row| row.node_id.clone())),
            string_array(rows.iter().map(|row| row.page_id.clone())),
            string_array(rows.iter().map(|row| row.parent_id.clone())),
            Arc::new(Int64Array::from(
                rows.iter().map(|row| row.depth).collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Int64Array::from(
                rows.iter().map(|row| row.rank).collect::<Vec<_>>(),
            )) as ArrayRef,
            string_array(rows.iter().map(|row| row.title.clone())),
            string_array(rows.iter().map(|row| row.summary.clone())),
            Arc::new(Int64Array::from(
                rows.iter().map(|row| row.line_start).collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Int64Array::from(
                rows.iter().map(|row| row.line_end).collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Int64Array::from(
                rows.iter().map(|row| row.token_count).collect::<Vec<_>>(),
            )) as ArrayRef,
        ],
    )
}

fn build_page_index_edges_batch(
    rows: &[PageIndexReasoningEdgeRow],
) -> Result<RecordBatch, LinkGraphWendaoGraphEvidenceError> {
    build_request_batch(
        PAGE_INDEX_EDGES_TABLE,
        vec![
            string_array(rows.iter().map(|row| row.source_id.clone())),
            string_array(rows.iter().map(|row| row.target_id.clone())),
            string_array(rows.iter().map(|row| row.edge_kind.clone())),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.weight).collect::<Vec<_>>(),
            )) as ArrayRef,
        ],
    )
}

fn build_page_index_seeds_batch(
    node_ids: &BTreeSet<String>,
    seeds: &[WendaoGraphPageIndexReasoningSeed],
) -> Result<RecordBatch, LinkGraphWendaoGraphEvidenceError> {
    let mut seed_node_ids = Vec::with_capacity(seeds.len());
    let mut weights = Vec::with_capacity(seeds.len());
    let mut seed_kinds = Vec::with_capacity(seeds.len());
    for seed in seeds {
        validate_page_index_seed(node_ids, seed)?;
        seed_node_ids.push(seed.node_id.clone());
        weights.push(seed.weight);
        seed_kinds.push(seed.seed_kind.clone());
    }

    build_request_batch(
        PAGE_INDEX_SEEDS_TABLE,
        vec![
            Arc::new(StringArray::from(seed_node_ids)) as ArrayRef,
            Arc::new(Float64Array::from(weights)) as ArrayRef,
            Arc::new(StringArray::from(seed_kinds)) as ArrayRef,
        ],
    )
}

fn validate_page_index_seed(
    node_ids: &BTreeSet<String>,
    seed: &WendaoGraphPageIndexReasoningSeed,
) -> Result<(), LinkGraphWendaoGraphEvidenceError> {
    if seed.node_id.trim().is_empty() {
        return Err(LinkGraphWendaoGraphEvidenceError::BlankSeedNode);
    }
    if !seed.weight.is_finite() || seed.weight < 0.0 {
        return Err(LinkGraphWendaoGraphEvidenceError::InvalidSeedWeight {
            node_id: seed.node_id.clone(),
        });
    }
    if seed.seed_kind.trim().is_empty() {
        return Err(LinkGraphWendaoGraphEvidenceError::BlankSeedKind {
            node_id: seed.node_id.clone(),
        });
    }
    if !node_ids.contains(&seed.node_id) {
        return Err(LinkGraphWendaoGraphEvidenceError::UnknownSeedNode {
            node_id: seed.node_id.clone(),
        });
    }
    Ok(())
}

fn build_request_batch(
    table_name: &'static str,
    columns: Vec<ArrayRef>,
) -> Result<RecordBatch, LinkGraphWendaoGraphEvidenceError> {
    let schema = wendao_graph_page_index_reasoning_table_schema(
        WendaoGraphEvidenceTableKind::Request,
        table_name,
    )
    .map_err(|message| LinkGraphWendaoGraphEvidenceError::Schema {
        table_name,
        message,
    })?;
    let batch = RecordBatch::try_new(schema, columns).map_err(|error| {
        LinkGraphWendaoGraphEvidenceError::Batch {
            table_name,
            message: error.to_string(),
        }
    })?;
    validate_wendao_graph_page_index_reasoning_request_schema(table_name, batch.schema().as_ref())
        .map_err(|message| LinkGraphWendaoGraphEvidenceError::Schema {
            table_name,
            message,
        })?;
    Ok(batch)
}

fn string_array(values: impl Iterator<Item = String>) -> ArrayRef {
    Arc::new(StringArray::from(values.collect::<Vec<_>>())) as ArrayRef
}

fn usize_to_i64(
    table_name: &'static str,
    column: &'static str,
    value: usize,
) -> Result<i64, LinkGraphWendaoGraphEvidenceError> {
    i64::try_from(value).map_err(|_| LinkGraphWendaoGraphEvidenceError::IntegerOverflow {
        table_name,
        column,
        value,
    })
}

pub(super) fn semantic_usize_to_i64(
    column: &'static str,
    value: usize,
) -> Result<i64, LinkGraphWendaoGraphEvidenceError> {
    usize_to_i64(PAGE_INDEX_NODES_TABLE, column, value)
}
