use std::fmt::{self, Write as _};

use super::api::{
    WorkflowPlan, WorkflowPlanEdge, WorkflowPlanEmitError, WorkflowPlanTask, escape_xml_attr,
    escape_xml_text,
};
use super::validation::validate_workflow_plan;

/// Emit a validated `WorkflowPlan` as deterministic BPMN XML.
///
/// # Errors
///
/// Returns validation diagnostics when the plan is outside the supported
/// construct subset.
pub(crate) fn emit_workflow_plan_bpmn(
    plan: &WorkflowPlan,
) -> Result<String, WorkflowPlanEmitError> {
    let validation = validate_workflow_plan(plan);
    if !validation.ok {
        return Err(WorkflowPlanEmitError { validation });
    }

    let process_id = stable_xml_id("Process", &plan.name);
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<definitions xmlns=\"http://www.omg.org/spec/BPMN/20100524/MODEL\"\n");
    xml.push_str("             xmlns:bpmndi=\"http://www.omg.org/spec/BPMN/20100524/DI\"\n");
    xml.push_str("             xmlns:dc=\"http://www.omg.org/spec/DD/20100524/DC\"\n");
    xml.push_str("             xmlns:di=\"http://www.omg.org/spec/DD/20100524/DI\"\n");
    xml.push_str("             xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"\n");
    xml.push_str("             id=\"Definitions_1\"\n");
    xml.push_str("             targetNamespace=\"https://qianji.dev\">\n");
    push_xml(
        &mut xml,
        format_args!(
            "  <process id=\"{}\" name=\"{}\" isExecutable=\"true\">\n",
            process_id,
            escape_xml_attr(&plan.name)
        ),
    );
    xml.push_str("    <startEvent id=\"Start_1\" name=\"Start\"/>\n");

    for task in &plan.tasks {
        push_task_xml(&mut xml, task);
    }
    let gateway_sources = conditional_gateway_sources(plan);
    for gateway in &gateway_sources {
        push_xml(
            &mut xml,
            format_args!(
                "    <exclusiveGateway id=\"{}\" name=\"Route {}\"{} />\n",
                gateway_id(gateway),
                escape_xml_attr(gateway),
                default_flow_for_source(plan, gateway, gateway_sources.len())
                    .map(|flow_id| format!(" default=\"{flow_id}\""))
                    .unwrap_or_default()
            ),
        );
    }

    xml.push_str("    <endEvent id=\"End_1\" name=\"End\"/>\n");
    for (index, source) in gateway_sources.iter().enumerate() {
        push_sequence_flow_xml(
            &mut xml,
            &flow_id(index),
            &node_ref(source),
            &gateway_id(source),
            None,
        );
    }
    for (index, edge) in plan.edges.iter().enumerate() {
        let flow_index = gateway_sources.len() + index;
        push_edge_xml(&mut xml, &gateway_sources, edge, flow_index);
    }
    xml.push_str("  </process>\n");
    push_bpmn_di_xml(&mut xml, plan, &process_id, &gateway_sources);
    xml.push_str("</definitions>");
    Ok(xml)
}

fn push_task_xml(xml: &mut String, task: &WorkflowPlanTask) {
    let element = match task.construct.as_str() {
        "user-task.interaction" => "userTask",
        _ => "serviceTask",
    };
    let implementation = if element == "serviceTask" {
        " implementation=\"${environment.services.runAgent}\""
    } else {
        ""
    };
    push_xml(
        xml,
        format_args!(
            "    <{element} id=\"{}\" name=\"{}\"{implementation}>\n",
            escape_xml_attr(&task.id),
            escape_xml_attr(&task.id)
        ),
    );
    push_xml(
        xml,
        format_args!(
            "      <documentation>{}</documentation>\n",
            escape_xml_text(&format!("Execute WorkflowPlan task {}.", task.id))
        ),
    );
    push_task_io_xml(xml, task, element);
    push_xml(xml, format_args!("    </{element}>\n"));
}

fn push_task_io_xml(xml: &mut String, task: &WorkflowPlanTask, element: &str) {
    let answer_output = task.outputs.first().map_or("answer", String::as_str);
    if task.inputs.is_empty() && task.outputs.is_empty() && element != "userTask" {
        return;
    }
    xml.push_str("      <ioSpecification>\n");
    push_task_io_declarations(xml, task, element);
    push_task_io_sets(xml, task, element);
    xml.push_str("      </ioSpecification>\n");
    if element == "userTask" {
        push_user_task_io_associations(xml, task, answer_output);
    }
}

fn push_task_io_declarations(xml: &mut String, task: &WorkflowPlanTask, element: &str) {
    if element == "userTask" {
        push_xml(
            xml,
            format_args!(
                "        <dataInput id=\"{}_interaction_type\" name=\"interactionType\"/>\n",
                stable_xml_id("Input", &task.id)
            ),
        );
        push_xml(
            xml,
            format_args!(
                "        <dataOutput id=\"{}_answer\" name=\"answer\"/>\n",
                stable_xml_id("Output", &task.id)
            ),
        );
    }
    for input in &task.inputs {
        push_xml(
            xml,
            format_args!(
                "        <dataInput id=\"{}\" name=\"{}\"/>\n",
                stable_xml_id("Input", &format!("{}_{}", task.id, input)),
                escape_xml_attr(input)
            ),
        );
    }
    if element != "userTask" {
        for output in &task.outputs {
            push_xml(
                xml,
                format_args!(
                    "        <dataOutput id=\"{}\" name=\"{}\"/>\n",
                    stable_xml_id("Output", &format!("{}_{}", task.id, output)),
                    escape_xml_attr(output)
                ),
            );
        }
    }
}

fn push_task_io_sets(xml: &mut String, task: &WorkflowPlanTask, element: &str) {
    xml.push_str("        <inputSet>\n");
    if element == "userTask" {
        push_xml(
            xml,
            format_args!(
                "          <dataInputRefs>{}_interaction_type</dataInputRefs>\n",
                stable_xml_id("Input", &task.id)
            ),
        );
    }
    for input in &task.inputs {
        push_xml(
            xml,
            format_args!(
                "          <dataInputRefs>{}</dataInputRefs>\n",
                stable_xml_id("Input", &format!("{}_{}", task.id, input))
            ),
        );
    }
    xml.push_str("        </inputSet>\n");
    xml.push_str("        <outputSet>\n");
    if element == "userTask" {
        push_xml(
            xml,
            format_args!(
                "          <dataOutputRefs>{}_answer</dataOutputRefs>\n",
                stable_xml_id("Output", &task.id)
            ),
        );
    }
    if element != "userTask" {
        for output in &task.outputs {
            push_xml(
                xml,
                format_args!(
                    "          <dataOutputRefs>{}</dataOutputRefs>\n",
                    stable_xml_id("Output", &format!("{}_{}", task.id, output))
                ),
            );
        }
    }
    xml.push_str("        </outputSet>\n");
}

fn push_user_task_io_associations(xml: &mut String, task: &WorkflowPlanTask, answer_output: &str) {
    push_xml(
        xml,
        format_args!(
            "      <dataInputAssociation><targetRef>{}_interaction_type</targetRef><assignment><from>input</from><to>{}_interaction_type</to></assignment></dataInputAssociation>\n",
            stable_xml_id("Input", &task.id),
            stable_xml_id("Input", &task.id)
        ),
    );
    push_xml(
        xml,
        format_args!(
            "      <dataOutputAssociation><sourceRef>{}_answer</sourceRef><targetRef>{}</targetRef></dataOutputAssociation>\n",
            stable_xml_id("Output", &task.id),
            escape_xml_text(answer_output)
        ),
    );
}

fn push_edge_xml(
    xml: &mut String,
    gateway_sources: &[&str],
    edge: &WorkflowPlanEdge,
    index: usize,
) {
    let flow_id = flow_id(index);
    let source_ref = if gateway_sources.contains(&edge.from.as_str()) {
        gateway_id(&edge.from)
    } else {
        node_ref(&edge.from)
    };
    let target_ref = node_ref(&edge.to);
    push_sequence_flow_xml(
        xml,
        &flow_id,
        &source_ref,
        &target_ref,
        edge.condition.as_deref(),
    );
}

fn push_sequence_flow_xml(
    xml: &mut String,
    flow_id: &str,
    source_ref: &str,
    target_ref: &str,
    condition: Option<&str>,
) {
    if let Some(condition) = condition {
        push_xml(
            xml,
            format_args!(
                "    <sequenceFlow id=\"{flow_id}\" sourceRef=\"{source_ref}\" targetRef=\"{target_ref}\">\n"
            ),
        );
        push_xml(
            xml,
            format_args!(
                "      <conditionExpression xsi:type=\"tFormalExpression\">{}</conditionExpression>\n",
                escape_xml_text(condition)
            ),
        );
        xml.push_str("    </sequenceFlow>\n");
    } else {
        push_xml(
            xml,
            format_args!(
                "    <sequenceFlow id=\"{flow_id}\" sourceRef=\"{source_ref}\" targetRef=\"{target_ref}\"/>\n"
            ),
        );
    }
}

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

fn push_bpmn_di_xml(
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

fn push_xml(xml: &mut String, args: fmt::Arguments<'_>) {
    let _ = xml.write_fmt(args);
}

fn conditional_gateway_sources(plan: &WorkflowPlan) -> Vec<&str> {
    let mut sources = Vec::new();
    for edge in &plan.edges {
        if edge.condition.is_some() && !sources.contains(&edge.from.as_str()) {
            sources.push(edge.from.as_str());
        }
    }
    sources
}

fn default_flow_for_source(
    plan: &WorkflowPlan,
    source: &str,
    gateway_source_count: usize,
) -> Option<String> {
    plan.edges
        .iter()
        .position(|edge| edge.from == source && edge.default)
        .map(|edge_index| flow_id(gateway_source_count + edge_index))
}

fn flow_id(index: usize) -> String {
    format!("Flow_{}", index + 1)
}

fn gateway_id(source: &str) -> String {
    stable_xml_id("Gateway", source)
}

fn node_ref(node: &str) -> String {
    match node {
        "start" => "Start_1".to_string(),
        "end" => "End_1".to_string(),
        other => other.to_string(),
    }
}

fn stable_xml_id(prefix: &str, value: &str) -> String {
    let mut output = String::with_capacity(prefix.len() + value.len() + 1);
    output.push_str(prefix);
    output.push('_');
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            output.push(ch);
        } else {
            output.push('_');
        }
    }
    if output.ends_with('_') {
        output.push('1');
    }
    output
}
