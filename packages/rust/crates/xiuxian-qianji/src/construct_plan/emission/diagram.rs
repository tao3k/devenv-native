use super::ids::{flow_id, gateway_id, node_ref, stable_xml_id};
use super::xml::push_xml;
use crate::construct_plan::api::{WorkflowPlan, escape_xml_attr};

#[derive(Debug, Clone)]
struct DiNode {
    bpmn_id: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl DiNode {
    fn event(bpmn_id: impl Into<String>, x: i32) -> Self {
        Self {
            bpmn_id: bpmn_id.into(),
            x,
            y: 122,
            width: 36,
            height: 36,
        }
    }

    fn task(bpmn_id: impl Into<String>, x: i32) -> Self {
        Self {
            bpmn_id: bpmn_id.into(),
            x,
            y: 100,
            width: 160,
            height: 80,
        }
    }

    fn gateway(bpmn_id: impl Into<String>, x: i32) -> Self {
        Self {
            bpmn_id: bpmn_id.into(),
            x,
            y: 115,
            width: 50,
            height: 50,
        }
    }

    fn right_center(&self) -> (i32, i32) {
        (self.x + self.width, self.y + self.height / 2)
    }

    fn left_center(&self) -> (i32, i32) {
        (self.x, self.y + self.height / 2)
    }
}

struct DiFlow {
    flow_id: String,
    source_ref: String,
    target_ref: String,
}

pub(super) fn push_bpmn_di_xml(
    xml: &mut String,
    plan: &WorkflowPlan,
    process_id: &str,
    gateway_sources: &[&str],
) {
    let nodes = layout_di_nodes(plan, gateway_sources);
    let flows = layout_di_flows(plan, gateway_sources);
    push_xml(
        xml,
        format_args!("  <bpmndi:BPMNDiagram id=\"BPMNDiagram_{process_id}\">\n"),
    );
    push_xml(
        xml,
        format_args!(
            "    <bpmndi:BPMNPlane id=\"BPMNPlane_{process_id}\" bpmnElement=\"{process_id}\">\n"
        ),
    );
    for node in &nodes {
        push_di_shape_xml(xml, node);
    }
    for flow in &flows {
        push_di_edge_xml(xml, flow, &nodes);
    }
    xml.push_str("    </bpmndi:BPMNPlane>\n");
    xml.push_str("  </bpmndi:BPMNDiagram>\n");
}

fn layout_di_nodes(plan: &WorkflowPlan, gateway_sources: &[&str]) -> Vec<DiNode> {
    let mut nodes = Vec::with_capacity(plan.tasks.len() + gateway_sources.len() + 2);
    let mut next_x = 80;
    push_di_node(&mut nodes, DiNode::event("Start_1", next_x), &mut next_x);
    if gateway_sources.contains(&"start") {
        push_di_node(
            &mut nodes,
            DiNode::gateway(gateway_id("start"), next_x),
            &mut next_x,
        );
    }
    for task in &plan.tasks {
        push_di_node(
            &mut nodes,
            DiNode::task(task.id.clone(), next_x),
            &mut next_x,
        );
        if gateway_sources.contains(&task.id.as_str()) {
            push_di_node(
                &mut nodes,
                DiNode::gateway(gateway_id(&task.id), next_x),
                &mut next_x,
            );
        }
    }
    push_di_node(&mut nodes, DiNode::event("End_1", next_x), &mut next_x);
    nodes
}

fn push_di_node(nodes: &mut Vec<DiNode>, node: DiNode, next_x: &mut i32) {
    *next_x = node.x + node.width + 120;
    nodes.push(node);
}

fn layout_di_flows(plan: &WorkflowPlan, gateway_sources: &[&str]) -> Vec<DiFlow> {
    let mut flows = Vec::with_capacity(plan.edges.len() + gateway_sources.len());
    for (index, source) in gateway_sources.iter().enumerate() {
        flows.push(DiFlow {
            flow_id: flow_id(index),
            source_ref: node_ref(source),
            target_ref: gateway_id(source),
        });
    }
    for (index, edge) in plan.edges.iter().enumerate() {
        let flow_index = gateway_sources.len() + index;
        let source_ref = if gateway_sources.contains(&edge.from.as_str()) {
            gateway_id(&edge.from)
        } else {
            node_ref(&edge.from)
        };
        flows.push(DiFlow {
            flow_id: flow_id(flow_index),
            source_ref,
            target_ref: node_ref(&edge.to),
        });
    }
    flows
}

fn push_di_shape_xml(xml: &mut String, node: &DiNode) {
    push_xml(
        xml,
        format_args!(
            "      <bpmndi:BPMNShape id=\"{}\" bpmnElement=\"{}\">\n",
            stable_xml_id("Shape", &node.bpmn_id),
            escape_xml_attr(&node.bpmn_id)
        ),
    );
    push_xml(
        xml,
        format_args!(
            "        <dc:Bounds x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>\n",
            node.x, node.y, node.width, node.height
        ),
    );
    xml.push_str("      </bpmndi:BPMNShape>\n");
}

fn push_di_edge_xml(xml: &mut String, flow: &DiFlow, nodes: &[DiNode]) {
    let Some(source) = find_di_node(nodes, &flow.source_ref) else {
        return;
    };
    let Some(target) = find_di_node(nodes, &flow.target_ref) else {
        return;
    };
    let (source_x, source_y) = source.right_center();
    let (target_x, target_y) = target.left_center();
    push_xml(
        xml,
        format_args!(
            "      <bpmndi:BPMNEdge id=\"{}\" bpmnElement=\"{}\">\n",
            stable_xml_id("Edge", &flow.flow_id),
            escape_xml_attr(&flow.flow_id)
        ),
    );
    push_xml(
        xml,
        format_args!("        <di:waypoint x=\"{source_x}\" y=\"{source_y}\"/>\n"),
    );
    push_xml(
        xml,
        format_args!("        <di:waypoint x=\"{target_x}\" y=\"{target_y}\"/>\n"),
    );
    xml.push_str("      </bpmndi:BPMNEdge>\n");
}

fn find_di_node<'a>(nodes: &'a [DiNode], bpmn_id: &str) -> Option<&'a DiNode> {
    nodes.iter().find(|node| node.bpmn_id == bpmn_id)
}
