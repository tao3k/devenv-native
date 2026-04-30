use crate::test_support::MustExt as _;

pub(super) use qianji_bpmn_engine::{
    BpmnSourceFile, DmnSourceFile, LintDomain, lint_bpmn_source, lint_dmn_source,
};

mod bpmn_core;
mod bpmn_loops;
mod bpmn_tasks;
mod compensation;
mod dmn;
mod smoke;
mod transaction;

pub(super) fn bpmn_fixture_source(name: &str) -> BpmnSourceFile {
    let path = format!("{}/tests/fixtures/bpmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    BpmnSourceFile::new(name, contents)
}

pub(super) fn dmn_fixture_source(name: &str) -> DmnSourceFile {
    let path = format!("{}/tests/fixtures/dmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    DmnSourceFile::new(name, contents)
}

pub(super) fn assert_lint_json_snapshot(name: &str, value: impl serde::Serialize) {
    insta::with_settings!({
        snapshot_path => "../../snapshots",
        prepend_module_to_snapshot => false,
        sort_maps => true,
    }, {
        insta::assert_json_snapshot!(name, value);
    });
}

pub(super) fn native_service_task(
    task_id: &str,
    prompt: &str,
    inputs: &[&str],
    outputs: &[&str],
) -> String {
    format!(
        r#"<bpmn:serviceTask id="{task_id}" implementation="${{environment.services.runAgent}}">
      <bpmn:documentation>{prompt}</bpmn:documentation>
      {}
    </bpmn:serviceTask>"#,
        native_io(task_id, inputs, outputs)
    )
}

pub(super) fn native_user_task(
    task_id: &str,
    prompt: &str,
    interaction_type: &str,
    inputs: &[&str],
    choices_ref: Option<&str>,
    answer_output: &str,
) -> String {
    let mut data_inputs = vec!["interactionType"];
    data_inputs.extend_from_slice(inputs);
    if choices_ref.is_some() {
        data_inputs.push("choices");
    }
    let input_associations = {
        let mut associations = vec![format!(
            r"<bpmn:dataInputAssociation><bpmn:targetRef>{task_id}_Input_interactionType</bpmn:targetRef><bpmn:assignment><bpmn:from>{interaction_type}</bpmn:from><bpmn:to>{task_id}_Input_interactionType</bpmn:to></bpmn:assignment></bpmn:dataInputAssociation>"
        )];
        associations.extend(inputs.iter().map(|input| {
            format!(
                r"<bpmn:dataInputAssociation><bpmn:sourceRef>{input}</bpmn:sourceRef><bpmn:targetRef>{}</bpmn:targetRef></bpmn:dataInputAssociation>",
                stable_xml_id(task_id, "Input", input)
            )
        }));
        if let Some(choices_ref) = choices_ref {
            associations.push(format!(
                r"<bpmn:dataInputAssociation><bpmn:sourceRef>{choices_ref}</bpmn:sourceRef><bpmn:targetRef>{task_id}_Input_choices</bpmn:targetRef></bpmn:dataInputAssociation>"
            ));
        }
        associations.join("")
    };
    format!(
        r#"<bpmn:userTask id="{task_id}">
      <bpmn:documentation>{prompt}</bpmn:documentation>
      {}
      {input_associations}
      <bpmn:dataOutputAssociation><bpmn:sourceRef>{task_id}_Output_answer</bpmn:sourceRef><bpmn:targetRef>{answer_output}</bpmn:targetRef></bpmn:dataOutputAssociation>
    </bpmn:userTask>"#,
        native_io(task_id, &data_inputs, &["answer"])
    )
}

pub(super) fn native_io(task_id: &str, inputs: &[&str], outputs: &[&str]) -> String {
    let mut data_inputs = String::new();
    let mut input_refs = String::new();
    for input in inputs {
        let input_id = stable_xml_id(task_id, "Input", input);
        data_inputs.push_str("<bpmn:dataInput id=\"");
        data_inputs.push_str(&input_id);
        data_inputs.push_str("\" name=\"");
        data_inputs.push_str(input);
        data_inputs.push_str("\"/>");
        input_refs.push_str("<bpmn:dataInputRefs>");
        input_refs.push_str(&input_id);
        input_refs.push_str("</bpmn:dataInputRefs>");
    }

    let mut data_outputs = String::new();
    let mut output_refs = String::new();
    for output in outputs {
        let output_id = stable_xml_id(task_id, "Output", output);
        data_outputs.push_str("<bpmn:dataOutput id=\"");
        data_outputs.push_str(&output_id);
        data_outputs.push_str("\" name=\"");
        data_outputs.push_str(output);
        data_outputs.push_str("\"/>");
        output_refs.push_str("<bpmn:dataOutputRefs>");
        output_refs.push_str(&output_id);
        output_refs.push_str("</bpmn:dataOutputRefs>");
    }

    format!(
        r"<bpmn:ioSpecification>{data_inputs}{data_outputs}<bpmn:inputSet>{input_refs}</bpmn:inputSet><bpmn:outputSet>{output_refs}</bpmn:outputSet></bpmn:ioSpecification>"
    )
}

fn stable_xml_id(task_id: &str, role: &str, value: &str) -> String {
    let mut id = format!("{task_id}_{role}_{value}")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if id
        .chars()
        .next()
        .is_none_or(|ch| !(ch.is_ascii_alphabetic() || ch == '_'))
    {
        id.insert(0, '_');
    }
    id
}
