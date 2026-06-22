use std::io;

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityReclaimArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityReclaimArgs {
    valkey_url: Option<String>,
    namespace: Option<String>,
    lease_json: Option<String>,
    now_ms: Option<u64>,
    json: bool,
}

impl ActivityReclaimArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
        match args[*index].as_str() {
            "--valkey-url" => {
                self.valkey_url = Some(parse_flag_value(args, index, "--valkey-url")?);
            }
            "--namespace" => {
                self.namespace = Some(parse_flag_value(args, index, "--namespace")?);
            }
            "--lease-json" => {
                self.lease_json = Some(parse_flag_value(args, index, "--lease-json")?);
            }
            "--now-ms" => {
                self.now_ms = Some(parse_u64(
                    "now-ms",
                    "control activity-reclaim",
                    &parse_flag_value(args, index, "--now-ms")?,
                )?);
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-reclaim` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        Ok(ControlCliCommand::ActivityReclaim {
            valkey_url: self.valkey_url.ok_or_else(|| {
                invalid_input("missing `--valkey-url <url>` for `control activity-reclaim`")
            })?,
            namespace: self.namespace,
            lease_json: self.lease_json.ok_or_else(|| {
                invalid_input("missing `--lease-json <json>` for `control activity-reclaim`")
            })?,
            now_ms: self.now_ms.ok_or_else(|| {
                invalid_input("missing `--now-ms <ms>` for `control activity-reclaim`")
            })?,
            json: self.json,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct WorkerActivityReclaimRunRequest<'a> {
    pub(super) valkey_url: &'a str,
    pub(super) namespace: Option<&'a str>,
    pub(super) lease_json: &'a str,
    pub(super) now_ms: u64,
    pub(super) json: bool,
}

#[cfg(any(feature = "valkey", test))]
#[derive(Clone, Copy)]
pub(crate) struct WorkerActivityReclaimStoreRequest<'a> {
    pub(crate) lease_json: &'a str,
    pub(crate) now_ms: u64,
    pub(crate) json: bool,
}

#[cfg(any(feature = "valkey", test))]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub(crate) struct WorkerActivityReclaimOutput {
    pub(crate) lease: xiuxian_qianji_control::ActivityTaskLease,
    pub(crate) now_ms: u64,
    pub(crate) reclaimed: bool,
}

#[cfg(feature = "valkey")]
pub(super) fn run(request: WorkerActivityReclaimRunRequest<'_>) -> io::Result<ControlCliOutput> {
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
    runtime.block_on(reclaim_with_hot_state(
        &store,
        WorkerActivityReclaimStoreRequest {
            lease_json: request.lease_json,
            now_ms: request.now_ms,
            json: request.json,
        },
    ))
}

#[cfg(not(feature = "valkey"))]
pub(super) fn run(request: WorkerActivityReclaimRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.valkey_url,
        request.namespace,
        request.lease_json,
        request.now_ms,
        request.json,
    );
    Err(invalid_input(
        "`control activity-reclaim` requires the `valkey` feature",
    ))
}

#[cfg(any(feature = "valkey", test))]
pub(crate) async fn reclaim_with_hot_state<H>(
    hot_state: &H,
    request: WorkerActivityReclaimStoreRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    let lease = parse_activity_task_lease(request.lease_json)?;
    let reclaimed = hot_state
        .reclaim_expired_activity_task_lease(&lease, request.now_ms)
        .await
        .map_err(|error| control_error(&error))?;
    let output = WorkerActivityReclaimOutput {
        lease,
        now_ms: request.now_ms,
        reclaimed,
    };
    let rendered = if request.json {
        serde_json::to_string_pretty(&output).map_err(io::Error::other)?
    } else {
        render_activity_reclaim_text(&output)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(any(feature = "valkey", test))]
fn parse_activity_task_lease(
    lease_json: &str,
) -> io::Result<xiuxian_qianji_control::ActivityTaskLease> {
    serde_json::from_str(lease_json).map_err(|error| {
        invalid_input(format!(
            "invalid `--lease-json` for `control activity-reclaim`: {error}"
        ))
    })
}

#[cfg(any(feature = "valkey", test))]
fn render_activity_reclaim_text(output: &WorkerActivityReclaimOutput) -> String {
    let step = output
        .lease
        .step_id
        .as_ref()
        .map_or("<run>", |step_id| step_id.as_str());
    format!(
        concat!(
            "# Qianji Control Activity Reclaim\n\n",
            "- Reclaimed: `{}`\n",
            "- Now ms: `{}`\n",
            "- Run: `{}`\n",
            "- Step: `{}`\n",
            "- Activity: `{}`\n",
            "- Worker: `{}`\n",
            "- Lease: `{}`\n"
        ),
        output.reclaimed,
        output.now_ms,
        output.lease.run_id.as_str(),
        step,
        output.lease.activity_id.as_str(),
        output.lease.worker_id.as_str(),
        output.lease.lease_id.as_str()
    )
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
