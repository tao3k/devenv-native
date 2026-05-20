use std::io;

#[cfg(all(feature = "duckdb", feature = "valkey"))]
use super::error::control_error;
#[cfg(all(feature = "duckdb", feature = "valkey"))]
use crate::qianji_cli::control_cli::render::{
    render_recovery_loop_json, render_recovery_loop_text,
};
use crate::qianji_cli::control_cli::{ControlCliCommand, ControlCliOutput};
#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
use crate::qianji_cli::invalid_input;

pub(super) fn run_apply_recovery_plan_from_command(
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
