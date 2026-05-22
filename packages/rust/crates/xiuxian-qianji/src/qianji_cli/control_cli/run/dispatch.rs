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
        ControlCliCommand::Activity { .. }
        | ControlCliCommand::ActivityClaim { .. }
        | ControlCliCommand::ActivityComplete { .. }
        | ControlCliCommand::ActivityCompleteWorkerTask { .. }
        | ControlCliCommand::ActivityFail { .. }
        | ControlCliCommand::ActivityFailWorkerTask { .. }
        | ControlCliCommand::ActivityMirror { .. }
        | ControlCliCommand::ActivityReclaim { .. }
        | ControlCliCommand::ActivityRelease { .. }
        | ControlCliCommand::ActivityScheduleLlm { .. }
        | ControlCliCommand::ActivitySettle { .. }
        | ControlCliCommand::ActivityStart { .. }
        | ControlCliCommand::ActivityStartWorkerTask { .. }
        | ControlCliCommand::ActivityTake { .. }
        | ControlCliCommand::ActivityWorkerLoop { .. }
        | ControlCliCommand::ActivityWorkerOnce { .. } => run_activity_control_command(command),
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
        ControlCliCommand::Heartbeat { .. } => run_heartbeat_from_command(command),
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
        ControlCliCommand::LlmActivities {
            ledger_path,
            run_id,
            require_request_audit,
            json,
        } => super::super::llm_inventory::run(ledger_path, run_id, *require_request_audit, *json),
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

fn run_heartbeat_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::Heartbeat {
        ledger_path,
        valkey_url,
        namespace,
        run_id,
        worker_id,
        observed_at_ms,
        expires_at_ms,
        metadata,
        json,
    } = command
    else {
        unreachable!("heartbeat runner received a non-heartbeat command");
    };
    super::super::heartbeat::run(super::super::heartbeat::HeartbeatRunRequest {
        ledger_path,
        valkey_url: valkey_url.as_deref(),
        namespace: namespace.as_deref(),
        run_id,
        worker_id,
        observed_at_ms: *observed_at_ms,
        expires_at_ms: *expires_at_ms,
        metadata: metadata.as_deref(),
        json: *json,
    })
}

fn run_activity_control_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    match command {
        ControlCliCommand::Activity { .. } => run_activity_from_command(command),
        ControlCliCommand::ActivityClaim { .. } => run_activity_claim_from_command(command),
        ControlCliCommand::ActivityComplete { .. } => run_activity_complete_from_command(command),
        ControlCliCommand::ActivityCompleteWorkerTask { .. } => {
            run_activity_complete_worker_task_command(command)
        }
        ControlCliCommand::ActivityFail { .. } => run_activity_fail_from_command(command),
        ControlCliCommand::ActivityFailWorkerTask { .. } => {
            run_activity_fail_worker_task_command(command)
        }
        ControlCliCommand::ActivityMirror { .. } => run_activity_mirror_from_command(command),
        ControlCliCommand::ActivityReclaim { .. } => run_activity_reclaim_from_command(command),
        ControlCliCommand::ActivityRelease { .. } => run_activity_release_from_command(command),
        ControlCliCommand::ActivityScheduleLlm { .. } => {
            run_activity_schedule_llm_from_command(command)
        }
        ControlCliCommand::ActivitySettle { .. } => run_activity_settle_from_command(command),
        ControlCliCommand::ActivityStart { .. } => run_activity_start_from_command(command),
        ControlCliCommand::ActivityStartWorkerTask { .. } => {
            run_activity_start_worker_task_command(command)
        }
        ControlCliCommand::ActivityTake { .. } => run_activity_take_from_command(command),
        ControlCliCommand::ActivityWorkerLoop { .. } => {
            run_activity_worker_loop_from_command(command)
        }
        ControlCliCommand::ActivityWorkerOnce { .. } => {
            run_activity_worker_once_from_command(command)
        }
        _ => unreachable!("activity control runner received a non-activity command"),
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

fn run_activity_claim_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityClaim {
        valkey_url,
        namespace,
        worker_id,
        task_queue,
        now_ms,
        lease_ttl_ms,
        json,
    } = command
    else {
        unreachable!("activity-claim runner received a non-activity-claim command");
    };
    super::super::activity_claim::run(
        super::super::activity_claim::WorkerActivityClaimRunRequest {
            valkey_url,
            namespace: namespace.as_deref(),
            worker_id,
            task_queue: task_queue.as_deref(),
            now_ms: *now_ms,
            lease_ttl_ms: *lease_ttl_ms,
            json: *json,
        },
    )
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

fn run_activity_mirror_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityMirror {
        ledger_path,
        valkey_url,
        namespace,
        run_id,
        task_queue,
        priority,
        not_before_ms,
        metadata,
        json,
    } = command
    else {
        unreachable!("activity-mirror runner received a non-activity-mirror command");
    };
    super::super::activity_mirror::run(
        super::super::activity_mirror::WorkerActivityMirrorRunRequest {
            ledger_path,
            valkey_url,
            namespace: namespace.as_deref(),
            run_id,
            task_queue: task_queue.as_deref(),
            priority: *priority,
            not_before_ms: *not_before_ms,
            metadata: metadata.as_deref(),
            json: *json,
        },
    )
}

fn run_activity_reclaim_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityReclaim {
        valkey_url,
        namespace,
        lease_json,
        now_ms,
        json,
    } = command
    else {
        unreachable!("activity-reclaim runner received a non-activity-reclaim command");
    };
    super::super::activity_reclaim::run(
        super::super::activity_reclaim::WorkerActivityReclaimRunRequest {
            valkey_url,
            namespace: namespace.as_deref(),
            lease_json,
            now_ms: *now_ms,
            json: *json,
        },
    )
}

fn run_activity_release_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityRelease {
        valkey_url,
        namespace,
        lease_json,
        json,
    } = command
    else {
        unreachable!("activity-release runner received a non-activity-release command");
    };
    super::super::activity_release::run(
        super::super::activity_release::WorkerActivityReleaseRunRequest {
            valkey_url,
            namespace: namespace.as_deref(),
            lease_json,
            json: *json,
        },
    )
}

fn run_activity_settle_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivitySettle {
        ledger_path,
        valkey_url,
        namespace,
        leased_task_json,
        outcome,
        settled_at_ms,
        output_hash,
        error_code,
        message,
        retryable,
        metadata,
        json,
    } = command
    else {
        unreachable!("activity-settle runner received a non-activity-settle command");
    };
    super::super::activity_settle::run(
        super::super::activity_settle::WorkerActivitySettleRunRequest {
            ledger_path,
            valkey_url,
            namespace: namespace.as_deref(),
            leased_task_json,
            outcome: *outcome,
            settled_at_ms: *settled_at_ms,
            output_hash: output_hash.as_deref(),
            error_code: error_code.as_deref(),
            message: message.as_deref(),
            retryable: *retryable,
            metadata: metadata.as_deref(),
            json: *json,
        },
    )
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

fn run_activity_schedule_llm_from_command(
    command: &ControlCliCommand,
) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityScheduleLlm {
        ledger_path,
        run_id,
        step_id,
        occurred_at_ms,
        llm_activity_json,
        json,
    } = command
    else {
        unreachable!("activity-schedule-llm runner received a non-activity-schedule-llm command");
    };
    super::super::activity_schedule_llm::run(
        super::super::activity_schedule_llm::ActivityScheduleLlmRunRequest {
            ledger_path,
            run_id,
            step_id: step_id.as_deref(),
            occurred_at_ms: *occurred_at_ms,
            llm_activity_json,
            json: *json,
        },
    )
}

fn run_activity_start_worker_task_command(
    command: &ControlCliCommand,
) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityStartWorkerTask {
        ledger_path,
        worker_task_json,
        worker_id,
        started_at_ms,
        json,
    } = command
    else {
        unreachable!("activity-start worker-task runner received a non-worker-task command");
    };
    super::super::activity_start::run_worker_task(
        super::super::activity_start::WorkerActivityStartRunRequest {
            ledger_path,
            worker_task_json,
            worker_id,
            started_at_ms: *started_at_ms,
            json: *json,
        },
    )
}

fn run_activity_take_from_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityTake {
        ledger_path,
        valkey_url,
        namespace,
        worker_id,
        task_queue,
        now_ms,
        lease_ttl_ms,
        json,
    } = command
    else {
        unreachable!("activity-take runner received a non-activity-take command");
    };
    super::super::activity_take::run(super::super::activity_take::WorkerActivityTakeRunRequest {
        ledger_path,
        valkey_url,
        namespace: namespace.as_deref(),
        worker_id,
        task_queue: task_queue.as_deref(),
        now_ms: *now_ms,
        lease_ttl_ms: *lease_ttl_ms,
        json: *json,
    })
}

fn run_activity_worker_once_from_command(
    command: &ControlCliCommand,
) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityWorkerOnce {
        ledger_path,
        valkey_url,
        namespace,
        worker_id,
        task_queue,
        now_ms,
        lease_ttl_ms,
        executor,
        outcome,
        settled_at_ms,
        output_ref_json,
        output_hash,
        output_artifact_path,
        output_artifact_content,
        output_artifact_id,
        output_artifact_kind,
        openai_compatible_base_url,
        openai_compatible_api_key,
        openai_compatible_timeout_ms,
        error_code,
        message,
        retryable,
        metadata,
        json,
    } = command
    else {
        unreachable!("activity-worker-once runner received a non-activity-worker-once command");
    };
    let request = super::super::activity_worker_once::ActivityWorkerOnceRunRequest {
        ledger_path,
        valkey_url,
        namespace: namespace.as_deref(),
        worker_id,
        task_queue: task_queue.as_deref(),
        now_ms: *now_ms,
        lease_ttl_ms: *lease_ttl_ms,
        executor: *executor,
        outcome: *outcome,
        settled_at_ms: *settled_at_ms,
        output_ref_json: output_ref_json.as_deref(),
        output_hash: output_hash.as_deref(),
        output_artifact_path: output_artifact_path.as_deref(),
        output_artifact_content: output_artifact_content.as_deref(),
        output_artifact_id: output_artifact_id.as_deref(),
        output_artifact_kind: output_artifact_kind.as_deref(),
        openai_compatible_base_url: openai_compatible_base_url.as_deref(),
        openai_compatible_api_key: openai_compatible_api_key.as_deref(),
        openai_compatible_timeout_ms: *openai_compatible_timeout_ms,
        error_code: error_code.as_deref(),
        message: message.as_deref(),
        retryable: *retryable,
        metadata: metadata.as_deref(),
        json: *json,
    };
    super::super::activity_worker_once::run(&request)
}

fn run_activity_worker_loop_from_command(
    command: &ControlCliCommand,
) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityWorkerLoop {
        ledger_path,
        valkey_url,
        namespace,
        worker_id,
        task_queue,
        now_ms,
        now_step_ms,
        lease_ttl_ms,
        heartbeat_ttl_ms,
        poll_limit,
        empty_limit,
        executor,
        outcome,
        settled_at_ms,
        settled_step_ms,
        output_hash,
        output_artifact_dir,
        output_artifact_kind,
        openai_compatible_base_url,
        openai_compatible_api_key,
        openai_compatible_timeout_ms,
        error_code,
        message,
        retryable,
        metadata,
        json,
    } = command
    else {
        unreachable!("activity-worker-loop runner received a non-activity-worker-loop command");
    };
    let request = super::super::activity_worker_loop::ActivityWorkerLoopRunRequest {
        ledger_path,
        valkey_url,
        namespace: namespace.as_deref(),
        worker_id,
        task_queue: task_queue.as_deref(),
        now_ms: *now_ms,
        now_step_ms: *now_step_ms,
        lease_ttl_ms: *lease_ttl_ms,
        heartbeat_ttl_ms: *heartbeat_ttl_ms,
        poll_limit: *poll_limit,
        empty_limit: *empty_limit,
        executor: *executor,
        outcome: *outcome,
        settled_at_ms: *settled_at_ms,
        settled_step_ms: *settled_step_ms,
        output_hash: output_hash.as_deref(),
        output_artifact_dir: output_artifact_dir.as_deref(),
        output_artifact_kind: output_artifact_kind.as_deref(),
        openai_compatible_base_url: openai_compatible_base_url.as_deref(),
        openai_compatible_api_key: openai_compatible_api_key.as_deref(),
        openai_compatible_timeout_ms: *openai_compatible_timeout_ms,
        error_code: error_code.as_deref(),
        message: message.as_deref(),
        retryable: *retryable,
        metadata: metadata.as_deref(),
        json: *json,
    };
    super::super::activity_worker_loop::run(&request)
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

fn run_activity_complete_worker_task_command(
    command: &ControlCliCommand,
) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityCompleteWorkerTask {
        ledger_path,
        worker_task_json,
        completed_at_ms,
        output_hash,
        metadata,
        json,
    } = command
    else {
        unreachable!("activity-complete worker-task runner received a non-worker-task command");
    };
    super::super::activity_finish::run_complete_worker_task(
        super::super::activity_finish::WorkerActivityCompleteRunRequest {
            ledger_path,
            worker_task_json,
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

fn run_activity_fail_worker_task_command(
    command: &ControlCliCommand,
) -> io::Result<ControlCliOutput> {
    let ControlCliCommand::ActivityFailWorkerTask {
        ledger_path,
        worker_task_json,
        failed_at_ms,
        error_code,
        message,
        retryable,
        metadata,
        json,
    } = command
    else {
        unreachable!("activity-fail worker-task runner received a non-worker-task command");
    };
    super::super::activity_finish::run_fail_worker_task(
        super::super::activity_finish::WorkerActivityFailRunRequest {
            ledger_path,
            worker_task_json,
            failed_at_ms: *failed_at_ms,
            error_code,
            message,
            retryable: *retryable,
            metadata: metadata.as_deref(),
            json: *json,
        },
    )
}
