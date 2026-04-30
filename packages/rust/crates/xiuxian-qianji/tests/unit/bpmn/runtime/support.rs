use super::*;

pub(crate) struct BusinessRuleBundlePaths {
    pub(crate) bpmn_path: std::path::PathBuf,
    pub(crate) dmn_path: std::path::PathBuf,
}

pub(crate) fn write_business_rule_bundle(temp_dir: &TempDir) -> BusinessRuleBundlePaths {
    let bpmn_path = temp_dir.path().join("review.bpmn");
    let dmn_path = temp_dir.path().join("loan-decision.dmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_review">
  <bpmn:process id="review" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:businessRuleTask id="review_task" decisionRef="loan-decision" decisionRefSource="loan-decision.dmn">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="review_task_output_approved" name="approved" />
        <bpmn:dataOutput id="review_task_output_path" name="path" />
        <bpmn:outputSet id="review_task_output_set">
          <bpmn:dataOutputRefs>review_task_output_approved</bpmn:dataOutputRefs>
          <bpmn:dataOutputRefs>review_task_output_path</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>review_task_output_approved</bpmn:sourceRef>
        <bpmn:targetRef>approved</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>review_task_output_path</bpmn:sourceRef>
        <bpmn:targetRef>path</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:businessRuleTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review_task" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review_task" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    write_file(
        &dmn_path,
        r#"<definitions xmlns="https://www.omg.org/spec/DMN/20191111/MODEL/"
  id="Definitions_loan"
  name="Loan DRD"
  namespace="http://example.com/dmn">
  <decision id="loan-decision" name="Loan Decision">
    <decisionTable id="decision_table_1" hitPolicy="UNIQUE">
      <input id="input_1" label="risk">
        <inputExpression id="input_expression_1" typeRef="string">
          <text>risk</text>
        </inputExpression>
      </input>
      <output id="output_1" name="approval" label="approval" typeRef="string" />
      <rule id="rule_approve">
        <inputEntry id="input_entry_1">
          <text>"low"</text>
        </inputEntry>
        <outputEntry id="output_entry_1">
          <text>"approve"</text>
        </outputEntry>
      </rule>
      <rule id="rule_review">
        <inputEntry id="input_entry_2">
          <text>-</text>
        </inputEntry>
        <outputEntry id="output_entry_2">
          <text>"review"</text>
        </outputEntry>
      </rule>
    </decisionTable>
  </decision>
</definitions>"#,
    );

    BusinessRuleBundlePaths {
        bpmn_path,
        dmn_path,
    }
}

pub(crate) fn write_wait_bundle(temp_dir: &TempDir) -> std::path::PathBuf {
    let bpmn_path = temp_dir.path().join("wait.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_wait">
  <bpmn:process id="wait_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:intermediateCatchEvent id="wait_message">
      <bpmn:messageEventDefinition messageRef="invoice_received" name="InvoiceReceived" />
    </bpmn:intermediateCatchEvent>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="wait_message" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="wait_message" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

pub(crate) fn write_event_race_bundle(temp_dir: &TempDir) -> std::path::PathBuf {
    let bpmn_path = temp_dir.path().join("event-race.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_event_race">
  <bpmn:process id="event_race" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:eventBasedGateway id="wait_race" />
    <bpmn:intermediateCatchEvent id="wait_message" name="InvoiceReceived">
      <bpmn:messageEventDefinition messageRef="invoice_received" name="InvoiceReceived" />
    </bpmn:intermediateCatchEvent>
    <bpmn:intermediateCatchEvent id="wait_timer" name="RaceTimeout">
      <bpmn:timerEventDefinition>
        <bpmn:timeDuration>PT5M</bpmn:timeDuration>
      </bpmn:timerEventDefinition>
    </bpmn:intermediateCatchEvent>
    <bpmn:endEvent id="message_end" />
    <bpmn:endEvent id="timer_end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="wait_race" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="wait_race" targetRef="wait_message" />
    <bpmn:sequenceFlow id="flow_3" sourceRef="wait_race" targetRef="wait_timer" />
    <bpmn:sequenceFlow id="flow_4" sourceRef="wait_message" targetRef="message_end" />
    <bpmn:sequenceFlow id="flow_5" sourceRef="wait_timer" targetRef="timer_end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

fn write_file(path: &Path, contents: &str) {
    fs::write(path, contents)
        .unwrap_or_else(|error| panic!("should write bundle file {}: {error}", path.display()));
}

pub(crate) fn ok_of<T, E: std::fmt::Debug>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error:?}"),
    }
}

pub(crate) fn err_of<T, E: std::fmt::Debug>(result: Result<T, E>) -> E {
    match result {
        Ok(_) => panic!("expected error result, got Ok value"),
        Err(error) => error,
    }
}
