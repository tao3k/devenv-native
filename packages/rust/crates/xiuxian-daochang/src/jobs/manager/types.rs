//! Job manager queue and record types.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::agent::Agent;

use crate::jobs::heartbeat::JobHealthState;

/// Runtime state of a background job managed by the channel job manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    /// Job has been accepted but has not started execution yet.
    Queued,
    /// Job is currently executing.
    Running,
    /// Job completed successfully.
    Succeeded,
    /// Job completed with an error.
    Failed,
    /// Job exceeded its configured timeout.
    TimedOut,
}

/// Terminal completion payload recorded for a finished background job.
#[derive(Debug, Clone)]
pub enum JobCompletionKind {
    /// Job completed successfully and produced output.
    Succeeded {
        /// Output returned by the finished job.
        output: String,
    },
    /// Job failed and recorded an error message.
    Failed {
        /// Error message returned by the failed job.
        error: String,
    },
    /// Job timed out after the reported number of seconds.
    TimedOut {
        /// Timeout budget that was exceeded.
        timeout_secs: u64,
    },
}

/// Completion event emitted after a background job leaves the running set.
#[derive(Debug, Clone)]
pub struct JobCompletion {
    /// Stable job identifier.
    pub job_id: String,
    /// Downstream recipient that should receive the completion notification.
    pub recipient: String,
    /// Parent foreground session that spawned the background job.
    pub parent_session_id: String,
    /// Completion payload describing the terminal outcome.
    pub kind: JobCompletionKind,
}

impl JobCompletion {
    #[must_use]
    /// Returns the logical foreground session key associated with this completion.
    pub fn parent_session_key(&self) -> &str {
        &self.parent_session_id
    }
}

/// Snapshot of one job as exposed to status and diagnostics surfaces.
#[derive(Debug, Clone)]
pub struct JobStatusSnapshot {
    /// Stable job identifier.
    pub job_id: String,
    /// Logical background session identifier used for the job turn.
    pub session_id: String,
    /// Current lifecycle state.
    pub state: JobState,
    /// Truncated prompt preview for dashboards and operator replies.
    pub prompt_preview: String,
    /// Age in seconds since submission.
    pub submitted_age_secs: u64,
    /// Age in seconds since execution started, when running.
    pub running_age_secs: Option<u64>,
    /// Age in seconds since completion, when finished.
    pub finished_age_secs: Option<u64>,
    /// Truncated output preview when the job succeeded.
    pub output_preview: Option<String>,
    /// Error message when the job failed.
    pub error: Option<String>,
}

/// Aggregate metrics for the in-memory job manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobMetricsSnapshot {
    /// Total number of jobs tracked by the manager.
    pub total_jobs: usize,
    /// Number of queued jobs.
    pub queued: usize,
    /// Number of running jobs.
    pub running: usize,
    /// Number of successfully completed jobs.
    pub succeeded: usize,
    /// Number of failed jobs.
    pub failed: usize,
    /// Number of timed out jobs.
    pub timed_out: usize,
    /// Age in seconds of the oldest queued job, when any exist.
    pub oldest_queued_age_secs: Option<u64>,
    /// Age in seconds of the longest-running job, when any exist.
    pub longest_running_age_secs: Option<u64>,
    /// Current heartbeat-derived health classification.
    pub health_state: JobHealthState,
}

/// Runtime configuration for the background job manager.
#[derive(Debug, Clone)]
pub struct JobManagerConfig {
    /// Maximum number of queued jobs before new submissions are rejected.
    pub queue_capacity: usize,
    /// Maximum number of concurrent running jobs.
    pub max_in_flight: usize,
    /// Hard timeout for each background job in seconds.
    pub job_timeout_secs: u64,
    /// Interval between heartbeat evaluations in seconds.
    pub heartbeat_interval_secs: u64,
    /// Timeout budget for each heartbeat probe in seconds.
    pub heartbeat_probe_timeout_secs: u64,
    /// Threshold after which queued jobs are considered unhealthy.
    pub max_queued_age_secs: u64,
    /// Threshold after which running jobs are considered unhealthy.
    pub max_running_age_secs: u64,
}

impl Default for JobManagerConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 64,
            max_in_flight: 4,
            job_timeout_secs: 300,
            heartbeat_interval_secs: 30,
            heartbeat_probe_timeout_secs: 5,
            max_queued_age_secs: 300,
            max_running_age_secs: 900,
        }
    }
}

#[async_trait]
/// Trait implemented by types that can execute one agent turn for a background job.
pub trait TurnRunner: Send + Sync {
    /// Runs one turn for the provided logical session and prompt.
    async fn run_turn(&self, session_id: &str, prompt: &str) -> Result<String>;
}

#[async_trait]
impl TurnRunner for Agent {
    async fn run_turn(&self, session_id: &str, prompt: &str) -> Result<String> {
        Agent::run_turn(self, session_id, prompt).await
    }
}

/// Mutable in-memory job record tracked by the scheduler.
#[derive(Debug, Clone)]
pub struct JobRecord {
    /// Logical background session identifier.
    pub session_id: String,
    /// Original prompt submitted to the job manager.
    pub prompt: String,
    /// Current lifecycle state.
    pub state: JobState,
    /// Submission timestamp.
    pub submitted_at: Instant,
    /// Start timestamp, when execution has begun.
    pub started_at: Option<Instant>,
    /// Finish timestamp, when execution has completed.
    pub finished_at: Option<Instant>,
    /// Truncated output preview for completed successful jobs.
    pub output_preview: Option<String>,
    /// Error text for failed jobs.
    pub error: Option<String>,
}

/// Queue payload for a submitted but not-yet-started job.
#[derive(Debug, Clone)]
pub struct QueuedJob {
    /// Stable job identifier.
    pub job_id: String,
    /// Downstream recipient that should receive responses.
    pub recipient: String,
    /// Foreground parent session that spawned the job.
    pub parent_session_id: String,
    /// Logical background session identifier.
    pub session_id: String,
    /// Original prompt submitted to the job manager.
    pub prompt: String,
}

#[must_use]
/// Returns the current Unix epoch timestamp in milliseconds.
pub fn epoch_millis() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis().try_into().unwrap_or(u64::MAX),
        Err(_) => 0,
    }
}

#[must_use]
/// Truncates a string for status surfaces, appending `...` when needed.
pub fn truncate_for_status(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let head_len = max_chars.saturating_sub(3);
    let head = input.chars().take(head_len).collect::<String>();
    format!("{head}...")
}

#[must_use]
/// Returns the elapsed whole seconds between two instants using saturating math.
pub fn elapsed_secs_from(now: Instant, then: Instant) -> u64 {
    now.saturating_duration_since(then).as_secs()
}

#[cfg(test)]
#[path = "../../../tests/unit/jobs/manager/types.rs"]
mod tests;
