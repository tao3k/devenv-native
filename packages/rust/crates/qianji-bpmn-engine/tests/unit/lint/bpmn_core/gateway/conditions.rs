use super::super::{BpmnSourceFile, LintDomain, lint_bpmn_source};

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
