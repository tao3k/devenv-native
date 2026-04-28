use super::{BpmnSourceFile, LintDomain, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_global_user_task_rendering_as_deferred() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "global-user-task-rendering.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <globalUserTask id="Global_Task_Review" name="Global review">
    <rendering id="Rendering_Global_Form"/>
  </globalUserTask>
  <process id="Process_GlobalUserTask" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_End" sourceRef="Start" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_human_task_rendering");
    assert!(issue.summary.contains("Global_Task_Review"));
    assert!(issue.why_it_failed.contains("qianji:interaction"));
    assert_eq!(issue.evidence["task_id"], "Global_Task_Review");
    assert_eq!(issue.evidence["task_kind"], "globalUserTask");
    assert_eq!(issue.evidence["element"], "rendering");
    assert!(issue.source_diagnostic.is_some());
}

#[test]
fn bpmn_linter_reports_global_manual_task_rendering_as_invalid_standard_surface() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "global-manual-task-rendering.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <globalManualTask id="Global_Task_Acknowledge" name="Global acknowledge">
    <rendering id="Rendering_Global_Form"/>
  </globalManualTask>
  <process id="Process_GlobalManualTask" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_End" sourceRef="Start" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.invalid_manual_task_rendering");
    assert!(issue.summary.contains("Global_Task_Acknowledge"));
    assert!(issue.why_it_failed.contains("globalManualTask"));
    assert_eq!(issue.evidence["task_id"], "Global_Task_Acknowledge");
    assert_eq!(issue.evidence["task_kind"], "globalManualTask");
    assert_eq!(issue.evidence["element"], "rendering");
    assert!(issue.source_diagnostic.is_some());
}

#[test]
fn bpmn_linter_reports_call_activity_to_global_user_task_as_unsupported() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "call-activity-global-user-task.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <globalUserTask id="Global_Task_Review" name="Global review"/>
  <process id="Process_GlobalUserTaskCall" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Call" sourceRef="Start" targetRef="Activity_GlobalReview"/>
    <callActivity id="Activity_GlobalReview" calledElement="Global_Task_Review"/>
    <sequenceFlow id="Flow_Call_End" sourceRef="Activity_GlobalReview" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_global_human_task_binding");
    assert!(issue.summary.contains("Activity_GlobalReview"));
    assert!(issue.summary.contains("Global_Task_Review"));
    assert!(issue.why_it_failed.contains("executable process"));
    assert!(issue.llm_fix_prompt.contains("typed `qianji:interaction`"));
    assert_eq!(issue.evidence["process_id"], "Process_GlobalUserTaskCall");
    assert_eq!(issue.evidence["call_activity_id"], "Activity_GlobalReview");
    assert_eq!(issue.evidence["called_element"], "Global_Task_Review");
    assert_eq!(issue.evidence["global_task_kind"], "globalUserTask");
    assert_eq!(issue.evidence["element"], "callActivity");
    assert!(issue.source_diagnostic.is_some());
}

#[test]
fn bpmn_linter_reports_call_activity_to_global_manual_task_as_unsupported() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "call-activity-global-manual-task.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <globalManualTask id="Global_Task_Acknowledge" name="Global acknowledge"/>
  <process id="Process_GlobalManualTaskCall" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Call" sourceRef="Start" targetRef="Activity_GlobalAcknowledge"/>
    <callActivity id="Activity_GlobalAcknowledge" calledElement="Global_Task_Acknowledge"/>
    <sequenceFlow id="Flow_Call_End" sourceRef="Activity_GlobalAcknowledge" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_global_human_task_binding");
    assert!(issue.summary.contains("Activity_GlobalAcknowledge"));
    assert!(issue.summary.contains("Global_Task_Acknowledge"));
    assert_eq!(issue.evidence["process_id"], "Process_GlobalManualTaskCall");
    assert_eq!(
        issue.evidence["call_activity_id"],
        "Activity_GlobalAcknowledge"
    );
    assert_eq!(issue.evidence["called_element"], "Global_Task_Acknowledge");
    assert_eq!(issue.evidence["global_task_kind"], "globalManualTask");
    assert!(issue.source_diagnostic.is_some());
}

#[test]
fn bpmn_linter_accepts_global_user_task_definition_without_runtime_binding() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "global-human-task-metadata-only.bpmn",
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <globalUserTask id="Global_Task_Review" name="Global review"/>
  <globalManualTask id="Global_Task_Acknowledge" name="Global acknowledge"/>
  <process id="Process_MetadataOnly" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_End" sourceRef="Start" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}
