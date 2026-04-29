//! Advanced BPMN 2.0 XML generation with full waypoint support for precision routing.

use std::fmt::Write as _;

use super::engine_types::{BpmnType, EdgeLayout, LayoutResult, NodePosition};

fn push_fmt(xml: &mut String, args: std::fmt::Arguments<'_>) {
    // Writing into `String` cannot fail.
    if xml.write_fmt(args).is_err() {
        unreachable!("writing into String cannot fail");
    }
}

fn bpmn_tag(node_type: &BpmnType) -> &'static str {
    match node_type {
        BpmnType::StartEvent => "bpmn:startEvent",
        BpmnType::EndEvent => "bpmn:endEvent",
        BpmnType::Task => "bpmn:task",
        BpmnType::ServiceTask => "bpmn:serviceTask",
        BpmnType::BusinessRule => "bpmn:businessRuleTask",
        BpmnType::ExclusiveGateway => "bpmn:exclusiveGateway",
    }
}

fn push_header(xml: &mut String) {
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(
        "<bpmn:definitions xmlns:bpmn=\"http://www.omg.org/spec/BPMN/20100524/MODEL\" \\
                  xmlns:bpmndi=\"http://www.omg.org/spec/BPMN/20100524/DI\" \\
                  xmlns:dc=\"http://www.omg.org/spec/DD/20100524/DC\" \\
                  xmlns:di=\"http://www.omg.org/spec/DD/20100524/DI\" \\
                  targetNamespace=\"http://bpmn.io/schema/bpmn\">\n",
    );
}

fn push_process_nodes(xml: &mut String, nodes: &[NodePosition]) {
    for node in nodes {
        let tag = bpmn_tag(&node.bpmn_type);
        if let Some(uri) = node.context_uri.as_ref() {
            push_fmt(
                xml,
                format_args!(
                    "    <{tag} id=\"{}\" name=\"{}\">\n      <bpmn:documentation>context_uri: {}</bpmn:documentation>\n    </{tag}>\n",
                    node.id,
                    node.label,
                    escape_xml_text(uri)
                ),
            );
        } else {
            push_fmt(
                xml,
                format_args!(
                    "    <{} id=\"{}\" name=\"{}\" />\n",
                    tag, node.id, node.label
                ),
            );
        }
    }
}

fn escape_xml_text(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn push_process_edges(xml: &mut String, edges: &[EdgeLayout]) {
    for edge in edges {
        let label_attr = edge
            .label
            .as_ref()
            .map(|label| format!(" name=\"{label}\""))
            .unwrap_or_default();
        push_fmt(
            xml,
            format_args!(
                "    <bpmn:sequenceFlow id=\"{}\" sourceRef=\"{}\" targetRef=\"{}\"{label_attr} />\n",
                edge.id, edge.from, edge.to
            ),
        );
    }
}

fn push_di_nodes(xml: &mut String, nodes: &[NodePosition]) {
    for node in nodes {
        push_fmt(
            xml,
            format_args!(
                "      <bpmndi:BPMNShape id=\"{}_di\" bpmnElement=\"{}\" isExpanded=\"true\">\n",
                node.id, node.id
            ),
        );
        push_fmt(
            xml,
            format_args!(
                "        <dc:Bounds x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />\n",
                node.x, node.y, node.width, node.height
            ),
        );
        xml.push_str("      </bpmndi:BPMNShape>\n");
    }
}

fn push_edge_label(xml: &mut String, edge: &EdgeLayout) {
    let midpoint = edge.waypoints.get(edge.waypoints.len() / 2).copied();
    if let Some((lx, ly)) = midpoint {
        push_fmt(
            xml,
            format_args!(
                "        <bpmndi:BPMNLabel>\n          <dc:Bounds x=\"{}\" y=\"{}\" width=\"100\" height=\"14\" />\n        </bpmndi:BPMNLabel>\n",
                lx - 50.0,
                ly - 20.0
            ),
        );
    }
}

fn push_di_edges(xml: &mut String, edges: &[EdgeLayout]) {
    for edge in edges {
        push_fmt(
            xml,
            format_args!(
                "      <bpmndi:BPMNEdge id=\"{}_di\" bpmnElement=\"{}\">\n",
                edge.id, edge.id
            ),
        );
        for (wx, wy) in &edge.waypoints {
            push_fmt(
                xml,
                format_args!("        <di:waypoint x=\"{wx}\" y=\"{wy}\" />\n"),
            );
        }
        if edge.label.is_some() {
            push_edge_label(xml, edge);
        }
        xml.push_str("      </bpmndi:BPMNEdge>\n");
    }
}

/// Serializes a computed layout into BPMN 2.0 XML.
#[must_use]
pub fn generate_bpmn_xml(layout: &LayoutResult) -> String {
    let mut xml = String::new();
    push_header(&mut xml);

    xml.push_str("  <bpmn:process id=\"Sovereign_Process\" isExecutable=\"false\">\n");
    push_process_nodes(&mut xml, &layout.nodes);
    push_process_edges(&mut xml, &layout.edges);
    xml.push_str("  </bpmn:process>\n");

    xml.push_str("  <bpmndi:BPMNDiagram id=\"Sovereign_Diagram\">\n");
    xml.push_str(
        "    <bpmndi:BPMNPlane id=\"Sovereign_Plane\" bpmnElement=\"Sovereign_Process\">\n",
    );
    push_di_nodes(&mut xml, &layout.nodes);
    push_di_edges(&mut xml, &layout.edges);
    xml.push_str("    </bpmndi:BPMNPlane>\n");
    xml.push_str("  </bpmndi:BPMNDiagram>\n");
    xml.push_str("</bpmn:definitions>");

    xml
}
