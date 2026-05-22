use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut valkey_url = None;
    let mut namespace = None;
    let mut run_id = None;
    let mut worker_id = None;
    let mut observed_at_ms = None;
    let mut expires_at_ms = None;
    let mut metadata = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--valkey-url" => {
                valkey_url = Some(parse_flag_value(args, &mut index, "--valkey-url")?);
            }
            "--namespace" => {
                namespace = Some(parse_flag_value(args, &mut index, "--namespace")?);
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--worker-id" => {
                worker_id = Some(parse_flag_value(args, &mut index, "--worker-id")?);
            }
            "--observed-at-ms" => {
                observed_at_ms = Some(parse_ms(
                    "observed-at-ms",
                    "control heartbeat",
                    &parse_flag_value(args, &mut index, "--observed-at-ms")?,
                )?);
            }
            "--expires-at-ms" => {
                expires_at_ms = Some(parse_ms(
                    "expires-at-ms",
                    "control heartbeat",
                    &parse_flag_value(args, &mut index, "--expires-at-ms")?,
                )?);
            }
            "--metadata" => {
                metadata = Some(parse_flag_value(args, &mut index, "--metadata")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control heartbeat` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::Heartbeat {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control heartbeat`"))?,
        valkey_url,
        namespace,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control heartbeat`"))?,
        worker_id: worker_id
            .ok_or_else(|| invalid_input("missing `--worker-id <id>` for `control heartbeat`"))?,
        observed_at_ms: observed_at_ms.ok_or_else(|| {
            invalid_input("missing `--observed-at-ms <ms>` for `control heartbeat`")
        })?,
        expires_at_ms: expires_at_ms.ok_or_else(|| {
            invalid_input("missing `--expires-at-ms <ms>` for `control heartbeat`")
        })?,
        metadata,
        json,
    })
}

#[derive(Clone, Copy)]
pub(super) struct HeartbeatRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) valkey_url: Option<&'a str>,
    pub(super) namespace: Option<&'a str>,
    pub(super) run_id: &'a str,
    pub(super) worker_id: &'a str,
    pub(super) observed_at_ms: u64,
    pub(super) expires_at_ms: u64,
    pub(super) metadata: Option<&'a str>,
    pub(super) json: bool,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Clone, Copy)]
pub(crate) struct HeartbeatHotStateRequest<'a> {
    pub(crate) run_id: &'a str,
    pub(crate) worker_id: &'a str,
    pub(crate) observed_at_ms: u64,
    pub(crate) expires_at_ms: u64,
    pub(crate) metadata: Option<&'a str>,
    pub(crate) json: bool,
}

#[cfg(feature = "duckdb")]
pub(super) fn run(request: HeartbeatRunRequest<'_>) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::DuckDbControlLedger;

    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    run_with_ledger_and_optional_hot_state(&ledger, request)
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run(request: HeartbeatRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.valkey_url,
        request.namespace,
        request.run_id,
        request.worker_id,
        request.observed_at_ms,
        request.expires_at_ms,
        request.metadata,
        request.json,
    );
    Err(invalid_input(
        "`control heartbeat` requires the `duckdb` feature",
    ))
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
fn run_with_ledger_and_optional_hot_state<L>(
    ledger: &L,
    request: HeartbeatRunRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
{
    validate_namespace_scope(request.valkey_url, request.namespace)?;
    if let Some(valkey_url) = request.valkey_url {
        use xiuxian_qianji_control::{ValkeyHotStateConfig, ValkeyHotStateStore};

        let config = ValkeyHotStateConfig::new(valkey_url.to_owned())
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
        return runtime.block_on(heartbeat_with_hot_state(
            ledger,
            &hot_state,
            HeartbeatHotStateRequest {
                run_id: request.run_id,
                worker_id: request.worker_id,
                observed_at_ms: request.observed_at_ms,
                expires_at_ms: request.expires_at_ms,
                metadata: request.metadata,
                json: request.json,
            },
        ));
    }
    heartbeat_with_ledger(ledger, request)
}

#[cfg(all(feature = "duckdb", not(feature = "valkey")))]
fn run_with_ledger_and_optional_hot_state<L>(
    ledger: &L,
    request: HeartbeatRunRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
{
    validate_namespace_scope(request.valkey_url, request.namespace)?;
    if request.valkey_url.is_some() {
        return Err(invalid_input(
            "`control heartbeat --valkey-url` requires the `valkey` feature",
        ));
    }
    heartbeat_with_ledger(ledger, request)
}

#[cfg(feature = "duckdb")]
fn validate_namespace_scope(valkey_url: Option<&str>, namespace: Option<&str>) -> io::Result<()> {
    if valkey_url.is_none() && namespace.is_some() {
        return Err(invalid_input(
            "`control heartbeat --namespace` requires `--valkey-url`",
        ));
    }
    Ok(())
}

#[cfg(feature = "duckdb")]
fn heartbeat_with_ledger<L>(
    ledger: &L,
    request: HeartbeatRunRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
{
    let heartbeat = build_worker_heartbeat(
        request.worker_id,
        request.observed_at_ms,
        request.expires_at_ms,
        request.metadata,
    )?;
    let run_id = xiuxian_qianji_control::RunId::new(request.run_id)
        .map_err(|error| control_error(&error))?;
    let record = xiuxian_qianji_control::record_worker_heartbeat(
        ledger,
        xiuxian_qianji_control::WorkerHeartbeatJournalRecord::new(run_id, heartbeat),
    )
    .map_err(|error| control_error(&error))?;
    render_heartbeat_record(&record, request.json)
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
pub(crate) async fn heartbeat_with_hot_state<L, H>(
    ledger: &L,
    hot_state: &H,
    request: HeartbeatHotStateRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    let heartbeat = build_worker_heartbeat(
        request.worker_id,
        request.observed_at_ms,
        request.expires_at_ms,
        request.metadata,
    )?;
    let run_id = xiuxian_qianji_control::RunId::new(request.run_id)
        .map_err(|error| control_error(&error))?;
    let record = xiuxian_qianji_control::record_worker_heartbeat_with_hot_state(
        ledger,
        hot_state,
        xiuxian_qianji_control::WorkerHeartbeatJournalRecord::new(run_id, heartbeat),
    )
    .await
    .map_err(|error| control_error(&error))?;
    render_heartbeat_record(&record, request.json)
}

#[cfg(any(feature = "duckdb", test))]
fn build_worker_heartbeat(
    worker_id: &str,
    observed_at_ms: u64,
    expires_at_ms: u64,
    metadata: Option<&str>,
) -> io::Result<xiuxian_qianji_control::WorkerHeartbeat> {
    if expires_at_ms <= observed_at_ms {
        return Err(invalid_input(
            "`control heartbeat` requires `--expires-at-ms` to be greater than `--observed-at-ms`",
        ));
    }

    let metadata = parse_metadata(metadata)?;
    let heartbeat = xiuxian_qianji_control::WorkerHeartbeat {
        worker_id: xiuxian_qianji_control::WorkerId::new(worker_id)
            .map_err(|error| control_error(&error))?,
        observed_at_ms,
        expires_at_ms,
        metadata,
    };
    Ok(heartbeat)
}

#[cfg(feature = "duckdb")]
fn render_heartbeat_record(
    record: &xiuxian_qianji_control::ControlEventRecord,
    json: bool,
) -> io::Result<ControlCliOutput> {
    let rendered = if json {
        serde_json::to_string_pretty(record).map_err(io::Error::other)?
    } else {
        render_heartbeat_text(record)
    };
    Ok(ControlCliOutput { rendered })
}

fn parse_ms(flag_name: &str, command_name: &str, value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{flag_name}` value `{value}` for `{command_name}`: {error}"
        ))
    })
}

fn parse_metadata(metadata: Option<&str>) -> io::Result<serde_json::Value> {
    match metadata {
        Some(value) => serde_json::from_str(value).map_err(|error| {
            invalid_input(format!(
                "invalid `--metadata` JSON for `control heartbeat`: {error}"
            ))
        }),
        None => Ok(serde_json::Value::Null),
    }
}

#[cfg(feature = "duckdb")]
fn render_heartbeat_text(record: &xiuxian_qianji_control::ControlEventRecord) -> String {
    let xiuxian_qianji_control::ControlEventKind::WorkerHeartbeatObserved { heartbeat } =
        &record.event.kind
    else {
        return "# Qianji Control Heartbeat\n\n- Status: `invalid-event`\n".to_string();
    };
    format!(
        concat!(
            "# Qianji Control Heartbeat\n\n",
            "- Sequence: `{}`\n",
            "- Run: `{}`\n",
            "- Worker: `{}`\n",
            "- Observed at ms: `{}`\n",
            "- Expires at ms: `{}`\n"
        ),
        record.sequence,
        record.event.run_id.as_str(),
        heartbeat.worker_id.as_str(),
        heartbeat.observed_at_ms,
        heartbeat.expires_at_ms
    )
}

#[cfg(feature = "duckdb")]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
