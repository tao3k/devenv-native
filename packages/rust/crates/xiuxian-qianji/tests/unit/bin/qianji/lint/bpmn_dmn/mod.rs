use super::*;

fn stable_temp_output(output: &str, temp_dir: &TempDir) -> String {
    output.replace(&temp_dir.path().display().to_string(), "$TEMP")
}

fn assert_llm_repair_snapshot_shape(output: &str, expected_fragments: &[&str]) {
    for required_section in [
        "Action:",
        "Fix:",
        "Patch focus:",
        "Examples:",
        "Forbidden forms:",
        "Structured repair:",
        "- strategy:",
        "- contract:",
    ] {
        assert!(
            output.contains(required_section),
            "compact diagnostic should include {required_section}"
        );
    }

    for expected_fragment in expected_fragments {
        assert!(
            output.contains(expected_fragment),
            "compact diagnostic should include {expected_fragment}"
        );
    }
}

fn native_service_task_io(
    task_id: &str,
    documentation: &str,
    inputs: &[&str],
    outputs: &[&str],
) -> String {
    let data_inputs = inputs
        .iter()
        .map(|input| format!(r#"<bpmn:dataInput id="{task_id}_input_{input}" name="{input}" />"#))
        .collect::<Vec<_>>()
        .join("\n        ");
    let data_outputs = outputs
        .iter()
        .map(|output| {
            format!(r#"<bpmn:dataOutput id="{task_id}_output_{output}" name="{output}" />"#)
        })
        .collect::<Vec<_>>()
        .join("\n        ");
    let input_refs = inputs
        .iter()
        .map(|input| format!(r#"<bpmn:dataInputRefs>{task_id}_input_{input}</bpmn:dataInputRefs>"#))
        .collect::<Vec<_>>()
        .join("\n          ");
    let output_refs = outputs
        .iter()
        .map(|output| {
            format!(r#"<bpmn:dataOutputRefs>{task_id}_output_{output}</bpmn:dataOutputRefs>"#)
        })
        .collect::<Vec<_>>()
        .join("\n          ");
    let input_associations = inputs
        .iter()
        .map(|input| {
            format!(
                r#"<bpmn:dataInputAssociation>
        <bpmn:sourceRef>{input}</bpmn:sourceRef>
        <bpmn:targetRef>{task_id}_input_{input}</bpmn:targetRef>
      </bpmn:dataInputAssociation>"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n      ");
    let output_associations = outputs
        .iter()
        .map(|output| {
            format!(
                r#"<bpmn:dataOutputAssociation>
        <bpmn:sourceRef>{task_id}_output_{output}</bpmn:sourceRef>
        <bpmn:targetRef>{output}</bpmn:targetRef>
      </bpmn:dataOutputAssociation>"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n      ");

    format!(
        r#"<bpmn:documentation>{documentation}</bpmn:documentation>
      <bpmn:ioSpecification>
        {data_inputs}
        {data_outputs}
        <bpmn:inputSet id="{task_id}_input_set">
          {input_refs}
        </bpmn:inputSet>
        <bpmn:outputSet id="{task_id}_output_set">
          {output_refs}
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      {input_associations}
      {output_associations}"#
    )
}

fn native_user_choice_io(
    task_id: &str,
    documentation: &str,
    interaction_type: &str,
    choices_json: &str,
    output_target: &str,
) -> String {
    format!(
        r#"<bpmn:documentation>{documentation}</bpmn:documentation>
      <bpmn:ioSpecification>
        <bpmn:dataInput id="{task_id}_input_interactionType" name="interactionType" />
        <bpmn:dataInput id="{task_id}_input_choices" name="choices" />
        <bpmn:dataOutput id="{task_id}_output_answer" name="answer" />
        <bpmn:inputSet id="{task_id}_input_set">
          <bpmn:dataInputRefs>{task_id}_input_interactionType</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>{task_id}_input_choices</bpmn:dataInputRefs>
        </bpmn:inputSet>
        <bpmn:outputSet id="{task_id}_output_set">
          <bpmn:dataOutputRefs>{task_id}_output_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataInputAssociation>
        <bpmn:assignment>
          <bpmn:from>{interaction_type}</bpmn:from>
          <bpmn:to>{task_id}_input_interactionType</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:assignment>
          <bpmn:from>{choices_json}</bpmn:from>
          <bpmn:to>{task_id}_input_choices</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>{task_id}_output_answer</bpmn:sourceRef>
        <bpmn:targetRef>{output_target}</bpmn:targetRef>
      </bpmn:dataOutputAssociation>"#
    )
}

fn native_user_dynamic_choice_io(
    task_id: &str,
    documentation: &str,
    question_source: &str,
    choices_source: &str,
    free_text_json: Option<&str>,
    output_target: &str,
) -> String {
    let free_text_input = free_text_json
        .map(|_| format!(r#"<bpmn:dataInput id="{task_id}_input_freeText" name="freeText" />"#))
        .unwrap_or_default();
    let free_text_ref = free_text_json
        .map(|_| format!(r#"<bpmn:dataInputRefs>{task_id}_input_freeText</bpmn:dataInputRefs>"#))
        .unwrap_or_default();
    let free_text_association = free_text_json
        .map(|json| {
            format!(
                r#"<bpmn:dataInputAssociation>
        <bpmn:assignment>
          <bpmn:from>{json}</bpmn:from>
          <bpmn:to>{task_id}_input_freeText</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>"#
            )
        })
        .unwrap_or_default();

    format!(
        r#"<bpmn:documentation>{documentation}</bpmn:documentation>
      <bpmn:ioSpecification>
        <bpmn:dataInput id="{task_id}_input_interactionType" name="interactionType" />
        <bpmn:dataInput id="{task_id}_input_question" name="question" />
        <bpmn:dataInput id="{task_id}_input_choices" name="choices" />
        {free_text_input}
        <bpmn:dataOutput id="{task_id}_output_answer" name="answer" />
        <bpmn:inputSet id="{task_id}_input_set">
          <bpmn:dataInputRefs>{task_id}_input_interactionType</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>{task_id}_input_question</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>{task_id}_input_choices</bpmn:dataInputRefs>
          {free_text_ref}
        </bpmn:inputSet>
        <bpmn:outputSet id="{task_id}_output_set">
          <bpmn:dataOutputRefs>{task_id}_output_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataInputAssociation>
        <bpmn:assignment>
          <bpmn:from>choice_input</bpmn:from>
          <bpmn:to>{task_id}_input_interactionType</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:sourceRef>{question_source}</bpmn:sourceRef>
        <bpmn:targetRef>{task_id}_input_question</bpmn:targetRef>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:sourceRef>{choices_source}</bpmn:sourceRef>
        <bpmn:targetRef>{task_id}_input_choices</bpmn:targetRef>
      </bpmn:dataInputAssociation>
      {free_text_association}
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>{task_id}_output_answer</bpmn:sourceRef>
        <bpmn:targetRef>{output_target}</bpmn:targetRef>
      </bpmn:dataOutputAssociation>"#
    )
}

mod cases_01;
mod cases_02;
mod cases_03;
mod cases_04;
mod cases_07;
mod cases_08;
mod cases_09;
mod cases_10;
