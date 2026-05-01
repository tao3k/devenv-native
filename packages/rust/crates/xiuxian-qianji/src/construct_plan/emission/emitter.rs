use super::diagram::push_bpmn_di_xml;
use super::gateways::{conditional_gateway_sources, default_flow_for_source};
use super::ids::{flow_id, gateway_id, node_ref, stable_xml_id};
use super::sequence::{push_edge_xml, push_sequence_flow_xml};
use super::task::push_task_xml;
use super::xml::push_xml;
use crate::construct_plan::api::{WorkflowPlan, WorkflowPlanEmitError, escape_xml_attr};
use crate::construct_plan::validation::validate_workflow_plan;

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
    push_gateway_xml(&mut xml, plan, &gateway_sources);

    xml.push_str("    <endEvent id=\"End_1\" name=\"End\"/>\n");
    push_process_flow_xml(&mut xml, plan, &gateway_sources);
    xml.push_str("  </process>\n");
    push_bpmn_di_xml(&mut xml, plan, &process_id, &gateway_sources);
    xml.push_str("</definitions>");
    Ok(xml)
}

fn push_gateway_xml(xml: &mut String, plan: &WorkflowPlan, gateway_sources: &[&str]) {
    for gateway in gateway_sources {
        push_xml(
            xml,
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
}

fn push_process_flow_xml(xml: &mut String, plan: &WorkflowPlan, gateway_sources: &[&str]) {
    for (index, source) in gateway_sources.iter().enumerate() {
        push_sequence_flow_xml(
            xml,
            &flow_id(index),
            &node_ref(source),
            &gateway_id(source),
            None,
        );
    }
    for (index, edge) in plan.edges.iter().enumerate() {
        let flow_index = gateway_sources.len() + index;
        push_edge_xml(xml, gateway_sources, edge, flow_index);
    }
}
