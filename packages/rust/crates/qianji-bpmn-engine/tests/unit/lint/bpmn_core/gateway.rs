use super::{
    BpmnSourceFile, LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source,
};

#[test]
fn bpmn_linter_reports_unsupported_gateway_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-unsupported-gateway.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    assert_lint_json_snapshot("bpmn_unsupported_gateway_lint_report", &report);
}

#[test]
fn bpmn_linter_reports_invalid_event_based_gateway_target_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-event-based-gateway-task-target.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(
        issue.code,
        "bpmn.unsupported_event_based_gateway_configuration"
    );
    assert!(issue.summary.contains("wait_race"));
    assert!(issue.llm_fix_prompt.contains("eventBasedGateway"));
}

#[test]
fn bpmn_linter_reports_unsupported_gateway_condition_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-exclusive-gateway-unsupported-condition.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_gateway_configuration");
    assert!(issue.summary.contains("decision"));
    assert!(issue.why_it_failed.contains("numeric comparisons"));
    assert!(issue.llm_fix_prompt.contains("amount > 100"));
    let Some(source_diagnostic) = issue.source_diagnostic.as_ref() else {
        panic!("unsupported condition should carry a source diagnostic");
    };
    assert!(source_diagnostic.span.start < source_diagnostic.span.end);
    assert!(source_diagnostic.label.contains("bounded subset"));
    assert_lint_json_snapshot("bpmn_gateway_condition_expression_lint_report", &report);
}

#[test]
fn bpmn_linter_aggregates_multiple_unsupported_gateway_conditions() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-multiple-unsupported-conditions.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_multiple_unsupported_conditions">
  <bpmn:process id="multiple_unsupported_conditions" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="choose" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return routeChoice.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>routeChoice</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="decision" default="flow_other" />
    <bpmn:serviceTask id="retry" />
    <bpmn:serviceTask id="skip" />
    <bpmn:endEvent id="other" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="choose" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="choose" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_retry" sourceRef="decision" targetRef="retry">
      <bpmn:conditionExpression>routeChoice == "retry"</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_skip" sourceRef="decision" targetRef="skip">
      <bpmn:conditionExpression>routeChoice == "skip"</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_other" sourceRef="decision" targetRef="other" />
    <bpmn:sequenceFlow id="flow_retry_done" sourceRef="retry" targetRef="done" />
    <bpmn:sequenceFlow id="flow_skip_done" sourceRef="skip" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    let unsupported_summaries = report
        .issues
        .iter()
        .filter(|issue| issue.title.contains("condition exceeds"))
        .map(|issue| issue.summary.as_str())
        .collect::<Vec<_>>();
    assert_eq!(unsupported_summaries.len(), 1, "{report:#?}");
    assert!(report.issues.iter().any(|issue| {
        issue.title.contains("condition exceeds") && issue.source_diagnostic.is_some()
    }));
    assert!(
        unsupported_summaries
            .iter()
            .any(|summary| summary.contains("routeChoice == \"retry\""))
    );
    assert!(
        unsupported_summaries
            .iter()
            .any(|summary| summary.contains("routeChoice == \"skip\""))
    );
}

#[test]
fn bpmn_linter_reports_gateway_condition_missing_upstream_output() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-gateway-undeclared-output.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_gateway_undeclared_output">
  <bpmn:process id="gateway_undeclared_output" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="analyze" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return JSON with agentTasks.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>failures</qianji:inputs>
          <qianji:outputs>agentTasks</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="decision" default="flow_sequential" />
    <bpmn:serviceTask id="parallel" />
    <bpmn:serviceTask id="sequential" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="analyze" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="analyze" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_parallel" sourceRef="decision" targetRef="parallel">
      <bpmn:conditionExpression>canParallelize</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_sequential" sourceRef="decision" targetRef="sequential" />
    <bpmn:sequenceFlow id="flow_parallel_done" sourceRef="parallel" targetRef="done" />
    <bpmn:sequenceFlow id="flow_sequential_done" sourceRef="sequential" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.undeclared_gateway_condition_output");
    assert!(issue.summary.contains("canParallelize"));
    assert!(issue.summary.contains("analyze"));
    assert!(issue.llm_fix_prompt.contains("qianji:outputs"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("return JSON field `canParallelize`")
    );
    assert!(issue.source_diagnostic.is_some());
}

#[test]
fn bpmn_linter_reports_count_like_boolean_condition_with_llm_guidance() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-count-like-boolean-condition.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_count_like">
  <bpmn:process id="count_like_condition" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="make_question" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return JSON with currentQuestion and questionsRemaining.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>currentQuestion,questionsRemaining</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    <bpmn:userTask id="ask" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="make_question" />
    <bpmn:sequenceFlow id="flow_gateway" sourceRef="make_question" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_more" sourceRef="decision" targetRef="ask">
      <bpmn:conditionExpression>questionsRemaining</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_ask_done" sourceRef="ask" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.ambiguous_boolean_gateway_condition");
    assert!(issue.summary.contains("questionsRemaining"));
    assert!(issue.llm_fix_prompt.contains("questionsRemaining > 0"));
    let Some(source_diagnostic) = issue.source_diagnostic.as_ref() else {
        panic!("count-like boolean condition should carry a source diagnostic");
    };
    assert!(source_diagnostic.span.start < source_diagnostic.span.end);
    assert!(source_diagnostic.help.contains("questionsRemaining > 0"));
}

#[test]
fn bpmn_linter_reports_content_like_boolean_condition_with_llm_guidance() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-content-like-boolean-condition.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_content_like">
  <bpmn:process id="content_like_condition" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_question" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return JSON with questions as user-facing text.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>questions</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    <bpmn:userTask id="ask" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_question" />
    <bpmn:sequenceFlow id="flow_gateway" sourceRef="prepare_question" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_more" sourceRef="decision" targetRef="ask">
      <bpmn:conditionExpression>questions</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_ask_done" sourceRef="ask" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.ambiguous_boolean_gateway_condition");
    assert!(issue.title.contains("content-like variable"));
    assert!(issue.summary.contains("questions"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("separate boolean route variable")
    );
    assert!(issue.llm_fix_prompt.contains("hasQuestions"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Keep 'questions' as content"))
    );
    let Some(source_diagnostic) = issue.source_diagnostic.as_ref() else {
        panic!("content-like boolean condition should carry a source diagnostic");
    };
    assert!(source_diagnostic.span.start < source_diagnostic.span.end);
    assert!(
        source_diagnostic
            .help
            .contains("separate boolean-shaped route output")
    );
}

#[test]
fn bpmn_linter_reports_content_like_condition_even_when_parse_fails() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-unsupported-and-content-like-conditions.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_mixed_conditions">
  <bpmn:process id="mixed_conditions" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="dispatch" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return implementerStatus and questions.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>implementerStatus,questions</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="question_gate" default="flow_status_gate" />
    <bpmn:exclusiveGateway id="status_gate" default="flow_done" />
    <bpmn:userTask id="ask" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="dispatch" />
    <bpmn:sequenceFlow id="flow_question_gate" sourceRef="dispatch" targetRef="question_gate" />
    <bpmn:sequenceFlow id="flow_ask" sourceRef="question_gate" targetRef="ask">
      <bpmn:conditionExpression>questions</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_status_gate" sourceRef="question_gate" targetRef="status_gate" />
    <bpmn:sequenceFlow id="flow_done_by_status" sourceRef="status_gate" targetRef="done">
      <bpmn:conditionExpression>implementerStatus == 'DONE'</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="status_gate" targetRef="done" />
    <bpmn:sequenceFlow id="flow_ask_done" sourceRef="ask" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.code == "bpmn.unsupported_gateway_configuration")
    );
    let content_issue = report
        .issues
        .iter()
        .find(|issue| issue.title.contains("content-like variable"))
        .unwrap_or_else(|| panic!("source scan should append content-like condition guidance"));
    assert!(content_issue.summary.contains("questions"));
    assert!(content_issue.llm_fix_prompt.contains("hasQuestions"));
}

#[test]
fn bpmn_linter_reports_duplicate_unconditional_default_branch_with_llm_guidance() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-duplicate-default-branch.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_duplicate_default_branch">
  <bpmn:process id="duplicate_default_branch" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="approve">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Approve this section?</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>sectionDraft</qianji:inputs>
          <qianji:outputs>sectionApproved</qianji:outputs>
          <qianji:interaction type="confirm">
            <qianji:question>Approve?</qianji:question>
            <qianji:result output="sectionApproved"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:exclusiveGateway id="decision" default="flow_revise" />
    <bpmn:serviceTask id="next" implementation="${environment.services.runAgent}" />
    <bpmn:serviceTask id="revise" implementation="${environment.services.runAgent}" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="approve" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="approve" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_approved" sourceRef="decision" targetRef="next">
      <bpmn:conditionExpression>sectionApproved</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_revise" sourceRef="decision" targetRef="revise" />
    <bpmn:sequenceFlow id="flow_duplicate_revise" sourceRef="decision" targetRef="revise" />
    <bpmn:sequenceFlow id="flow_next_done" sourceRef="next" targetRef="done" />
    <bpmn:sequenceFlow id="flow_revise_done" sourceRef="revise" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    let issue = report
        .issues
        .iter()
        .find(|issue| {
            issue
                .evidence
                .get("duplicate_default_flow_ids")
                .is_some_and(|value| {
                    value
                        .as_array()
                        .is_some_and(|items| items.iter().any(|item| item == "flow_revise"))
                })
        })
        .unwrap_or_else(|| panic!("duplicate default branch guidance should be reported"));
    assert_eq!(issue.code, "bpmn.unsupported_gateway_configuration");
    assert!(
        issue
            .llm_fix_prompt
            .contains("duplicates default fallback branch")
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not change the gateway default")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("default fallback branch"))
    );
    let Some(source_diagnostic) = issue.source_diagnostic.as_ref() else {
        panic!("duplicate default branch should carry a source diagnostic");
    };
    assert!(source_diagnostic.label.contains("duplicate fallback"));
    assert!(source_diagnostic.help.contains("flow_revise"));
    let Some(structured_repair) = issue.structured_repair.as_ref() else {
        panic!("duplicate default branch should carry structured repair");
    };
    assert_eq!(
        structured_repair["strategy"],
        "remove_duplicate_unconditional_gateway_branch"
    );
    assert_eq!(
        structured_repair["target"]["duplicate_default_flow_ids"][0],
        "flow_revise"
    );
}

#[test]
fn bpmn_linter_reports_unsupported_condition_even_when_default_flow_is_stale() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-stale-default-and-string-condition.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_stale_default_string_condition">
  <bpmn:process id="stale_default_string_condition" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="execute_task" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Execute one task and return taskStatus.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>taskStatus</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="task_gate" default="flow_done" />
    <bpmn:userTask id="ask_human" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="execute_task" />
    <bpmn:sequenceFlow id="flow_gate" sourceRef="execute_task" targetRef="task_gate" />
    <bpmn:sequenceFlow id="flow_blocked" sourceRef="task_gate" targetRef="ask_human">
      <bpmn:conditionExpression>taskStatus == "blocked"</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_completed" sourceRef="task_gate" targetRef="done">
      <bpmn:conditionExpression>taskStatus == "completed"</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_ask_done" sourceRef="ask_human" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.title.contains("default flow reference"))
    );
    let unsupported_condition = report
        .issues
        .iter()
        .find(|issue| {
            issue.title.contains("condition exceeds")
                && issue.summary.contains("taskStatus == \"blocked\"")
        })
        .unwrap_or_else(|| {
            panic!("source scan should append unsupported string condition guidance")
        });
    assert!(
        unsupported_condition
            .llm_fix_prompt
            .contains("bounded boolean route variables")
    );
    let Some(source_diagnostic) = unsupported_condition.source_diagnostic.as_ref() else {
        panic!("unsupported condition should carry source span");
    };
    assert!(source_diagnostic.span.start < source_diagnostic.span.end);
}

#[test]
fn bpmn_linter_accepts_boolean_route_names_that_end_with_context() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "valid-human-provide-context-route.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_context_route">
  <bpmn:process id="context_route" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="human_decision">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Ask whether the human should provide context.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>humanProvideContext</qianji:outputs>
          <qianji:interaction type="confirm">
            <qianji:question>Provide more context?</qianji:question>
            <qianji:result output="humanProvideContext"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    <bpmn:serviceTask id="provide" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Provide context.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>provided</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="human_decision" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="human_decision" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_provide" sourceRef="decision" targetRef="provide">
      <bpmn:conditionExpression>humanProvideContext</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_provide_done" sourceRef="provide" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{report:#?}");
}

#[test]
fn bpmn_linter_accepts_boolean_route_names_with_embedded_marker() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "valid-status-is-done-with-concerns-route.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_status_route">
  <bpmn:process id="status_route" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="dispatch" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return statusIsDoneWithConcerns as a JSON boolean.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>statusIsDoneWithConcerns</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    <bpmn:serviceTask id="review" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Review concerns.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>reviewed</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="dispatch" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="dispatch" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_review" sourceRef="decision" targetRef="review">
      <bpmn:conditionExpression>statusIsDoneWithConcerns</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_review_done" sourceRef="review" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{report:#?}");
}

#[test]
fn bpmn_linter_accepts_escaped_numeric_gateway_condition() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "valid-escaped-numeric-gateway-condition.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_count_like">
  <bpmn:process id="count_like_condition" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="make_question" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return JSON with questionsRemaining as a number.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>questionsRemaining</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    <bpmn:userTask id="ask" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="make_question" />
    <bpmn:sequenceFlow id="flow_gateway" sourceRef="make_question" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_more" sourceRef="decision" targetRef="ask">
      <bpmn:conditionExpression>questionsRemaining &gt; 0</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_ask_done" sourceRef="ask" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{report:#?}");
}

#[test]
fn bpmn_linter_reports_single_outgoing_default_with_complete_fallback_guidance() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-single-outgoing-default.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_default">
  <bpmn:process id="default_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:exclusiveGateway id="decision" default="flow_fallback" />
    <bpmn:serviceTask id="retry" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_retry" sourceRef="decision" targetRef="retry">
      <bpmn:conditionExpression>needsRetry</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="retry" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_gateway_configuration");
    assert!(
        issue
            .llm_fix_prompt
            .contains("add the fallback sequenceFlow")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("set the gateway `default` attribute"))
    );
    let structured_repair = issue
        .structured_repair
        .as_ref()
        .unwrap_or_else(|| panic!("default-flow issue should carry structured repair"));
    assert!(
        structured_repair
            .to_string()
            .contains("add_or_retarget_unconditional_default_flow")
    );
}
