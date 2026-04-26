//! Qianji construct-card catalog for LLM-facing progressive disclosure.

use serde::Serialize;

/// Lifecycle status for a construct-card contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructStatus {
    /// Stable enough for downstream compilers to depend on.
    Stable,
    /// Available as guidance while the contract is still being hardened.
    Draft,
}

impl ConstructStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Draft => "draft",
        }
    }
}

/// One lint diagnostic mapping for a construct card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConstructLintMapping {
    /// Diagnostic code emitted by qianji lint.
    pub diagnostic: &'static str,
    /// Human and LLM readable repair guidance.
    pub repair: &'static str,
}

/// One executable construct card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConstructCard {
    /// Stable construct id used by CLI and downstream compilers.
    pub id: &'static str,
    /// Short display title.
    pub title: &'static str,
    /// BPMN or DMN domain.
    pub domain: &'static str,
    /// Lifecycle status.
    pub status: ConstructStatus,
    /// Compact index summary.
    pub summary: &'static str,
    /// When an LLM should choose this construct.
    pub purpose: &'static str,
    /// Required preconditions or neighboring constructs.
    pub requires: &'static [&'static str],
    /// Supported bounded forms.
    pub allows: &'static [&'static str],
    /// Explicit anti-patterns.
    pub forbids: &'static [&'static str],
    /// Minimal BPMN or DMN scaffold for this construct.
    pub example: &'static str,
    /// Lint diagnostic repair hints connected to this construct.
    pub lint_mappings: &'static [ConstructLintMapping],
    /// Follow-up cards that are commonly useful with this card.
    pub next_cards: &'static [&'static str],
}

/// Compact machine-readable index entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConstructIndexEntry {
    /// Stable construct id.
    pub id: &'static str,
    /// BPMN or DMN domain.
    pub domain: &'static str,
    /// Lifecycle status.
    pub status: ConstructStatus,
    /// Compact summary.
    pub summary: &'static str,
}

const GATEWAY_LINT: &[ConstructLintMapping] = &[ConstructLintMapping {
    diagnostic: "bpmn.unsupported_gateway_configuration",
    repair: "Move rich logic into an upstream serviceTask or DMN decision that outputs a declared boolean, then route on the plain variable or use the gateway default branch.",
}];

const TASK_CONFIG_LINT: &[ConstructLintMapping] = &[
    ConstructLintMapping {
        diagnostic: "bpmn.missing_host_task_contract",
        repair: "Add a qianji-owned extension config with prompt, inputs, outputs, and implementation metadata expected by the selected host adapter.",
    },
    ConstructLintMapping {
        diagnostic: "bpmn.unsupported_qianji_interaction_type",
        repair: "Use one supported qianji interaction type: input, confirm, choice, or choice_input. Use input for plain free-form text and choice_input for option selection plus free-form feedback.",
    },
];

const DMN_LINT: &[ConstructLintMapping] = &[ConstructLintMapping {
    diagnostic: "dmn.invalid_decision_table",
    repair: "Keep one explicit decision id, declared inputs, typed outputs, and rules that match the declared hit policy.",
}];

const CONSTRUCT_CARDS: &[ConstructCard] = &[
    ConstructCard {
        id: "service-task.agent",
        title: "Agent Service Task",
        domain: "bpmn",
        status: ConstructStatus::Draft,
        summary: "Run one host-owned agent step and return declared outputs.",
        purpose: "Use when workflow progress needs an LLM/tool host to perform a bounded unit of work.",
        requires: &[
            "implementation points at the host adapter",
            "outputs are declared before any gateway uses them",
            "prompt describes one bounded responsibility",
        ],
        allows: &[
            "qianji extension config",
            "declared input variable names",
            "declared output variable names",
            "host-specific tools when the host adapter supports them",
        ],
        forbids: &[
            "implicit outputs consumed by gateways",
            "multiple unrelated responsibilities in one task",
            "BPMN boundary error events for recoverable host failure",
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
          <qianji:inputs>designNotes</qianji:inputs>
          <qianji:outputs>ready</qianji:outputs>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <sequenceFlow id="Flow_Check_End" sourceRef="Task_Check" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
        lint_mappings: TASK_CONFIG_LINT,
        next_cards: &["gateway.exclusive.bounded", "user-task.interaction"],
    },
    ConstructCard {
        id: "user-task.interaction",
        title: "Structured User Interaction Task",
        domain: "bpmn",
        status: ConstructStatus::Draft,
        summary: "Ask the user a bounded question and map the answer to declared outputs.",
        purpose: "Use when a workflow checkpoint needs human approval, selection, or free-form input.",
        requires: &[
            "userTask with qianji config",
            "interaction type is one of input, confirm, choice, or choice_input",
            "outputs declared for downstream routing",
        ],
        allows: &[
            "input",
            "confirm",
            "choice",
            "choice_input",
            "free-form answer text through input or choice_input",
        ],
        forbids: &[
            "hardcoded downstream prompt behavior outside the card contract",
            "unsupported interaction type values such as free_form",
            "choices whose values do not match downstream output mapping",
            "gateway conditions over undeclared answers",
        ],
        example: r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions"
  targetNamespace="https://qianji.dev/examples">
  <process id="Process_UserInteraction" isExecutable="true">
    <startEvent id="Start"/>
    <sequenceFlow id="Flow_Start_Approve" sourceRef="Start" targetRef="Task_Approve"/>
    <userTask id="Task_Approve" name="Approve design">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Review the proposed design.</qianji:prompt>
          <qianji:outputs>approved,feedback</qianji:outputs>
          <qianji:interaction type="choice_input">
            <qianji:question>Does this design look right?</qianji:question>
            <qianji:choice value="approved" label="Approve">Continue.</qianji:choice>
            <qianji:choice value="changes" label="Request changes">Revise first.</qianji:choice>
            <qianji:freeText name="feedback" optional="true"/>
          </qianji:interaction>
        </qianji:config>
      </extensionElements>
    </userTask>
    <sequenceFlow id="Flow_Approve_End" sourceRef="Task_Approve" targetRef="End"/>
    <endEvent id="End"/>
  </process>
</definitions>"#,
        lint_mappings: TASK_CONFIG_LINT,
        next_cards: &["gateway.exclusive.bounded"],
    },
    ConstructCard {
        id: "gateway.exclusive.bounded",
        title: "Bounded Exclusive Gateway",
        domain: "bpmn",
        status: ConstructStatus::Stable,
        summary: "Route one branch using qianji's bounded condition subset.",
        purpose: "Use when one declared workflow variable decides which path runs next.",
        requires: &[
            "condition variables are declared upstream outputs",
            "fallback branch uses the gateway default attribute",
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
        lint_mappings: GATEWAY_LINT,
        next_cards: &["service-task.agent", "dmn.decision-table.unique"],
    },
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
        lint_mappings: DMN_LINT,
        next_cards: &["gateway.exclusive.bounded"],
    },
];

/// Return all registered construct cards in deterministic index order.
#[must_use]
pub fn construct_cards() -> &'static [ConstructCard] {
    CONSTRUCT_CARDS
}

/// Find a construct card by stable id.
#[must_use]
pub fn find_construct_card(id: &str) -> Option<&'static ConstructCard> {
    CONSTRUCT_CARDS.iter().find(|card| card.id == id)
}

/// Return compact index entries in deterministic order.
#[must_use]
pub fn construct_index_entries(cards: &[ConstructCard]) -> Vec<ConstructIndexEntry> {
    cards
        .iter()
        .map(|card| ConstructIndexEntry {
            id: card.id,
            domain: card.domain,
            status: card.status,
            summary: card.summary,
        })
        .collect()
}

/// Render a compact construct-card table of contents.
#[must_use]
pub fn render_construct_index(cards: &[ConstructCard]) -> String {
    let mut lines = vec![
        "# Qianji Construct Index".to_string(),
        String::new(),
        "Use this as a table of contents after reading the source task or `SKILL.md`. The source file is semantic input, not automatically a workflow artifact.".to_string(),
        String::new(),
        "First decide the scenario shape from the source: autonomous workflow, interactive workflow, or planning workflow that must ask the user before execution. Then select only the cards needed for that scenario and run `qianji construct show <id>` for details.".to_string(),
        String::new(),
        "| ID | Domain | Status | Summary |".to_string(),
        "| --- | --- | --- | --- |".to_string(),
    ];
    for card in cards {
        lines.push(format!(
            "| `{}` | {} | {} | {} |",
            card.id,
            card.domain,
            card.status.as_str(),
            card.summary
        ));
    }
    lines.push(String::new());
    lines.push("Suggested LLM flow: read source skill/task -> classify autonomous vs interactive vs planning scenario -> pick construct ids -> inspect cards -> fill a BPMN or DMN scaffold -> run `qianji lint`.".to_string());
    lines.join("\n")
}

/// Render the construct index as pretty JSON.
///
/// # Errors
///
/// Returns an error if the static catalog cannot be serialized.
pub fn render_construct_index_json(cards: &[ConstructCard]) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&construct_index_entries(cards))
}

/// Render one detailed construct card.
#[must_use]
pub fn render_construct_card(card: &ConstructCard) -> String {
    let mut lines = vec![
        format!("# Qianji Construct Card: {}", card.id),
        String::new(),
        format!("Title: {}", card.title),
        format!("Domain: {}", card.domain),
        format!("Status: {}", card.status.as_str()),
        String::new(),
        "## Purpose".to_string(),
        String::new(),
        card.purpose.to_string(),
        String::new(),
        "## Requires".to_string(),
        String::new(),
    ];
    push_bullets(&mut lines, card.requires);
    lines.extend([String::new(), "## Allows".to_string(), String::new()]);
    push_bullets(&mut lines, card.allows);
    lines.extend([String::new(), "## Forbids".to_string(), String::new()]);
    push_bullets(&mut lines, card.forbids);
    lines.extend([
        String::new(),
        "## Example".to_string(),
        String::new(),
        fence_for_domain(card.domain).to_string(),
        card.example.to_string(),
        "```".to_string(),
        String::new(),
        "## Lint Repair Map".to_string(),
        String::new(),
    ]);
    for mapping in card.lint_mappings {
        lines.push(format!("- `{}`: {}", mapping.diagnostic, mapping.repair));
    }
    lines.extend([String::new(), "## Related Cards".to_string(), String::new()]);
    push_bullets(&mut lines, card.next_cards);
    lines.join("\n")
}

fn fence_for_domain(_domain: &str) -> &'static str {
    "```xml"
}

/// Render one detailed construct card as pretty JSON.
///
/// # Errors
///
/// Returns an error if the static construct card cannot be serialized.
pub fn render_construct_card_json(card: &ConstructCard) -> serde_json::Result<String> {
    serde_json::to_string_pretty(card)
}

fn push_bullets(lines: &mut Vec<String>, values: &[&str]) {
    if values.is_empty() {
        lines.push("- none".to_string());
        return;
    }
    for value in values {
        lines.push(format!("- {value}"));
    }
}
