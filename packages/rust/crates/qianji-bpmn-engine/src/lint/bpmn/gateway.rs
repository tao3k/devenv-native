use crate::lint_api::LintIssue;
use serde_json::{Value, json};

pub(super) fn gateway_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    match detail {
        "condition_expression_requires_conditional_gateway" => {
            condition_expression_requires_conditional_gateway_issue(process_id, node_id, detail)
        }
        "default_flow_requires_multiple_outgoing" => {
            default_flow_requires_multiple_outgoing_issue(process_id, node_id, detail)
        }
        "unknown_default_flow" | "default_flow_not_outgoing" => {
            invalid_default_flow_issue(process_id, node_id, detail)
        }
        "default_flow_must_not_have_condition_expression" => {
            default_flow_must_not_have_condition_expression_issue(process_id, node_id, detail)
        }
        "missing_condition_expression" => {
            missing_condition_expression_issue(process_id, node_id, detail)
        }
        "unsupported_condition_expression" => {
            unsupported_condition_expression_issue(process_id, node_id, detail)
        }
        "no_matching_condition_or_default" => {
            no_matching_condition_or_default_issue(process_id, node_id, detail)
        }
        "unresolved_condition_variable" => {
            unresolved_condition_variable_issue(process_id, node_id, detail)
        }
        "inclusive_gateway_requires_structured_split_or_join"
        | "inclusive_join_default_not_supported"
        | "inclusive_join_condition_expression_not_supported"
        | "inclusive_split_missing_join"
        | "inclusive_split_branch_not_linear"
        | "inclusive_split_branch_unsupported_gateway"
        | "inclusive_split_branch_duplicate_join_input"
        | "inclusive_split_branch_mismatched_join"
        | "inclusive_split_branch_ends_before_join"
        | "inclusive_join_missing_activation_hint"
        | "inclusive_join_missing_peer_token" => {
            structured_inclusive_gateway_issue(process_id, node_id, detail)
        }
        _ => generic_gateway_configuration_issue(process_id, node_id, detail),
    }
}

fn condition_expression_requires_conditional_gateway_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Sequence-flow condition requires a bounded conditional gateway source",
        format!(
            "Process '{process_id}' gateway or source node '{node_id}' uses `conditionExpression` on a sequence flow whose source is not a bounded conditional gateway."
        ),
        "The bounded engine currently supports `conditionExpression` only on outgoing sequence flows of one branching `exclusiveGateway` or one structured diverging `inclusiveGateway`.",
        vec![
            "Move the conditional routing behind one `exclusiveGateway` or one structured diverging `inclusiveGateway`, or remove the `conditionExpression` from the current non-conditional source flow.".to_string(),
            "Keep the condition as one simple boolean variable path such as `approved` or `not approved`, or one numeric comparison such as `amount > 100`, once it is attached to the bounded gateway branch.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' so node '{node_id}' uses `conditionExpression` only on outgoing sequence flows of one bounded `exclusiveGateway` or one structured diverging `inclusiveGateway`. Preserve workflow intent, but do not leave conditional sequence flows attached to non-conditional sources."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
    .with_structured_repair(gateway_repair_plan(
        "move_condition_to_bounded_gateway",
        process_id,
        node_id,
        detail,
        vec![
            json!({
                "op": "insert_or_reuse_gateway",
                "element": "exclusiveGateway",
                "construct_card": "gateway.exclusive.bounded",
                "reason": "conditionExpression must be owned by a bounded conditional gateway"
            }),
            json!({
                "op": "move_condition_expression",
                "from": "non_gateway_sequence_flow",
                "to": "outgoing_gateway_sequence_flow"
            }),
        ],
    ))
}

fn default_flow_requires_multiple_outgoing_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Conditional gateway default flow requires branching",
        format!(
            "Process '{process_id}' gateway '{node_id}' declares a `default` flow without two or more outgoing sequence flows."
        ),
        "A bounded default flow is only meaningful when one conditional gateway branches across multiple outgoing sequence flows.",
        vec![
            "Either add the missing additional outgoing branches to this conditional gateway or remove the `default` attribute.".to_string(),
            "Keep one unconditional fallback flow as the `default` branch only when the gateway has multiple outgoing sequence flows.".to_string(),
        ],
        format!(
            "Edit gateway '{node_id}' in process '{process_id}' so the `default` attribute is used only when that bounded conditional gateway has multiple outgoing sequence flows. Preserve workflow intent, but do not leave a default branch on a single-route gateway."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
    .with_structured_repair(gateway_repair_plan(
        "remove_or_complete_default_branch",
        process_id,
        node_id,
        detail,
        vec![
            json!({
                "op": "choose_one",
                "options": [
                    {
                        "op": "remove_gateway_default_attribute",
                        "when": "the gateway has only one real outgoing path"
                    },
                    {
                        "op": "add_branching_gateway_flow",
                        "when": "the workflow really needs conditional branching",
                        "requires": "at least one conditional branch plus one unconditional default branch"
                    }
                ]
            }),
        ],
    ))
}

fn invalid_default_flow_issue(process_id: &str, node_id: &str, detail: &'static str) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Conditional gateway default flow reference is invalid",
        format!(
            "Process '{process_id}' gateway '{node_id}' points `default` at a sequence flow that is missing or not one of its outgoing branches."
        ),
        "The bounded engine requires `default` to name one real outgoing sequence flow owned by that same conditional gateway.",
        vec![
            "Set the gateway `default` attribute to the id of one existing outgoing sequence flow from this conditional gateway.".to_string(),
            "If the referenced flow id is stale, either rename the flow id consistently or retarget `default` to the intended existing branch.".to_string(),
        ],
        format!(
            "Repair gateway '{node_id}' in process '{process_id}' so its `default` attribute points to one existing outgoing sequence flow from that same bounded conditional gateway. Preserve branch intent, but do not leave `default` referencing a missing or unrelated flow."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
    .with_structured_repair(gateway_repair_plan(
        "retarget_default_flow",
        process_id,
        node_id,
        detail,
        vec![json!({
            "op": "set_gateway_default",
            "value": "one existing outgoing sequenceFlow id from this gateway",
            "forbid": "missing flow ids or flow ids owned by another source"
        })],
    ))
}

fn default_flow_must_not_have_condition_expression_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Default bounded conditional branch must stay unconditional",
        format!(
            "Process '{process_id}' gateway '{node_id}' marks one branch as `default`, but that same sequence flow still carries `conditionExpression`."
        ),
        "The bounded engine treats the `default` branch as the unconditional fallback after all non-default branch conditions evaluate to false.",
        vec![
            "Remove `conditionExpression` from the flow named by the gateway `default` attribute.".to_string(),
            "Keep conditions only on non-default outgoing branches, and leave the default branch as the unconditional fallback.".to_string(),
        ],
        format!(
            "Edit gateway '{node_id}' in process '{process_id}' so the sequence flow named by `default` has no `conditionExpression`. Preserve workflow intent, but keep the fallback branch unconditional."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
    .with_structured_repair(gateway_repair_plan(
        "make_default_branch_unconditional",
        process_id,
        node_id,
        detail,
        vec![json!({
            "op": "remove_condition_expression",
            "target": "sequenceFlow named by gateway default"
        })],
    ))
}

fn missing_condition_expression_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Non-default bounded conditional branches need conditions",
        format!(
            "Process '{process_id}' gateway '{node_id}' has one non-default outgoing sequence flow without `conditionExpression`."
        ),
        "In the bounded conditional-gateway slice, every non-default outgoing branch on a branching gateway must carry one supported `conditionExpression`.",
        vec![
            "Add one `conditionExpression` to every non-default outgoing branch of this bounded conditional gateway.".to_string(),
            "Use one simple boolean variable path such as `approved`, `vip`, or `not approved`, or one numeric comparison such as `amount > 100`, and keep exactly one unconditional fallback only through `default`.".to_string(),
        ],
        format!(
            "Repair gateway '{node_id}' in process '{process_id}' so every non-default outgoing sequence flow has one supported `conditionExpression`, and reserve unconditional routing only for the optional `default` branch."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
    .with_structured_repair(gateway_repair_plan(
        "add_missing_branch_condition",
        process_id,
        node_id,
        detail,
        vec![json!({
            "op": "add_condition_expression",
            "target": "every non-default outgoing sequenceFlow",
            "allowed_forms": allowed_gateway_condition_forms()
        })],
    ))
}

fn unsupported_condition_expression_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Conditional gateway condition exceeds the bounded subset",
        format!(
            "Process '{process_id}' gateway '{node_id}' uses a `conditionExpression` that is outside the bounded subset."
        ),
        "The current engine accepts only one bounded gateway-condition subset on exclusive-gateway branches and structured inclusive-gateway branches: simple boolean variable paths such as `approved`, `not approved`, or dotted paths such as `flags.approved`, plus numeric comparisons such as `amount > 100` or `risk >= 7`.",
        vec![
            "Rewrite the branch condition as one simple boolean variable path, optionally prefixed with `not`, or as one numeric comparison from an identifier path to one numeric literal.".to_string(),
            "Do not use FEEL, boolean-literal comparisons like `approved == true`, scripts, function calls, arithmetic, or logical combinations such as `approved and vip` in this bounded slice.".to_string(),
        ],
        format!(
            "Rewrite the `conditionExpression` on gateway '{node_id}' in process '{process_id}' so it stays inside the bounded subset: one boolean variable path like `approved`, `not approved`, or `flags.approved`, or one numeric comparison like `amount > 100` or `risk >= 7`. Preserve workflow intent, but remove FEEL, boolean-literal comparisons, logical combinations, and script-style expressions."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
    .with_structured_repair(gateway_repair_plan(
        "rewrite_condition_to_bounded_subset",
        process_id,
        node_id,
        detail,
        vec![json!({
            "op": "rewrite_condition_expression",
            "allowed_forms": allowed_gateway_condition_forms(),
            "forbidden_forms": [
                "approved == true",
                "approved == false",
                "approved and vip",
                "${approved}",
                "functions",
                "scripts",
                "FEEL expressions"
            ],
            "examples": ["approved", "not approved", "flags.approved", "amount > 100", "risk >= 7"]
        })],
    ))
}

fn no_matching_condition_or_default_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Conditional gateway can dead-end when no branch matches",
        format!(
            "Process '{process_id}' gateway '{node_id}' can reach runtime with no matching branch and no `default` fallback."
        ),
        "A branching conditional gateway in the bounded runtime must either have one condition that evaluates to true at runtime or provide one unconditional `default` sequence flow.",
        vec![
            "Add a `default` flow to this bounded conditional gateway if more than one branch can evaluate to false at the same time.".to_string(),
            "If no fallback is intended, ensure the incoming workflow data always sets one supported condition path to true before reaching this gateway.".to_string(),
        ],
        format!(
            "Repair gateway '{node_id}' in process '{process_id}' so runtime execution cannot stall when all supported branch conditions evaluate to false. Either add one unconditional `default` branch or redesign the upstream data flow so one condition always becomes true."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
    .with_structured_repair(gateway_repair_plan(
        "add_default_or_guarantee_condition",
        process_id,
        node_id,
        detail,
        vec![json!({
            "op": "choose_one",
            "options": [
                {
                    "op": "add_unconditional_default_branch",
                    "preferred": true,
                    "reason": "prevents runtime dead-end when all conditions are false"
                },
                {
                    "op": "guarantee_upstream_boolean_or_numeric_value",
                    "reason": "only valid when source data guarantees one condition will match"
                }
            ]
        })],
    ))
}

fn unresolved_condition_variable_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Conditional gateway condition reads a missing or incompatible variable",
        format!(
            "Process '{process_id}' gateway '{node_id}' evaluates one branch condition against a variable path that is missing or incompatible with the bounded condition subset at runtime."
        ),
        "The bounded runtime resolves each gateway `conditionExpression` against JSON variables and expects the referenced path to exist with a compatible value type: boolean for boolean-path conditions, or finite JSON number for numeric comparisons.",
        vec![
            "Populate the referenced variable path before this gateway runs, and ensure the value type matches the bounded condition subset: `true`/`false` for boolean-path conditions, or one JSON number for numeric comparisons.".to_string(),
            "If the variable is optional, redesign the gateway to use a different guaranteed path plus an unconditional `default` fallback.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' so gateway '{node_id}' reads only existing compatible variable paths at runtime. Preserve workflow intent, but do not leave branch conditions depending on missing values, booleans where numbers are required, or numbers where boolean paths are required."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
    .with_structured_repair(gateway_repair_plan(
        "declare_gateway_condition_variable_upstream",
        process_id,
        node_id,
        detail,
        vec![
            json!({
                "op": "declare_upstream_output",
                "elements": ["qianji:outputs"],
                "construct_cards": ["service-task.agent", "user-task.interaction"],
                "value_type": "boolean for boolean-path conditions, number for numeric comparisons"
            }),
            json!({
                "op": "route_on_declared_output_only",
                "forbid": "gateway conditions that read variables not produced by an earlier qianji task"
            }),
            json!({
                "op": "add_unconditional_default_branch",
                "when": "the variable may be absent at runtime"
            }),
        ],
    ))
}

fn structured_inclusive_gateway_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Inclusive gateway exceeds the structured bounded subset",
        format!(
            "Process '{process_id}' inclusive gateway '{node_id}' uses unsupported configuration '{detail}'."
        ),
        "The current engine supports one structured inclusive-gateway subset only: one diverging split with exactly one incoming and multiple outgoing conditional/default branches, plus one matching converging join with multiple incoming and exactly one unconditional outgoing sequence flow, where every branch follows one linear path to that same join.",
        vec![
            "Rewrite this inclusive gateway pair into one structured split/join fragment: one diverging `inclusiveGateway`, one matching converging `inclusiveGateway`, and one linear path from each branch to that join.".to_string(),
            "Keep branch conditions inside the simple boolean-path subset such as `approved`, `vip`, or `not approved`, and avoid nested gateways, branch endings before the join, or unstructured reachability.".to_string(),
        ],
        format!(
            "Repair inclusive gateway '{node_id}' in process '{process_id}' so it fits the bounded structured subset already supported by the engine: one diverging split with multiple outgoing branches, one matching converging join with exactly one unconditional outgoing sequence flow, and one linear branch path from each split branch to that same join. Preserve workflow intent, but remove unsupported inclusive configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
    .with_structured_repair(gateway_repair_plan(
        "rewrite_inclusive_gateway_to_structured_subset",
        process_id,
        node_id,
        detail,
        vec![json!({
            "op": "rewrite_inclusive_gateway_pair",
            "shape": "one diverging inclusiveGateway, one matching converging inclusiveGateway, linear branch paths only"
        })],
    ))
}

fn generic_gateway_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_gateway_configuration",
        "Gateway configuration exceeds the bounded subset",
        format!(
            "Process '{process_id}' gateway '{node_id}' uses unsupported configuration '{detail}'."
        ),
        "The current engine supports bounded `parallelGateway`, bounded `eventBasedGateway`, bounded `exclusiveGateway` routing with simple boolean-path `conditionExpression` values plus one optional `default` flow, and one structured `inclusiveGateway` split/join subset.",
        vec![
            "Rewrite this gateway so it stays inside the bounded exclusive, structured inclusive, parallel, or event-based subset already supported by the engine.".to_string(),
            "If the workflow needs a richer gateway form such as unstructured inclusive routing or FEEL-based branching, keep the intent documented and defer execution until a later slice lands.".to_string(),
        ],
        format!(
            "Rewrite gateway '{node_id}' in process '{process_id}' so it fits the bounded gateway subset already supported by the engine. Preserve workflow intent, but remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
    .with_structured_repair(gateway_repair_plan(
        "rewrite_gateway_to_supported_construct",
        process_id,
        node_id,
        detail,
        vec![json!({
            "op": "select_supported_gateway_construct",
            "construct_cards": ["gateway.exclusive.bounded"],
            "supported_gateway_elements": ["exclusiveGateway", "inclusiveGateway", "parallelGateway", "eventBasedGateway"]
        })],
    ))
}

fn gateway_repair_plan(
    strategy: &'static str,
    process_id: &str,
    node_id: &str,
    detail: &'static str,
    actions: Vec<Value>,
) -> Value {
    let actions = Value::Array(actions);
    json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.gateway.bounded.v1",
        "strategy": strategy,
        "target": {
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        },
        "construct_cards": ["gateway.exclusive.bounded"],
        "actions": actions,
    })
}

fn allowed_gateway_condition_forms() -> Value {
    json!({
        "boolean_path": {
            "examples": ["approved", "not approved", "flags.approved"],
            "value_type": "boolean"
        },
        "numeric_comparison": {
            "operators": ["==", "!=", ">", ">=", "<", "<="],
            "examples": ["amount > 100", "risk >= 7"],
            "left": "identifier path",
            "right": "finite numeric literal"
        }
    })
}
