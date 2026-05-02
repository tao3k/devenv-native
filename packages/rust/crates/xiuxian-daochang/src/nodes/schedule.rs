use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    JobManager, JobManagerConfig, RecurringScheduleConfig, RuntimeSettings, TurnRunner,
    run_recurring_schedule,
};

use crate::agent_builder::build_agent;

pub(crate) struct ScheduleModeRequest {
    pub prompt: String,
    pub interval_secs: u64,
    pub max_runs: Option<u64>,
    pub schedule_id: String,
    pub session_prefix: String,
    pub recipient: String,
    pub wait_for_completion_secs: u64,
    pub tool_config_path: PathBuf,
}

pub(crate) async fn run_schedule_mode(
    request: ScheduleModeRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let ScheduleModeRequest {
        prompt,
        interval_secs,
        max_runs,
        schedule_id,
        session_prefix,
        recipient,
        wait_for_completion_secs,
        tool_config_path,
    } = request;
    let runner: Arc<dyn TurnRunner> =
        Arc::new(build_agent(&tool_config_path, runtime_settings).await?);
    let (job_manager, completion_rx) = JobManager::start(runner, JobManagerConfig::default());

    println!(
        "Starting scheduler: schedule_id={schedule_id} interval={}s max_runs={:?}",
        interval_secs.max(1),
        max_runs
    );
    let outcome = run_recurring_schedule(
        job_manager,
        completion_rx,
        RecurringScheduleConfig {
            schedule_id,
            session_prefix,
            recipient,
            prompt,
            interval_secs,
            max_runs,
            wait_for_completion_secs,
        },
    )
    .await?;

    println!(
        "Scheduler finished: submitted={} completed={} succeeded={} failed={} timed_out={} pending={}",
        outcome.submitted,
        outcome.completed,
        outcome.succeeded,
        outcome.failed,
        outcome.timed_out,
        outcome.submitted.saturating_sub(outcome.completed),
    );
    Ok(())
}
