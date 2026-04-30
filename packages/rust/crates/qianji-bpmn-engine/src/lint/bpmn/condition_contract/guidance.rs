use super::{AmbiguousBooleanPathKind, json};

pub(super) fn ambiguous_boolean_condition_guidance(
    process_id: &str,
    gateway_id: &str,
    condition: &str,
    variable_path: &str,
    kind: AmbiguousBooleanPathKind,
) -> (
    String,
    String,
    String,
    Vec<String>,
    String,
    serde_json::Value,
) {
    match kind {
        AmbiguousBooleanPathKind::CountLike => (
            "Gateway boolean condition uses a count-like variable".to_string(),
            format!(
                "Process '{process_id}' gateway '{gateway_id}' uses boolean condition '{condition}', but variable '{variable_path}' is count-like and commonly resolves to a JSON number."
            ),
            "The bounded runtime evaluates bare gateway condition paths as JSON booleans only. Numeric counters must use an explicit numeric comparison, or the producer must emit a boolean-shaped variable.".to_string(),
            vec![
                format!(
                    "If '{variable_path}' is a count, rewrite the branch condition to `{variable_path} > 0` and ensure the upstream task emits a JSON number."
                ),
                "If the branch is boolean, rename the output to a boolean-shaped variable such as `hasMoreQuestions` or `needsMoreQuestions` and emit only `true` or `false`.".to_string(),
                "Keep the fallback branch as the gateway `default` flow without a conditionExpression.".to_string(),
            ],
            format!(
                "Repair process '{process_id}' gateway '{gateway_id}' by replacing boolean-path condition `{condition}` with either `{variable_path} > 0` for numeric counts, or by renaming the upstream native BPMN output to a boolean-shaped variable and routing on that boolean."
            ),
            json!([
                {
                    "conditionExpression": format!("{variable_path} > 0"),
                    "producer_output_type": "json_number"
                },
                {
                    "conditionExpression": "hasMoreQuestions",
                    "producer_output_type": "json_boolean"
                }
            ]),
        ),
        AmbiguousBooleanPathKind::ContentLike => (
            "Gateway boolean condition uses a content-like variable".to_string(),
            format!(
                "Process '{process_id}' gateway '{gateway_id}' uses boolean condition '{condition}', but variable '{variable_path}' is content-like and commonly resolves to text, arrays, objects, or enum strings."
            ),
            "The bounded runtime evaluates bare gateway condition paths as JSON booleans only. User-facing content, status strings, arrays, and details must stay separate from route booleans.".to_string(),
            vec![
                format!(
                    "Keep '{variable_path}' as content for prompts or task inputs, but do not route directly on it."
                ),
                "Add a separate boolean-shaped route output such as `hasQuestions`, `hasConcerns`, `needsHumanInput`, or `shouldEscalate`, and emit only JSON true or false.".to_string(),
                "Route the gateway on that boolean output, and make every upstream task that can enter this gateway declare and produce the same route boolean.".to_string(),
            ],
            format!(
                "Repair process '{process_id}' gateway '{gateway_id}' by replacing content-like boolean condition `{condition}` with a separate boolean route variable such as `hasQuestions`, `hasConcerns`, `needsHumanInput`, or `shouldEscalate`. Preserve `{variable_path}` for user-facing text or arrays, add that route boolean to upstream native BPMN outputs and task prompts, and route only on the JSON boolean."
            ),
            json!([
                {
                    "conditionExpression": "hasQuestions",
                    "producer_output_type": "json_boolean",
                    "content_variable": variable_path
                },
                {
                    "conditionExpression": "hasConcerns",
                    "producer_output_type": "json_boolean",
                    "content_variable": variable_path
                },
                {
                    "conditionExpression": "shouldEscalate",
                    "producer_output_type": "json_boolean",
                    "content_variable": variable_path
                }
            ]),
        ),
    }
}

pub(super) fn ambiguous_boolean_condition_help(
    variable_path: &str,
    kind: AmbiguousBooleanPathKind,
) -> String {
    match kind {
        AmbiguousBooleanPathKind::CountLike => format!(
            "Use `{variable_path} > 0` for a numeric count, or rename the producer output to a boolean-shaped variable and emit true/false."
        ),
        AmbiguousBooleanPathKind::ContentLike => format!(
            "Keep `{variable_path}` as content, add a separate boolean-shaped route output, and route on that JSON boolean."
        ),
    }
}

pub(super) fn ambiguous_boolean_condition_repair(
    process_id: &str,
    gateway_id: &str,
    condition: &str,
    variable_path: &str,
    kind: AmbiguousBooleanPathKind,
) -> serde_json::Value {
    match kind {
        AmbiguousBooleanPathKind::CountLike => json!({
            "schema_version": 1,
            "contract": "bpmn.native.gateway.condition.v1",
            "strategy": "disambiguate_count_like_boolean_condition",
            "target": {
                "process_id": process_id,
                "gateway_id": gateway_id,
                "condition": condition,
                "variable_path": variable_path
            },
            "actions": [
                {
                    "op": "choose_one",
                    "examples": [
                        format!("<conditionExpression xsi:type=\"tFormalExpression\">{variable_path} > 0</conditionExpression>"),
                        format!("Return JSON with currentQuestion and {variable_path} as a JSON number."),
                        "native BPMN dataOutput names currentQuestion and hasMoreQuestions with <conditionExpression>hasMoreQuestions</conditionExpression>".to_string()
                    ],
                    "forbidden_forms": [
                        format!("<conditionExpression>{condition}</conditionExpression>"),
                        format!("{variable_path} as a string such as \"5\""),
                        format!("{variable_path} as a boolean while the gateway uses `{variable_path} > 0`"),
                        "routing count-like variables through bare boolean paths".to_string()
                    ],
                    "options": [
                        {
                            "op": "rewrite_condition_expression",
                            "from": condition,
                            "to": format!("{variable_path} > 0"),
                            "requires": "upstream task emits a JSON number",
                            "producer_change": format!("update every upstream task that outputs `{variable_path}` so its prompt says `{variable_path}` is a JSON number count, not a boolean or string")
                        },
                        {
                            "op": "rename_gateway_variable_to_boolean",
                            "examples": ["hasMoreQuestions", "needsMoreQuestions"],
                            "requires": "upstream task emits true or false",
                            "producer_change": format!("rename every native BPMN input, output, and condition use of `{variable_path}` consistently if choosing the boolean route")
                        }
                    ]
                }
            ]
        }),
        AmbiguousBooleanPathKind::ContentLike => json!({
            "schema_version": 1,
            "contract": "bpmn.native.gateway.condition.v1",
            "strategy": "split_content_variable_from_boolean_route",
            "target": {
                "process_id": process_id,
                "gateway_id": gateway_id,
                "condition": condition,
                "content_variable": variable_path
            },
            "actions": [
                {
                    "op": "add_boolean_route_output",
                    "examples": ["hasQuestions", "hasConcerns", "needsHumanInput", "shouldEscalate"],
                    "producer_change": format!("update every task that can reach gateway `{gateway_id}` so it declares and emits a JSON boolean route variable separate from `{variable_path}`"),
                    "route_change": format!("replace `<conditionExpression>{condition}</conditionExpression>` with the new boolean route variable"),
                    "preserve": format!("keep `{variable_path}` available for user-facing question text, concerns, details, or other content")
                }
            ],
            "forbidden_forms": [
                format!("<conditionExpression>{condition}</conditionExpression>"),
                "routing directly on text, arrays, objects, or enum status values".to_string(),
                "using the same variable as both user-facing prompt content and boolean route signal".to_string()
            ]
        }),
    }
}
