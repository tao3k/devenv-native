use super::super::{LintDomain, lint_bpmn_source};
use qianji_bpmn_engine::BpmnSourceFile;

#[test]
fn bpmn_linter_accepts_native_choice_input_contract() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "native-choice-input.bpmn",
        native_user_task("choice_input", Some("currentChoices"), None, Some("answer")),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{:#?}", report.issues);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_all_supported_native_interaction_types() {
    for (interaction_type, choices) in [
        ("input", None),
        ("confirm", None),
        (
            "choice",
            Some(r#"[{"value":"approve"},{"value":"reject"}]"#),
        ),
        (
            "choice_input",
            Some(r#"[{"value":"approve"},{"value":"revise"}]"#),
        ),
    ] {
        let report = lint_bpmn_source(&BpmnSourceFile::new(
            format!("native-{interaction_type}.bpmn"),
            native_user_task(interaction_type, None, choices, Some("answer")),
        ));

        assert!(
            report.ok,
            "{interaction_type} should be accepted as native BPMN IO: {:#?}",
            report.issues
        );
        assert!(report.issues.is_empty());
    }
}

#[test]
fn bpmn_linter_rejects_legacy_custom_interaction_xml() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "legacy-custom-interaction.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_Legacy" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Question" sourceRef="Start" targetRef="Task_Question"/>
    <userTask id="Task_Question">
      <extensionElements>
        <qianji:config>
          <qianji:interaction type="input">
            <qianji:result output="answer"/>
          </qianji:interaction>
        </qianji:config>
      </extensionElements>
    </userTask>
    <sequenceFlow id="Flow_Question_End" sourceRef="Task_Question" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.legacy_custom_interaction_xml");
    assert!(issue.summary.contains("qianji:config"));
    assert!(issue.why_it_failed.contains("native BPMN"));
}

#[test]
fn bpmn_linter_rejects_unsupported_native_interaction_type() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "unsupported-native-interaction-type.bpmn",
        native_user_task("free_form", None, None, Some("answer")),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(
        report.issues[0].code,
        "bpmn.unsupported_native_interaction_type"
    );
    assert!(report.issues[0].summary.contains("free_form"));
}

#[test]
fn bpmn_linter_rejects_choice_without_choices() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "missing-native-choices.bpmn",
        native_user_task("choice", None, None, Some("answer")),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues[0].code, "bpmn.missing_native_choice_contract");
}

#[test]
fn bpmn_linter_rejects_choice_with_dynamic_and_static_choices() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "ambiguous-native-choices.bpmn",
        native_user_task(
            "choice_input",
            Some("currentChoices"),
            Some(r#"[{"value":"fallback","label":"Fallback"}]"#),
            Some("answer"),
        ),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues[0].code, "bpmn.ambiguous_native_choices");
}

#[test]
fn bpmn_linter_rejects_multiple_native_free_text_fields() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "multiple-native-free-text.bpmn",
        native_user_task_with_free_text(
            "choice_input",
            Some(r#"[{"value":"approve"},{"value":"revise"}]"#),
            r#"[{"name":"feedback","optional":true},{"name":"rationale","optional":true}]"#,
            Some("answer"),
        ),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(
        report.issues[0].code,
        "bpmn.unsupported_native_free_text_cardinality"
    );
}

#[test]
fn bpmn_linter_rejects_native_interaction_without_answer_output() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "missing-native-answer-output.bpmn",
        native_user_task("input", None, None, None),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues[0].code, "bpmn.missing_native_answer_output");
}

fn native_user_task(
    interaction_type: &str,
    choices_ref: Option<&str>,
    static_choices: Option<&str>,
    answer_output: Option<&str>,
) -> String {
    let choices_association = match (choices_ref, static_choices) {
        (Some(ref_name), Some(literal)) => format!(
            r"
      <dataInputAssociation>
        <sourceRef>{ref_name}</sourceRef>
        <targetRef>Task_Question_choices</targetRef>
      </dataInputAssociation>
      <dataInputAssociation>
        <targetRef>Task_Question_choices</targetRef>
        <assignment><from>{literal}</from><to>Task_Question_choices</to></assignment>
      </dataInputAssociation>"
        ),
        (Some(ref_name), None) => format!(
            r"
      <dataInputAssociation>
        <sourceRef>{ref_name}</sourceRef>
        <targetRef>Task_Question_choices</targetRef>
      </dataInputAssociation>"
        ),
        (None, Some(literal)) => format!(
            r"
      <dataInputAssociation>
        <targetRef>Task_Question_choices</targetRef>
        <assignment><from>{literal}</from><to>Task_Question_choices</to></assignment>
      </dataInputAssociation>"
        ),
        (None, None) => String::new(),
    };
    native_user_task_body(interaction_type, &choices_association, "", answer_output)
}

fn native_user_task_with_free_text(
    interaction_type: &str,
    static_choices: Option<&str>,
    free_text: &str,
    answer_output: Option<&str>,
) -> String {
    let choices_association = static_choices
        .map(|literal| {
            format!(
                r"
      <dataInputAssociation>
        <targetRef>Task_Question_choices</targetRef>
        <assignment><from>{literal}</from><to>Task_Question_choices</to></assignment>
      </dataInputAssociation>"
            )
        })
        .unwrap_or_default();
    let free_text_association = format!(
        r"
      <dataInputAssociation>
        <targetRef>Task_Question_free_text</targetRef>
        <assignment><from>{free_text}</from><to>Task_Question_free_text</to></assignment>
      </dataInputAssociation>"
    );
    native_user_task_body(
        interaction_type,
        &choices_association,
        &free_text_association,
        answer_output,
    )
}

fn native_user_task_body(
    interaction_type: &str,
    choices_association: &str,
    free_text_association: &str,
    answer_output: Option<&str>,
) -> String {
    let answer_association = answer_output
        .map(|output| {
            format!(
                r"
      <dataOutputAssociation>
        <sourceRef>Task_Question_answer</sourceRef>
        <targetRef>{output}</targetRef>
      </dataOutputAssociation>"
            )
        })
        .unwrap_or_default();
    format!(
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Question" sourceRef="Start" targetRef="Task_Question"/>
    <userTask id="Task_Question" name="Ask question">
      <documentation>What should happen next?</documentation>
      <ioSpecification>
        <dataInput id="Task_Question_interaction_type" name="interactionType"/>
        <dataInput id="Task_Question_choices" name="choices"/>
        <dataInput id="Task_Question_free_text" name="freeText"/>
        <dataOutput id="Task_Question_answer" name="answer"/>
        <inputSet>
          <dataInputRefs>Task_Question_interaction_type</dataInputRefs>
          <dataInputRefs>Task_Question_choices</dataInputRefs>
          <dataInputRefs>Task_Question_free_text</dataInputRefs>
        </inputSet>
        <outputSet>
          <dataOutputRefs>Task_Question_answer</dataOutputRefs>
        </outputSet>
      </ioSpecification>
      <dataInputAssociation>
        <targetRef>Task_Question_interaction_type</targetRef>
        <assignment><from>{interaction_type}</from><to>Task_Question_interaction_type</to></assignment>
      </dataInputAssociation>{choices_association}{free_text_association}{answer_association}
    </userTask>
    <sequenceFlow id="Flow_Question_End" sourceRef="Task_Question" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#
    )
}
