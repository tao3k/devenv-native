use super::{BpmnSourceFile, LintDomain, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_native_user_task_rendering_as_deferred() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "native-user-task-rendering.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_UserInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Review" sourceRef="Start" targetRef="Task_Review"/>
    <userTask id="Task_Review" name="Review request">
      <rendering id="Rendering_Form"/>
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
    assert_eq!(issue.code, "bpmn.unsupported_human_task_rendering");
    assert!(issue.summary.contains("Task_Review"));
    assert!(issue.why_it_failed.contains("native BPMN IO"));
    assert!(issue.llm_fix_prompt.contains("native BPMN IO"));
    assert_eq!(issue.evidence["task_id"], "Task_Review");
    assert_eq!(issue.evidence["element"], "rendering");
    assert!(issue.source_diagnostic.is_some());
}

#[test]
fn bpmn_linter_reports_manual_task_rendering_as_invalid_standard_surface() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "manual-task-rendering.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <process id="Process_ManualInput" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Review" sourceRef="Start" targetRef="Task_Acknowledge"/>
    <manualTask id="Task_Acknowledge" name="Acknowledge external action">
      <rendering id="Rendering_Form"/>
    </manualTask>
    <sequenceFlow id="Flow_Review_End" sourceRef="Task_Acknowledge" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_manual_task_rendering");
    assert!(issue.summary.contains("Task_Acknowledge"));
    assert!(issue.why_it_failed.contains("userTask"));
    assert!(issue.why_it_failed.contains("manualTask"));
    assert!(issue.llm_fix_prompt.contains("native BPMN IO"));
    assert_eq!(issue.evidence["task_id"], "Task_Acknowledge");
    assert_eq!(issue.evidence["task_kind"], "manualTask");
    assert_eq!(issue.evidence["element"], "rendering");
    assert!(issue.source_diagnostic.is_some());
}
