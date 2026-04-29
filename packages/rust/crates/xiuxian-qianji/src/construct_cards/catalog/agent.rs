use super::{ConstructCard, ConstructLintMapping};
use crate::construct_cards::ConstructStatus;

pub(super) const fn card(lint_mappings: &'static [ConstructLintMapping]) -> ConstructCard {
    ConstructCard {
        id: "service-task.agent",
        title: "Agent Service Task",
        domain: "bpmn",
        status: ConstructStatus::Draft,
        summary: "Run one host-owned agent step and return declared outputs.",
        purpose: "Use when workflow progress needs an LLM/tool host to perform a bounded unit of work.",
        requires: &[
            "implementation points at the host adapter",
            "documentation gives the host prompt",
            "ioSpecification declares every consumed data input and emitted data output",
            "dataInputAssociation and dataOutputAssociation map workflow variables explicitly",
            "outputs are declared before any gateway uses them",
            "prompt describes one bounded responsibility",
        ],
        allows: &[
            "native BPMN task documentation",
            "declared input variable names",
            "declared output variable names",
            "empty input sets for source tasks",
            "host-owned capability policy outside the BPMN XML envelope",
        ],
        forbids: &[
            "implicit outputs consumed by gateways",
            "multiple unrelated responsibilities in one task",
            "no-tool store or rename tasks that only persist a prior userTask result",
            "workflow routing, approval, or retry policy hidden inside prompt prose",
            "BPMN boundary error events for recoverable host failure",
            "custom QName extension XML for prompt, tool, input, or output metadata",
        ],
        example: r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  targetNamespace="https://example.test/bpmn">
  <process id="Process_AgentStep" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Check" sourceRef="Start" targetRef="Task_Check"/>
    <serviceTask id="Task_Check" name="Check readiness" implementation="${environment.services.runAgent}">
      <documentation>Check whether the design is ready. Return JSON with ready.</documentation>
      <ioSpecification>
        <dataInput id="Task_Check_Input_designNotes" name="designNotes"/>
        <dataOutput id="Task_Check_Output_ready" name="ready"/>
        <inputSet>
          <dataInputRefs>Task_Check_Input_designNotes</dataInputRefs>
        </inputSet>
        <outputSet>
          <dataOutputRefs>Task_Check_Output_ready</dataOutputRefs>
        </outputSet>
      </ioSpecification>
      <dataInputAssociation>
        <sourceRef>designNotes</sourceRef>
        <targetRef>Task_Check_Input_designNotes</targetRef>
      </dataInputAssociation>
      <dataOutputAssociation>
        <sourceRef>Task_Check_Output_ready</sourceRef>
        <targetRef>ready</targetRef>
      </dataOutputAssociation>
    </serviceTask>
    <sequenceFlow id="Flow_Check_End" sourceRef="Task_Check" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
        lint_mappings,
        next_cards: &["gateway.exclusive.bounded", "user-task.interaction"],
    }
}
