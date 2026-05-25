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
    assert!(issue.why_it_failed.contains("native BPMN IO"));
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
    assert_eq!(issue.code, "bpmn.unsupported_global_task_binding");
    assert!(issue.summary.contains("Activity_GlobalReview"));
    assert!(issue.summary.contains("Global_Task_Review"));
    assert!(issue.why_it_failed.contains("executable process"));
    assert!(issue.llm_fix_prompt.contains("native BPMN IO"));
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
    assert_eq!(issue.code, "bpmn.unsupported_global_task_binding");
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
fn bpmn_linter_reports_call_activity_to_non_human_global_task_as_unsupported() {
    for (task_kind, task_id, process_id, activity_id) in [
        (
            "globalTask",
            "Global_Task_Generic",
            "Process_GlobalTaskCall",
            "Activity_GlobalGeneric",
        ),
        (
            "globalBusinessRuleTask",
            "Global_Task_Rule",
            "Process_GlobalRuleTaskCall",
            "Activity_GlobalRule",
        ),
        (
            "globalScriptTask",
            "Global_Task_Script",
            "Process_GlobalScriptTaskCall",
            "Activity_GlobalScript",
        ),
    ] {
        let report = lint_bpmn_source(&BpmnSourceFile::new(
            format!("call-activity-{task_kind}.bpmn"),
            format!(
                r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://qianji.dev/tests">
  <{task_kind} id="{task_id}" name="Reusable task"/>
  <process id="{process_id}" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Call" sourceRef="Start" targetRef="{activity_id}"/>
    <callActivity id="{activity_id}" calledElement="{task_id}"/>
    <sequenceFlow id="Flow_Call_End" sourceRef="{activity_id}" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#
            ),
        ));

        assert_eq!(report.domain, LintDomain::Bpmn);
        assert!(!report.ok);
        assert_eq!(report.issues.len(), 1);
        let issue = &report.issues[0];
        assert_eq!(issue.code, "bpmn.unsupported_global_task_binding");
        assert!(issue.summary.contains(activity_id));
        assert!(issue.summary.contains(task_id));
        assert!(issue.why_it_failed.contains("executable process"));
        assert_eq!(issue.evidence["process_id"], process_id);
        assert_eq!(issue.evidence["call_activity_id"], activity_id);
        assert_eq!(issue.evidence["called_element"], task_id);
        assert_eq!(issue.evidence["global_task_kind"], task_kind);
        assert_eq!(issue.evidence["element"], "callActivity");
        assert_eq!(issue.evidence["unsupported_binding"], "global task");
        assert!(issue.source_diagnostic.is_some());
    }
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
