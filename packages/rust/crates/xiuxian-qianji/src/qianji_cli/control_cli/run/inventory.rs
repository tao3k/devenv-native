use std::io;

use super::error::control_error;
use crate::qianji_cli::control_cli::ControlCliOutput;
use crate::qianji_cli::control_cli::render::{
    render_activity_queue_projection_json, render_activity_queue_projection_text,
    render_cost_inventory_json, render_cost_inventory_text, render_signal_append_json,
    render_signal_append_text, render_signal_inventory_json, render_signal_inventory_text,
    render_step_lease_json, render_step_lease_text, render_step_leases_json,
    render_step_leases_text, render_timer_inventory_json, render_timer_inventory_text,
};
use crate::qianji_cli::invalid_input;

#[cfg(feature = "duckdb")]
pub(super) fn run_leases_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let view = ledger
        .load_run_view(&run_id)
        .map_err(|error| control_error(&error))?;
    let leases = view
        .steps
        .values()
        .filter_map(|step| step.active_lease.clone())
        .collect::<Vec<_>>();
    let rendered = if json {
        render_step_leases_json(&leases)?
    } else {
        render_step_leases_text(&leases)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_leases_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control leases` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_lease_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    step_id: &str,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId, StepId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let step_id = StepId::new(step_id).map_err(|error| control_error(&error))?;
    let view = ledger
        .load_run_view(&run_id)
        .map_err(|error| control_error(&error))?;
    let step = view.steps.get(&step_id).ok_or_else(|| {
        invalid_input(format!(
            "could not find step `{}` in run `{}`",
            step_id.as_str(),
            run_id.as_str()
        ))
    })?;
    let lease = step.active_lease.as_ref().ok_or_else(|| {
        invalid_input(format!(
            "step `{}` in run `{}` has no active lease",
            step_id.as_str(),
            run_id.as_str()
        ))
    })?;
    let rendered = if json {
        render_step_lease_json(lease)?
    } else {
        render_step_lease_text(lease)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_lease_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _step_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control lease` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_costs_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let projection = ledger
        .load_cost_inventory_projection(&run_id)
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_cost_inventory_json(&projection)?
    } else {
        render_cost_inventory_text(&projection)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_costs_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control costs` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_activity_queue_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    task_queue: Option<&str>,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId, TaskQueue};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let task_queue = task_queue
        .map(TaskQueue::new)
        .transpose()
        .map_err(|error| control_error(&error))?;
    let projection = ledger
        .load_activity_queue_projection(&run_id, task_queue.as_ref())
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_activity_queue_projection_json(&projection)?
    } else {
        render_activity_queue_projection_text(&projection)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_activity_queue_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _task_queue: Option<&str>,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control activity-queue` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_signal_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    step_id: Option<&str>,
    signal_name: &str,
    payload: &str,
    received_at_ms: u64,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        DuckDbControlLedger, RecoveryItemScope, RunId, SignalName, SignalReceiveJournalRecord,
        SignalRecord, StepId, record_signal_received,
    };

    let payload_metadata = serde_json::from_str::<serde_json::Value>(payload).map_err(|error| {
        invalid_input(format!(
            "invalid `--payload` JSON for `control signal`: {error}"
        ))
    })?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let signal_name = SignalName::new(signal_name).map_err(|error| control_error(&error))?;
    let signal = SignalRecord {
        signal_name,
        payload_ref: None,
        payload_hash: None,
        metadata: payload_metadata,
    };
    let scope = if let Some(step_id) = step_id {
        RecoveryItemScope::step(StepId::new(step_id).map_err(|error| control_error(&error))?)
    } else {
        RecoveryItemScope::run()
    };
    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let record = record_signal_received(
        &ledger,
        SignalReceiveJournalRecord::new(run_id, scope, signal, received_at_ms),
    )
    .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_signal_append_json(&record)?
    } else {
        render_signal_append_text(&record)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_signal_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _step_id: Option<&str>,
    _signal_name: &str,
    _payload: &str,
    _received_at_ms: u64,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control signal` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_signals_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let projection = ledger
        .load_signal_inventory_projection(&run_id)
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_signal_inventory_json(&projection)?
    } else {
        render_signal_inventory_text(&projection)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_signals_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control signals` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_timers_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let projection = ledger
        .load_timer_inventory_projection(&run_id)
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_timer_inventory_json(&projection)?
    } else {
        render_timer_inventory_text(&projection)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_timers_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control timers` requires the `duckdb` feature",
    ))
}
