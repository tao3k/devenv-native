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
        "The current engine supports only one bounded message-task family: `receiveTask` waits for exactly one message binding through `messageRef` or one nested `messageEventDefinition`, and `sendTask` dispatches exactly one message binding through that same bounded source shape. Script execution, correlations, and broader collaboration-aware routing remain outside the slice.",
        vec![
            "For `receiveTask` or `sendTask`, keep exactly one message binding source: either `messageRef` on the task or one nested `messageEventDefinition`, but not both.".to_string(),
            "Do not add script bodies, correlation execution, signal/timer task events, or collaboration-aware routing to this bounded task slice.".to_string(),
        ],
        format!(
            "Repair task node '{node_id}' in process '{process_id}' so it fits the bounded message-task slice: for `receiveTask` or `sendTask`, keep exactly one message binding through task-level `messageRef` or one nested `<bpmn:messageEventDefinition>`, but not both, and remove unsupported configuration '{detail}'. Preserve workflow intent, but keep `scriptTask`, correlations, and broader collaboration-aware message routing deferred."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}
