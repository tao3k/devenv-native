use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, LlmActivityAdmission, LlmActivityRequest, LlmActivityTask, LlmModelId,
};

use crate::control::support::activity_task;

pub(super) fn llm_admission(
    activity_id: ActivityId,
) -> Result<LlmActivityAdmission, Box<dyn Error>> {
    let task = activity_task(activity_id)?;
    let prompt_ref = task
        .input_ref
        .clone()
        .ok_or_else(|| io::Error::other("missing prompt input ref"))?;
    Ok(LlmActivityAdmission::from_activity(LlmActivityTask::new(
        task,
        LlmActivityRequest::new(LlmModelId::new("openai/gpt-5.2")?, prompt_ref),
    ))?)
}
