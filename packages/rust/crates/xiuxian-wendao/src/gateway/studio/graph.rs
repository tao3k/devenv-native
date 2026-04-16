//! Graph intelligence and visualization endpoints for Studio API.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use serde::Deserialize;

use crate::gateway::studio::pathing::studio_display_path;
use crate::gateway::studio::router::{GatewayState, StudioApiError};
use crate::gateway::studio::search::{
    DefinitionMatchMode, DefinitionResolveOptions, resolve_best_definition,
};
use crate::gateway::studio::types::{
    GraphNeighborsResult, GraphNode, StudioNavigationTarget, Topology3dPayload,
};
use crate::link_graph::LinkGraphDirection;

#[derive(Debug, Deserialize)]
pub struct GraphNeighborsQuery {
    pub direction: Option<String>,
    pub hops: Option<usize>,
    pub limit: Option<usize>,
}

/// Gets node neighbors.
pub async fn node_neighbors(
    State(state): State<Arc<GatewayState>>,
    AxumPath(node_id): AxumPath<String>,
) -> Result<Json<GraphNeighborsResult>, StudioApiError> {
    let index = state.link_graph_index().await?;
    let neighbors = index.neighbors(node_id.as_str(), LinkGraphDirection::Both, 1, 100);

    let mut nodes = Vec::new();
    let edges = Vec::new();

    for neighbor in neighbors {
        let node_id_value = neighbor.stem.clone();
        let node_label = if neighbor.title.is_empty() {
            node_id_value.clone()
        } else {
            neighbor.title.clone()
        };
        nodes.push(GraphNode {
            id: node_id_value.clone(),
            label: node_label,
            path: neighbor.path.clone(),
            navigation_target: StudioNavigationTarget {
                path: studio_display_path(state.studio.as_ref(), neighbor.path.as_str()),
                category: "doc".to_string(),
                project_name: None,
                root_label: None,
                line: None,
                line_end: None,
                column: None,
            },
            node_type: "note".to_string(),
            is_center: node_id_value == node_id,
            distance: neighbor.distance,
        });
    }

    Ok(Json(GraphNeighborsResult { nodes, edges }))
}

/// Gets graph neighbors.
pub async fn graph_neighbors(
    State(state): State<Arc<GatewayState>>,
    AxumPath(node_id): AxumPath<String>,
) -> Result<Json<GraphNeighborsResult>, StudioApiError> {
    node_neighbors(State(state), AxumPath(node_id)).await
}

/// Gets 3D topology.
pub async fn topology_3d(
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<Topology3dPayload>, StudioApiError> {
    let _index = state.link_graph_index().await?;
    Ok(Json(Topology3dPayload {
        nodes: Vec::new(),
        links: Vec::new(),
        clusters: Vec::new(),
    }))
}

pub(crate) fn enhance_graph_response_with_definitions(
    state: &Arc<GatewayState>,
    response: &mut GraphNeighborsResult,
) {
    let ast_index = match state.studio.ast_index.read() {
        Ok(guard) => match guard.as_ref() {
            Some(index) => Arc::clone(index),
            None => return,
        },
        Err(_) => return,
    };

    let project_root = state.studio.project_root.clone();
    let config_root = state.studio.config_root.clone();
    let projects = state.studio.configured_projects();
    let navigation_hints = markdown_symbol_navigation_hints(response);

    for node in &mut response.nodes {
        if !node.id.contains("symbol:") {
            continue;
        }

        let Some(definition) = resolve_best_definition(
            node.label.as_str(),
            ast_index.as_slice(),
            project_root.as_path(),
            config_root.as_path(),
            projects.as_slice(),
            &DefinitionResolveOptions {
                scope_patterns: navigation_hints
                    .get(node.id.as_str())
                    .map(|hints| hints.scopes.iter().map(|s| s.to_string()).collect()),
                languages: navigation_hints
                    .get(node.id.as_str())
                    .map(|hints| hints.languages.iter().map(|s| s.to_string()).collect()),
                match_mode: DefinitionMatchMode::ExactOnly,
                include_markdown: false,
                ..DefinitionResolveOptions::default()
            },
        ) else {
            continue;
        };

        let mut navigation_target = definition.navigation_target.clone();
        navigation_target.path =
            studio_display_path(state.studio.as_ref(), definition.path.as_str());
        node.navigation_target = navigation_target;
    }
}

struct SymbolNavigationHints {
    scopes: Vec<String>,
    languages: Vec<String>,
}

fn markdown_symbol_navigation_hints(
    response: &GraphNeighborsResult,
) -> HashMap<String, SymbolNavigationHints> {
    let mut hints = HashMap::new();
    for edge in &response.edges {
        if edge.kind != "Mentions" {
            continue;
        }
        let target_hints =
            hints
                .entry(edge.target_id.clone())
                .or_insert_with(|| SymbolNavigationHints {
                    scopes: Vec::new(),
                    languages: Vec::new(),
                });

        // Simple heuristic: if source node is a file, use its extension/path as hints
        if let Some(source_node) = response.nodes.iter().find(|n| n.id == edge.source_id) {
            let path = Path::new(source_node.path.as_str());
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext = ext.to_string();
                if !target_hints.languages.contains(&ext) {
                    target_hints.languages.push(ext);
                }
            }
        }
    }
    hints
}
