use serde_json::json;
use xiuxian_qianji_bpmn_engine::{HostBridgeError, PendingHostWorkRequest, UserTaskRequest};

use super::support::err_of;
use crate::{BpmnAdapterError, QianjiBpmnHostBridge, dispatch_pending_host_work_request};

#[tokio::test(flavor = "current_thread")]
async fn default_bridge_keeps_unsupported_host_operations_explicit() {
    let host = QianjiBpmnHostBridge::default();
    let error = err_of(
        dispatch_pending_host_work_request(
            &host,
            PendingHostWorkRequest::User(UserTaskRequest {
                instance_id: "wf_user".into(),
                process_id: "review".into(),
                token_id: 7.into(),
                node_index: 3,
                activity_id: "Task_Review".into(),
                variables: json!({ "approved": false }),
                inputs: json!({}),
                output_bindings: vec![],
                repeat: None,
                lane: None,
                form: None,
                assignment: None,
                claim: None,
            }),
        )
        .await,
    );

    match error {
        BpmnAdapterError::Host(HostBridgeError::UnsupportedOperation { operation }) => {
            assert_eq!(operation, "dispatch_user_task");
        }
        other => panic!("expected explicit unsupported host error, got {other:?}"),
    }
}
