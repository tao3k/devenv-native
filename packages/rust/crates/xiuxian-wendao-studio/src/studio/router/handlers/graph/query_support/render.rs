use std::path::Path;

use crate::studio::pathing::{normalize_path_like, studio_display_path};
use crate::studio::router::GatewayState;
use crate::studio::types::GraphNode;
use crate::studio::vfs::resolve_navigation_target;
use xiuxian_wendao::link_graph::LinkGraphIndex;

pub(crate) fn preferred_label(title: &str, fallback_path: &str) -> String {
    if !title.trim().is_empty() {
        return title.to_string();
    }
    if let Some(stem) = Path::new(fallback_path)
        .file_stem()
        .and_then(|value| value.to_str())
        && !stem.trim().is_empty()
    {
        return stem.to_string();
    }
    fallback_path.to_string()
}

fn resolve_graph_node_id_by_display_path(
    state: &GatewayState,
    index: &LinkGraphIndex,
    node_id: &str,
) -> Option<String> {
    let normalized_target = normalize_path_like(node_id)?;
    for doc in index.docs() {
        let display_path = studio_display_path(state.studio.as_ref(), doc.path.as_str());
        let Some(normalized_display_path) = normalize_path_like(display_path.as_str()) else {
            continue;
        };
        if normalized_display_path == normalized_target {
            return Some(doc.id.clone());
        }
    }
    None
}

fn resolve_graph_node_id_by_internal_path(index: &LinkGraphIndex, node_id: &str) -> Option<String> {
    let normalized = normalize_path_like(node_id)?;
    index
        .resolve_doc_id_pub(normalized.as_str())
        .map(str::to_owned)
}

pub(crate) fn resolve_graph_node_id(
    state: &GatewayState,
    index: &LinkGraphIndex,
    node_id: &str,
) -> Option<String> {
    resolve_graph_node_id_by_display_path(state, index, node_id)
        .or_else(|| resolve_graph_node_id_by_internal_path(index, node_id))
}

pub(crate) fn graph_node(
    state: &GatewayState,
    internal_path: &str,
    label: &str,
    is_center: bool,
    distance: usize,
) -> GraphNode {
    let display_path = studio_display_path(state.studio.as_ref(), internal_path);
    let navigation_target = resolve_navigation_target(state.studio.as_ref(), display_path.as_str());
    GraphNode {
        id: display_path.clone().into(),
        label: preferred_label(label, display_path.as_str()),
        path: display_path.into(),
        navigation_target: Some(navigation_target),
        node_type: "doc".into(),
        is_center: is_center.into(),
        distance,
    }
}

pub(crate) fn topology_position(index: usize, total: usize) -> [f32; 3] {
    if total == 0 {
        return [0.0, 0.0, 0.0];
    }

    let angle = std::f32::consts::TAU * layout_scalar(index) / layout_scalar(total);
    let radius = 14.0 + layout_scalar(index % 7) * 2.5;
    let depth = layout_scalar(index % 9) - 4.0;
    [radius * angle.cos(), radius * angle.sin(), depth]
}

pub(crate) fn topology_color(index: usize) -> &'static str {
    const PALETTE: [&str; 8] = [
        "#9ece6a", "#73daca", "#7aa2f7", "#f7768e", "#e0af68", "#bb9af7", "#7dcfff", "#c0caf5",
    ];
    PALETTE[index % PALETTE.len()]
}

pub(crate) fn layout_scalar(value: usize) -> f32 {
    f32::from(u16::try_from(value).unwrap_or(u16::MAX))
}
