use super::evidence::{augment_evidence, decision_display, root_context};
use crate::dmn::{
    validate_dmn_context_expression_syntax, validate_dmn_literal_expression_syntax,
    validate_dmn_relation_expression_syntax,
};
use crate::dmn_model_api::{DmnDecisionDefinition, DmnDocumentSnapshot};
use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn issue_from_dmn_literal_expression_error(
    error: &BpmnEngineError,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_list_unsupported_child",
        } => unsupported_list_child_issue(snapshot),
        BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_context_unsupported_child",
        } => unsupported_context_child_issue(snapshot),
        BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_relation_unsupported_child",
        } => unsupported_relation_child_issue(snapshot),
        _ => return None,
    })
}

pub(super) fn issue_from_dmn_literal_expression_contract(
    decisions: &[DmnDecisionDefinition],
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    issue_from_direct_literal_expression_contract(decisions, snapshot)
        .or_else(|| issue_from_list_expression_contract(decisions, snapshot))
        .or_else(|| issue_from_context_expression_contract(decisions, snapshot))
        .or_else(|| issue_from_relation_expression_contract(decisions, snapshot))
        .or_else(|| issue_from_invocation_expression_contract(decisions, snapshot))
}

fn unsupported_list_child_issue(snapshot: Option<&DmnDocumentSnapshot>) -> LintIssue {
    let decision_id = snapshot
        .and_then(|snapshot| {
            snapshot
                .decisions
                .iter()
                .find(|decision| decision.list_count > 0)
        })
        .map_or("<unknown>", |decision| decision.decision_id.as_str());
    LintIssue::new(
        "dmn.unsupported_list_child",
        "DMN list contains a non-literal child",
        format!(
            "{} contains a direct `<list>` child that is not a direct `<literalExpression>`.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded DMN list runtime executes only direct lists made of direct `literalExpression` items.{}",
            root_context(snapshot)
        ),
        vec![
            "Keep the existing decision id and list item ordering.".to_string(),
            "Replace each direct list child with one direct `<literalExpression><text>...</text></literalExpression>` item.".to_string(),
            "If a list item needs context, invocation, function, relation, or nested-list semantics, keep the source non-executable until that FEEL subset is expanded.".to_string(),
        ],
        "Rewrite the direct `<list>` so every direct child is a supported `<literalExpression>` item. Do not flatten nested boxed expressions into guessed decision-table rules.".to_string(),
        augment_evidence(
            json!({
                "operation": "parse_dmn_list_unsupported_child",
            }),
            snapshot,
            (decision_id != "<unknown>").then_some(decision_id),
        ),
    )
}

fn unsupported_context_child_issue(snapshot: Option<&DmnDocumentSnapshot>) -> LintIssue {
    let decision_id = snapshot
        .and_then(|snapshot| {
            snapshot
                .decisions
                .iter()
                .find(|decision| decision.context_count > 0)
        })
        .map_or("<unknown>", |decision| decision.decision_id.as_str());
    LintIssue::new(
        "dmn.unsupported_context_child",
        "DMN context contains an unsupported child",
        format!(
            "{} contains a direct `<context>` child outside the bounded context-entry subset.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded DMN context runtime executes only direct `contextEntry` children with optional `variable name` metadata and one direct `literalExpression` body.{}",
            root_context(snapshot)
        ),
        vec![
            "Keep the existing decision id and context-entry ordering.".to_string(),
            "Rewrite each direct context child as one `<contextEntry>` with at most one `<variable name=\"...\"/>` and one direct `<literalExpression><text>...</text></literalExpression>` body.".to_string(),
            "If an entry needs invocation, function, relation, nested context, or nested-list semantics, keep the source non-executable until that FEEL subset is expanded.".to_string(),
        ],
        "Rewrite the direct `<context>` so every entry fits the bounded context-entry literal-expression subset. Do not flatten context entries into guessed decision-table rules.".to_string(),
        augment_evidence(
            json!({
                "operation": "parse_dmn_context_unsupported_child",
            }),
            snapshot,
            (decision_id != "<unknown>").then_some(decision_id),
        ),
    )
}

fn unsupported_relation_child_issue(snapshot: Option<&DmnDocumentSnapshot>) -> LintIssue {
    let decision_id = snapshot
        .and_then(|snapshot| {
            snapshot
                .decisions
                .iter()
                .find(|decision| decision.relation_count > 0)
        })
        .map_or("<unknown>", |decision| decision.decision_id.as_str());
    LintIssue::new(
        "dmn.unsupported_relation_child",
        "DMN relation contains an unsupported child",
        format!(
            "{} contains a direct `<relation>` child outside the bounded column/row/cell subset.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded DMN relation runtime executes only direct `column` metadata plus direct `row` children whose cells are direct `literalExpression` bodies.{}",
            root_context(snapshot)
        ),
        vec![
            "Keep the existing decision id, column order, and row ordering.".to_string(),
            "Rewrite each row cell as one direct `<literalExpression><text>...</text></literalExpression>` body.".to_string(),
            "If a cell needs context, invocation, function, relation, or nested-list semantics, keep the source non-executable until that FEEL subset is expanded.".to_string(),
        ],
        "Rewrite the direct `<relation>` so every row contains only supported direct literal-expression cells. Do not flatten relation rows into guessed decision-table rules.".to_string(),
        augment_evidence(
            json!({
                "operation": "parse_dmn_relation_unsupported_child",
            }),
            snapshot,
            (decision_id != "<unknown>").then_some(decision_id),
        ),
    )
}

fn issue_from_direct_literal_expression_contract(
    decisions: &[DmnDecisionDefinition],
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    decisions.iter().find_map(|decision| {
        let literal = decision.literal_expression.as_ref()?;
        validate_dmn_literal_expression_syntax(decision.source_id.as_ref(), literal.text.as_ref())
            .err()
            .map(|_| {
                unsupported_literal_expression_subset_issue(
                    decision.decision.decision_id.as_ref(),
                    literal.text.as_ref(),
                    snapshot,
                )
            })
    })
}

fn issue_from_list_expression_contract(
    decisions: &[DmnDecisionDefinition],
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    decisions.iter().find_map(|decision| {
        let list = decision.list_expression.as_ref()?;
        list.items.iter().find_map(|item| {
            validate_dmn_literal_expression_syntax(decision.source_id.as_ref(), item.text.as_ref())
                .err()
                .map(|_| {
                    unsupported_list_expression_subset_issue(
                        decision.decision.decision_id.as_ref(),
                        item.text.as_ref(),
                        snapshot,
                    )
                })
        })
    })
}

fn issue_from_context_expression_contract(
    decisions: &[DmnDecisionDefinition],
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    decisions.iter().find_map(|decision| {
        let context = decision.context_expression.as_ref()?;
        validate_dmn_context_expression_syntax(decision.source_id.as_ref(), context)
            .err()
            .map(|_| {
                unsupported_context_expression_subset_issue(
                    decision.decision.decision_id.as_ref(),
                    context
                        .entries
                        .iter()
                        .map(|entry| entry.expression.text.as_ref())
                        .find(|text| {
                            validate_dmn_literal_expression_syntax(
                                decision.source_id.as_ref(),
                                text,
                            )
                            .is_err()
                        })
                        .unwrap_or("<context-shape>"),
                    snapshot,
                )
            })
    })
}

fn issue_from_relation_expression_contract(
    decisions: &[DmnDecisionDefinition],
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    decisions.iter().find_map(|decision| {
        let relation = decision.relation_expression.as_ref()?;
        validate_dmn_relation_expression_syntax(decision.source_id.as_ref(), relation)
            .err()
            .map(|_| {
                unsupported_relation_expression_subset_issue(
                    decision.decision.decision_id.as_ref(),
                    relation
                        .rows
                        .iter()
                        .flat_map(|row| &row.cells)
                        .map(|cell| cell.text.as_ref())
                        .find(|text| {
                            validate_dmn_literal_expression_syntax(
                                decision.source_id.as_ref(),
                                text,
                            )
                            .is_err()
                        })
                        .unwrap_or("<relation-shape>"),
                    snapshot,
                )
            })
    })
}

fn issue_from_invocation_expression_contract(
    decisions: &[DmnDecisionDefinition],
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    decisions.iter().find_map(|decision| {
        let invocation = decision.invocation.as_ref()?;
        issue_from_single_invocation_contract(decision, invocation, snapshot)
    })
}

fn issue_from_single_invocation_contract(
    decision: &DmnDecisionDefinition,
    invocation: &crate::dmn_model_api::DmnInvocation,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    let decision_id = decision.decision.decision_id.as_ref();
    let Some(target_name) = invocation_target_name(invocation) else {
        return Some(unsupported_invocation_contract_issue(
            decision_id,
            "Use one simple same-source BKM id or invocable variable name as the direct invocation target.",
            snapshot,
        ));
    };

    let snapshot = snapshot?;
    let target = match resolve_snapshot_invocation_target(snapshot, decision, target_name) {
        SnapshotInvocationTarget::Found(target) => target,
        SnapshotInvocationTarget::Missing => {
            return Some(unsupported_invocation_contract_issue(
                decision_id,
                "Add one same-source top-level `businessKnowledgeModel` whose `id` or nested invocable `variable name` matches the invocation target text.",
                Some(snapshot),
            ));
        }
        SnapshotInvocationTarget::Ambiguous => {
            return Some(unsupported_invocation_contract_issue(
                decision_id,
                "Make the invocation target resolve to exactly one same-source `businessKnowledgeModel`; duplicate local ids or invocable variable names are not executable in this slice.",
                Some(snapshot),
            ));
        }
        SnapshotInvocationTarget::OutsideDeclaredKnowledge => {
            return Some(unsupported_invocation_contract_issue(
                decision_id,
                "Make the invocation target match one direct same-source `<requiredKnowledge href=\"#...\">` edge already declared on the decision, or add the missing same-source required-knowledge edge before expecting the invocation to execute locally.",
                Some(snapshot),
            ));
        }
    };

    issue_from_invocation_body_contract(decision, target, snapshot)
        .or_else(|| issue_from_invocation_binding_contract(decision, invocation, snapshot))
}

fn invocation_target_name(invocation: &crate::dmn_model_api::DmnInvocation) -> Option<&str> {
    invocation
        .invoked_expression
        .as_ref()
        .map(|expression| expression.text.trim())
        .filter(|text| is_simple_identifier(text))
}

enum SnapshotInvocationTarget<'a> {
    Missing,
    Ambiguous,
    OutsideDeclaredKnowledge,
    Found(&'a crate::dmn_model_api::DmnBusinessKnowledgeModelSnapshot),
}

fn resolve_snapshot_invocation_target<'a>(
    snapshot: &'a DmnDocumentSnapshot,
    decision: &DmnDecisionDefinition,
    target_name: &str,
) -> SnapshotInvocationTarget<'a> {
    let required_knowledge_targets =
        resolve_snapshot_required_knowledge_targets(snapshot, decision).transpose();
    let Ok(required_knowledge_targets) = required_knowledge_targets else {
        return SnapshotInvocationTarget::Missing;
    };
    let matches = match required_knowledge_targets.as_ref() {
        Some(targets) => targets
            .iter()
            .copied()
            .filter(|business_knowledge_model| {
                business_knowledge_model
                    .business_knowledge_model_id
                    .as_deref()
                    == Some(target_name)
                    || business_knowledge_model
                        .variable
                        .as_ref()
                        .and_then(|variable| variable.name.as_deref())
                        == Some(target_name)
            })
            .collect::<Vec<_>>(),
        None => snapshot
            .root
            .business_knowledge_models
            .iter()
            .filter(|business_knowledge_model| {
                business_knowledge_model
                    .business_knowledge_model_id
                    .as_deref()
                    == Some(target_name)
                    || business_knowledge_model
                        .variable
                        .as_ref()
                        .and_then(|variable| variable.name.as_deref())
                        == Some(target_name)
            })
            .collect::<Vec<_>>(),
    };

    match matches.as_slice() {
        [target] => SnapshotInvocationTarget::Found(target),
        [] if required_knowledge_targets.is_some() => {
            SnapshotInvocationTarget::OutsideDeclaredKnowledge
        }
        [] => SnapshotInvocationTarget::Missing,
        _ => SnapshotInvocationTarget::Ambiguous,
    }
}

fn resolve_snapshot_required_knowledge_targets<'a>(
    snapshot: &'a DmnDocumentSnapshot,
    decision: &DmnDecisionDefinition,
) -> Option<Result<Vec<&'a crate::dmn_model_api::DmnBusinessKnowledgeModelSnapshot>, ()>> {
    if decision.knowledge_requirements.is_empty() {
        return None;
    }

    let mut targets = Vec::with_capacity(decision.knowledge_requirements.len());
    for requirement in &decision.knowledge_requirements {
        let Some(href) = requirement.href.as_deref() else {
            return Some(Err(()));
        };
        let Some(target_id) = href.strip_prefix('#').filter(|target| !target.is_empty()) else {
            return Some(Err(()));
        };
        let Some(target) =
            snapshot
                .root
                .business_knowledge_models
                .iter()
                .find(|business_knowledge_model| {
                    business_knowledge_model
                        .business_knowledge_model_id
                        .as_deref()
                        == Some(target_id)
                })
        else {
            return Some(Err(()));
        };
        targets.push(target);
    }
    Some(Ok(targets))
}

fn issue_from_invocation_body_contract(
    decision: &DmnDecisionDefinition,
    target: &crate::dmn_model_api::DmnBusinessKnowledgeModelSnapshot,
    snapshot: &DmnDocumentSnapshot,
) -> Option<LintIssue> {
    let decision_id = decision.decision.decision_id.as_ref();
    let Some(logic) = target.encapsulated_logic.as_ref() else {
        return Some(unsupported_invocation_contract_issue(
            decision_id,
            "Keep one same-source `businessKnowledgeModel`, but add one direct `encapsulatedLogic` block with one supported literal-expression body before expecting the invocation to execute locally.",
            Some(snapshot),
        ));
    };
    let Some(body_text) = logic
        .body
        .as_ref()
        .and_then(|body| body.text.as_deref())
        .map(str::trim)
        .filter(|text| !text.is_empty())
    else {
        return Some(unsupported_invocation_contract_issue(
            decision_id,
            "Keep the BKM target, but give its `encapsulatedLogic` one non-empty direct `<literalExpression><text>...</text></literalExpression>` body.",
            Some(snapshot),
        ));
    };
    validate_dmn_literal_expression_syntax(decision.source_id.as_ref(), body_text)
        .err()
        .map(|_| {
            unsupported_invocation_contract_issue(
                decision_id,
                "Reduce the BKM body to one supported bounded literal expression such as one literal, one variable path, or one simple numeric path operation.",
                Some(snapshot),
            )
        })
}

fn issue_from_invocation_binding_contract(
    decision: &DmnDecisionDefinition,
    invocation: &crate::dmn_model_api::DmnInvocation,
    snapshot: &DmnDocumentSnapshot,
) -> Option<LintIssue> {
    for binding in &invocation.bindings {
        let Some(parameter_name) = binding
            .parameter
            .as_ref()
            .and_then(|parameter| parameter.name.as_deref())
            .filter(|name| is_simple_identifier(name))
        else {
            return Some(unsupported_invocation_contract_issue(
                decision.decision.decision_id.as_ref(),
                "Each direct invocation binding must expose one simple named `parameter` so the local runtime can map the bound value into the BKM scope.",
                Some(snapshot),
            ));
        };
        let Some(argument_text) = binding
            .argument
            .as_ref()
            .map(|argument| argument.text.as_ref())
        else {
            return Some(unsupported_invocation_contract_issue(
                decision.decision.decision_id.as_ref(),
                "Each direct invocation binding must include one direct literal-expression argument body.",
                Some(snapshot),
            ));
        };
        if validate_dmn_literal_expression_syntax(decision.source_id.as_ref(), argument_text)
            .is_err()
        {
            return Some(unsupported_invocation_contract_issue(
                decision.decision.decision_id.as_ref(),
                &format!(
                    "Rewrite binding parameter '{parameter_name}' so its argument stays within the supported bounded literal-expression subset."
                ),
                Some(snapshot),
            ));
        }
    }
    None
}

fn unsupported_literal_expression_subset_issue(
    decision_id: &str,
    literal: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_literal_expression_subset",
        "DMN literal expression is outside the supported executable subset",
        format!(
            "{} uses direct `<literalExpression>` text '{literal}', which exceeds the bounded direct-expression runtime.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded DMN runtime executes direct literal expressions only when the text is one supported literal, one variable path, or one whitespace-delimited numeric path operation such as `applicant.age + 1`.{}",
            root_context(snapshot)
        ),
        vec![
            "Keep the existing decision id and name.".to_string(),
            "If the expression is a constant string, quote it explicitly instead of relying on a bare token.".to_string(),
            "If the expression reads input data, reduce it to one variable path such as `applicant.age`.".to_string(),
            "If the expression performs arithmetic, reduce it to one whitespace-delimited `path + number` or `path - number` operation.".to_string(),
            "For broader FEEL logic, convert it to an explicit bounded `<decisionTable>` or keep it non-executable until the FEEL subset is expanded.".to_string(),
        ],
        format!(
            "Edit decision '{decision_id}' so its direct `<literalExpression>` text '{literal}' uses one supported bounded form: quoted string, number, boolean, `null`, ISO date/time/datetime/duration literal, variable path like `applicant.age`, or one whitespace-delimited numeric operation like `applicant.age + 1` or `applicant.age - 1`. Do not approximate broader FEEL expressions; convert them to an explicit bounded decision table only when the mapping is lossless."
        ),
        augment_evidence(
            json!({
                "decision_id": decision_id,
                "literal_expression": literal,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}

fn unsupported_invocation_contract_issue(
    decision_id: &str,
    bounded_fix: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_invocation_contract",
        "DMN invocation is outside the supported local callable subset",
        format!(
            "{} uses direct `<invocation>` logic, but at least one part of the callable target, binding, or body contract is outside the bounded local runtime subset.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded evaluator executes direct invocations only when the invoked text resolves to exactly one same-source top-level `businessKnowledgeModel` by `id` or invocable `variable name`, every binding exposes one simple named parameter plus one supported literal-expression argument, the target `encapsulatedLogic` provides one supported direct literal-expression body, and any preserved executable `<requiredKnowledge>` edges on the decision also point at that same-source target.{}",
            root_context(snapshot)
        ),
        vec![
            "Preserve the existing decision id, invocation id, invoked-expression text, and binding order.".to_string(),
            bounded_fix.to_string(),
            "Do not fabricate or inline broader BKM semantics when the target, parameter contract, or body logic is missing; keep the source non-executable until the callable contract is explicit and lossless.".to_string(),
        ],
        format!(
            "Inspect decision '{decision_id}' and keep its direct `<invocation>` honest. Preserve the reported invocation evidence, then repair only the missing same-source BKM target, parameter names, binding arguments, or BKM literal body needed for the bounded local callable subset. Do not invent broader BKM logic or guessed decision-table rules."
        ),
        augment_evidence(
            json!({
                "decision_id": decision_id,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}

fn is_simple_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn unsupported_list_expression_subset_issue(
    decision_id: &str,
    literal: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_list_expression_subset",
        "DMN list item is outside the supported executable subset",
        format!(
            "{} uses direct `<list>` item text '{literal}', which exceeds the bounded list runtime.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded DMN runtime executes direct lists only when every direct `literalExpression` item is one supported literal, one variable path, or one whitespace-delimited numeric path operation such as `applicant.age + 1`.{}",
            root_context(snapshot)
        ),
        vec![
            "Keep the existing decision id and list item ordering.".to_string(),
            "If an item is a constant string, quote it explicitly instead of relying on a bare token.".to_string(),
            "If an item reads input data, reduce it to one variable path such as `applicant.age`.".to_string(),
            "If an item performs arithmetic, reduce it to one whitespace-delimited `path + number` or `path - number` operation.".to_string(),
            "For broader FEEL list items, convert the decision to an explicit bounded `<decisionTable>` or keep it non-executable until the FEEL subset is expanded.".to_string(),
        ],
        format!(
            "Edit decision '{decision_id}' so direct `<list>` item '{literal}' uses one supported bounded form: quoted string, number, boolean, `null`, ISO date/time/datetime/duration literal, variable path like `applicant.age`, or one whitespace-delimited numeric operation like `applicant.age + 1` or `applicant.age - 1`. Preserve list item order and do not flatten the list into guessed decision-table rules."
        ),
        augment_evidence(
            json!({
                "decision_id": decision_id,
                "list_item_expression": literal,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}

fn unsupported_context_expression_subset_issue(
    decision_id: &str,
    literal: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_context_expression_subset",
        "DMN context entry is outside the supported executable subset",
        format!(
            "{} uses direct `<context>` entry text '{literal}', which exceeds the bounded context runtime.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded DMN runtime executes direct contexts only when each context entry contains one supported literal expression and unnamed result entries appear only as the final entry.{}",
            root_context(snapshot)
        ),
        vec![
            "Keep the existing decision id and context-entry ordering.".to_string(),
            "If an entry is a constant string, quote it explicitly instead of relying on a bare token.".to_string(),
            "If an entry reads input data or a prior context variable, reduce it to one variable path such as `applicant.age` or `nextAge`.".to_string(),
            "If an entry performs arithmetic, reduce it to one whitespace-delimited `path + number` or `path - number` operation.".to_string(),
            "For broader FEEL context entries, convert the decision to an explicit bounded `<decisionTable>` or keep it non-executable until the FEEL subset is expanded.".to_string(),
        ],
        format!(
            "Edit decision '{decision_id}' so direct `<context>` entry '{literal}' uses one supported bounded form: quoted string, number, boolean, `null`, ISO date/time/datetime/duration literal, variable path like `applicant.age` or `nextAge`, or one whitespace-delimited numeric operation like `applicant.age + 1` or `nextAge - 1`. Preserve entry order and do not flatten the context into guessed decision-table rules."
        ),
        augment_evidence(
            json!({
                "decision_id": decision_id,
                "context_entry_expression": literal,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}

fn unsupported_relation_expression_subset_issue(
    decision_id: &str,
    literal: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.unsupported_relation_expression_subset",
        "DMN relation cell is outside the supported executable subset",
        format!(
            "{} uses direct `<relation>` cell text '{literal}', which exceeds the bounded relation runtime.",
            decision_display(decision_id, snapshot)
        ),
        format!(
            "The bounded DMN runtime executes direct relations only when each row cell contains one supported literal expression and every row matches the relation column count.{}",
            root_context(snapshot)
        ),
        vec![
            "Keep the existing decision id, column order, and row ordering.".to_string(),
            "If a cell is a constant string, quote it explicitly instead of relying on a bare token.".to_string(),
            "If a cell reads input data, reduce it to one variable path such as `applicant.age`.".to_string(),
            "If a cell performs arithmetic, reduce it to one whitespace-delimited `path + number` or `path - number` operation.".to_string(),
            "For broader FEEL relation cells, convert the decision to an explicit bounded `<decisionTable>` or keep it non-executable until the FEEL subset is expanded.".to_string(),
        ],
        format!(
            "Edit decision '{decision_id}' so direct `<relation>` cell '{literal}' uses one supported bounded form: quoted string, number, boolean, `null`, ISO date/time/datetime/duration literal, variable path like `applicant.age`, or one whitespace-delimited numeric operation like `applicant.age + 1` or `applicant.age - 1`. Preserve column and row ordering and do not flatten the relation into guessed decision-table rules."
        ),
        augment_evidence(
            json!({
                "decision_id": decision_id,
                "relation_cell_expression": literal,
            }),
            snapshot,
            Some(decision_id),
        ),
    )
}
