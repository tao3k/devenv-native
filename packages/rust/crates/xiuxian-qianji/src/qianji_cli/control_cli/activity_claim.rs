#[cfg(any(feature = "valkey", test))]
use std::fmt::Write as _;
use std::io;

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityClaimArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityClaimArgs {
    valkey_url: Option<String>,
    namespace: Option<String>,
    worker_id: Option<String>,
    task_queue: Option<String>,
    now_ms: Option<u64>,
    lease_ttl_ms: Option<u64>,
    json: bool,
}

impl ActivityClaimArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
        match args[*index].as_str() {
            "--valkey-url" => {
                self.valkey_url = Some(parse_flag_value(args, index, "--valkey-url")?);
            }
            "--namespace" => {
                self.namespace = Some(parse_flag_value(args, index, "--namespace")?);
            }
            "--worker-id" => {
                self.worker_id = Some(parse_flag_value(args, index, "--worker-id")?);
            }
            "--task-queue" => {
                self.task_queue = Some(parse_flag_value(args, index, "--task-queue")?);
            }
            "--now-ms" => {
                self.now_ms = Some(parse_u64(
                    "now-ms",
                    "control activity-claim",
                    &parse_flag_value(args, index, "--now-ms")?,
                )?);
            }
            "--lease-ttl-ms" => {
                self.lease_ttl_ms = Some(parse_u64(
                    "lease-ttl-ms",
                    "control activity-claim",
                    &parse_flag_value(args, index, "--lease-ttl-ms")?,
                )?);
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-claim` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        Ok(ControlCliCommand::ActivityClaim {
            valkey_url: self.valkey_url.ok_or_else(|| {
                invalid_input("missing `--valkey-url <url>` for `control activity-claim`")
            })?,
            namespace: self.namespace,
            worker_id: self.worker_id.ok_or_else(|| {
                invalid_input("missing `--worker-id <id>` for `control activity-claim`")
            })?,
            task_queue: self.task_queue,
            now_ms: self.now_ms.ok_or_else(|| {
                invalid_input("missing `--now-ms <ms>` for `control activity-claim`")
            })?,
            lease_ttl_ms: self.lease_ttl_ms.ok_or_else(|| {
                invalid_input("missing `--lease-ttl-ms <ms>` for `control activity-claim`")
            })?,
            json: self.json,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct WorkerActivityClaimRunRequest<'a> {
    pub(super) valkey_url: &'a str,
    pub(super) namespace: Option<&'a str>,
    pub(super) worker_id: &'a str,
    pub(super) task_queue: Option<&'a str>,
    pub(super) now_ms: u64,
    pub(super) lease_ttl_ms: u64,
    pub(super) json: bool,
}

#[cfg(any(feature = "valkey", test))]
#[derive(Clone, Copy)]
pub(crate) struct WorkerActivityClaimStoreRequest<'a> {
    pub(crate) worker_id: &'a str,
    pub(crate) task_queue: Option<&'a str>,
    pub(crate) now_ms: u64,
    pub(crate) lease_ttl_ms: u64,
    pub(crate) json: bool,
}

#[cfg(any(feature = "valkey", test))]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub(crate) struct WorkerActivityClaimOutput {
    pub(crate) worker_id: xiuxian_qianji_control::WorkerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) task_queue: Option<xiuxian_qianji_control::TaskQueue>,
    pub(crate) now_ms: u64,
    pub(crate) lease_ttl_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) claimed: Option<xiuxian_qianji_control::HotStateLeasedActivityTask>,
}

#[cfg(feature = "valkey")]
pub(super) fn run(request: WorkerActivityClaimRunRequest<'_>) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ValkeyHotStateConfig, ValkeyHotStateStore};

    let config = ValkeyHotStateConfig::new(request.valkey_url.to_owned())
        .map_err(|error| control_error(&error))?;
    let config = if let Some(namespace) = request.namespace {
        config
            .with_namespace(namespace)
            .map_err(|error| control_error(&error))?
    } else {
        config
    };
    let store = ValkeyHotStateStore::new(config);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(io::Error::other)?;
    runtime.block_on(claim_with_hot_state(
        &store,
        WorkerActivityClaimStoreRequest {
            worker_id: request.worker_id,
            task_queue: request.task_queue,
            now_ms: request.now_ms,
            lease_ttl_ms: request.lease_ttl_ms,
            json: request.json,
        },
    ))
}

#[cfg(not(feature = "valkey"))]
pub(super) fn run(request: WorkerActivityClaimRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.valkey_url,
        request.namespace,
        request.worker_id,
        request.task_queue,
        request.now_ms,
        request.lease_ttl_ms,
        request.json,
    );
    Err(invalid_input(
        "`control activity-claim` requires the `valkey` feature",
    ))
}

#[cfg(any(feature = "valkey", test))]
pub(crate) async fn claim_with_hot_state<H>(
    hot_state: &H,
    request: WorkerActivityClaimStoreRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    use xiuxian_qianji_control::{TaskQueue, WorkerId, WorkerRef};

    let worker_id = WorkerId::new(request.worker_id).map_err(|error| control_error(&error))?;
    let task_queue = request
        .task_queue
        .map(TaskQueue::new)
        .transpose()
        .map_err(|error| control_error(&error))?;
    let worker = WorkerRef {
        worker_id: worker_id.clone(),
        capabilities: Vec::new(),
        metadata: serde_json::Value::Null,
    };
    let claimed = hot_state
        .claim_activity_task(
            worker,
            task_queue.as_ref(),
            request.now_ms,
            request.lease_ttl_ms,
        )
        .await
        .map_err(|error| control_error(&error))?;
    let output = WorkerActivityClaimOutput {
        worker_id,
        task_queue,
        now_ms: request.now_ms,
        lease_ttl_ms: request.lease_ttl_ms,
        claimed,
    };
    let rendered = if request.json {
        serde_json::to_string_pretty(&output).map_err(io::Error::other)?
    } else {
        render_activity_claim_text(&output)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(any(feature = "valkey", test))]
fn render_activity_claim_text(output: &WorkerActivityClaimOutput) -> String {
    let task_queue = output
        .task_queue
        .as_ref()
        .map_or("<all>", |queue| queue.as_str());
    let mut rendered = format!(
        concat!(
            "# Qianji Control Activity Claim\n\n",
            "- Worker: `{}`\n",
            "- Task queue: `{}`\n",
            "- Now ms: `{}`\n",
            "- Lease ttl ms: `{}`\n",
            "- Claimed: `{}`\n"
        ),
        output.worker_id.as_str(),
        task_queue,
        output.now_ms,
        output.lease_ttl_ms,
        output.claimed.is_some()
    );
    if let Some(claimed) = &output.claimed {
        let step = claimed
            .activity_task
            .task
            .step_id
            .as_ref()
            .map_or("<run>", |step_id| step_id.as_str());
        rendered.push_str("\n## Activity Task\n\n");
        if write!(
            rendered,
            concat!(
                "- Run: `{}`\n",
                "- Step: `{}`\n",
                "- Activity: `{}`\n",
                "- Activity type: `{}`\n",
                "- Task queue: `{}`\n",
                "- Next attempt: `{}`\n",
                "- Lease: `{}`\n",
                "- Expires at ms: `{}`\n"
            ),
            claimed.activity_task.task.run_id.as_str(),
            step,
            claimed.activity_task.task.activity_id.as_str(),
            claimed.activity_task.task.activity_type.as_str(),
            claimed.activity_task.task.task_queue.as_str(),
            claimed.activity_task.task.next_attempt,
            claimed.lease.lease_id.as_str(),
            claimed.lease.expires_at_ms
        )
        .is_err()
        {
            return rendered;
        }
    }
    rendered
}

fn parse_u64(field: &'static str, command: &'static str, value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{field}` for `{command}`; expected u64: {error}"
        ))
    })
}

#[cfg(any(feature = "valkey", test))]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
