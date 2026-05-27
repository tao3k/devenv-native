use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn boundary_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    match detail {
        "cancel_boundary_requires_transaction_shell" => {
            cancel_boundary_requires_transaction_shell_issue(process_id, node_id, detail)
        }
        "multiple_transaction_cancel_boundaries" => {
            multiple_transaction_cancel_boundaries_issue(process_id, node_id, detail)
        }
        "error_boundary_requires_supported_subprocess_shell" => {
            error_boundary_requires_supported_subprocess_shell_issue(process_id, node_id, detail)
        }
        "escalation_boundary_requires_supported_subprocess_shell" => {
            escalation_boundary_requires_supported_subprocess_shell_issue(
                process_id, node_id, detail,
            )
        }
        "non_interrupting_escalation_boundary_deferred" => {
            non_interrupting_escalation_boundary_issue(process_id, node_id, detail)
        }
        "non_interrupting_boundary_requires_supported_task_repeat_owner" => {
            non_interrupting_boundary_requires_supported_task_repeat_owner_issue(
                process_id, node_id, detail,
            )
        }
        _ => generic_boundary_configuration_issue(process_id, node_id, detail),
    }
}

fn cancel_boundary_requires_transaction_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_boundary_configuration",
        "Cancel boundary must attach to a transaction shell",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses `<cancelEventDefinition>` without attaching it to a bounded transaction shell."
        ),
        "The bounded engine supports one interrupting cancel boundary path only when the boundary event is attached to one bounded `<transaction>` shell and matches the transaction shell's nested cancel end.",
        vec![
            "Attach this cancel boundary to one `<bpmn:transaction>` node, not to a task, embedded subprocess, or call activity.".to_string(),
            "Keep `cancelActivity=\"true\"` and pair the boundary with exactly one nested transaction end event that carries `<bpmn:cancelEventDefinition>`.".to_string(),
        ],
        format!(
            "Rewrite boundary event '{node_id}' in process '{process_id}' so `<cancelEventDefinition>` is used only as one interrupting boundary event attached to one bounded `<bpmn:transaction>` shell, paired with exactly one nested cancel end inside that same transaction. Preserve workflow intent, but do not leave the cancel boundary attached to a task or non-transaction subprocess."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn multiple_transaction_cancel_boundaries_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_boundary_configuration",
        "Transaction owner exposes more than one cancel boundary",
        format!(
            "Process '{process_id}' boundary event '{node_id}' adds a second `<cancelEventDefinition>` boundary to the same bounded transaction owner."
        ),
        "The bounded engine allows one transaction owner to expose either one interrupting timer/message/signal/conditional boundary on its own, one interrupting timer/message/signal/conditional boundary plus one interrupting cancel boundary, one interrupting timer/message/signal/conditional boundary plus one or more interrupting error boundaries, one interrupting timer/message/signal/conditional boundary plus one interrupting cancel boundary plus one or more interrupting error boundaries, or one interrupting cancel boundary plus one or more interrupting error boundaries, but it still permits only one cancel boundary on that same transaction shell.",
        vec![
            "Keep exactly one interrupting cancel boundary attached to this `<bpmn:transaction>` node.".to_string(),
            "If the second branch is really an external wait, keep one timer/message/signal/conditional boundary and one cancel boundary, and if needed also keep one or more error boundaries, but remove the extra cancel boundary because the bounded slice still permits only one cancel boundary on that transaction owner.".to_string(),
            "If the second branch is really an error path, rewrite it as `<bpmn:errorEventDefinition>` and pair it with the transaction shell's nested error end.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' so boundary event '{node_id}' no longer creates a second transaction cancel boundary. Keep one interrupting `<bpmn:cancelEventDefinition>` boundary on that transaction owner, and convert any extra recovery branch into one or more interrupting `<bpmn:errorEventDefinition>` boundaries only if the nested transaction end throws matching errors."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn error_boundary_requires_supported_subprocess_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_boundary_configuration",
        "Error boundary must attach to a supported subprocess shell",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses `<errorEventDefinition>` without attaching it to one bounded transaction shell, one bounded embedded subprocess shell, or one bounded call activity owner."
        ),
        "The bounded engine supports one or more interrupting error boundary paths only when those boundary events are attached to one bounded `<transaction>` shell, one bounded embedded `subProcess` shell, or one bounded same-package `callActivity` owner and match one or more bounded error ends that execute under that same owner.",
        vec![
            "Attach this error boundary to one bounded `<bpmn:transaction>` node, one bounded embedded `<bpmn:subProcess>` node, or one bounded same-package `<bpmn:callActivity>` node, not to a task.".to_string(),
            "Keep `cancelActivity=\"true\"` and pair the boundary with one or more error ends that execute under that same owner, using a matching `errorRef` or omitting `errorRef` on the boundary as a catch-all.".to_string(),
        ],
        format!(
            "Rewrite boundary event '{node_id}' in process '{process_id}' so `<errorEventDefinition>` is used only as one interrupting boundary event attached to one bounded `<bpmn:transaction>` shell, one bounded embedded `<bpmn:subProcess>` shell, or one bounded same-package `<bpmn:callActivity>` owner, paired with one or more error ends that execute under that same owner. Preserve workflow intent, but do not leave the error boundary attached to a task."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn escalation_boundary_requires_supported_subprocess_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_boundary_configuration",
        "Escalation boundary must attach to a supported subprocess shell",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses `<escalationEventDefinition>` without attaching it to one bounded embedded subprocess, same-package call activity, or transaction owner."
        ),
        "The bounded engine supports interrupting escalation boundaries only on parent owners that can run a child scope: one bounded embedded subprocess, one bounded same-package call activity, or one bounded transaction shell. Task-owned escalation boundaries require a broader escalation model and are deferred.",
        vec![
            "Attach the escalation boundary to one bounded embedded `<bpmn:subProcess>`, one bounded same-package `<bpmn:callActivity>`, or one bounded `<bpmn:transaction>` owner.".to_string(),
            "Keep `cancelActivity=\"true\"` and pair the boundary with one child-scope escalation end event or intermediate escalation throw whose optional `escalationRef` matches the boundary, or omit the boundary reference as a catch-all.".to_string(),
        ],
        format!(
            "Rewrite boundary event '{node_id}' in process '{process_id}' so `<escalationEventDefinition>` is used only as one interrupting boundary event attached to one bounded embedded `<bpmn:subProcess>`, same-package `<bpmn:callActivity>`, or `<bpmn:transaction>` owner. Preserve workflow intent, but do not leave escalation boundary configuration '{detail}' on a task or unsupported owner."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn non_interrupting_escalation_boundary_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_boundary_configuration",
        "Non-interrupting escalation boundaries are deferred",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses `<escalationEventDefinition>` with `cancelActivity=\"false\"`, which is outside the bounded slice."
        ),
        "The current runtime can execute non-interrupting timer, message, signal, and conditional boundaries on supported task owners, but escalation routing is currently interrupting-only and parent-scope based. Non-interrupting escalation requires concurrent parent/child escalation semantics and remains deferred.",
        vec![
            "If the escalation must cancel the child scope, use `cancelActivity=\"true\"` and attach the escalation boundary to one bounded embedded subprocess, same-package call activity, or transaction owner.".to_string(),
            "If the escalation must be non-interrupting, keep the BPMN requirement explicit but defer runtime execution until non-interrupting escalation semantics land.".to_string(),
        ],
        format!(
            "Repair boundary event '{node_id}' in process '{process_id}' by either converting deferred non-interrupting escalation configuration '{detail}' into one supported interrupting parent escalation boundary on a bounded subprocess-like owner, or preserving the requirement as deferred rather than relying on runtime execution."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn non_interrupting_boundary_requires_supported_task_repeat_owner_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_boundary_configuration",
        "Non-interrupting boundary must attach to a supported task owner",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses `cancelActivity=\"false\"` on a task owner whose repeat metadata falls outside the bounded task-repeat subset."
        ),
        "The bounded engine currently allows one non-interrupting timer, message, signal, or conditional boundary only on one task, sendTask, receiveTask, serviceTask, scriptTask, userTask, manualTask, or businessRuleTask whose repeat metadata is either omitted, one bounded `standardLoopCharacteristics` owner, or one bounded sequential or parallel `multiInstanceLoopCharacteristics` owner.",
        vec![
            "Keep `cancelActivity=\"false\"` only when the attached task is non-repeating, uses one bounded `<bpmn:standardLoopCharacteristics>` owner, or uses one bounded `<bpmn:multiInstanceLoopCharacteristics>` owner whether `isSequential=\"true\"` or `isSequential=\"false\"`.".to_string(),
            "If the owner needs a broader repeat family than that bounded subset, either simplify the repeat metadata, convert it to one bounded standard-loop or multi-instance shape when that preserves intent, or switch the boundary back to `cancelActivity=\"true\"`.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' so boundary event '{node_id}' uses one non-interrupting timer, message, signal, or conditional boundary only on one non-repeating task, sendTask, receiveTask, serviceTask, scriptTask, userTask, manualTask, or businessRuleTask, or on one bounded standard-loop, sequential multi-instance, or parallel multi-instance owner of those same task kinds, with exactly one timer, message, signal, or conditional definition. Preserve workflow intent, but do not keep `cancelActivity=\"false\"` on a task owner whose repeat metadata falls outside that bounded subset."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn generic_boundary_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_boundary_configuration",
        "Boundary event configuration exceeds the bounded slice",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses unsupported configuration '{detail}'."
        ),
        "The current engine supports one interrupting timer, message, signal, or conditional boundary on one host-blocking task, one interrupting timer, message, signal, or conditional boundary on one bounded embedded subprocess owner either alone or paired with one or more interrupting error boundaries on that same owner, one interrupting timer, message, signal, or conditional boundary on one bounded same-package call activity owner either alone or paired with one or more interrupting error boundaries on that same owner, one interrupting timer, message, signal, or conditional boundary on one bounded transaction shell either on its own, paired with one interrupting cancel boundary, paired with one or more interrupting error boundaries on that same owner, or paired with one interrupting cancel boundary plus one or more interrupting error boundaries on that same owner, one non-interrupting timer, message, signal, or conditional boundary on one non-repeating or bounded standard-loop, sequential multi-instance, or parallel multi-instance host-blocking task, one interrupting cancel boundary on one bounded transaction shell, and one or more interrupting error boundaries on one bounded transaction shell, one bounded embedded subprocess shell, or one bounded same-package call activity owner.",
        vec![
            "Keep the timeout, escalation, or transaction-cancel intent, but rewrite the boundary to one supported bounded shape.".to_string(),
            "Use one interrupting timer, message, signal, or conditional boundary on one task, sendTask, receiveTask, serviceTask, scriptTask, userTask, manualTask, or businessRuleTask, or on one bounded embedded `<bpmn:subProcess>` owner including the bounded mixed-owner shape with that single interrupting timer/message/signal/conditional boundary plus one or more interrupting error boundaries on the same owner, or on one bounded same-package `<bpmn:callActivity>` owner including that same bounded mixed-owner shape with one interrupting timer/message/signal/conditional boundary plus one or more interrupting error boundaries on the same owner, or on one bounded `<bpmn:transaction>` shell either on its own, with one interrupting cancel boundary, with one or more interrupting error boundaries, or with one interrupting cancel boundary plus one or more interrupting error boundaries, or use one non-interrupting timer, message, signal, or conditional boundary on one non-repeating or bounded standard-loop, sequential multi-instance, or parallel multi-instance task of those same kinds, or use one cancel boundary on one bounded `<transaction>` shell, or use one or more error boundaries on one bounded `<transaction>` shell, one bounded embedded `<bpmn:subProcess>` shell, or one bounded same-package `<bpmn:callActivity>` owner.".to_string(),
        ],
        format!(
            "Rewrite boundary event '{node_id}' in process '{process_id}' so it fits the bounded slice: either one interrupting timer `boundaryEvent` attached to one task, sendTask, receiveTask, serviceTask, scriptTask, userTask, manualTask, or businessRuleTask with `cancelActivity=\"true\"` and exactly one timer expression, one interrupting message, signal, or conditional `boundaryEvent` attached to one task, sendTask, receiveTask, serviceTask, scriptTask, userTask, manualTask, or businessRuleTask with `cancelActivity=\"true\"` and exactly one `messageEventDefinition`, `signalEventDefinition`, or `conditionalEventDefinition` with a bounded `condition`, one interrupting timer, message, signal, or conditional `boundaryEvent` attached to one bounded embedded `<bpmn:subProcess>` owner with `cancelActivity=\"true\"`, exactly one matching event definition, and optionally one or more interrupting error `boundaryEvent` nodes on that same owner, one interrupting timer, message, signal, or conditional `boundaryEvent` attached to one bounded same-package `<bpmn:callActivity>` owner with `cancelActivity=\"true\"`, exactly one matching event definition, and optionally one or more interrupting error `boundaryEvent` nodes on that same owner, one interrupting timer, message, signal, or conditional `boundaryEvent` attached to one bounded `<bpmn:transaction>` shell with `cancelActivity=\"true\"`, exactly one matching event definition, and optionally one interrupting cancel `boundaryEvent`, one or more interrupting error `boundaryEvent` nodes, or both on that same owner, one non-interrupting timer `boundaryEvent` attached to one non-repeating or bounded standard-loop, sequential multi-instance, or parallel multi-instance task, sendTask, receiveTask, serviceTask, scriptTask, userTask, manualTask, or businessRuleTask with `cancelActivity=\"false\"` and exactly one timer expression, one non-interrupting message, signal, or conditional `boundaryEvent` attached to one non-repeating or bounded standard-loop, sequential multi-instance, or parallel multi-instance task, sendTask, receiveTask, serviceTask, scriptTask, userTask, manualTask, or businessRuleTask with `cancelActivity=\"false\"` and exactly one `messageEventDefinition`, `signalEventDefinition`, or `conditionalEventDefinition` with a bounded `condition`, one interrupting cancel `boundaryEvent` attached to one bounded `<bpmn:transaction>` shell with a matching nested cancel end, or one or more interrupting error `boundaryEvent` nodes attached to one bounded `<bpmn:transaction>` shell, one bounded embedded `<bpmn:subProcess>` shell, or one bounded same-package `<bpmn:callActivity>` owner with one or more matching error ends whose optional `errorRef` either matches the thrown error or stays omitted as a catch-all. Preserve workflow intent, but remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}
