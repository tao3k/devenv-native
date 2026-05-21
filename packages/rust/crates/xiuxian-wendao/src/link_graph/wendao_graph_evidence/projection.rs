//! Project `LinkGraph` evidence into the generic `WendaoGraph.jl` request tables.

use std::collections::BTreeSet;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_julia::{
    WendaoGraphEvidenceTableKind, validate_wendao_graph_evidence_request_schema,
    wendao_graph_evidence_table_schema,
};

use super::types::{
    LinkGraphWendaoGraphEvidenceError, WendaoGraphEvidenceRequestBundle,
    WendaoGraphEvidenceRequestOptions, WendaoGraphEvidenceSeed, WendaoGraphSemanticNeighbor,
    WendaoGraphSemanticOverlayEdge,
};
use crate::link_graph::{LinkGraphIndex, PageIndexNode};

const NODES_TABLE: &str = "nodes";
const EDGES_TABLE: &str = "edges";
const SEEDS_TABLE: &str = "seeds";
const SEMANTIC_NEIGHBORS_TABLE: &str = "semantic_neighbors";
const SEMANTIC_OVERLAY_TABLE: &str = "semantic_overlay";

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
    if !options.semantic_neighbors.is_empty() && !options.semantic_overlay.is_empty() {
        return Err(LinkGraphWendaoGraphEvidenceError::ConflictingSemanticEvidence);
    }

    let projection = ProjectedLinkGraph::from_index(index, options.include_page_index);
    let nodes = build_nodes_batch(&projection.nodes)?;
    let edges = build_edges_batch(&projection.edges)?;
    let seeds = if options.seeds.is_empty() {
        None
    } else {
        Some(build_seeds_batch(&projection.nodes, &options.seeds)?)
    };
    let semantic_neighbors = if options.semantic_neighbors.is_empty() {
        None
    } else {
        Some(build_semantic_neighbors_batch(
            &projection.nodes,
            &options.semantic_neighbors,
        )?)
    };
    let semantic_overlay = if options.semantic_overlay.is_empty() {
        None
    } else {
        Some(build_semantic_overlay_batch(
            &projection.nodes,
            &options.semantic_overlay,
        )?)
    };

    Ok(WendaoGraphEvidenceRequestBundle {
        nodes,
        edges,
        seeds,
        semantic_neighbors,
        semantic_overlay,
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

fn build_semantic_neighbors_batch(
    nodes: &BTreeSet<String>,
    semantic_neighbors: &[WendaoGraphSemanticNeighbor],
) -> Result<RecordBatch, LinkGraphWendaoGraphEvidenceError> {
    let mut query_ids = Vec::with_capacity(semantic_neighbors.len());
    let mut neighbor_ids = Vec::with_capacity(semantic_neighbors.len());
    let mut query_indices = Vec::with_capacity(semantic_neighbors.len());
    let mut neighbor_indices = Vec::with_capacity(semantic_neighbors.len());
    let mut ranks = Vec::with_capacity(semantic_neighbors.len());
    let mut distances = Vec::with_capacity(semantic_neighbors.len());

    for semantic_neighbor in semantic_neighbors {
        validate_semantic_neighbor(nodes, semantic_neighbor)?;
        query_ids.push(semantic_neighbor.query_id.clone());
        neighbor_ids.push(semantic_neighbor.neighbor_id.clone());
        query_indices.push(semantic_neighbor.query_index);
        neighbor_indices.push(semantic_neighbor.neighbor_index);
        ranks.push(semantic_neighbor.rank);
        distances.push(semantic_neighbor.distance);
    }

    build_request_batch(
        SEMANTIC_NEIGHBORS_TABLE,
        vec![
            Arc::new(StringArray::from(query_ids)) as ArrayRef,
            Arc::new(StringArray::from(neighbor_ids)) as ArrayRef,
            Arc::new(Int64Array::from(query_indices)) as ArrayRef,
            Arc::new(Int64Array::from(neighbor_indices)) as ArrayRef,
            Arc::new(Int64Array::from(ranks)) as ArrayRef,
            Arc::new(Float64Array::from(distances)) as ArrayRef,
        ],
    )
}

fn build_semantic_overlay_batch(
    nodes: &BTreeSet<String>,
    semantic_overlay: &[WendaoGraphSemanticOverlayEdge],
) -> Result<RecordBatch, LinkGraphWendaoGraphEvidenceError> {
    let mut source_ids = Vec::with_capacity(semantic_overlay.len());
    let mut target_ids = Vec::with_capacity(semantic_overlay.len());
    let mut source_indices = Vec::with_capacity(semantic_overlay.len());
    let mut target_indices = Vec::with_capacity(semantic_overlay.len());
    let mut ranks = Vec::with_capacity(semantic_overlay.len());
    let mut distances = Vec::with_capacity(semantic_overlay.len());
    let mut weights = Vec::with_capacity(semantic_overlay.len());
    let mut edge_kinds = Vec::with_capacity(semantic_overlay.len());

    for edge in semantic_overlay {
        validate_semantic_overlay_edge(nodes, edge)?;
        source_ids.push(edge.source_id.clone());
        target_ids.push(edge.target_id.clone());
        source_indices.push(edge.source_index);
        target_indices.push(edge.target_index);
        ranks.push(edge.rank);
        distances.push(edge.distance);
        weights.push(edge.weight);
        edge_kinds.push(edge.edge_kind.clone());
    }

    build_request_batch(
        SEMANTIC_OVERLAY_TABLE,
        vec![
            Arc::new(StringArray::from(source_ids)) as ArrayRef,
            Arc::new(StringArray::from(target_ids)) as ArrayRef,
            Arc::new(Int64Array::from(source_indices)) as ArrayRef,
            Arc::new(Int64Array::from(target_indices)) as ArrayRef,
            Arc::new(Int64Array::from(ranks)) as ArrayRef,
            Arc::new(Float64Array::from(distances)) as ArrayRef,
            Arc::new(Float64Array::from(weights)) as ArrayRef,
            Arc::new(StringArray::from(edge_kinds)) as ArrayRef,
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

fn validate_semantic_neighbor(
    nodes: &BTreeSet<String>,
    semantic_neighbor: &WendaoGraphSemanticNeighbor,
) -> Result<(), LinkGraphWendaoGraphEvidenceError> {
    validate_semantic_neighbor_node(nodes, &semantic_neighbor.query_id)?;
    validate_semantic_neighbor_node(nodes, &semantic_neighbor.neighbor_id)?;
    if semantic_neighbor.query_index <= 0 {
        return Err(invalid_semantic_neighbor(semantic_neighbor, "query_index"));
    }
    if semantic_neighbor.neighbor_index <= 0 {
        return Err(invalid_semantic_neighbor(
            semantic_neighbor,
            "neighbor_index",
        ));
    }
    if semantic_neighbor.rank <= 0 {
        return Err(invalid_semantic_neighbor(semantic_neighbor, "rank"));
    }
    if !semantic_neighbor.distance.is_finite() || semantic_neighbor.distance < 0.0 {
        return Err(invalid_semantic_neighbor(semantic_neighbor, "distance"));
    }
    Ok(())
}

fn validate_semantic_neighbor_node(
    nodes: &BTreeSet<String>,
    node_id: &str,
) -> Result<(), LinkGraphWendaoGraphEvidenceError> {
    if node_id.trim().is_empty() || !nodes.contains(node_id) {
        return Err(
            LinkGraphWendaoGraphEvidenceError::UnknownSemanticNeighborNode {
                node_id: node_id.to_string(),
            },
        );
    }
    Ok(())
}

fn invalid_semantic_neighbor(
    semantic_neighbor: &WendaoGraphSemanticNeighbor,
    field: &'static str,
) -> LinkGraphWendaoGraphEvidenceError {
    LinkGraphWendaoGraphEvidenceError::InvalidSemanticNeighbor {
        query_id: semantic_neighbor.query_id.clone(),
        neighbor_id: semantic_neighbor.neighbor_id.clone(),
        field,
    }
}

fn validate_semantic_overlay_edge(
    nodes: &BTreeSet<String>,
    edge: &WendaoGraphSemanticOverlayEdge,
) -> Result<(), LinkGraphWendaoGraphEvidenceError> {
    validate_semantic_overlay_node(nodes, &edge.source_id)?;
    validate_semantic_overlay_node(nodes, &edge.target_id)?;
    if edge.source_index <= 0 {
        return Err(invalid_semantic_overlay(edge, "source_index"));
    }
    if edge.target_index <= 0 {
        return Err(invalid_semantic_overlay(edge, "target_index"));
    }
    if edge.rank <= 0 {
        return Err(invalid_semantic_overlay(edge, "rank"));
    }
    if !edge.distance.is_finite() || edge.distance < 0.0 {
        return Err(invalid_semantic_overlay(edge, "distance"));
    }
    if !edge.weight.is_finite() || edge.weight < 0.0 {
        return Err(invalid_semantic_overlay(edge, "weight"));
    }
    if edge.edge_kind.trim().is_empty() {
        return Err(invalid_semantic_overlay(edge, "edge_kind"));
    }
    Ok(())
}

fn validate_semantic_overlay_node(
    nodes: &BTreeSet<String>,
    node_id: &str,
) -> Result<(), LinkGraphWendaoGraphEvidenceError> {
    if node_id.trim().is_empty() || !nodes.contains(node_id) {
        return Err(
            LinkGraphWendaoGraphEvidenceError::UnknownSemanticOverlayNode {
                node_id: node_id.to_string(),
            },
        );
    }
    Ok(())
}

fn invalid_semantic_overlay(
    edge: &WendaoGraphSemanticOverlayEdge,
    field: &'static str,
) -> LinkGraphWendaoGraphEvidenceError {
    LinkGraphWendaoGraphEvidenceError::InvalidSemanticOverlay {
        source_id: edge.source_id.clone(),
        target_id: edge.target_id.clone(),
        field,
    }
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
