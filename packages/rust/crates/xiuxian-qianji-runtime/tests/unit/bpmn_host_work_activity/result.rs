use std::error::Error;

use serde_json::json;
use xiuxian_qianji_runtime::{
    BPMN_HOST_WORK_COMPLETION_METADATA_KEY, BpmnHostWorkCompletion, BpmnHostWorkCompletionKind,
    QianjiRuntimeBpmnActivityId, QianjiRuntimeBpmnProcessId, QianjiRuntimeBpmnTokenId,
    build_bpmn_host_work_activity_result,
};

#[test]
fn bpmn_host_work_completion_result_preserves_metadata_hash() -> Result<(), Box<dyn Error>> {
    let completion = BpmnHostWorkCompletion {
        token_id: QianjiRuntimeBpmnTokenId::new(9),
        process_id: QianjiRuntimeBpmnProcessId::new("Process_1"),
        activity_id: QianjiRuntimeBpmnActivityId::new("Task_Review"),
        kind: BpmnHostWorkCompletionKind::Service,
        data: json!({"approved": true}),
        claimant: Some("worker-1".to_owned()),
    };

    let result = build_bpmn_host_work_activity_result(&completion)?;

    assert!(result.output_ref.is_none());
    assert!(
        result
            .output_hash
            .as_deref()
            .is_some_and(|hash| hash.starts_with("sha256:"))
    );
    let metadata = &result.metadata[BPMN_HOST_WORK_COMPLETION_METADATA_KEY];
    assert_eq!(
        metadata["schema"],
        "xiuxian_qianji.bpmn.host_work_completion.v1"
    );
    assert_eq!(metadata["tokenId"], 9);
    assert_eq!(metadata["processId"], "Process_1");
    assert_eq!(metadata["activityId"], "Task_Review");
    assert_eq!(metadata["kind"], "service");
    assert_eq!(metadata["data"], json!({"approved": true}));
    assert_eq!(metadata["claimant"], "worker-1");

    Ok(())
}
