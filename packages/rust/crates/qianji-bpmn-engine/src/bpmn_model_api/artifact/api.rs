/// Snapshot of one BPMN artifact `association`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnAssociationSnapshot {
    /// Optional stable association identifier.
    pub association_id: Option<String>,
    /// Optional source reference.
    pub source_ref: Option<String>,
    /// Optional target reference.
    pub target_ref: Option<String>,
    /// Optional BPMN association direction.
    pub association_direction: Option<String>,
}

/// Snapshot of one BPMN artifact `group`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnGroupSnapshot {
    /// Optional stable group identifier.
    pub group_id: Option<String>,
    /// Optional referenced category value.
    pub category_value_ref: Option<String>,
}

/// Snapshot of one BPMN artifact `textAnnotation`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnTextAnnotationSnapshot {
    /// Optional stable text-annotation identifier.
    pub annotation_id: Option<String>,
    /// Optional BPMN text format.
    pub text_format: Option<String>,
    /// Optional nested text payload.
    pub text: Option<String>,
}
