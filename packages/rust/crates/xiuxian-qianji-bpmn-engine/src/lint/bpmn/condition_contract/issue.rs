use super::{
    AmbiguousBooleanPathKind, BpmnSourceFile, LintIssue, LintSourceDiagnostic, LintSourceSpan,
    StaticInteractionChoiceOutput, UnsupportedGatewayConditionGroup,
    ambiguous_boolean_condition_guidance, ambiguous_boolean_condition_help,
    ambiguous_boolean_condition_repair, find_condition_expression_span, json,
};

pub(super) fn non_boolean_interaction_choice_condition_issue(
    source: &BpmnSourceFile,
    process_id: &str,
    gateway_id: &str,
    condition: &str,
    variable_path: &str,
    interaction_output: &StaticInteractionChoiceOutput,
) -> LintIssue {
    let choice_values = interaction_output.choice_values.join(", ");
    let issue = LintIssue::from_parts(
        "bpmn.non_boolean_interaction_choice_condition",
        "Gateway boolean condition consumes non-boolean interaction choices",
        format!(
            "Process '{process_id}' gateway '{gateway_id}' uses bare boolean condition '{condition}', but userTask '{}' maps static choice values [{choice_values}] into '{variable_path}'.",
            interaction_output.task_id
        ),
        "The bounded runtime evaluates bare gateway condition paths as JSON booleans only. A user interaction result that drives such a gateway must produce a boolean-shaped answer, or an upstream service task must derive a separate boolean output.",
        vec![
            format!(
                "For a boolean branch, make userTask '{}' return boolean/approval values such as `true`/`false`, `yes`/`no`, or `approved`/`rejected` into '{variable_path}'.",
                interaction_output.task_id
            ),
            format!(
                "Prefer a boolean-shaped output name such as `needsMoreQuestions` or `hasMoreQuestions`, then route with `<conditionExpression>{variable_path}</conditionExpression>` only when the host output is a JSON boolean."
            ),
            "If the choices are semantic strings, keep that result as a text answer and add a serviceTask that emits a separate JSON boolean consumed by the gateway.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' by aligning userTask '{}' choice output '{variable_path}' with gateway '{gateway_id}': either use boolean/approval choice values for the gateway variable, or add a serviceTask that converts semantic choice strings into a JSON boolean before routing.",
            interaction_output.task_id
        ),
        json!({
            "process_id": process_id,
            "gateway_id": gateway_id,
            "condition": condition,
            "user_task_id": interaction_output.task_id,
            "output": variable_path,
            "choice_values": interaction_output.choice_values,
            "expected_runtime_type": "json_boolean for bare gateway condition path"
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "bpmn.native.gateway.condition.v1",
        "strategy": "align_interaction_choice_output_with_boolean_gateway",
        "target": {
            "process_id": process_id,
            "gateway_id": gateway_id,
            "user_task_id": interaction_output.task_id,
            "condition": condition,
            "output": variable_path,
            "choice_values": interaction_output.choice_values
        },
        "actions": [{
            "op": "choose_one",
            "examples": [
                "choices dataInput assignment with boolean-compatible values",
                "dataOutput name=\"answer\" associated to needsMoreQuestions",
                "<conditionExpression xsi:type=\"tFormalExpression\">needsMoreQuestions</conditionExpression>"
            ],
            "options": [
                {
                    "op": "use_boolean_choice_values",
                    "examples": [
                        "choices dataInput assignment with values true and false",
                        "dataOutput name=\"answer\" associated to needsMoreQuestions",
                        "<conditionExpression xsi:type=\"tFormalExpression\">needsMoreQuestions</conditionExpression>"
                    ],
                    "requires": "the host maps the interaction result to a JSON boolean before completing the userTask"
                },
                {
                    "op": "derive_boolean_with_service_task",
                    "when": "choice values must remain semantic strings",
                    "requires": "serviceTask outputs a separate JSON boolean such as needsMoreQuestions, and the gateway routes on that boolean"
                }
            ],
            "forbidden_forms": [
                "semantic choice strings such as need_more_clarification routed directly by a bare boolean condition",
                "bare gateway condition over a userTask text answer",
                "string values pretending to be booleans without a host/runtime mapping"
            ]
        }]
    }));

    let Some(span) =
        find_condition_expression_span(source.contents.as_str(), gateway_id, condition)
    else {
        return issue;
    };
    issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(span.start, span.end),
        "this bare gateway condition needs a JSON boolean",
        format!(
            "The condition consumes userTask '{}' choices [{choice_values}] through '{variable_path}'. Convert the interaction result to a boolean output, or derive a separate boolean before this gateway.",
            interaction_output.task_id
        ),
    ))
}

pub(super) fn ambiguous_boolean_condition_issue(
    source: &BpmnSourceFile,
    process_id: &str,
    gateway_id: &str,
    condition: &str,
    variable_path: &str,
    kind: AmbiguousBooleanPathKind,
) -> LintIssue {
    let (title, summary, why_it_failed, repair_guidance, llm_fix_prompt, valid_repairs) =
        ambiguous_boolean_condition_guidance(
            process_id,
            gateway_id,
            condition,
            variable_path,
            kind,
        );
    let issue = LintIssue::from_parts(
        "bpmn.ambiguous_boolean_gateway_condition",
        title,
        summary,
        why_it_failed,
        repair_guidance,
        llm_fix_prompt,
        json!({
            "process_id": process_id,
            "gateway_id": gateway_id,
            "condition": condition,
            "variable_path": variable_path,
            "expected_runtime_type": "boolean for bare path, number for numeric comparison",
            "valid_repairs": valid_repairs
        }),
    )
    .with_structured_repair(ambiguous_boolean_condition_repair(
        process_id,
        gateway_id,
        condition,
        variable_path,
        kind,
    ));

    let Some(span) = find_condition_expression_span(&source.contents, gateway_id, condition) else {
        return issue;
    };
    issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(span.start, span.end),
        "bare condition paths must resolve to JSON boolean values",
        ambiguous_boolean_condition_help(variable_path, kind),
    ))
}

pub(super) fn unsupported_gateway_condition_issue(
    source: &BpmnSourceFile,
    group: UnsupportedGatewayConditionGroup,
) -> LintIssue {
    let process_id = group.process_id;
    let gateway_id = group.gateway_id;
    let conditions = group.conditions;
    let first_condition = conditions.first().cloned().unwrap_or_default();
    let condition_list = conditions
        .iter()
        .map(|condition| format!("`{condition}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let issue = LintIssue::from_parts(
        "bpmn.unsupported_gateway_configuration",
        "Conditional gateway condition exceeds the bounded subset",
        format!(
            "Process '{process_id}' gateway '{gateway_id}' uses unsupported conditionExpression value(s): {condition_list}."
        ),
        "The bounded runtime accepts only boolean variable paths such as `approved` or `not approved`, dotted boolean paths such as `flags.approved`, and numeric comparisons against numeric literals such as `amount > 100`. String or enum equality must be converted into an upstream boolean route output.",
        vec![
            "Rewrite the branch condition as one boolean variable path, optionally prefixed with `not`, or as one numeric comparison from an identifier path to one numeric literal.".to_string(),
            "For string or enum decisions, add one top-level boolean native BPMN output on the upstream task, then route on that boolean output.".to_string(),
            "If the string or enum value comes from a userTask answer output, keep that answer output declared on the userTask and add a following serviceTask to derive route booleans. Do not replace the userTask answer mapping with derived booleans.".to_string(),
            "Do not use string equality, boolean-literal comparisons, variable-to-variable comparisons, function calls, scripts, or logical combinations in gateway conditions.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' gateway '{gateway_id}' by replacing every unsupported conditionExpression in [{condition_list}] with bounded boolean route variables or numeric comparisons. If these conditions compare user choice/status strings, preserve the original userTask answer output, add a following serviceTask that consumes it and emits JSON boolean route outputs, then route on those booleans."
        ),
        json!({
            "process_id": process_id,
            "gateway_id": gateway_id,
            "conditions": conditions,
            "valid_condition_forms": [
                "approved",
                "not approved",
                "flags.approved",
                "amount > 100",
                "risk >= 7"
            ],
            "forbidden_condition_forms": [
                "taskStatus == \"completed\"",
                "choice == 'merge'",
                "approved == true",
                "currentCount > requiredCount",
                "approved and verified"
            ]
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
            "contract": "bpmn.native.gateway.condition.v1",
            "strategy": "rewrite_condition_to_bounded_subset",
            "target": {
                "process_id": process_id,
                "gateway_id": gateway_id,
                "conditions": conditions
            },
            "actions": [{
            "op": "replace_unsupported_conditions",
            "allowed_forms": [
                "boolean variable path",
                "not boolean variable path",
                "numeric variable path compared to numeric literal"
            ],
            "examples": ["taskCompleted", "not blocked", "questionsRemaining > 0"],
            "producer_change": "if the existing condition compares a string/enum/status from a userTask answer output, keep that result output declared on the userTask, then add a following serviceTask that consumes it and declares/emits JSON boolean route outputs",
            "forbid": "replacing a userTask answer mapping with derived booleans while the answer output still points at the original reply",
            "forbidden_forms": [
                "taskStatus == \"completed\"",
                "choice == 'merge'",
                "approved == true",
                "approved and verified",
                "currentCount > requiredCount"
            ]
        }]
    }));

    let Some(span) =
        find_condition_expression_span(&source.contents, &gateway_id, &first_condition)
    else {
        return issue;
    };
    issue.with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(span.start, span.end),
        "rewrite this condition into the bounded native subset",
        "Use a JSON boolean route variable such as `taskCompleted`, or a numeric comparison against a literal. For status or choice strings, emit a separate boolean output upstream and route on it.",
    ))
}
