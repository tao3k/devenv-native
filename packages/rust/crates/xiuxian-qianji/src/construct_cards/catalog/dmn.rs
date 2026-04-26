use super::{ConstructCard, ConstructLintMapping};
use crate::construct_cards::ConstructStatus;

pub(super) const fn card(lint_mappings: &'static [ConstructLintMapping]) -> ConstructCard {
    ConstructCard {
        id: "dmn.decision-table.unique",
        title: "Unique DMN Decision Table",
        domain: "dmn",
        status: ConstructStatus::Draft,
        summary: "Represent stable tabular business rules with one unique matching rule.",
        purpose: "Use when the task has explicit rule rows that are clearer as DMN than as prompt text.",
        requires: &[
            "stable decision id",
            "declared input expressions",
            "typed outputs",
            "UNIQUE hit policy semantics",
        ],
        allows: &[
            "string, boolean, and numeric typed outputs supported by the engine",
            "wildcard input entries where appropriate",
            "businessRuleTask references from BPMN",
        ],
        forbids: &[
            "using DMN for vague LLM judgment",
            "multiple matching rows under UNIQUE",
            "unreferenced DMN decisions",
        ],
        example: r#"<decision id="risk-decision" name="Risk Decision">
  <decisionTable id="risk_table" hitPolicy="UNIQUE">
    <input id="Input_1"><inputExpression id="InputExpression_1" typeRef="number"><text>risk</text></inputExpression></input>
    <output id="Output_1" name="needsReview" typeRef="boolean"/>
    <rule id="Rule_1"><inputEntry id="InputEntry_1"><text>&gt;= 7</text></inputEntry><outputEntry id="OutputEntry_1"><text>true</text></outputEntry></rule>
  </decisionTable>
</decision>"#,
        lint_mappings,
        next_cards: &["gateway.exclusive.bounded"],
    }
}
