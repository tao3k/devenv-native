use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn task_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    let summary = match detail {
        "multiple_task_message_bindings" => "Message task declares more than one binding source",
        "unsupported_send_task_event_kind" => "Send task uses an unsupported event binding",
        "unsupported_receive_task_event_kind" => "Receive task uses an unsupported event binding",
        _ => "Task configuration exceeds the bounded slice",
    };
    LintIssue::new(
        "bpmn.unsupported_task_configuration",
        summary,
        format!(
            "Process '{process_id}' task node '{node_id}' uses unsupported configuration '{detail}'."
        ),
        "The current engine supports one bounded message-task family plus one bounded host-dispatched script-task family: `receiveTask` waits for exactly one message binding through `messageRef` or one nested `messageEventDefinition`, `sendTask` dispatches exactly one message binding through that same bounded source shape, and `scriptTask` preserves one optional `scriptFormat` plus one optional nested `<bpmn:script>` body for host execution. Correlations, signal/timer task events, and broader collaboration-aware routing remain outside the slice.",
        vec![
            "For `receiveTask` or `sendTask`, keep exactly one message binding source: either `messageRef` on the task or one nested `messageEventDefinition`, but not both.".to_string(),
            "If the task is executable logic rather than messaging, convert it to one bounded `scriptTask` with optional `scriptFormat` and one optional nested `<bpmn:script>` body instead of overloading `sendTask` or `receiveTask` message bindings.".to_string(),
            "Do not add correlation execution, signal/timer task events, or broader collaboration-aware routing to this bounded task slice.".to_string(),
        ],
        format!(
            "Repair task node '{node_id}' in process '{process_id}' so it fits the bounded task slice: for `receiveTask` or `sendTask`, keep exactly one message binding through task-level `messageRef` or one nested `<bpmn:messageEventDefinition>`, but not both. If the node is executable logic instead of messaging, convert it to one bounded `scriptTask` with optional `scriptFormat` and one optional nested `<bpmn:script>` body. Remove unsupported configuration '{detail}', preserve workflow intent, and defer correlations or broader collaboration-aware routing."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}
