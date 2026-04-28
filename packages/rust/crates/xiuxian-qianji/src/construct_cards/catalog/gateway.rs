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
            "bare condition paths resolve to JSON booleans at runtime",
            "count-like variables use explicit numeric comparisons",
            "two-way boolean routing uses one conditional true branch plus one unconditional default else branch",
            "fallback branch uses the gateway default attribute only when the gateway has two or more outgoing sequence flows",
            "the default sequenceFlow is one of the gateway's outgoing flows and has no conditionExpression",
            "each non-default outgoing sequenceFlow has one bounded conditionExpression",
            "rich decisions are normalized before the gateway",
        ],
        allows: &[
            "plain boolean path such as approved",
            "negated boolean path such as not approved",
            "numeric comparison such as retryCount >= 3",
            "one unconditional fallback branch named by the gateway default attribute",
            "one boolean branch with condition `ready` and one default branch that means else/not ready",
        ],
        forbids: &[
            "bare count-like conditions such as questionsRemaining; use `questionsRemaining > 0` or a boolean name such as hasMoreQuestions",
            "${...}",
            "== true or == false",
            "!approved; use `not approved` for negation",
            "paired boolean conditions such as `ready` and `not ready` when one branch is the gateway default",
            "&& or ||",
            "string comparisons",
            "function calls, scripts, or FEEL expressions",
            "a default attribute on a gateway with only one outgoing sequence flow",
            "a default attribute that points at a missing sequenceFlow id or another node's flow",
            "conditionExpression on the default sequenceFlow",
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
