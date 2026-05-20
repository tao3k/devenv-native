use std::io;

use crate::qianji_cli::invalid_input;

use super::render::{
    ControlStateQueryView, render_activity_queue_projection_json,
    render_activity_queue_projection_text, render_activity_view_json, render_activity_view_text,
    render_agent_decision_json, render_agent_decision_text, render_control_history_json,
    render_control_history_text, render_control_state_query_json, render_control_state_query_text,
    render_recovery_snapshot_json, render_recovery_snapshot_text, render_run_view_json,
    render_run_view_text, render_signal_append_json, render_signal_append_text,
    render_step_lease_json, render_step_lease_text, render_step_view_json, render_step_view_text,
    render_timer_view_json, render_timer_view_text,
};
#[cfg(feature = "valkey")]
use super::render::{render_hot_state_snapshot_json, render_hot_state_snapshot_text};
#[cfg(all(feature = "duckdb", feature = "valkey"))]
use super::render::{render_recovery_loop_json, render_recovery_loop_text};
use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn run_control_command_impl(
    command: &ControlCliCommand,
) -> io::Result<ControlCliOutput> {
    match command {
        ControlCliCommand::Activity { .. } => run_activity_from_command(command),
        ControlCliCommand::ActivityComplete { .. } => run_activity_complete_from_command(command),
        ControlCliCommand::ActivityFail { .. } => run_activity_fail_from_command(command),
        ControlCliCommand::ActivityStart {
            ledger_path,
            run_id,
            step_id,
            activity_id,
            worker_id,
            started_at_ms,
            attempt,
            json,
        } => super::activity_start::run(super::activity_start::ActivityStartRunRequest {
            ledger_path,
            run_id,
            step_id: step_id.as_deref(),
            activity_id,
            worker_id,
            started_at_ms: *started_at_ms,
            attempt: *attempt,
            json: *json,
        }),
        ControlCliCommand::ActivityQueue {
            ledger_path,
            run_id,
            task_queue,
            json,
        } => run_activity_queue_command(ledger_path, run_id, task_queue.as_deref(), *json),
        ControlCliCommand::ApplyRecoveryPlan { .. } => {
            run_apply_recovery_plan_from_command(command)
        }
        ControlCliCommand::Decision {
            ledger_path,
            run_id,
            step_id,
            decision_id,
            json,
        } => run_decision_command(ledger_path, run_id, step_id.as_deref(), decision_id, *json),
        ControlCliCommand::History {
            ledger_path,
            run_id,
            json,
        } => run_history_command(ledger_path, run_id, *json),
        ControlCliCommand::Heartbeat {
            ledger_path,
            run_id,
            worker_id,
            observed_at_ms,
            expires_at_ms,
            metadata,
            json,
        } => super::heartbeat::run(
            ledger_path,
            run_id,
            worker_id,
            *observed_at_ms,
            *expires_at_ms,
            metadata.as_ref(),
            *json,
        ),
        ControlCliCommand::HotState {
            valkey_url,
            namespace,
            now_ms,
            json,
        } => run_hot_state_command(valkey_url, namespace.as_deref(), *now_ms, *json),
        ControlCliCommand::Lease {
            ledger_path,
            run_id,
            step_id,
            json,
        } => run_lease_command(ledger_path, run_id, step_id, *json),
        ControlCliCommand::QueryState {
            ledger_path,
            run_id,
            now_ms,
            json,
        } => run_query_state_command(ledger_path, run_id, *now_ms, *json),
        ControlCliCommand::RecoverySnapshot {
            ledger_path,
            run_id,
            now_ms,
            json,
        } => run_recovery_snapshot_command(ledger_path, run_id, *now_ms, *json),
        _ => run_control_command_tail(command),
    }
}

#[cfg(feature = "duckdb")]
fn run_lease_command(
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
fn run_lease_command(
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
fn run_activity_queue_command(
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
fn run_activity_queue_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _task_queue: Option<&str>,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control activity-queue` requires the `duckdb` feature",
    ))
}

fn run_control_command_tail(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    match command {
        ControlCliCommand::Signal {
            ledger_path,
            run_id,
            step_id,
            signal_name,
            payload,
            received_at_ms,
            json,
        } => run_signal_command(
            ledger_path,
            run_id,
            step_id.as_deref(),
            signal_name,
            payload,
            *received_at_ms,
            *json,
        ),
        ControlCliCommand::View {
            ledger_path,
            run_id,
            json,
        } => run_view_command(ledger_path, run_id, *json),
        ControlCliCommand::Step {
            ledger_path,
            run_id,
            step_id,
            json,
        } => run_step_command(ledger_path, run_id, step_id, *json),
        ControlCliCommand::Timer {
            ledger_path,
            run_id,
            step_id,
            timer_id,
            json,
        } => run_timer_command(ledger_path, run_id, step_id.as_deref(), timer_id, *json),
        _ => unreachable!("control command tail received a head command"),
    }
}

fn run_activity_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::Activity {
        ledger_path,
        run_id,
        step_id,
        activity_id,
        json,
    } = command
    else {
        unreachable!("activity runner received a non-activity command");
    };
    run_activity_command(ledger_path, run_id, step_id.as_deref(), activity_id, *json)
}

fn run_activity_complete_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityComplete {
        ledger_path,
        run_id,
        step_id,
        activity_id,
        completed_at_ms,
        output_hash,
        metadata,
        json,
    } = command
    else {
        unreachable!("activity-complete runner received a non-activity-complete command");
    };
    super::activity_finish::run_complete(super::activity_finish::ActivityCompleteRunRequest {
        ledger_path,
        run_id,
        step_id: step_id.as_deref(),
        activity_id,
        completed_at_ms: *completed_at_ms,
        output_hash: output_hash.as_deref(),
        metadata: metadata.as_deref(),
        json: *json,
    })
}

fn run_activity_fail_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityFail {
        ledger_path,
        run_id,
        step_id,
        activity_id,
        failed_at_ms,
        error_code,
        message,
        retryable,
        attempt,
        metadata,
        json,
    } = command
    else {
        unreachable!("activity-fail runner received a non-activity-fail command");
    };
    super::activity_finish::run_fail(super::activity_finish::ActivityFailRunRequest {
        ledger_path,
        run_id,
        step_id: step_id.as_deref(),
        activity_id,
        failed_at_ms: *failed_at_ms,
        error_code,
        message,
        retryable: *retryable,
        attempt: *attempt,
        metadata: metadata.as_deref(),
        json: *json,
    })
}

fn run_apply_recovery_plan_from_command(
    command: &ControlCliCommand,
) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ApplyRecoveryPlan { json, .. } = command else {
        unreachable!("recovery runner received a non-recovery command");
    };
    run_apply_recovery_plan_command(ApplyRecoveryPlanCliRequest::from_command(command), *json)
}

#[derive(Clone, Copy)]
struct ApplyRecoveryPlanCliRequest<'a> {
    ledger_path: &'a std::path::Path,
    valkey_url: &'a str,
    namespace: Option<&'a str>,
    run_id: &'a str,
    now_ms: u64,
    attempt: u32,
    reason: &'a str,
    max_attempts: u32,
    backoff_ms: u64,
    require_human_approval: bool,
    priority: i64,
}

impl<'a> ApplyRecoveryPlanCliRequest<'a> {
    fn from_command(command: &'a ControlCliCommand) -> Self {
        let ControlCliCommand::ApplyRecoveryPlan {
            ledger_path,
            valkey_url,
            namespace,
            run_id,
            now_ms,
            attempt,
            reason,
            max_attempts,
            backoff_ms,
            require_human_approval,
            priority,
            json: _,
        } = command
        else {
            unreachable!("recovery request received a non-recovery command");
        };
        Self {
            ledger_path,
            valkey_url,
            namespace: namespace.as_deref(),
            run_id,
            now_ms: *now_ms,
            attempt: *attempt,
            reason,
            max_attempts: *max_attempts,
            backoff_ms: *backoff_ms,
            require_human_approval: *require_human_approval,
            priority: *priority,
        }
    }
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
fn run_apply_recovery_plan_command(
    request: ApplyRecoveryPlanCliRequest<'_>,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        ControlLedger, DuckDbControlLedger, RecoveryAttempt, RecoveryLoopApplicationRequest,
        RecoveryPolicy, RunId, ValkeyHotStateConfig, ValkeyHotStateStore, apply_recovery_plan,
    };

    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(request.run_id).map_err(|error| control_error(&error))?;
    let plan = ledger
        .load_recovery_plan(&run_id, request.now_ms)
        .map_err(|error| control_error(&error))?;
    let attempt = RecoveryAttempt {
        attempt: request.attempt,
        reason: request.reason.to_owned(),
        policy: RecoveryPolicy {
            max_attempts: request.max_attempts,
            backoff_ms: request.backoff_ms,
            require_human_approval: request.require_human_approval,
        },
    };
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
    let application = runtime
        .block_on(apply_recovery_plan(
            &ledger,
            &hot_state,
            RecoveryLoopApplicationRequest::new(plan, attempt, request.now_ms, request.priority),
        ))
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_recovery_loop_json(&application)?
    } else {
        render_recovery_loop_text(&application)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
fn run_apply_recovery_plan_command(
    request: ApplyRecoveryPlanCliRequest<'_>,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.valkey_url,
        request.namespace,
        request.run_id,
        request.now_ms,
        request.attempt,
        request.reason,
        request.max_attempts,
        request.backoff_ms,
        request.require_human_approval,
        request.priority,
    );
    Err(invalid_input(
        "`control apply-recovery-plan` requires the `duckdb` and `valkey` features",
    ))
}

#[cfg(feature = "valkey")]
fn run_hot_state_command(
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
fn run_hot_state_command(
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
fn run_query_state_command(
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
fn run_query_state_command(
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
fn run_signal_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    step_id: Option<&str>,
    signal_name: &str,
    payload: &str,
    received_at_ms: u64,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        ControlEvent, ControlEventKind, ControlLedger, DuckDbControlLedger, RunId, SignalName,
        SignalRecord, StepId,
    };

    let payload_metadata = serde_json::from_str::<serde_json::Value>(payload).map_err(|error| {
        invalid_input(format!(
            "invalid `--payload` JSON for `control signal`: {error}"
        ))
    })?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let signal_name = SignalName::new(signal_name).map_err(|error| control_error(&error))?;
    let event_kind = ControlEventKind::SignalReceived {
        signal: SignalRecord {
            signal_name,
            payload_ref: None,
            payload_hash: None,
            metadata: payload_metadata,
        },
    };
    let event = if let Some(step_id) = step_id {
        ControlEvent::step(
            run_id,
            StepId::new(step_id).map_err(|error| control_error(&error))?,
            received_at_ms,
            event_kind,
        )
    } else {
        ControlEvent::run(run_id, received_at_ms, event_kind)
    };
    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let record = ledger
        .append_event(event)
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_signal_append_json(&record)?
    } else {
        render_signal_append_text(&record)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
fn run_signal_command(
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
fn run_decision_command(
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
fn run_decision_command(
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
fn run_history_command(
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
fn run_history_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control history` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
fn run_recovery_snapshot_command(
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

#[cfg(feature = "duckdb")]
fn run_view_command(
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
fn run_activity_command(
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
fn run_activity_command(
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
fn run_step_command(
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
fn run_timer_command(
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
fn run_timer_command(
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
fn run_step_command(
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
fn run_view_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control view` requires the `duckdb` feature",
    ))
}

#[cfg(not(feature = "duckdb"))]
fn run_recovery_snapshot_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _now_ms: u64,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control recovery-snapshot` requires the `duckdb` feature",
    ))
}

fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
