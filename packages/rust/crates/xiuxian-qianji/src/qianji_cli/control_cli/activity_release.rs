use std::io;

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityReleaseArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityReleaseArgs {
    valkey_url: Option<String>,
    namespace: Option<String>,
    lease_json: Option<String>,
    json: bool,
}

impl ActivityReleaseArgs {
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
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-release` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        Ok(ControlCliCommand::ActivityRelease {
            valkey_url: self.valkey_url.ok_or_else(|| {
                invalid_input("missing `--valkey-url <url>` for `control activity-release`")
            })?,
            namespace: self.namespace,
            lease_json: self.lease_json.ok_or_else(|| {
                invalid_input("missing `--lease-json <json>` for `control activity-release`")
            })?,
            json: self.json,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct WorkerActivityReleaseRunRequest<'a> {
    pub(super) valkey_url: &'a str,
    pub(super) namespace: Option<&'a str>,
    pub(super) lease_json: &'a str,
    pub(super) json: bool,
}

#[cfg(any(feature = "valkey", test))]
#[derive(Clone, Copy)]
pub(crate) struct WorkerActivityReleaseStoreRequest<'a> {
    pub(crate) lease_json: &'a str,
    pub(crate) json: bool,
}

#[cfg(any(feature = "valkey", test))]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub(crate) struct WorkerActivityReleaseOutput {
    pub(crate) lease: xiuxian_qianji_control::ActivityTaskLease,
    pub(crate) released: bool,
}

#[cfg(feature = "valkey")]
pub(super) fn run(request: WorkerActivityReleaseRunRequest<'_>) -> io::Result<ControlCliOutput> {
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
    runtime.block_on(release_with_hot_state(
        &store,
        WorkerActivityReleaseStoreRequest {
            lease_json: request.lease_json,
            json: request.json,
        },
    ))
}

#[cfg(not(feature = "valkey"))]
pub(super) fn run(request: WorkerActivityReleaseRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.valkey_url,
        request.namespace,
        request.lease_json,
        request.json,
    );
    Err(invalid_input(
        "`control activity-release` requires the `valkey` feature",
    ))
}

#[cfg(any(feature = "valkey", test))]
pub(crate) async fn release_with_hot_state<H>(
    hot_state: &H,
    request: WorkerActivityReleaseStoreRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    let lease = parse_activity_task_lease(request.lease_json)?;
    let released = hot_state
        .release_activity_task_lease(&lease)
        .await
        .map_err(|error| control_error(&error))?;
    let output = WorkerActivityReleaseOutput { lease, released };
    let rendered = if request.json {
        serde_json::to_string_pretty(&output).map_err(io::Error::other)?
    } else {
        render_activity_release_text(&output)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(any(feature = "valkey", test))]
fn parse_activity_task_lease(
    lease_json: &str,
) -> io::Result<xiuxian_qianji_control::ActivityTaskLease> {
    serde_json::from_str(lease_json).map_err(|error| {
        invalid_input(format!(
            "invalid `--lease-json` for `control activity-release`: {error}"
        ))
    })
}

#[cfg(any(feature = "valkey", test))]
fn render_activity_release_text(output: &WorkerActivityReleaseOutput) -> String {
    let step = output
        .lease
        .step_id
        .as_ref()
        .map_or("<run>", |step_id| step_id.as_str());
    format!(
        concat!(
            "# Qianji Control Activity Release\n\n",
            "- Released: `{}`\n",
            "- Run: `{}`\n",
            "- Step: `{}`\n",
            "- Activity: `{}`\n",
            "- Worker: `{}`\n",
            "- Lease: `{}`\n"
        ),
        output.released,
        output.lease.run_id.as_str(),
        step,
        output.lease.activity_id.as_str(),
        output.lease.worker_id.as_str(),
        output.lease.lease_id.as_str()
    )
}

#[cfg(any(feature = "valkey", test))]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
