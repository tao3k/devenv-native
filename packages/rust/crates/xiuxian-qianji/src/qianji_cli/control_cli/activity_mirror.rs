use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityMirrorArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityMirrorArgs {
    ledger_path: Option<PathBuf>,
    valkey_url: Option<String>,
    namespace: Option<String>,
    run_id: Option<String>,
    task_queue: Option<String>,
    priority: Option<i64>,
    not_before_ms: Option<u64>,
    metadata: Option<String>,
    json: bool,
}

impl ActivityMirrorArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
        match args[*index].as_str() {
            "--ledger" => {
                self.ledger_path = Some(PathBuf::from(parse_flag_value(args, index, "--ledger")?));
            }
            "--valkey-url" => {
                self.valkey_url = Some(parse_flag_value(args, index, "--valkey-url")?);
            }
            "--namespace" => {
                self.namespace = Some(parse_flag_value(args, index, "--namespace")?);
            }
            "--run-id" => {
                self.run_id = Some(parse_flag_value(args, index, "--run-id")?);
            }
            "--task-queue" => {
                self.task_queue = Some(parse_flag_value(args, index, "--task-queue")?);
            }
            "--priority" => {
                self.priority = Some(parse_i64(
                    "priority",
                    "control activity-mirror",
                    &parse_flag_value(args, index, "--priority")?,
                )?);
            }
            "--not-before-ms" => {
                self.not_before_ms = Some(parse_u64(
                    "not-before-ms",
                    "control activity-mirror",
                    &parse_flag_value(args, index, "--not-before-ms")?,
                )?);
            }
            "--metadata" => {
                self.metadata = Some(parse_flag_value(args, index, "--metadata")?);
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-mirror` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        Ok(ControlCliCommand::ActivityMirror {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control activity-mirror`")
            })?,
            valkey_url: self.valkey_url.ok_or_else(|| {
                invalid_input("missing `--valkey-url <url>` for `control activity-mirror`")
            })?,
            namespace: self.namespace,
            run_id: self.run_id.ok_or_else(|| {
                invalid_input("missing `--run-id <id>` for `control activity-mirror`")
            })?,
            task_queue: self.task_queue,
            priority: self.priority.unwrap_or(0),
            not_before_ms: self.not_before_ms.unwrap_or(0),
            metadata: self.metadata,
            json: self.json,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct WorkerActivityMirrorRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) valkey_url: &'a str,
    pub(super) namespace: Option<&'a str>,
    pub(super) run_id: &'a str,
    pub(super) task_queue: Option<&'a str>,
    pub(super) priority: i64,
    pub(super) not_before_ms: u64,
    pub(super) metadata: Option<&'a str>,
    pub(super) json: bool,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Clone, Copy)]
pub(crate) struct WorkerActivityMirrorStoreRequest<'a> {
    pub(crate) run_id: &'a str,
    pub(crate) task_queue: Option<&'a str>,
    pub(crate) priority: i64,
    pub(crate) not_before_ms: u64,
    pub(crate) metadata: Option<&'a str>,
    pub(crate) json: bool,
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
pub(super) fn run(request: WorkerActivityMirrorRunRequest<'_>) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{DuckDbControlLedger, ValkeyHotStateConfig, ValkeyHotStateStore};

    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    let config = ValkeyHotStateConfig::new(request.valkey_url.to_owned())
        .map_err(|error| control_error(&error))?;
    let config = if let Some(namespace) = request.namespace {
        config
            .with_namespace(namespace)
            .map_err(|error| control_error(&error))?
    } else {
        config
    };
    let hot_state = ValkeyHotStateStore::new(config);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(io::Error::other)?;
    runtime.block_on(mirror_with_hot_state(
        &ledger,
        &hot_state,
        WorkerActivityMirrorStoreRequest {
            run_id: request.run_id,
            task_queue: request.task_queue,
            priority: request.priority,
            not_before_ms: request.not_before_ms,
            metadata: request.metadata,
            json: request.json,
        },
    ))
}

#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
pub(super) fn run(request: WorkerActivityMirrorRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.valkey_url,
        request.namespace,
        request.run_id,
        request.task_queue,
        request.priority,
        request.not_before_ms,
        request.metadata,
        request.json,
    );
    Err(invalid_input(
        "`control activity-mirror` requires the `duckdb` and `valkey` features",
    ))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
pub(crate) async fn mirror_with_hot_state<L, H>(
    ledger: &L,
    hot_state: &H,
    request: WorkerActivityMirrorStoreRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    use xiuxian_qianji_control::{
        RunId, TaskQueue, WorkerActivityHotStateMirrorRequest,
        mirror_worker_activity_tasks_to_hot_state,
    };

    let run_id = RunId::new(request.run_id).map_err(|error| control_error(&error))?;
    let task_queue = request
        .task_queue
        .map(TaskQueue::new)
        .transpose()
        .map_err(|error| control_error(&error))?;
    let metadata = parse_metadata(request.metadata)?;
    let mut mirror_request = WorkerActivityHotStateMirrorRequest::new(run_id)
        .with_priority(request.priority)
        .with_not_before_ms(request.not_before_ms)
        .with_metadata(metadata);
    if let Some(task_queue) = task_queue {
        mirror_request = mirror_request.with_task_queue(task_queue);
    }
    let outcome = mirror_worker_activity_tasks_to_hot_state(ledger, hot_state, mirror_request)
        .await
        .map_err(|error| control_error(&error))?;
    let rendered = if request.json {
        serde_json::to_string_pretty(&outcome).map_err(io::Error::other)?
    } else {
        render_activity_mirror_text(&outcome)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn render_activity_mirror_text(
    outcome: &xiuxian_qianji_control::WorkerActivityHotStateMirrorOutcome,
) -> String {
    let task_queue = outcome
        .task_queue
        .as_ref()
        .map_or("<all>", |queue| queue.as_str());
    format!(
        concat!(
            "# Qianji Control Activity Mirror\n\n",
            "- Run: `{}`\n",
            "- Task queue: `{}`\n",
            "- Mirrored tasks: `{}`\n"
        ),
        outcome.run_id.as_str(),
        task_queue,
        outcome.mirrored_count
    )
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn parse_metadata(metadata: Option<&str>) -> io::Result<serde_json::Value> {
    let Some(metadata) = metadata else {
        return Ok(serde_json::Value::Null);
    };
    serde_json::from_str(metadata).map_err(|error| {
        invalid_input(format!(
            "invalid `--metadata` for `control activity-mirror`: {error}"
        ))
    })
}

fn parse_i64(field: &'static str, command: &'static str, value: &str) -> io::Result<i64> {
    value.parse::<i64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{field}` for `{command}`; expected i64: {error}"
        ))
    })
}

fn parse_u64(field: &'static str, command: &'static str, value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{field}` for `{command}`; expected u64: {error}"
        ))
    })
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
