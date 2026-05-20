use std::io;
use std::path::PathBuf;

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::ControlCliCommand;

pub(super) fn parse_control_command_impl(args: &[String]) -> io::Result<Option<ControlCliCommand>> {
    let Some(command_name) = args.get(1).map(String::as_str) else {
        return Ok(None);
    };
    if command_name != "control" {
        return Ok(None);
    }

    match args.get(2).map(String::as_str) {
        Some("activity") => parse_activity(args).map(Some),
        Some("activity-complete") => super::activity_finish::parse_complete(args).map(Some),
        Some("activity-fail") => super::activity_finish::parse_fail(args).map(Some),
        Some("activity-queue") => parse_activity_queue(args).map(Some),
        Some("activity-start") => super::activity_start::parse(args).map(Some),
        Some("apply-recovery-plan") => parse_apply_recovery_plan(args).map(Some),
        Some("decision") => parse_decision(args).map(Some),
        Some("heartbeat") => super::heartbeat::parse(args).map(Some),
        Some("history") => parse_history(args).map(Some),
        Some("hot-state") => parse_hot_state(args).map(Some),
        Some("query") => parse_query(args).map(Some),
        Some("recovery-snapshot") => parse_recovery_snapshot(args).map(Some),
        Some("signal") => parse_signal(args).map(Some),
        Some("step") => parse_step(args).map(Some),
        Some("timer") => parse_timer(args).map(Some),
        Some("view") => parse_view(args).map(Some),
        Some(other) => Err(invalid_input(format!(
            "unsupported `control` subcommand `{other}`"
        ))),
        None => Err(invalid_input(
            "missing `control` subcommand; expected `activity`, `activity-complete`, `activity-fail`, `activity-queue`, `activity-start`, `apply-recovery-plan`, `decision`, `heartbeat`, `history`, `hot-state`, `query`, `recovery-snapshot`, `signal`, `step`, `timer`, or `view`",
        )),
    }
}

fn parse_activity_queue(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut task_queue = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--task-queue" => {
                task_queue = Some(parse_flag_value(args, &mut index, "--task-queue")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-queue` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::ActivityQueue {
        ledger_path: ledger_path.ok_or_else(|| {
            invalid_input("missing `--ledger <path>` for `control activity-queue`")
        })?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control activity-queue`"))?,
        task_queue,
        json,
    })
}

fn parse_apply_recovery_plan(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ApplyRecoveryPlanArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ApplyRecoveryPlanArgs {
    ledger_path: Option<PathBuf>,
    valkey_url: Option<String>,
    namespace: Option<String>,
    run_id: Option<String>,
    now_ms: Option<u64>,
    attempt: Option<u32>,
    reason: Option<String>,
    max_attempts: Option<u32>,
    backoff_ms: u64,
    require_human_approval: bool,
    priority: i64,
    json: bool,
}

impl ApplyRecoveryPlanArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
        match args[*index].as_str() {
            "--ledger" => {
                self.ledger_path = Some(PathBuf::from(parse_flag_value(args, index, "--ledger")?));
            }
            "--valkey-url" => {
                self.valkey_url = Some(parse_flag_value(args, index, "--valkey-url")?);
            }
            "--namespace" => {
                self.namespace = Some(parse_flag_value(args, index, "--namespace")?);
            }
            "--run-id" => {
                self.run_id = Some(parse_flag_value(args, index, "--run-id")?);
            }
            "--now-ms" => {
                self.now_ms = Some(parse_apply_recovery_plan_now_ms(&parse_flag_value(
                    args, index, "--now-ms",
                )?)?);
            }
            "--attempt" => {
                self.attempt = Some(parse_apply_recovery_plan_attempt(&parse_flag_value(
                    args,
                    index,
                    "--attempt",
                )?)?);
            }
            "--reason" => {
                self.reason = Some(parse_flag_value(args, index, "--reason")?);
            }
            "--max-attempts" => {
                self.max_attempts = Some(parse_apply_recovery_plan_max_attempts(
                    &parse_flag_value(args, index, "--max-attempts")?,
                )?);
            }
            "--backoff-ms" => {
                self.backoff_ms = parse_apply_recovery_plan_backoff_ms(&parse_flag_value(
                    args,
                    index,
                    "--backoff-ms",
                )?)?;
            }
            "--require-human-approval" => {
                self.require_human_approval = true;
            }
            "--priority" => {
                self.priority = parse_apply_recovery_plan_priority(&parse_flag_value(
                    args,
                    index,
                    "--priority",
                )?)?;
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control apply-recovery-plan` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        Ok(ControlCliCommand::ApplyRecoveryPlan {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control apply-recovery-plan`")
            })?,
            valkey_url: self.valkey_url.ok_or_else(|| {
                invalid_input("missing `--valkey-url <url>` for `control apply-recovery-plan`")
            })?,
            namespace: self.namespace,
            run_id: self.run_id.ok_or_else(|| {
                invalid_input("missing `--run-id <id>` for `control apply-recovery-plan`")
            })?,
            now_ms: self.now_ms.ok_or_else(|| {
                invalid_input("missing `--now-ms <ms>` for `control apply-recovery-plan`")
            })?,
            attempt: self.attempt.ok_or_else(|| {
                invalid_input("missing `--attempt <n>` for `control apply-recovery-plan`")
            })?,
            reason: self.reason.ok_or_else(|| {
                invalid_input("missing `--reason <text>` for `control apply-recovery-plan`")
            })?,
            max_attempts: self.max_attempts.ok_or_else(|| {
                invalid_input("missing `--max-attempts <n>` for `control apply-recovery-plan`")
            })?,
            backoff_ms: self.backoff_ms,
            require_human_approval: self.require_human_approval,
            priority: self.priority,
            json: self.json,
        })
    }
}

fn parse_hot_state(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut valkey_url = None;
    let mut namespace = None;
    let mut now_ms = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--valkey-url" => {
                valkey_url = Some(parse_flag_value(args, &mut index, "--valkey-url")?);
            }
            "--namespace" => {
                namespace = Some(parse_flag_value(args, &mut index, "--namespace")?);
            }
            "--now-ms" => {
                now_ms = Some(parse_hot_state_now_ms(&parse_flag_value(
                    args, &mut index, "--now-ms",
                )?)?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control hot-state` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::HotState {
        valkey_url: valkey_url
            .ok_or_else(|| invalid_input("missing `--valkey-url <url>` for `control hot-state`"))?,
        namespace,
        now_ms: now_ms
            .ok_or_else(|| invalid_input("missing `--now-ms <ms>` for `control hot-state`"))?,
        json,
    })
}

fn parse_query(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut now_ms = None;
    let mut state = false;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--now-ms" => {
                now_ms = Some(parse_query_now_ms(&parse_flag_value(
                    args, &mut index, "--now-ms",
                )?)?);
            }
            "--state" => {
                state = true;
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control query` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    if !state {
        return Err(invalid_input(
            "missing `--state` for `control query`; no other query type is supported yet",
        ));
    }

    Ok(ControlCliCommand::QueryState {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control query`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control query`"))?,
        now_ms: now_ms
            .ok_or_else(|| invalid_input("missing `--now-ms <ms>` for `control query`"))?,
        json,
    })
}

fn parse_signal(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut step_id = None;
    let mut signal_name = None;
    let mut payload = None;
    let mut received_at_ms = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--step-id" => {
                step_id = Some(parse_flag_value(args, &mut index, "--step-id")?);
            }
            "--signal-name" => {
                signal_name = Some(parse_flag_value(args, &mut index, "--signal-name")?);
            }
            "--payload" => {
                payload = Some(parse_flag_value(args, &mut index, "--payload")?);
            }
            "--received-at-ms" => {
                received_at_ms = Some(parse_signal_ms(&parse_flag_value(
                    args,
                    &mut index,
                    "--received-at-ms",
                )?)?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control signal` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::Signal {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control signal`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control signal`"))?,
        step_id,
        signal_name: signal_name
            .ok_or_else(|| invalid_input("missing `--signal-name <name>` for `control signal`"))?,
        payload: payload
            .ok_or_else(|| invalid_input("missing `--payload <json>` for `control signal`"))?,
        received_at_ms: received_at_ms
            .ok_or_else(|| invalid_input("missing `--received-at-ms <ms>` for `control signal`"))?,
        json,
    })
}

fn parse_activity(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut step_id = None;
    let mut activity_id = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--step-id" => {
                step_id = Some(parse_flag_value(args, &mut index, "--step-id")?);
            }
            "--activity-id" => {
                activity_id = Some(parse_flag_value(args, &mut index, "--activity-id")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::Activity {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control activity`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control activity`"))?,
        step_id,
        activity_id: activity_id
            .ok_or_else(|| invalid_input("missing `--activity-id <id>` for `control activity`"))?,
        json,
    })
}

fn parse_decision(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut step_id = None;
    let mut decision_id = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--step-id" => {
                step_id = Some(parse_flag_value(args, &mut index, "--step-id")?);
            }
            "--decision-id" => {
                decision_id = Some(parse_flag_value(args, &mut index, "--decision-id")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control decision` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::Decision {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control decision`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control decision`"))?,
        step_id,
        decision_id: decision_id
            .ok_or_else(|| invalid_input("missing `--decision-id <id>` for `control decision`"))?,
        json,
    })
}

fn parse_history(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control history` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::History {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control history`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control history`"))?,
        json,
    })
}

fn parse_recovery_snapshot(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut now_ms = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--now-ms" => {
                now_ms = Some(parse_now_ms(&parse_flag_value(
                    args, &mut index, "--now-ms",
                )?)?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control recovery-snapshot` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::RecoverySnapshot {
        ledger_path: ledger_path.ok_or_else(|| {
            invalid_input("missing `--ledger <path>` for `control recovery-snapshot`")
        })?,
        run_id: run_id.ok_or_else(|| {
            invalid_input("missing `--run-id <id>` for `control recovery-snapshot`")
        })?,
        now_ms: now_ms.ok_or_else(|| {
            invalid_input("missing `--now-ms <ms>` for `control recovery-snapshot`")
        })?,
        json,
    })
}

fn parse_view(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control view` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::View {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control view`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control view`"))?,
        json,
    })
}

fn parse_step(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut step_id = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--step-id" => {
                step_id = Some(parse_flag_value(args, &mut index, "--step-id")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control step` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::Step {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control step`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control step`"))?,
        step_id: step_id
            .ok_or_else(|| invalid_input("missing `--step-id <id>` for `control step`"))?,
        json,
    })
}

fn parse_timer(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut step_id = None;
    let mut timer_id = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--step-id" => {
                step_id = Some(parse_flag_value(args, &mut index, "--step-id")?);
            }
            "--timer-id" => {
                timer_id = Some(parse_flag_value(args, &mut index, "--timer-id")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control timer` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::Timer {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control timer`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control timer`"))?,
        step_id,
        timer_id: timer_id
            .ok_or_else(|| invalid_input("missing `--timer-id <id>` for `control timer`"))?,
        json,
    })
}

fn parse_now_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--now-ms` value `{value}` for `control recovery-snapshot`: {error}"
        ))
    })
}

fn parse_signal_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--received-at-ms` value `{value}` for `control signal`: {error}"
        ))
    })
}

fn parse_query_now_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--now-ms` value `{value}` for `control query`: {error}"
        ))
    })
}

fn parse_hot_state_now_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--now-ms` value `{value}` for `control hot-state`: {error}"
        ))
    })
}

fn parse_apply_recovery_plan_now_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--now-ms` value `{value}` for `control apply-recovery-plan`: {error}"
        ))
    })
}

fn parse_apply_recovery_plan_attempt(value: &str) -> io::Result<u32> {
    value.parse::<u32>().map_err(|error| {
        invalid_input(format!(
            "invalid `--attempt` value `{value}` for `control apply-recovery-plan`: {error}"
        ))
    })
}

fn parse_apply_recovery_plan_max_attempts(value: &str) -> io::Result<u32> {
    value.parse::<u32>().map_err(|error| {
        invalid_input(format!(
            "invalid `--max-attempts` value `{value}` for `control apply-recovery-plan`: {error}"
        ))
    })
}

fn parse_apply_recovery_plan_backoff_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--backoff-ms` value `{value}` for `control apply-recovery-plan`: {error}"
        ))
    })
}

fn parse_apply_recovery_plan_priority(value: &str) -> io::Result<i64> {
    value.parse::<i64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--priority` value `{value}` for `control apply-recovery-plan`: {error}"
        ))
    })
}
