use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use log::debug;

use crate::studio::pathing::studio_display_path;
use crate::studio::router::{GatewayState, StudioApiError};
use crate::studio::types::{
    GraphLink, GraphNeighborsResponse, GraphNode, Topology3dPayload, TopologyCluster, TopologyLink,
    TopologyNode,
};
use xiuxian_wendao::link_graph::{LinkGraphDirection, LinkGraphIndex};
use xiuxian_wendao::query_core::{
    GraphDirection as QueryCoreGraphDirection, InMemoryWendaoExplainSink, WendaoExplainEvent,
    WendaoGraphNode, WendaoGraphProjection, explain_events_summary,
    query_graph_neighbors_projection,
};

use crate::studio::router::handlers::graph::shared::{
    graph_node, layout_scalar, preferred_label, resolve_graph_node_id, topology_color,
    topology_position,
};

pub(crate) async fn run_graph_neighbors(
    state: Arc<GatewayState>,
    node_id: &str,
    direction: LinkGraphDirection,
    hops: usize,
    limit: usize,
) -> Result<GraphNeighborsResponse, StudioApiError> {
    let index = state.link_graph_index().await?;
    let (resolved_node_id, center_path, center_title) =
        resolve_center_node(state.as_ref(), index.as_ref(), node_id)?;
    let explain_sink = Arc::new(InMemoryWendaoExplainSink::new());
    let projection = query_graph_neighbors_projection(
        Arc::clone(&index),
        resolved_node_id.as_str(),
        query_core_graph_direction(direction),
        hops,
        limit,
        Some(explain_sink.clone()),
    )
    .await
    .map_err(|error| {
        StudioApiError::internal(
            "GRAPH_NEIGHBORS_QUERY_CORE_FAILED",
            "Failed to query graph neighbors through query core",
            Some(error.to_string()),
        )
    })?;
    record_graph_query_core_explain(
        "graph_neighbors",
        resolved_node_id.as_str(),
        direction,
        hops,
        limit,
        explain_sink.events().as_slice(),
    );

    Ok(graph_neighbors_response_from_projection(
        state.as_ref(),
        center_path.as_str(),
        center_title.as_str(),
        &projection,
    ))
}

pub(crate) async fn run_topology_3d(
    state: Arc<GatewayState>,
) -> Result<Topology3dPayload, StudioApiError> {
    let index = state.link_graph_index().await?;
    let docs = index.toc(usize::MAX);
    let total = docs.len();

    let mut nodes = Vec::with_capacity(total);
    let mut cluster_members = BTreeMap::<String, Vec<[f32; 3]>>::new();
    for (position_index, doc) in docs.iter().enumerate() {
        let display_path = studio_display_path(state.studio.as_ref(), doc.path.as_str());
        let cluster_id = display_path
            .split('/')
            .next()
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(ToOwned::to_owned);
        let position = topology_position(position_index, total);

        if let Some(cluster_id) = cluster_id.as_ref() {
            cluster_members
                .entry(cluster_id.clone())
                .or_default()
                .push(position);
        }

        nodes.push(TopologyNode {
            id: display_path.clone(),
            name: preferred_label(doc.title.as_str(), display_path.as_str()),
            node_type: "doc".to_string(),
            position,
            cluster_id,
        });
    }

    let mut seen_links = BTreeSet::<(String, String)>::new();
    let mut links = Vec::new();
    for doc in &docs {
        let from = studio_display_path(state.studio.as_ref(), doc.path.as_str());
        for neighbor in
            index.neighbors(doc.id.as_str(), LinkGraphDirection::Outgoing, 1, usize::MAX)
        {
            let to = studio_display_path(state.studio.as_ref(), neighbor.path.as_str());
            if seen_links.insert((from.clone(), to.clone())) {
                links.push(TopologyLink {
                    from: from.clone(),
                    to,
                    label: None,
                });
            }
        }
    }

    let mut clusters = cluster_members
        .into_iter()
        .enumerate()
        .map(|(index, (cluster_id, positions))| {
            let node_count = positions.len();
            let (sum_x, sum_y, sum_z) = positions.into_iter().fold(
                (0.0_f32, 0.0_f32, 0.0_f32),
                |(acc_x, acc_y, acc_z), [x, y, z]| (acc_x + x, acc_y + y, acc_z + z),
            );
            let scale = layout_scalar(node_count.max(1));
            TopologyCluster {
                id: cluster_id.clone(),
                name: cluster_id,
                centroid: [sum_x / scale, sum_y / scale, sum_z / scale],
                node_count,
                color: topology_color(index).to_string(),
            }
        })
        .collect::<Vec<_>>();
    clusters.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(Topology3dPayload {
        nodes,
        links,
        clusters,
    })
}

fn record_graph_query_core_explain(
    route: &str,
    node_id: &str,
    direction: LinkGraphDirection,
    hops: usize,
    limit: usize,
    events: &[WendaoExplainEvent],
) {
    if events.is_empty() {
        return;
    }
    debug!(
        "query_core graph explain route={route} node_id={node_id} direction={direction:?} hops={hops} limit={limit} summary={}",
        explain_events_summary(events)
    );
}

fn query_core_graph_direction(direction: LinkGraphDirection) -> QueryCoreGraphDirection {
    match direction {
        LinkGraphDirection::Incoming => QueryCoreGraphDirection::Incoming,
        LinkGraphDirection::Outgoing => QueryCoreGraphDirection::Outgoing,
        LinkGraphDirection::Both => QueryCoreGraphDirection::Both,
    }
}

fn resolve_center_node(
    state: &GatewayState,
    index: &LinkGraphIndex,
    node_id: &str,
) -> Result<(String, String, String), StudioApiError> {
    let Some(resolved_node_id) = resolve_graph_node_id(state, index, node_id) else {
        return Err(graph_node_not_found(node_id));
    };
    let Some(center_metadata) = index.metadata(resolved_node_id.as_str()) else {
        return Err(graph_node_not_found(node_id));
    };
    Ok((
        resolved_node_id,
        center_metadata.path.clone(),
        center_metadata.title.clone(),
    ))
}

fn graph_neighbors_response_from_projection(
    state: &GatewayState,
    center_path: &str,
    center_title: &str,
    projection: &WendaoGraphProjection,
) -> GraphNeighborsResponse {
    let mut nodes = projection
        .nodes
        .iter()
        .map(|node| graph_node_from_projection(state, center_title, node))
        .collect::<Vec<_>>();
    let mut links = projection
        .links
        .iter()
        .map(|link| GraphLink {
            source: studio_display_path(state.studio.as_ref(), link.source_path.as_str()),
            target: studio_display_path(state.studio.as_ref(), link.target_path.as_str()),
            direction: link.direction.clone(),
            distance: link.distance,
        })
        .collect::<Vec<_>>();
    let center = graph_node(state, center_path, center_title, true, 0);

    nodes.sort_by(|left, right| {
        right
            .is_center
            .cmp(&left.is_center)
            .then_with(|| left.distance.cmp(&right.distance))
            .then_with(|| left.id.cmp(&right.id))
    });
    links.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then_with(|| left.target.cmp(&right.target))
            .then_with(|| left.direction.cmp(&right.direction))
    });

    GraphNeighborsResponse {
        center,
        total_nodes: nodes.len(),
        total_links: links.len(),
        nodes,
        links,
    }
}

fn graph_node_from_projection(
    state: &GatewayState,
    center_title: &str,
    node: &WendaoGraphNode,
) -> GraphNode {
    let title = if node.title.trim().is_empty() {
        center_title
    } else {
        node.title.as_str()
    };
    graph_node(
        state,
        node.path.as_str(),
        title,
        node.is_center,
        node.distance,
    )
}

fn graph_node_not_found(node_id: &str) -> StudioApiError {
    StudioApiError::not_found(format!("graph node `{node_id}` was not found"))
}
