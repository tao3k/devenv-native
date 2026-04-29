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
    let answer_output = task.outputs.first().map(String::as_str).unwrap_or("answer");
    if task.inputs.is_empty() && task.outputs.is_empty() && element != "userTask" {
        return;
    }
    xml.push_str("      <ioSpecification>\n");
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
    xml.push_str("      </ioSpecification>\n");
    if element == "userTask" {
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
