use super::{
    BpmnSourceFile, HashSet, LintIssue, LintSourceDiagnostic, LintSourceSpan, Range, json,
};

pub(super) struct UndeclaredGatewayConditionIssue<'a> {
    pub(super) source: &'a BpmnSourceFile,
    pub(super) process_id: &'a str,
    pub(super) gateway_id: &'a str,
    pub(super) target_id: &'a str,
    pub(super) condition: &'a str,
    pub(super) variable_path: &'a str,
    pub(super) producer_ids: &'a [String],
    pub(super) producer_outputs: &'a HashSet<String>,
    pub(super) condition_span: Option<Range<usize>>,
}

pub(super) fn undeclared_gateway_condition_output_issue(
    context: UndeclaredGatewayConditionIssue<'_>,
) -> LintIssue {
    let producer_list = context.producer_ids.join(", ");
    let mut output_list = context.producer_outputs.iter().cloned().collect::<Vec<_>>();
    output_list.sort();
    let output_summary = output_list.join(", ");
    let mut issue = LintIssue::new(
        "bpmn.undeclared_gateway_condition_output",
        "Gateway condition variable is not declared by upstream task outputs",
        format!(
            "Process '{}' gateway '{}' routes to '{}' with condition `{}`, but direct upstream task(s) [{producer_list}] do not declare native BPMN output '{}'.",
            context.process_id,
            context.gateway_id,
            context.target_id,
            context.condition,
            context.variable_path
        ),
        "Gateway conditions resolve against runtime variables. A task immediately before a gateway must declare any route variable it is expected to produce through native BPMN output metadata, and its prompt should say to return that JSON field.",
        vec![
            format!("Add '{}' as a native BPMN data output or output association target on upstream task(s) [{producer_list}].", context.variable_path),
            format!("Update the same upstream task prompt to return JSON with boolean or numeric field '{}', matching the gateway condition type.", context.variable_path),
            "Keep the gateway condition unchanged after the producer declares and emits the variable.".to_string(),
        ],
        format!(
            "Repair process '{}' by aligning gateway '{}' condition `{}` with upstream native BPMN outputs. Add `{}` to task output metadata for task(s) [{producer_list}] and update their prompt to return JSON field `{}`. Preserve the branch target '{}' and keep the condition inside the bounded gateway subset.",
            context.process_id,
            context.gateway_id,
            context.condition,
            context.variable_path,
            context.variable_path,
            context.target_id
        ),
        json!({
            "process_id": context.process_id,
            "gateway_id": context.gateway_id,
            "target_id": context.target_id,
            "condition": context.condition,
            "variable_path": context.variable_path,
            "producer_task_ids": context.producer_ids,
            "producer_outputs": output_list,
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "bpmn.native.gateway.data_contract.v1",
        "strategy": "declare_gateway_condition_variable_on_upstream_task",
        "actions": [{
            "op": "add_native_bpmn_output",
            "tasks": context.producer_ids,
            "output": context.variable_path,
        }, {
            "op": "update_task_prompt",
            "tasks": context.producer_ids,
            "requires": format!("return JSON field `{}`", context.variable_path)
        }, {
            "op": "keep_gateway_condition",
            "gateway": context.gateway_id,
            "condition": context.condition
        }],
        "forbid": [
            "routing on variables missing from direct upstream task outputs",
            "renaming the gateway condition without updating the producer prompt and outputs"
        ]
    }));
    if let Some(span) = context.condition_span {
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            &context.source.source_id,
            LintSourceSpan::new(span.start, span.end),
            format!(
                "condition uses undeclared upstream output `{}`",
                context.variable_path
            ),
            format!(
                "Add `{}` to native BPMN task outputs and prompt JSON on upstream task(s) [{producer_list}]. Current outputs: {output_summary}.",
                context.variable_path
            ),
        ));
    }
    issue
}
