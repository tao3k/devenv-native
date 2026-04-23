//! Advanced layout engine with precision Manhattan routing and protocol metadata.

use std::collections::HashMap;

use petgraph::stable_graph::NodeIndex;
use petgraph::visit::{EdgeRef, IntoEdgeReferences, Topo};

use crate::engine::QianjiEngine;

use super::engine_types::{
    BpmnType, DeepEdge, DeepKnowledgeGraph, DeepNode, EdgeLayout, EntityKind, LayoutResult,
    NodePosition, ZoneLayout,
};
use super::style::QgsTheme;

/// Layout engine for BPMN process and deep graph exports.
pub struct QianjiLayoutEngine {
    theme: QgsTheme,
    layer_spacing: f32,
    node_spacing: f32,
    origin_x: f32,
    origin_y: f32,
}

impl QianjiLayoutEngine {
    /// Creates a layout engine with tuned defaults.
    #[must_use]
    pub fn new(theme: QgsTheme) -> Self {
        Self {
            theme,
            layer_spacing: 450.0,
            node_spacing: 300.0,
            origin_x: 200.0,
            origin_y: 200.0,
        }
    }

    fn usize_to_f32_saturating(value: usize) -> f32 {
        f32::from(u16::try_from(value).unwrap_or(u16::MAX))
    }

    fn resolve_context_uri(node_id: &str) -> Option<String> {
        if node_id.contains("Alpha") || node_id.contains("Beta") {
            Some("$wendao://skills/paper-banana/references/cognitive_specs.md".to_string())
        } else if node_id.contains("Audit") {
            Some("$wendao://skills/paper-banana/references/formal_validator.md".to_string())
        } else if node_id.contains("Foundation") {
            Some("$wendao://skills/paper-banana/references/epistemic_anchor.md".to_string())
        } else {
            None
        }
    }

    fn resolve_bpmn_type(node_id: &str, in_degree: usize, out_degree: usize) -> BpmnType {
        let lower = node_id.to_lowercase();
        if lower.contains("start") {
            BpmnType::StartEvent
        } else if lower.contains("end") || lower.contains("ready") {
            BpmnType::EndEvent
        } else if lower.contains("decide") || out_degree > 1 || in_degree > 1 {
            BpmnType::ExclusiveGateway
        } else if node_id.contains("Audit") {
            BpmnType::BusinessRule
        } else if node_id.contains("Alpha") || node_id.contains("Beta") {
            BpmnType::ServiceTask
        } else {
            BpmnType::Task
        }
    }

    fn node_dimensions(node_type: &BpmnType) -> (f32, f32) {
        match node_type {
            BpmnType::StartEvent | BpmnType::EndEvent => (36.0, 36.0),
            BpmnType::ExclusiveGateway => (50.0, 50.0),
            _ => (140.0, 90.0),
        }
    }

    fn parent_level(
        engine: &QianjiEngine,
        node_idx: NodeIndex,
        levels: &HashMap<NodeIndex, usize>,
    ) -> usize {
        let mut max_parent_level = 0;
        for edge in engine
            .graph
            .edges_directed(node_idx, petgraph::Direction::Incoming)
        {
            if let Some(&parent_level) = levels.get(&edge.source()) {
                max_parent_level = max_parent_level.max(parent_level + 1);
            }
        }
        max_parent_level
    }

    fn build_node(
        &self,
        engine: &QianjiEngine,
        node_idx: NodeIndex,
        level: usize,
        row: usize,
    ) -> NodePosition {
        let node = &engine.graph[node_idx];
        let in_degree = engine
            .graph
            .edges_directed(node_idx, petgraph::Direction::Incoming)
            .count();
        let out_degree = engine.graph.neighbors(node_idx).count();
        let bpmn_type = Self::resolve_bpmn_type(&node.id, in_degree, out_degree);
        let (width, height) = Self::node_dimensions(&bpmn_type);
        NodePosition {
            id: node.id.clone(),
            x: Self::usize_to_f32_saturating(level) * self.layer_spacing + self.origin_x,
            y: Self::usize_to_f32_saturating(row) * self.node_spacing + self.origin_y,
            width,
            height,
            label: node.id.replace('_', " "),
            bpmn_type,
            context_uri: Self::resolve_context_uri(&node.id),
        }
    }

    fn build_waypoints(source: &NodePosition, target: &NodePosition) -> Vec<(f32, f32)> {
        let start_x = source.x + source.width;
        let start_y = source.y + source.height / 2.0;
        let end_x = target.x;
        let end_y = target.y + target.height / 2.0;
        let mid_x = start_x + (end_x - start_x) / 2.0;
        vec![
            (start_x, start_y),
            (mid_x, start_y),
            (mid_x, end_y),
            (end_x, end_y),
        ]
    }

    fn build_edge_label(label: Option<String>, weight: f32) -> Option<String> {
        if weight < 0.99 {
            let probability = format!("{:.0}%", weight * 100.0);
            Some(
                label
                    .map(|value| format!("{value} ({probability})"))
                    .unwrap_or(probability),
            )
        } else {
            label
        }
    }

    fn default_zones() -> Vec<ZoneLayout> {
        Vec::new()
    }

    /// Computes a BPMN-ready layout from the execution graph.
    #[must_use]
    pub fn compute_from_engine(&self, engine: &QianjiEngine) -> LayoutResult {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut topo = Topo::new(&engine.graph);
        let mut levels: HashMap<NodeIndex, usize> = HashMap::new();
        let mut level_counts: HashMap<usize, usize> = HashMap::new();

        while let Some(node_idx) = topo.next(&engine.graph) {
            let level = Self::parent_level(engine, node_idx, &levels);
            levels.insert(node_idx, level);
            let row = *level_counts.entry(level).or_insert(0);
            nodes.push(self.build_node(engine, node_idx, level, row));
            *level_counts.entry(level).or_insert(0) += 1;
        }

        for (index, edge) in engine.graph.edge_references().enumerate() {
            let source_id = &engine.graph[edge.source()].id;
            let target_id = &engine.graph[edge.target()].id;
            let Some(source) = nodes.iter().find(|node| &node.id == source_id) else {
                continue;
            };
            let Some(target) = nodes.iter().find(|node| &node.id == target_id) else {
                continue;
            };

            let weight = edge.weight().weight;
            edges.push(EdgeLayout {
                id: format!("Flow_{index}"),
                from: source.id.clone(),
                to: target.id.clone(),
                label: Self::build_edge_label(edge.weight().label.clone(), weight),
                weight,
                waypoints: Self::build_waypoints(source, target),
            });
        }

        LayoutResult {
            nodes,
            edges,
            zones: Self::default_zones(),
            theme: self.theme.clone(),
        }
    }

    /// Builds an Obsidian-style deep graph from the execution graph.
    #[must_use]
    pub fn compute_obsidian_graph(engine: &QianjiEngine) -> DeepKnowledgeGraph {
        let nodes = engine
            .graph
            .node_indices()
            .map(|node_idx| {
                let node = &engine.graph[node_idx];
                DeepNode {
                    id: node.id.clone(),
                    label: node.id.clone(),
                    kind: EntityKind::Mechanism,
                    parent_id: None,
                }
            })
            .collect();
        let edges = engine
            .graph
            .edge_references()
            .map(|edge| DeepEdge {
                from: engine.graph[edge.source()].id.clone(),
                to: engine.graph[edge.target()].id.clone(),
                relation: "FLOW".to_string(),
            })
            .collect();
        DeepKnowledgeGraph { nodes, edges }
    }
}
