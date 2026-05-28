use super::compile::{xml_attr, xml_id, xml_text};
use super::markdown::WorkflowTask;
use std::fmt::Write as _;

pub(super) fn render_workflow_bpmn_xml(
    source_id: &str,
    process_id: &str,
    workflow_name: &str,
    tasks: &[WorkflowTask],
) -> String {
    let (flows, bounds) = workflow_layout(tasks);
    let process_body = render_process_body(tasks, &flows);
    let diagram_body = render_diagram_body(&bounds, &flows);
    render_bpmn_xml(
        source_id,
        process_id,
        workflow_name,
        process_body.as_str(),
        diagram_body.as_str(),
    )
}

fn workflow_layout(tasks: &[WorkflowTask]) -> (Vec<WorkflowFlow>, Vec<WorkflowBounds>) {
    let start_id = "StartEvent_1";
    let end_id = "EndEvent_1";
    let node_ids = std::iter::once(start_id.to_owned())
        .chain(tasks.iter().map(|task| task.id.clone()))
        .chain(std::iter::once(end_id.to_owned()))
        .collect::<Vec<_>>();
    let flows = node_ids
        .windows(2)
        .enumerate()
        .map(|(index, pair)| WorkflowFlow {
            id: format!("Flow_{}", index + 1),
            source_id: pair[0].clone(),
            target_id: pair[1].clone(),
        })
        .collect::<Vec<_>>();
    let bounds = node_ids
        .iter()
        .enumerate()
        .map(|(index, id)| WorkflowBounds {
            id: id.clone(),
            x: 120 + index * 210,
            y: if id == start_id || id == end_id {
                186
            } else {
                160
            },
            width: if id == start_id || id == end_id {
                48
            } else {
                150
            },
            height: if id == start_id || id == end_id {
                48
            } else {
                100
            },
        })
        .collect::<Vec<_>>();
    (flows, bounds)
}

fn render_process_body(tasks: &[WorkflowTask], flows: &[WorkflowFlow]) -> String {
    let mut process_body = String::new();
    let _ = write!(
        process_body,
        "    <bpmn:startEvent id=\"StartEvent_1\" name=\"Start\">\n      <bpmn:outgoing>{}</bpmn:outgoing>\n    </bpmn:startEvent>\n",
        xml_text(flows.first().map_or("", |flow| flow.id.as_str()))
    );
    for (index, task) in tasks.iter().enumerate() {
        render_service_task(&mut process_body, task, flows, index);
    }
    let _ = write!(
        process_body,
        "    <bpmn:endEvent id=\"EndEvent_1\" name=\"Done\">\n      <bpmn:incoming>{}</bpmn:incoming>\n    </bpmn:endEvent>\n",
        xml_text(flows.last().map_or("", |flow| flow.id.as_str()))
    );
    for flow in flows {
        let _ = writeln!(
            process_body,
            "    <bpmn:sequenceFlow id=\"{}\" sourceRef=\"{}\" targetRef=\"{}\" />",
            xml_attr(&flow.id),
            xml_attr(&flow.source_id),
            xml_attr(&flow.target_id),
        );
    }
    process_body
}

fn render_service_task(
    process_body: &mut String,
    task: &WorkflowTask,
    flows: &[WorkflowFlow],
    index: usize,
) {
    let incoming = flows.get(index).map_or("", |flow| flow.id.as_str());
    let outgoing = flows.get(index + 1).map_or("", |flow| flow.id.as_str());
    let _ = write!(
        process_body,
        "    <bpmn:serviceTask id=\"{}\" name=\"{}\">\n      <bpmn:documentation>{}</bpmn:documentation>\n      <bpmn:incoming>{}</bpmn:incoming>\n      <bpmn:outgoing>{}</bpmn:outgoing>\n      <bpmn:ioSpecification>\n        <bpmn:dataOutput id=\"{}_output_result\" name=\"result\" />\n        <bpmn:inputSet id=\"{}_input_set\" />\n        <bpmn:outputSet id=\"{}_output_set\">\n          <bpmn:dataOutputRefs>{}_output_result</bpmn:dataOutputRefs>\n        </bpmn:outputSet>\n      </bpmn:ioSpecification>\n      <bpmn:dataOutputAssociation>\n        <bpmn:sourceRef>{}_output_result</bpmn:sourceRef>\n        <bpmn:targetRef>{}</bpmn:targetRef>\n      </bpmn:dataOutputAssociation>\n    </bpmn:serviceTask>\n",
        xml_attr(&task.id),
        xml_attr(&task.name),
        xml_text(&task.documentation),
        xml_text(incoming),
        xml_text(outgoing),
        xml_attr(&task.id),
        xml_attr(&task.id),
        xml_attr(&task.id),
        xml_text(&task.id),
        xml_text(&task.id),
        xml_text(&task_result_variable(&task.id)),
    );
}

fn render_diagram_body(bounds: &[WorkflowBounds], flows: &[WorkflowFlow]) -> String {
    let mut diagram_body = String::new();
    for box_bounds in bounds {
        let _ = write!(
            diagram_body,
            "      <bpmndi:BPMNShape id=\"{}_di\" bpmnElement=\"{}\">\n        <dc:Bounds x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />\n      </bpmndi:BPMNShape>\n",
            xml_attr(&box_bounds.id),
            xml_attr(&box_bounds.id),
            box_bounds.x,
            box_bounds.y,
            box_bounds.width,
            box_bounds.height,
        );
    }
    for (index, flow) in flows.iter().enumerate() {
        render_diagram_edge(&mut diagram_body, bounds, flow, index);
    }
    diagram_body
}

fn render_diagram_edge(
    diagram_body: &mut String,
    bounds: &[WorkflowBounds],
    flow: &WorkflowFlow,
    index: usize,
) {
    let source = &bounds[index];
    let target = &bounds[index + 1];
    let source_x = source.x + source.width;
    let source_y = source.y + source.height / 2;
    let target_x = target.x;
    let target_y = target.y + target.height / 2;
    let _ = write!(
        diagram_body,
        "      <bpmndi:BPMNEdge id=\"{}_di\" bpmnElement=\"{}\">\n        <di:waypoint x=\"{source_x}\" y=\"{source_y}\" />\n        <di:waypoint x=\"{target_x}\" y=\"{target_y}\" />\n      </bpmndi:BPMNEdge>\n",
        xml_attr(&flow.id),
        xml_attr(&flow.id),
    );
}

fn render_bpmn_xml(
    source_id: &str,
    process_id: &str,
    workflow_name: &str,
    process_body: &str,
    diagram_body: &str,
) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<bpmn:definitions xmlns:bpmn=\"http://www.omg.org/spec/BPMN/20100524/MODEL\" xmlns:bpmndi=\"http://www.omg.org/spec/BPMN/20100524/DI\" xmlns:dc=\"http://www.omg.org/spec/DD/20100524/DC\" xmlns:di=\"http://www.omg.org/spec/DD/20100524/DI\" id=\"Definitions_{}\" targetNamespace=\"https://wendao.ai/qianji\">\n  <bpmn:process id=\"{}\" name=\"{}\" isExecutable=\"true\">\n{}  </bpmn:process>\n  <bpmndi:BPMNDiagram id=\"Diagram_{}\">\n    <bpmndi:BPMNPlane id=\"Plane_{}\" bpmnElement=\"{}\">\n{}    </bpmndi:BPMNPlane>\n  </bpmndi:BPMNDiagram>\n</bpmn:definitions>\n",
        xml_attr(source_id),
        xml_attr(process_id),
        xml_attr(workflow_name),
        process_body,
        xml_attr(source_id),
        xml_attr(source_id),
        xml_attr(process_id),
        diagram_body,
    )
}

#[derive(Debug, Clone)]
struct WorkflowFlow {
    id: String,
    source_id: String,
    target_id: String,
}

#[derive(Debug, Clone)]
struct WorkflowBounds {
    id: String,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

fn task_result_variable(task_id: &str) -> String {
    xml_id(&format!("{task_id}_result"))
}
