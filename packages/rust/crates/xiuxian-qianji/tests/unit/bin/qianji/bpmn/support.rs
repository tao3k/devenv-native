use super::write_file;
use serde_json::Value;
use std::path::PathBuf;
use tempfile::TempDir;

pub(crate) struct BusinessRuleBundlePaths {
    pub(crate) bpmn_path: PathBuf,
    pub(crate) dmn_path: PathBuf,
}

pub(crate) fn write_linear_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("linear.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_linear">
  <bpmn:process id="linear" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

pub(crate) fn write_service_task_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("service-task.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_review">
  <bpmn:process id="review" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="review_task" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review_task" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review_task" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

#[cfg(feature = "sqlite")]
pub(crate) fn write_parallel_multi_instance_loop_input_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir
        .path()
        .join("parallel-multi-instance-loop-input.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_parallel_mi">
  <bpmn:process id="parallel_mi" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="review">
      <bpmn:multiInstanceLoopCharacteristics>
        <bpmn:loopDataInputRef>items</bpmn:loopDataInputRef>
        <bpmn:inputDataItem id="item" />
        <bpmn:loopDataOutputRef>results</bpmn:loopDataOutputRef>
        <bpmn:outputDataItem id="result" />
      </bpmn:multiInstanceLoopCharacteristics>
    </bpmn:serviceTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

pub(crate) fn write_send_task_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("send-task.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_send">
  <bpmn:process id="send_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:sendTask id="send_invoice_message" messageRef="invoice_dispatched" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="send_invoice_message" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="send_invoice_message" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
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
    <bpmn:businessRuleTask id="review_task" decisionRef="loan-decision" decisionRefSource="loan-decision.dmn" />
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

pub(crate) fn write_json_fixture(path: PathBuf, value: &Value) -> PathBuf {
    write_file(
        &path,
        &serde_json::to_string_pretty(value)
            .unwrap_or_else(|error| panic!("host fixture should serialize: {error}")),
    );
    path
}

pub(crate) fn write_event_wait_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("event-wait.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_wait">
  <bpmn:process id="wait_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:intermediateCatchEvent id="wait_message">
      <bpmn:messageEventDefinition messageRef="invoice_received" />
    </bpmn:intermediateCatchEvent>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="wait_message" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="wait_message" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

pub(crate) fn write_event_race_bundle(temp_dir: &TempDir) -> PathBuf {
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

#[cfg(feature = "sqlite")]
pub(crate) fn write_waiting_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("wait.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_wait">
  <bpmn:process id="wait_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:intermediateCatchEvent id="wait_message">
      <bpmn:messageEventDefinition />
    </bpmn:intermediateCatchEvent>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="wait_message" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="wait_message" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}
