use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, ActivityTask, ActivityType, ArtifactId, ArtifactKind, ArtifactRef, IdempotencyKey,
    TaskQueue,
};

pub(crate) fn activity_task(activity_id: ActivityId) -> Result<ActivityTask, Box<dyn Error>> {
    Ok(ActivityTask::new(
        activity_id,
        ActivityType::new("llm.plan")?,
        TaskQueue::new("llm.openai")?,
        IdempotencyKey::new("run/activity/1")?,
    )
    .with_input_ref(artifact_ref("artifact-llm-plan-input")?)
    .with_timeout_ms(30_000))
}

pub(crate) fn artifact_ref(artifact_id: &str) -> Result<ArtifactRef, Box<dyn Error>> {
    Ok(ArtifactRef {
        artifact_id: ArtifactId::new(artifact_id)?,
        artifact_kind: ArtifactKind::new("claim_check")?,
        uri: format!("artifact://{artifact_id}"),
        content_digest: None,
        metadata: serde_json::Value::Null,
    })
}
