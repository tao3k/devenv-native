use std::io;

use super::error::control_error;
use crate::qianji_cli::control_cli::ControlCliOutput;
use crate::qianji_cli::control_cli::render::{
    ControlStateQueryView, render_control_state_query_json, render_control_state_query_text,
    render_operator_summary_json, render_operator_summary_text, render_recovery_snapshot_json,
    render_recovery_snapshot_text,
};
#[cfg(feature = "valkey")]
use crate::qianji_cli::control_cli::render::{
    render_hot_state_snapshot_json, render_hot_state_snapshot_text,
};
#[cfg(any(not(feature = "duckdb"), not(feature = "valkey")))]
use crate::qianji_cli::invalid_input;

#[cfg(feature = "valkey")]
pub(super) fn run_hot_state_command(
    valkey_url: &str,
    namespace: Option<&str>,
    now_ms: u64,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{HotStateStore, ValkeyHotStateConfig, ValkeyHotStateStore};

    let config =
        ValkeyHotStateConfig::new(valkey_url.to_string()).map_err(|error| control_error(&error))?;
    let config = if let Some(namespace) = namespace {
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
    let snapshot = runtime
        .block_on(store.load_snapshot(now_ms))
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_hot_state_snapshot_json(&snapshot)?
    } else {
        render_hot_state_snapshot_text(&snapshot)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "valkey"))]
pub(super) fn run_hot_state_command(
    _valkey_url: &str,
    _namespace: Option<&str>,
    _now_ms: u64,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control hot-state` requires the `valkey` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_query_state_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    now_ms: u64,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        ControlLedger, DuckDbControlLedger, RunId, RunRecoverySnapshot, replay_run_view,
    };

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let records = ledger
        .load_events(&run_id)
        .map_err(|error| control_error(&error))?;
    let event_count = records.len();
    let view = replay_run_view(records).map_err(|error| control_error(&error))?;
    let recovery_snapshot = RunRecoverySnapshot::from_view(
        view.recovery_view(now_ms)
            .map_err(|error| control_error(&error))?,
    );
    let state = ControlStateQueryView {
        event_count,
        run_view: &view,
        recovery_snapshot: &recovery_snapshot,
    };
    let rendered = if json {
        render_control_state_query_json(&state)?
    } else {
        render_control_state_query_text(&state)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_query_state_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _now_ms: u64,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control query --state` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_recovery_snapshot_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    now_ms: u64,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let snapshot = ledger
        .load_recovery_snapshot(&run_id, now_ms)
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_recovery_snapshot_json(&snapshot)?
    } else {
        render_recovery_snapshot_text(&snapshot)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_recovery_snapshot_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _now_ms: u64,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control recovery-snapshot` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_summary_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    now_ms: u64,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let summary = ledger
        .load_operator_summary(&run_id, now_ms)
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_operator_summary_json(&summary)?
    } else {
        render_operator_summary_text(&summary)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_summary_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _now_ms: u64,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control summary` requires the `duckdb` feature",
    ))
}
