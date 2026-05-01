use super::ids::stable_xml_id;
use super::xml::push_xml;
use crate::construct_plan::api::{WorkflowPlanTask, escape_xml_attr, escape_xml_text};

pub(super) fn push_task_xml(xml: &mut String, task: &WorkflowPlanTask) {
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
        push_user_task_default_io(xml, task);
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
        push_service_task_output_declarations(xml, task);
    }
}

fn push_user_task_default_io(xml: &mut String, task: &WorkflowPlanTask) {
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

fn push_service_task_output_declarations(xml: &mut String, task: &WorkflowPlanTask) {
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
    push_output_set(xml, task, element);
}

fn push_output_set(xml: &mut String, task: &WorkflowPlanTask, element: &str) {
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
