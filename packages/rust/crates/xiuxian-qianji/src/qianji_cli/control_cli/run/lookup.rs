use std::io;

use super::error::control_error;
use crate::qianji_cli::control_cli::ControlCliOutput;
use crate::qianji_cli::control_cli::render::{
    render_activity_view_json, render_activity_view_text, render_agent_decision_json,
    render_agent_decision_text, render_control_history_json, render_control_history_text,
    render_run_view_json, render_run_view_text, render_step_view_json, render_step_view_text,
    render_timer_view_json, render_timer_view_text,
};
use crate::qianji_cli::invalid_input;

#[cfg(feature = "duckdb")]
pub(super) fn run_decision_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    step_id: Option<&str>,
    decision_id: &str,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        AgentDecisionId, ControlLedger, DuckDbControlLedger, RunId, StepId,
    };

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let decision_id = AgentDecisionId::new(decision_id).map_err(|error| control_error(&error))?;
    let view = ledger
        .load_run_view(&run_id)
        .map_err(|error| control_error(&error))?;
    let decision = if let Some(step_id) = step_id {
        let step_id = StepId::new(step_id).map_err(|error| control_error(&error))?;
        let step = view.steps.get(&step_id).ok_or_else(|| {
            invalid_input(format!(
                "`control decision` could not find step `{}` in run `{}`",
                step_id.as_str(),
                run_id.as_str()
            ))
        })?;
        step.agent_decisions.get(&decision_id)
    } else {
        view.agent_decisions.get(&decision_id)
    }
    .ok_or_else(|| {
        invalid_input(format!(
            "`control decision` could not find decision `{}` in run `{}`",
            decision_id.as_str(),
            run_id.as_str()
        ))
    })?;

    let rendered = if json {
        render_agent_decision_json(decision)?
    } else {
        render_agent_decision_text(decision)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_decision_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _step_id: Option<&str>,
    _decision_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control decision` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_history_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let records = ledger
        .load_events(&run_id)
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_control_history_json(&records)?
    } else {
        render_control_history_text(&run_id, &records)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_history_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control history` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_view_command(
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
    let rendered = if json {
        render_run_view_json(&view)?
    } else {
        render_run_view_text(&view)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(feature = "duckdb")]
pub(super) fn run_activity_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    step_id: Option<&str>,
    activity_id: &str,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ActivityId, ControlLedger, DuckDbControlLedger, RunId, StepId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let activity_id = ActivityId::new(activity_id).map_err(|error| control_error(&error))?;
    let view = ledger
        .load_run_view(&run_id)
        .map_err(|error| control_error(&error))?;
    let activity = if let Some(step_id) = step_id {
        let step_id = StepId::new(step_id).map_err(|error| control_error(&error))?;
        let step = view.steps.get(&step_id).ok_or_else(|| {
            invalid_input(format!(
                "`control activity` could not find step `{}` in run `{}`",
                step_id.as_str(),
                run_id.as_str()
            ))
        })?;
        step.activities.get(&activity_id)
    } else {
        view.activities.get(&activity_id)
    }
    .ok_or_else(|| {
        invalid_input(format!(
            "`control activity` could not find activity `{}` in run `{}`",
            activity_id.as_str(),
            run_id.as_str()
        ))
    })?;

    let rendered = if json {
        render_activity_view_json(activity)?
    } else {
        render_activity_view_text(activity)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_activity_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _step_id: Option<&str>,
    _activity_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control activity` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_step_command(
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
            "`control step` could not find step `{}` in run `{}`",
            step_id.as_str(),
            run_id.as_str()
        ))
    })?;
    let rendered = if json {
        render_step_view_json(step)?
    } else {
        render_step_view_text(step)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(feature = "duckdb")]
pub(super) fn run_timer_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    step_id: Option<&str>,
    timer_id: &str,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId, StepId, TimerId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let timer_id = TimerId::new(timer_id).map_err(|error| control_error(&error))?;
    let view = ledger
        .load_run_view(&run_id)
        .map_err(|error| control_error(&error))?;
    let timer = if let Some(step_id) = step_id {
        let step_id = StepId::new(step_id).map_err(|error| control_error(&error))?;
        let step = view.steps.get(&step_id).ok_or_else(|| {
            invalid_input(format!(
                "`control timer` could not find step `{}` in run `{}`",
                step_id.as_str(),
                run_id.as_str()
            ))
        })?;
        step.timers.get(&timer_id)
    } else {
        view.timers.get(&timer_id)
    }
    .ok_or_else(|| {
        invalid_input(format!(
            "`control timer` could not find timer `{}` in run `{}`",
            timer_id.as_str(),
            run_id.as_str()
        ))
    })?;

    let rendered = if json {
        render_timer_view_json(timer)?
    } else {
        render_timer_view_text(timer)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_timer_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _step_id: Option<&str>,
    _timer_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control timer` requires the `duckdb` feature",
    ))
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_step_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _step_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control step` requires the `duckdb` feature",
    ))
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_view_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control view` requires the `duckdb` feature",
    ))
}
