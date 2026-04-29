/// Snapshot of one top-level DMN `dmndi:DMNDI` block.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDmndiSnapshot {
    /// Optional stable DMNDI block identifier.
    pub dmndi_id: Option<String>,
    /// Direct nested `DMNDiagram` placeholder metadata preserved for this bounded slice.
    pub diagrams: Vec<DmnDiagramSnapshot>,
}

/// Snapshot of one direct nested DMNDI `DMNDiagram`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDiagramSnapshot {
    /// Optional stable DMN diagram identifier.
    pub diagram_id: Option<String>,
    /// Number of direct nested `DMNShape` elements discovered for the diagram.
    pub shape_count: usize,
    /// Number of direct nested `DMNEdge` elements discovered for the diagram.
    pub edge_count: usize,
    /// Direct nested `DMNShape` placeholder metadata preserved for this bounded slice.
    pub shapes: Vec<DmnShapeSnapshot>,
    /// Direct nested `DMNEdge` placeholder metadata preserved for this bounded slice.
    pub edges: Vec<DmnEdgeSnapshot>,
}

/// Snapshot of one direct nested DMNDI `DMNShape`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnShapeSnapshot {
    /// Optional stable DMN shape identifier.
    pub shape_id: Option<String>,
    /// Optional referenced DMN element identifier.
    pub dmn_element_ref: Option<String>,
    /// Optional direct `isListedInputData` marker preserved for this bounded slice.
    pub is_listed_input_data: Option<bool>,
    /// Optional direct `isCollapsed` marker preserved for this bounded slice.
    pub is_collapsed: Option<bool>,
    /// Optional direct nested `dc:Bounds` placeholder preserved for this bounded slice.
    pub bounds: Option<DmnBoundsSnapshot>,
    /// Optional direct nested `DMNDecisionServiceDividerLine` placeholder preserved for this bounded slice.
    pub decision_service_divider_line: Option<DmnDecisionServiceDividerLineSnapshot>,
    /// Optional direct nested `DMNLabel` placeholder preserved for this bounded slice.
    pub label: Option<DmnLabelSnapshot>,
}

/// Snapshot of one direct nested `dc:Bounds` placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnBoundsSnapshot {
    /// Optional direct `x` payload preserved from `dc:Bounds`.
    pub x: Option<String>,
    /// Optional direct `y` payload preserved from `dc:Bounds`.
    pub y: Option<String>,
    /// Optional direct `width` payload preserved from `dc:Bounds`.
    pub width: Option<String>,
    /// Optional direct `height` payload preserved from `dc:Bounds`.
    pub height: Option<String>,
}

/// Snapshot of one direct nested DMNDI `DMNEdge`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnEdgeSnapshot {
    /// Optional stable DMN edge identifier.
    pub edge_id: Option<String>,
    /// Optional referenced DMN element identifier.
    pub dmn_element_ref: Option<String>,
    /// Direct nested `di:waypoint` placeholders preserved for this bounded slice.
    pub waypoints: Vec<DmnWaypointSnapshot>,
    /// Optional direct nested `DMNLabel` placeholder preserved for this bounded slice.
    pub label: Option<DmnLabelSnapshot>,
}

/// Snapshot of one direct nested `di:waypoint` placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnWaypointSnapshot {
    /// Optional direct `x` payload preserved from `di:waypoint`.
    pub x: Option<String>,
    /// Optional direct `y` payload preserved from `di:waypoint`.
    pub y: Option<String>,
}

/// Snapshot of one direct nested `DMNDecisionServiceDividerLine` placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionServiceDividerLineSnapshot {
    /// Direct nested `di:waypoint` placeholders preserved for this bounded slice.
    pub waypoints: Vec<DmnWaypointSnapshot>,
}

/// Snapshot of one direct nested DMNDI `DMNLabel`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnLabelSnapshot {
    /// Optional stable DMN label identifier.
    pub label_id: Option<String>,
    /// Optional direct nested `dc:Bounds` placeholder preserved for this bounded slice.
    pub bounds: Option<DmnBoundsSnapshot>,
    /// Optional direct `DMNLabel/Text` payload preserved for this bounded slice.
    pub text: Option<String>,
}
