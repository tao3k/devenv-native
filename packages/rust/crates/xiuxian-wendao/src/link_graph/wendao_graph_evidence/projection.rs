use std::collections::BTreeSet;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, StringArray};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_julia::{
    WendaoGraphEvidenceTableKind, validate_wendao_graph_evidence_request_schema,
    wendao_graph_evidence_table_schema,
};

use super::types::{
    LinkGraphWendaoGraphEvidenceError, WendaoGraphEvidenceRequestBundle,
    WendaoGraphEvidenceRequestOptions, WendaoGraphEvidenceSeed,
};
use crate::link_graph::{LinkGraphIndex, PageIndexNode};

const NODES_TABLE: &str = "nodes";
const EDGES_TABLE: &str = "edges";
const SEEDS_TABLE: &str = "seeds";

/// Build the default `WendaoGraph` evidence request bundle for a `LinkGraphIndex`.
///
/// # Errors
///
/// Returns an error when Arrow batch construction or request schema validation fails.
pub fn build_wendao_graph_evidence_request_bundle(
    index: &LinkGraphIndex,
) -> Result<WendaoGraphEvidenceRequestBundle, LinkGraphWendaoGraphEvidenceError> {
    build_wendao_graph_evidence_request_bundle_with_options(
        index,
        &WendaoGraphEvidenceRequestOptions::default(),
    )
}

/// Build a `WendaoGraph` evidence request bundle for a `LinkGraphIndex`.
///
/// # Errors
///
/// Returns an error when seed rows are invalid, reference nodes outside the
/// projected graph, or Arrow batch construction/request schema validation fails.
pub fn build_wendao_graph_evidence_request_bundle_with_options(
    index: &LinkGraphIndex,
    options: &WendaoGraphEvidenceRequestOptions,
) -> Result<WendaoGraphEvidenceRequestBundle, LinkGraphWendaoGraphEvidenceError> {
    let projection = ProjectedLinkGraph::from_index(index, options.include_page_index);
    let nodes = build_nodes_batch(&projection.nodes)?;
    let edges = build_edges_batch(&projection.edges)?;
    let seeds = if options.seeds.is_empty() {
        None
    } else {
        Some(build_seeds_batch(&projection.nodes, &options.seeds)?)
    };

    Ok(WendaoGraphEvidenceRequestBundle {
        nodes,
        edges,
        seeds,
        semantic_neighbors: None,
        semantic_overlay: None,
    })
}

struct ProjectedLinkGraph {
    nodes: BTreeSet<String>,
    edges: BTreeSet<(String, String)>,
}

impl ProjectedLinkGraph {
    fn from_index(index: &LinkGraphIndex, include_page_index: bool) -> Self {
        let mut nodes = BTreeSet::new();
        let mut edges = BTreeSet::new();

        nodes.extend(index.docs_by_id.keys().cloned());
        nodes.extend(index.virtual_nodes.keys().cloned());

        collect_adjacency(&mut nodes, &mut edges, index);
        if include_page_index {
            collect_page_index(&mut nodes, &mut edges, index);
        }

        Self { nodes, edges }
    }
}

fn collect_adjacency(
    nodes: &mut BTreeSet<String>,
    edges: &mut BTreeSet<(String, String)>,
    index: &LinkGraphIndex,
) {
    for (source, targets) in &index.outgoing {
        nodes.insert(source.clone());
        for target in targets {
            nodes.insert(target.clone());
            edges.insert((source.clone(), target.clone()));
        }
    }
    for (target, sources) in &index.incoming {
        nodes.insert(target.clone());
        for source in sources {
            nodes.insert(source.clone());
            edges.insert((source.clone(), target.clone()));
        }
    }
}

fn collect_page_index(
    nodes: &mut BTreeSet<String>,
    edges: &mut BTreeSet<(String, String)>,
    index: &LinkGraphIndex,
) {
    for roots in index.trees_by_doc.values() {
        for node in roots {
            collect_page_index_node(nodes, edges, node);
        }
    }
    for (node_id, parent_id) in &index.node_parent_map {
        nodes.insert(node_id.clone());
        if let Some(parent_id) = parent_id {
            nodes.insert(parent_id.clone());
            edges.insert((parent_id.clone(), node_id.clone()));
        }
    }
}

fn collect_page_index_node(
    nodes: &mut BTreeSet<String>,
    edges: &mut BTreeSet<(String, String)>,
    node: &PageIndexNode,
) {
    nodes.insert(node.node_id.clone());
    if let Some(parent_id) = &node.parent_id {
        nodes.insert(parent_id.clone());
        edges.insert((parent_id.clone(), node.node_id.clone()));
    }
    for child in &node.children {
        edges.insert((node.node_id.clone(), child.node_id.clone()));
        collect_page_index_node(nodes, edges, child);
    }
}

fn build_nodes_batch(
    nodes: &BTreeSet<String>,
) -> Result<RecordBatch, LinkGraphWendaoGraphEvidenceError> {
    let node_ids = nodes.iter().cloned().collect::<Vec<_>>();
    build_request_batch(
        NODES_TABLE,
        vec![Arc::new(StringArray::from(node_ids)) as ArrayRef],
    )
}

fn build_edges_batch(
    edges: &BTreeSet<(String, String)>,
) -> Result<RecordBatch, LinkGraphWendaoGraphEvidenceError> {
    let mut sources = Vec::with_capacity(edges.len());
    let mut targets = Vec::with_capacity(edges.len());
    for (source, target) in edges {
        sources.push(source.clone());
        targets.push(target.clone());
    }

    build_request_batch(
        EDGES_TABLE,
        vec![
            Arc::new(StringArray::from(sources)) as ArrayRef,
            Arc::new(StringArray::from(targets)) as ArrayRef,
        ],
    )
}

fn build_seeds_batch(
    nodes: &BTreeSet<String>,
    seeds: &[WendaoGraphEvidenceSeed],
) -> Result<RecordBatch, LinkGraphWendaoGraphEvidenceError> {
    let mut node_ids = Vec::with_capacity(seeds.len());
    let mut weights = Vec::with_capacity(seeds.len());
    for seed in seeds {
        validate_seed(nodes, seed)?;
        node_ids.push(seed.node_id.clone());
        weights.push(seed.weight);
    }

    build_request_batch(
        SEEDS_TABLE,
        vec![
            Arc::new(StringArray::from(node_ids)) as ArrayRef,
            Arc::new(Float64Array::from(weights)) as ArrayRef,
        ],
    )
}

fn validate_seed(
    nodes: &BTreeSet<String>,
    seed: &WendaoGraphEvidenceSeed,
) -> Result<(), LinkGraphWendaoGraphEvidenceError> {
    if seed.node_id.trim().is_empty() {
        return Err(LinkGraphWendaoGraphEvidenceError::BlankSeedNode);
    }
    if !seed.weight.is_finite() || seed.weight < 0.0 {
        return Err(LinkGraphWendaoGraphEvidenceError::InvalidSeedWeight {
            node_id: seed.node_id.clone(),
        });
    }
    if !nodes.contains(&seed.node_id) {
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
    let schema =
        wendao_graph_evidence_table_schema(WendaoGraphEvidenceTableKind::Request, table_name)
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
    validate_wendao_graph_evidence_request_schema(table_name, batch.schema().as_ref()).map_err(
        |message| LinkGraphWendaoGraphEvidenceError::Schema {
            table_name,
            message,
        },
    )?;
    Ok(batch)
}
