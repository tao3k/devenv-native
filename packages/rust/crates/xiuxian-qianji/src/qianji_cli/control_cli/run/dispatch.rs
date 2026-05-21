use std::io;

use super::inventory::{
    run_activity_queue_command, run_costs_command, run_lease_command, run_leases_command,
    run_signal_command, run_signals_command, run_timers_command,
};
use super::lookup::{
    run_activity_command, run_decision_command, run_history_command, run_step_command,
    run_timer_command, run_view_command,
};
use super::recovery::run_apply_recovery_plan_from_command;
use super::state::{
    run_hot_state_command, run_query_state_command, run_recovery_snapshot_command,
    run_summary_command,
};
use crate::qianji_cli::control_cli::{ControlCliCommand, ControlCliOutput};

pub(crate) fn run_control_command_impl(
    command: &ControlCliCommand,
) -> io::Result<ControlCliOutput> {
    match command {
        ControlCliCommand::Activity { .. } => run_activity_from_command(command),
        ControlCliCommand::ActivityComplete { .. } => run_activity_complete_from_command(command),
        ControlCliCommand::ActivityFail { .. } => run_activity_fail_from_command(command),
        ControlCliCommand::ActivityStart { .. } => run_activity_start_from_command(command),
        ControlCliCommand::ActivityQueue {
            ledger_path,
            run_id,
            task_queue,
            json,
        } => run_activity_queue_command(ledger_path, run_id, task_queue.as_deref(), *json),
        ControlCliCommand::ApplyRecoveryPlan { .. } => {
            run_apply_recovery_plan_from_command(command)
        }
        ControlCliCommand::Costs {
            ledger_path,
            run_id,
            json,
        } => run_costs_command(ledger_path, run_id, *json),
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
        } => super::super::heartbeat::run(
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
        ControlCliCommand::Leases {
            ledger_path,
            run_id,
            json,
        } => run_leases_command(ledger_path, run_id, *json),
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
        ControlCliCommand::Summary {
            ledger_path,
            run_id,
            now_ms,
            json,
        } => run_summary_command(ledger_path, run_id, *now_ms, *json),
        _ => run_control_command_tail(command),
    }
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
        ControlCliCommand::Signals {
            ledger_path,
            run_id,
            json,
        } => run_signals_command(ledger_path, run_id, *json),
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
        ControlCliCommand::Timers {
            ledger_path,
            run_id,
            json,
        } => run_timers_command(ledger_path, run_id, *json),
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

fn run_activity_start_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityStart {
        ledger_path,
        run_id,
        step_id,
        activity_id,
        worker_id,
        started_at_ms,
        attempt,
        json,
    } = command
    else {
        unreachable!("activity-start runner received a non-activity-start command");
    };
    super::super::activity_start::run(super::super::activity_start::ActivityStartRunRequest {
        ledger_path,
        run_id,
        step_id: step_id.as_deref(),
        activity_id,
        worker_id,
        started_at_ms: *started_at_ms,
        attempt: *attempt,
        json: *json,
    })
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
    super::super::activity_finish::run_complete(
        super::super::activity_finish::ActivityCompleteRunRequest {
            ledger_path,
            run_id,
            step_id: step_id.as_deref(),
            activity_id,
            completed_at_ms: *completed_at_ms,
            output_hash: output_hash.as_deref(),
            metadata: metadata.as_deref(),
            json: *json,
        },
    )
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
    super::super::activity_finish::run_fail(super::super::activity_finish::ActivityFailRunRequest {
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
