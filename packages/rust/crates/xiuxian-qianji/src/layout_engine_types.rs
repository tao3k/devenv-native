use serde::{Deserialize, Serialize};

use super::style::QgsTheme;

/// BPMN node kinds emitted by Qianji layout export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BpmnType {
    /// Start event.
    StartEvent,
    /// End event.
    EndEvent,
    /// Generic task.
    Task,
    /// Service task.
    ServiceTask,
    /// Business rule task.
    BusinessRule,
    /// Exclusive gateway.
    ExclusiveGateway,
}

/// Position and semantic metadata for one rendered node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    /// Stable node identifier.
    pub id: String,
    /// Top-left x position in diagram space.
    pub x: f32,
    /// Top-left y position in diagram space.
    pub y: f32,
    /// Node width.
    pub width: f32,
    /// Node height.
    pub height: f32,
    /// Human-readable label.
    pub label: String,
    /// Semantic BPMN type.
    pub bpmn_type: BpmnType,
    /// Optional protocol metadata pointing to Wendao resources.
    pub context_uri: Option<String>,
}

/// Rendered edge metadata and route for BPMN DI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeLayout {
    /// Stable edge identifier.
    pub id: String,
    /// Source node id.
    pub from: String,
    /// Target node id.
    pub to: String,
    /// Optional edge label shown in process and DI layers.
    pub label: Option<String>,
    /// Branch weight from runtime graph edge.
    pub weight: f32,
    /// Ordered waypoint list for orthogonal routing.
    pub waypoints: Vec<(f32, f32)>,
}

/// Full layout payload used by BPMN renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    /// Positioned nodes.
    pub nodes: Vec<NodePosition>,
    /// Routed edges.
    pub edges: Vec<EdgeLayout>,
    /// Optional semantic zones.
    pub zones: Vec<ZoneLayout>,
    /// Theme snapshot used during layout.
    pub theme: QgsTheme,
}

/// Visual grouping rectangle for optional diagram zones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneLayout {
    /// Stable zone id.
    pub id: String,
    /// Zone label.
    pub label: String,
    /// Top-left x position.
    pub x: f32,
    /// Top-left y position.
    pub y: f32,
    /// Zone width.
    pub width: f32,
    /// Zone height.
    pub height: f32,
    /// Zone color token.
    pub color: String,
}

/// Deep graph entity kinds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityKind {
    /// Executable mechanism node.
    Mechanism,
    /// Variable/attribute node.
    Variable,
    /// Knowledge/resource node.
    Knowledge,
}

/// Deep graph node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepNode {
    /// Stable node identifier.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Semantic kind.
    pub kind: EntityKind,
    /// Optional parent node id.
    pub parent_id: Option<String>,
}

/// Deep graph edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepEdge {
    /// Source node id.
    pub from: String,
    /// Target node id.
    pub to: String,
    /// Relation label.
    pub relation: String,
}

/// Serializable deep graph payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepKnowledgeGraph {
    /// Graph nodes.
    pub nodes: Vec<DeepNode>,
    /// Graph edges.
    pub edges: Vec<DeepEdge>,
}
