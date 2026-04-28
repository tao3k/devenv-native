use super::{ConstructCard, ConstructLintMapping};
use crate::construct_cards::ConstructStatus;

pub(super) const fn card(lint_mappings: &'static [ConstructLintMapping]) -> ConstructCard {
    ConstructCard {
        id: "service-task.multi-instance.parallel",
        title: "Parallel Multi-Instance Service Task",
        domain: "bpmn",
        status: ConstructStatus::Draft,
        summary: "Run one service task per item in a collection using bounded parallel multi-instance semantics.",
        purpose: "Use when the source task explicitly needs the same bounded service step to run once per assignment, domain, reviewer, or agent task, and those iterations may run in parallel.",
        requires: &[
            "one serviceTask with qianji extension config",
            "multiInstanceLoopCharacteristics with omitted or isSequential=\"false\"",
            "either loopCardinality or collection-backed loopDataInputRef plus inputDataItem",
            "when collection-backed, qianji:inputs should include the per-iteration inputDataItem name",
            "when aggregating outputs, loopDataOutputRef and outputDataItem are both present",
        ],
        allows: &[
            "bounded parallel fan-out over a JSON array or object variable",
            "per-iteration variable overlay from inputDataItem",
            "optional output aggregation into a different collection variable",
            "bounded completionCondition using a boolean path or completed/active/total counter comparison",
        ],
        forbids: &[
            "hiding per-item parallel dispatch inside one serviceTask prompt when BPMN should own the fan-out",
            "in-place output binding where loopDataOutputRef equals loopDataInputRef",
            "completionCondition expressions outside the bounded qianji subset",
            "parallel gateway fan-out when the branch count is dynamic data",
        ],
        example: r#"<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/examples">
  <bpmn:process id="Process_ParallelAgents" isExecutable="true">
    <bpmn:startEvent id="Start"/>
    <bpmn:sequenceFlow id="Flow_Start_Dispatch" sourceRef="Start" targetRef="Task_RunAgent"/>
    <bpmn:serviceTask id="Task_RunAgent" name="Run one agent task" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Run the focused agent task in agentTask. Return JSON with agentResult.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs>agentTask</qianji:inputs>
          <qianji:outputs>agentResult</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
      <bpmn:multiInstanceLoopCharacteristics isSequential="false">
        <bpmn:loopDataInputRef>agentTasks</bpmn:loopDataInputRef>
        <bpmn:inputDataItem id="agentTask"/>
        <bpmn:loopDataOutputRef>agentResults</bpmn:loopDataOutputRef>
        <bpmn:outputDataItem id="agentResult"/>
      </bpmn:multiInstanceLoopCharacteristics>
    </bpmn:serviceTask>
    <bpmn:sequenceFlow id="Flow_Dispatch_End" sourceRef="Task_RunAgent" targetRef="End"/>
    <bpmn:endEvent id="End"/>
  </bpmn:process>
</bpmn:definitions>"#,
        lint_mappings,
        next_cards: &[
            "service-task.agent",
            "gateway.exclusive.bounded",
            "user-task.interaction",
        ],
    }
}
