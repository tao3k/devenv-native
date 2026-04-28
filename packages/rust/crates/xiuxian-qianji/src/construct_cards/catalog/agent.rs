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
            "qianji:prompt, qianji:tools, qianji:inputs, and qianji:outputs are present",
            "outputs are declared before any gateway uses them",
            "prompt describes one bounded responsibility",
            "qianji:tools is empty unless the prompt explicitly needs workspace inspection, artifact writing, or command execution",
            "every non-empty qianji:tools declaration has matching qianji:toolScope entries that bound command strings, paths, and side effects",
        ],
        allows: &[
            "qianji extension config",
            "declared input variable names",
            "declared output variable names",
            "empty tools for input-only analysis, summarization, classification, routing, and UI-metadata preparation",
            "write for explicit artifact or file creation",
            "read/grep/find/ls for explicit workspace inspection",
            "bash for explicit shell commands, tests, builds, lint commands, or git operations",
            "qianji:toolScope entries such as exact bash command plus timeout, or read/write path boundaries",
        ],
        forbids: &[
            "implicit outputs consumed by gateways",
            "multiple unrelated responsibilities in one task",
            "no-tool store or rename tasks that only persist a prior userTask result",
            "workflow routing, approval, or retry policy hidden inside prompt prose",
            "BPMN boundary error events for recoverable host failure",
            "bash/read tools for reading declared qianji input variables such as specContent",
        ],
        example: r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/examples">
  <process id="Process_AgentStep" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Check" sourceRef="Start" targetRef="Task_Check"/>
    <serviceTask id="Task_Check" name="Check readiness" implementation="${environment.services.runAgent}">
      <extensionElements>
          <qianji:config>
            <qianji:prompt>Check whether the design is ready. Return JSON with ready.</qianji:prompt>
            <qianji:tools></qianji:tools>
            <qianji:inputs>designNotes</qianji:inputs>
            <qianji:outputs>ready</qianji:outputs>
          </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Check_End" sourceRef="Task_Check" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
        lint_mappings,
        next_cards: &["gateway.exclusive.bounded", "user-task.interaction"],
    }
}
