//! Public bpmn model api di contracts for BPMN/DMN engine integration.

use super::types::{BpmnSnapshotFlag, BpmnSnapshotId, BpmnSnapshotKind};

/// Snapshot of one BPMN DI `BPMNDiagram`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDiagramSnapshot {
    /// Optional stable BPMN diagram identifier.
    pub diagram_id: Option<String>,
    /// Optional human-readable BPMN diagram name.
    pub name: Option<String>,
    /// Optional BPMN diagram documentation attribute.
    pub documentation: Option<String>,
    /// Optional BPMN diagram resolution attribute.
    pub resolution: Option<String>,
    /// Optional direct nested BPMN DI plane metadata.
    pub plane: Option<BpmnPlaneSnapshot>,
    /// Direct nested BPMN DI label styles preserved in source order.
    #[serde(default)]
    pub label_styles: Vec<BpmnLabelStyleSnapshot>,
}

/// Snapshot of one BPMN DI `BPMNPlane`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnPlaneSnapshot {
    /// Optional stable BPMN plane identifier.
    pub plane_id: Option<String>,
    /// Optional referenced BPMN semantic element.
    pub bpmn_element: Option<String>,
    /// Direct nested BPMN DI shapes preserved in source order.
    #[serde(default)]
    pub shapes: Vec<BpmnShapeSnapshot>,
    /// Direct nested BPMN DI edges preserved in source order.
    #[serde(default)]
    pub edges: Vec<BpmnEdgeSnapshot>,
}

/// Snapshot of one BPMN DI `BPMNShape`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnShapeSnapshot {
    /// Optional stable BPMN shape identifier.
    pub shape_id: Option<BpmnSnapshotId>,
    /// Optional referenced BPMN semantic element.
    pub bpmn_element: Option<String>,
    /// Optional horizontal marker.
    pub is_horizontal: Option<BpmnSnapshotFlag>,
    /// Optional expanded marker.
    pub is_expanded: Option<BpmnSnapshotFlag>,
    /// Optional marker-visibility marker.
    pub is_marker_visible: Option<BpmnSnapshotFlag>,
    /// Optional message-visibility marker.
    pub is_message_visible: Option<BpmnSnapshotFlag>,
    /// Optional participant band kind.
    pub participant_band_kind: Option<BpmnSnapshotKind>,
    /// Optional choreography activity shape reference.
    pub choreography_activity_shape: Option<String>,
    /// Optional direct nested `dc:Bounds` metadata.
    pub bounds: Option<BpmnBoundsSnapshot>,
    /// Optional direct nested BPMN DI label metadata.
    pub label: Option<BpmnLabelSnapshot>,
}

/// Snapshot of one BPMN DI `BPMNEdge`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnEdgeSnapshot {
    /// Optional stable BPMN edge identifier.
    pub edge_id: Option<String>,
    /// Optional referenced BPMN semantic element.
    pub bpmn_element: Option<String>,
    /// Optional source diagram element reference.
    pub source_element: Option<String>,
    /// Optional target diagram element reference.
    pub target_element: Option<String>,
    /// Optional message visible kind.
    pub message_visible_kind: Option<BpmnSnapshotKind>,
    /// Direct nested `di:waypoint` metadata preserved in source order.
    #[serde(default)]
    pub waypoints: Vec<BpmnWaypointSnapshot>,
    /// Optional direct nested BPMN DI label metadata.
    pub label: Option<BpmnLabelSnapshot>,
}

/// Snapshot of one direct nested `dc:Bounds` payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnBoundsSnapshot {
    /// Optional direct `x` payload preserved from `dc:Bounds`.
    pub x: Option<String>,
    /// Optional direct `y` payload preserved from `dc:Bounds`.
    pub y: Option<String>,
    /// Optional direct `width` payload preserved from `dc:Bounds`.
    pub width: Option<String>,
    /// Optional direct `height` payload preserved from `dc:Bounds`.
    pub height: Option<String>,
}

/// Snapshot of one direct nested `di:waypoint` payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnWaypointSnapshot {
    /// Optional direct `x` payload preserved from `di:waypoint`.
    pub x: Option<String>,
    /// Optional direct `y` payload preserved from `di:waypoint`.
    pub y: Option<String>,
}

/// Snapshot of one BPMN DI `BPMNLabel`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnLabelSnapshot {
    /// Optional stable BPMN label identifier.
    pub label_id: Option<String>,
    /// Optional referenced BPMN label style.
    pub label_style: Option<String>,
    /// Optional direct nested `dc:Bounds` metadata.
    pub bounds: Option<BpmnBoundsSnapshot>,
}

/// Snapshot of one BPMN DI `BPMNLabelStyle`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnLabelStyleSnapshot {
    /// Optional stable BPMN label style identifier.
    pub style_id: Option<String>,
    /// Optional direct nested `dc:Font` metadata.
    pub font: Option<BpmnFontSnapshot>,
}

/// Snapshot of one direct nested `dc:Font` payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnFontSnapshot {
    /// Optional font family name.
    pub name: Option<String>,
    /// Optional font size payload.
    pub size: Option<String>,
    /// Optional bold marker.
    pub is_bold: Option<BpmnSnapshotFlag>,
    /// Optional italic marker.
    pub is_italic: Option<BpmnSnapshotFlag>,
    /// Optional underline marker.
    pub is_underline: Option<BpmnSnapshotFlag>,
    /// Optional strike-through marker.
    pub is_strike_through: Option<BpmnSnapshotFlag>,
}
