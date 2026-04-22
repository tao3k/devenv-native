use crate::dmn_model_api::{
    DmnAssociationSnapshot, DmnBusinessKnowledgeModelSnapshot, DmnDecisionServiceSnapshot,
    DmnDiagramSnapshot, DmnDmndiSnapshot, DmnDocumentSnapshot, DmnElementCollectionSnapshot,
    DmnGroupSnapshot, DmnInputDataSnapshot, DmnItemDefinitionSnapshot, DmnKnowledgeSourceSnapshot,
    DmnOrganizationUnitSnapshot, DmnPerformanceIndicatorSnapshot, DmnTextAnnotationSnapshot,
};
use serde_json::{Value, json};

pub(super) fn augment_evidence(
    mut evidence: Value,
    snapshot: Option<&DmnDocumentSnapshot>,
    decision_id: Option<&str>,
) -> Value {
    let Some(snapshot) = snapshot else {
        return evidence;
    };
    let Value::Object(ref mut map) = evidence else {
        return evidence;
    };
    map.insert("document_root".to_string(), json!(snapshot.root));
    map.insert(
        "document_decision_count".to_string(),
        json!(snapshot.decisions.len()),
    );
    if let Some(decision_id) = decision_id
        && let Some(decision) = snapshot.decision(decision_id)
    {
        map.insert("decision_snapshot".to_string(), json!(decision));
    }
    evidence
}

pub(super) fn root_context(snapshot: Option<&DmnDocumentSnapshot>) -> String {
    let Some(snapshot) = snapshot else {
        return String::new();
    };
    let root = &snapshot.root;
    let mut parts = root_intro_parts(root);
    push_root_counts(root, &mut parts);
    push_root_metadata(root, &mut parts);
    format!(" {}", parts.join(", ") + ".")
}

pub(super) fn dmndi_metadata_context(snapshot: Option<&DmnDocumentSnapshot>) -> String {
    let Some(snapshot) = snapshot else {
        return String::new();
    };
    let metadata = snapshot
        .root
        .dmndi_blocks
        .iter()
        .take(2)
        .map(summarize_dmndi)
        .collect::<Vec<_>>()
        .join("; ");
    if metadata.is_empty() {
        String::new()
    } else {
        format!(" Snapshot DMNDI metadata: {metadata}.")
    }
}

fn root_intro_parts(root: &crate::dmn_model_api::DmnRootSnapshot) -> Vec<String> {
    let mut parts = vec![format!(
        "The scanned document root was `<{}>`",
        root.element_name
    )];
    if let Some(model_namespace_uri) = root.model_namespace_uri.as_deref() {
        parts.push(format!("model namespace '{model_namespace_uri}'"));
    }
    if let Some(model_version_hint) = root.model_version_hint.as_deref() {
        parts.push(format!("version hint '{model_version_hint}'"));
    }
    parts
}

fn push_root_counts(root: &crate::dmn_model_api::DmnRootSnapshot, parts: &mut Vec<String>) {
    push_root_count(parts, "import", root.import_count);
    push_root_count(parts, "itemDefinition", root.item_definition_count);
    push_root_count(parts, "inputData", root.input_data_count);
    push_root_count(parts, "knowledgeSource", root.knowledge_source_count);
    push_root_count(
        parts,
        "businessKnowledgeModel",
        root.business_knowledge_model_count,
    );
    push_root_count(parts, "decisionService", root.decision_service_count);
    push_root_count(parts, "organizationUnit", root.organization_unit_count);
    push_root_count(
        parts,
        "performanceIndicator",
        root.performance_indicator_count,
    );
    push_root_count(parts, "textAnnotation", root.text_annotation_count);
    push_root_count(parts, "association", root.association_count);
    push_root_count(parts, "elementCollection", root.element_collection_count);
    push_root_count(parts, "group", root.group_count);
    push_root_count(parts, "DMNDI", root.dmndi_count);
}

fn push_root_count(parts: &mut Vec<String>, label: &str, count: usize) {
    if count > 0 {
        parts.push(format!("top-level {label} count {count}"));
    }
}

fn push_root_metadata(root: &crate::dmn_model_api::DmnRootSnapshot, parts: &mut Vec<String>) {
    push_metadata_summary(
        parts,
        "itemDefinition metadata",
        root.item_definitions.iter().map(summarize_item_definition),
    );
    push_metadata_summary(
        parts,
        "inputData metadata",
        root.input_data.iter().map(summarize_input_data),
    );
    push_metadata_summary(
        parts,
        "knowledgeSource metadata",
        root.knowledge_sources
            .iter()
            .map(summarize_knowledge_source),
    );
    push_metadata_summary(
        parts,
        "businessKnowledgeModel metadata",
        root.business_knowledge_models
            .iter()
            .map(summarize_business_knowledge_model),
    );
    push_metadata_summary(
        parts,
        "decisionService metadata",
        root.decision_services
            .iter()
            .map(summarize_decision_service),
    );
    push_metadata_summary(
        parts,
        "organizationUnit metadata",
        root.organization_units
            .iter()
            .map(summarize_organization_unit),
    );
    push_metadata_summary(
        parts,
        "performanceIndicator metadata",
        root.performance_indicators
            .iter()
            .map(summarize_performance_indicator),
    );
    push_metadata_summary(
        parts,
        "textAnnotation metadata",
        root.text_annotations.iter().map(summarize_text_annotation),
    );
    push_metadata_summary(
        parts,
        "association metadata",
        root.associations.iter().map(summarize_association),
    );
    push_metadata_summary(
        parts,
        "elementCollection metadata",
        root.element_collections
            .iter()
            .map(summarize_element_collection),
    );
    push_metadata_summary(
        parts,
        "group metadata",
        root.groups.iter().map(summarize_group),
    );
    push_metadata_summary(
        parts,
        "DMNDI metadata",
        root.dmndi_blocks.iter().map(summarize_dmndi),
    );
}

fn push_metadata_summary<I>(parts: &mut Vec<String>, label: &str, summaries: I)
where
    I: Iterator<Item = String>,
{
    let metadata = summaries.take(3).collect::<Vec<_>>().join("; ");
    if !metadata.is_empty() {
        parts.push(format!("{label} [{metadata}]"));
    }
}

fn summarize_item_definition(item_definition: &DmnItemDefinitionSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(item_definition_id) = item_definition.item_definition_id.as_deref() {
        parts.push(format!("id '{item_definition_id}'"));
    }
    if let Some(name) = item_definition.name.as_deref() {
        parts.push(format!("name '{name}'"));
    }
    if let Some(type_ref) = item_definition.type_ref.as_deref() {
        parts.push(format!("typeRef '{type_ref}'"));
    }
    if let Some(is_collection) = item_definition.is_collection {
        parts.push(format!("isCollection {is_collection}"));
    }
    if !item_definition.item_components.is_empty() {
        parts.push(format!(
            "{} direct itemComponent(s)",
            item_definition.item_components.len()
        ));
    }
    if parts.is_empty() {
        "<itemDefinition>".to_string()
    } else {
        format!("<itemDefinition> with {}", parts.join(", "))
    }
}

fn summarize_input_data(input_data: &DmnInputDataSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(input_data_id) = input_data.input_data_id.as_deref() {
        parts.push(format!("id '{input_data_id}'"));
    }
    if let Some(name) = input_data.name.as_deref() {
        parts.push(format!("name '{name}'"));
    }
    if let Some(variable) = input_data.variable.as_ref() {
        let mut variable_parts = Vec::new();
        if let Some(variable_id) = variable.variable_id.as_deref() {
            variable_parts.push(format!("id '{variable_id}'"));
        }
        if let Some(name) = variable.name.as_deref() {
            variable_parts.push(format!("name '{name}'"));
        }
        if let Some(type_ref) = variable.type_ref.as_deref() {
            variable_parts.push(format!("typeRef '{type_ref}'"));
        }
        if variable_parts.is_empty() {
            parts.push("direct variable".to_string());
        } else {
            parts.push(format!(
                "direct variable with {}",
                variable_parts.join(", ")
            ));
        }
    }
    if parts.is_empty() {
        "<inputData>".to_string()
    } else {
        format!("<inputData> with {}", parts.join(", "))
    }
}

fn summarize_knowledge_source(knowledge_source: &DmnKnowledgeSourceSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(knowledge_source_id) = knowledge_source.knowledge_source_id.as_deref() {
        parts.push(format!("id '{knowledge_source_id}'"));
    }
    if let Some(name) = knowledge_source.name.as_deref() {
        parts.push(format!("name '{name}'"));
    }
    if parts.is_empty() {
        "<knowledgeSource>".to_string()
    } else {
        format!("<knowledgeSource> with {}", parts.join(", "))
    }
}

fn summarize_business_knowledge_model(
    business_knowledge_model: &DmnBusinessKnowledgeModelSnapshot,
) -> String {
    let mut parts = Vec::new();
    if let Some(business_knowledge_model_id) = business_knowledge_model
        .business_knowledge_model_id
        .as_deref()
    {
        parts.push(format!("id '{business_knowledge_model_id}'"));
    }
    if let Some(name) = business_knowledge_model.name.as_deref() {
        parts.push(format!("name '{name}'"));
    }
    if parts.is_empty() {
        "<businessKnowledgeModel>".to_string()
    } else {
        format!("<businessKnowledgeModel> with {}", parts.join(", "))
    }
}

fn summarize_decision_service(decision_service: &DmnDecisionServiceSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(decision_service_id) = decision_service.decision_service_id.as_deref() {
        parts.push(format!("id '{decision_service_id}'"));
    }
    if let Some(name) = decision_service.name.as_deref() {
        parts.push(format!("name '{name}'"));
    }
    if parts.is_empty() {
        "<decisionService>".to_string()
    } else {
        format!("<decisionService> with {}", parts.join(", "))
    }
}

fn summarize_organization_unit(organization_unit: &DmnOrganizationUnitSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(organization_unit_id) = organization_unit.organization_unit_id.as_deref() {
        parts.push(format!("id '{organization_unit_id}'"));
    }
    if let Some(name) = organization_unit.name.as_deref() {
        parts.push(format!("name '{name}'"));
    }
    if parts.is_empty() {
        "<organizationUnit>".to_string()
    } else {
        format!("<organizationUnit> with {}", parts.join(", "))
    }
}

fn summarize_performance_indicator(
    performance_indicator: &DmnPerformanceIndicatorSnapshot,
) -> String {
    let mut parts = Vec::new();
    if let Some(performance_indicator_id) =
        performance_indicator.performance_indicator_id.as_deref()
    {
        parts.push(format!("id '{performance_indicator_id}'"));
    }
    if let Some(name) = performance_indicator.name.as_deref() {
        parts.push(format!("name '{name}'"));
    }
    if parts.is_empty() {
        "<performanceIndicator>".to_string()
    } else {
        format!("<performanceIndicator> with {}", parts.join(", "))
    }
}

fn summarize_text_annotation(text_annotation: &DmnTextAnnotationSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(text_annotation_id) = text_annotation.text_annotation_id.as_deref() {
        parts.push(format!("id '{text_annotation_id}'"));
    }
    if let Some(text) = text_annotation.text.as_deref() {
        parts.push(format!("text '{text}'"));
    }
    if parts.is_empty() {
        "<textAnnotation>".to_string()
    } else {
        format!("<textAnnotation> with {}", parts.join(", "))
    }
}

fn summarize_association(association: &DmnAssociationSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(association_id) = association.association_id.as_deref() {
        parts.push(format!("id '{association_id}'"));
    }
    if let Some(association_direction) = association.association_direction.as_deref() {
        parts.push(format!("associationDirection '{association_direction}'"));
    }
    if let Some(source_ref) = association.source_ref.as_deref() {
        parts.push(format!("sourceRef '{source_ref}'"));
    }
    if let Some(target_ref) = association.target_ref.as_deref() {
        parts.push(format!("targetRef '{target_ref}'"));
    }
    if parts.is_empty() {
        "<association>".to_string()
    } else {
        format!("<association> with {}", parts.join(", "))
    }
}

fn summarize_element_collection(element_collection: &DmnElementCollectionSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(element_collection_id) = element_collection.element_collection_id.as_deref() {
        parts.push(format!("id '{element_collection_id}'"));
    }
    if let Some(name) = element_collection.name.as_deref() {
        parts.push(format!("name '{name}'"));
    }
    if parts.is_empty() {
        "<elementCollection>".to_string()
    } else {
        format!("<elementCollection> with {}", parts.join(", "))
    }
}

fn summarize_group(group: &DmnGroupSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(group_id) = group.group_id.as_deref() {
        parts.push(format!("id '{group_id}'"));
    }
    if let Some(name) = group.name.as_deref() {
        parts.push(format!("name '{name}'"));
    }
    if parts.is_empty() {
        "<group>".to_string()
    } else {
        format!("<group> with {}", parts.join(", "))
    }
}

fn summarize_dmndi(dmndi: &DmnDmndiSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(dmndi_id) = dmndi.dmndi_id.as_deref() {
        parts.push(format!("id '{dmndi_id}'"));
    }
    if !dmndi.diagrams.is_empty() {
        let diagrams = dmndi
            .diagrams
            .iter()
            .take(2)
            .map(summarize_dmn_diagram)
            .collect::<Vec<_>>()
            .join("; ");
        parts.push(format!("diagrams [{diagrams}]"));
    }
    if parts.is_empty() {
        "<dmndi:DMNDI>".to_string()
    } else {
        format!("<dmndi:DMNDI> with {}", parts.join(", "))
    }
}

fn summarize_dmn_diagram(diagram: &DmnDiagramSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(diagram_id) = diagram.diagram_id.as_deref() {
        parts.push(format!("id '{diagram_id}'"));
    }
    if diagram.shape_count > 0 {
        let shapes = diagram
            .shapes
            .iter()
            .take(2)
            .map(summarize_dmn_shape)
            .collect::<Vec<_>>()
            .join("; ");
        if shapes.is_empty() {
            parts.push(format!("{} direct DMNShape(s)", diagram.shape_count));
        } else {
            parts.push(format!(
                "{} direct DMNShape(s) [{shapes}]",
                diagram.shape_count
            ));
        }
    }
    if diagram.edge_count > 0 {
        let edges = diagram
            .edges
            .iter()
            .take(2)
            .map(summarize_dmn_edge)
            .collect::<Vec<_>>()
            .join("; ");
        if edges.is_empty() {
            parts.push(format!("{} direct DMNEdge(s)", diagram.edge_count));
        } else {
            parts.push(format!(
                "{} direct DMNEdge(s) [{edges}]",
                diagram.edge_count
            ));
        }
    }
    if parts.is_empty() {
        "<DMNDiagram>".to_string()
    } else {
        format!("<DMNDiagram> with {}", parts.join(", "))
    }
}

fn summarize_dmn_shape(shape: &crate::dmn_model_api::DmnShapeSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(shape_id) = shape.shape_id.as_deref() {
        parts.push(format!("id '{shape_id}'"));
    }
    if let Some(dmn_element_ref) = shape.dmn_element_ref.as_deref() {
        parts.push(format!("dmnElementRef '{dmn_element_ref}'"));
    }
    if let Some(is_listed_input_data) = shape.is_listed_input_data {
        parts.push(format!("isListedInputData {is_listed_input_data}"));
    }
    if let Some(is_collapsed) = shape.is_collapsed {
        parts.push(format!("isCollapsed {is_collapsed}"));
    }
    if let Some(bounds) = shape.bounds.as_ref() {
        parts.push(summarize_dmn_bounds(bounds));
    }
    if let Some(divider_line) = shape.decision_service_divider_line.as_ref() {
        parts.push(summarize_dmn_decision_service_divider_line(divider_line));
    }
    if let Some(label) = shape.label.as_ref() {
        parts.push(summarize_dmn_label(label));
    }
    if parts.is_empty() {
        "<DMNShape>".to_string()
    } else {
        format!("<DMNShape> with {}", parts.join(", "))
    }
}

fn summarize_dmn_edge(edge: &crate::dmn_model_api::DmnEdgeSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(edge_id) = edge.edge_id.as_deref() {
        parts.push(format!("id '{edge_id}'"));
    }
    if let Some(dmn_element_ref) = edge.dmn_element_ref.as_deref() {
        parts.push(format!("dmnElementRef '{dmn_element_ref}'"));
    }
    if !edge.waypoints.is_empty() {
        let waypoints = edge
            .waypoints
            .iter()
            .take(2)
            .map(summarize_dmn_waypoint)
            .collect::<Vec<_>>()
            .join("; ");
        parts.push(format!(
            "{} waypoint(s) [{waypoints}]",
            edge.waypoints.len()
        ));
    }
    if let Some(label) = edge.label.as_ref() {
        parts.push(summarize_dmn_label(label));
    }
    if parts.is_empty() {
        "<DMNEdge>".to_string()
    } else {
        format!("<DMNEdge> with {}", parts.join(", "))
    }
}

fn summarize_dmn_label(label: &crate::dmn_model_api::DmnLabelSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(label_id) = label.label_id.as_deref() {
        parts.push(format!("id '{label_id}'"));
    }
    if let Some(bounds) = label.bounds.as_ref() {
        parts.push(summarize_dmn_bounds(bounds));
    }
    if let Some(text) = label.text.as_deref() {
        parts.push(format!("text '{}'", text.replace('\n', "\\n")));
    }
    if parts.is_empty() {
        "DMNLabel".to_string()
    } else {
        format!("DMNLabel {}", parts.join(", "))
    }
}

fn summarize_dmn_bounds(bounds: &crate::dmn_model_api::DmnBoundsSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(x) = bounds.x.as_deref() {
        parts.push(format!("x '{x}'"));
    }
    if let Some(y) = bounds.y.as_deref() {
        parts.push(format!("y '{y}'"));
    }
    if let Some(width) = bounds.width.as_deref() {
        parts.push(format!("width '{width}'"));
    }
    if let Some(height) = bounds.height.as_deref() {
        parts.push(format!("height '{height}'"));
    }
    if parts.is_empty() {
        "dc:Bounds".to_string()
    } else {
        format!("dc:Bounds {}", parts.join(", "))
    }
}

fn summarize_dmn_waypoint(waypoint: &crate::dmn_model_api::DmnWaypointSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(x) = waypoint.x.as_deref() {
        parts.push(format!("x '{x}'"));
    }
    if let Some(y) = waypoint.y.as_deref() {
        parts.push(format!("y '{y}'"));
    }
    if parts.is_empty() {
        "di:waypoint".to_string()
    } else {
        format!("di:waypoint {}", parts.join(", "))
    }
}

fn summarize_dmn_decision_service_divider_line(
    divider_line: &crate::dmn_model_api::DmnDecisionServiceDividerLineSnapshot,
) -> String {
    if divider_line.waypoints.is_empty() {
        return "DMNDecisionServiceDividerLine".to_string();
    }
    let waypoints = divider_line
        .waypoints
        .iter()
        .take(2)
        .map(summarize_dmn_waypoint)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "DMNDecisionServiceDividerLine {} waypoint(s) [{waypoints}]",
        divider_line.waypoints.len()
    )
}

pub(super) fn decision_display(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> String {
    if let Some(decision_name) = snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .and_then(|decision| decision.name.as_deref())
    {
        format!("Decision '{decision_id}' ('{decision_name}')")
    } else {
        format!("Decision '{decision_id}'")
    }
}
