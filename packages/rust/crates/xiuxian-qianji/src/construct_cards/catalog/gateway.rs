use super::{ConstructCard, ConstructLintMapping};
use crate::construct_cards::ConstructStatus;

pub(super) const fn card(lint_mappings: &'static [ConstructLintMapping]) -> ConstructCard {
    ConstructCard {
        id: "gateway.exclusive.bounded",
        title: "Bounded Exclusive Gateway",
        domain: "bpmn",
        status: ConstructStatus::Stable,
        summary: "Route one branch using qianji's bounded condition subset.",
        purpose: "Use when one declared workflow variable decides which path runs next.",
        requires: &[
            "condition variables are declared upstream outputs",
            "fallback branch uses the gateway default attribute only when the gateway has two or more outgoing sequence flows",
            "rich decisions are normalized before the gateway",
        ],
        allows: &[
            "plain boolean path such as approved",
            "negated boolean path such as not approved",
            "numeric comparison such as retryCount >= 3",
        ],
        forbids: &[
            "${...}",
            "== true or == false",
            "&& or ||",
            "string comparisons",
            "function calls, scripts, or FEEL expressions",
            "a default attribute on a gateway with only one outgoing sequence flow",
            "a gateway that has only one outgoing sequence flow; connect directly instead",
            "gateway nodes inside WorkflowPlan tasks",
        ],
        example: r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  targetNamespace="https://qianji.dev/examples">
  <process id="Process_Gateway" isExecutable="true">
    <exclusiveGateway id="Gateway_Approved" default="Flow_Revise"/>
    <sequenceFlow id="Flow_Approved" sourceRef="Gateway_Approved" targetRef="Task_Next">
      <conditionExpression xsi:type="tFormalExpression">approved</conditionExpression>
    </sequenceFlow>
    <sequenceFlow id="Flow_Revise" sourceRef="Gateway_Approved" targetRef="Task_Revise"/>
  </process>
</definitions>"#,
        lint_mappings,
        next_cards: &["service-task.agent", "dmn.decision-table.unique"],
    }
}
