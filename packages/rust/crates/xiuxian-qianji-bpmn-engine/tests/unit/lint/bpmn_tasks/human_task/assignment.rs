use super::{BpmnSourceFile, LintDomain, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_resource_parameter_binding_assignment_semantics() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "human-task-resource-parameter-binding.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Review" sourceRef="Start" targetRef="Task_Review"/>
    <userTask id="Task_Review" name="Review request">
      <potentialOwner name="review_team">
        <resourceRef>reviewers</resourceRef>
        <resourceParameterBinding parameterRef="region">
          <formalExpression>emea</formalExpression>
        </resourceParameterBinding>
      </potentialOwner>
    </userTask>
    <sequenceFlow id="Flow_Review_End" sourceRef="Task_Review" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(
        issue.code,
        "bpmn.unsupported_human_task_assignment_semantics"
    );
    assert!(issue.summary.contains("resourceParameterBinding"));
    assert!(issue.why_it_failed.contains("routing metadata only"));
    assert!(issue.llm_fix_prompt.contains("potentialOwner"));
    assert_eq!(issue.evidence["task_id"], "Task_Review");
    assert_eq!(issue.evidence["element"], "resourceParameterBinding");
    assert!(issue.source_diagnostic.is_some());
}

#[test]
fn bpmn_linter_reports_generic_performer_assignment_semantics() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "human-task-generic-performer.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Review" sourceRef="Start" targetRef="Task_Review"/>
    <userTask id="Task_Review" name="Review request">
      <performer name="reviewer"/>
    </userTask>
    <sequenceFlow id="Flow_Review_End" sourceRef="Task_Review" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(
        issue.code,
        "bpmn.unsupported_human_task_assignment_semantics"
    );
    assert!(issue.summary.contains("<performer>"));
    assert_eq!(issue.evidence["task_id"], "Task_Review");
    assert_eq!(issue.evidence["element"], "performer");
}

#[test]
fn bpmn_linter_reports_generic_resource_role_assignment_semantics() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "human-task-generic-resource-role.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Review" sourceRef="Start" targetRef="Task_Review"/>
    <userTask id="Task_Review" name="Review request">
      <resourceRole name="regional_reviewer">
        <resourceRef>reviewers</resourceRef>
      </resourceRole>
    </userTask>
    <sequenceFlow id="Flow_Review_End" sourceRef="Task_Review" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(
        issue.code,
        "bpmn.unsupported_human_task_assignment_semantics"
    );
    assert!(issue.summary.contains("<resourceRole>"));
    assert!(issue.why_it_failed.contains("routing metadata only"));
    assert_eq!(issue.evidence["task_id"], "Task_Review");
    assert_eq!(issue.evidence["element"], "resourceRole");
    assert!(issue.source_diagnostic.is_some());
}

#[test]
fn bpmn_linter_reports_participant_ref_assignment_semantics() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "human-task-participant-ref.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Review" sourceRef="Start" targetRef="Task_Review"/>
    <userTask id="Task_Review" name="Review request">
      <potentialOwner name="review_team">
        <participantRef>Participant_Reviewers</participantRef>
      </potentialOwner>
    </userTask>
    <sequenceFlow id="Flow_Review_End" sourceRef="Task_Review" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(
        issue.code,
        "bpmn.unsupported_human_task_assignment_semantics"
    );
    assert!(issue.summary.contains("<participantRef>"));
    assert!(issue.why_it_failed.contains("participant refs"));
    assert_eq!(issue.evidence["task_id"], "Task_Review");
    assert_eq!(issue.evidence["element"], "participantRef");
    assert!(issue.source_diagnostic.is_some());
}
