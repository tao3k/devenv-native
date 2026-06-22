//! Parser for one-shot activity worker execution.

use std::io;
use std::path::PathBuf;

use crate::qianji_cli::{invalid_input, parse_flag_value};

use crate::qianji_cli::control_cli::activity_args::ActivitySettleOutcomeArg;
use crate::qianji_cli::control_cli::activity_executor::{self, ActivityExecutorKindArg};
use crate::qianji_cli::control_cli::types::ControlCliCommand;

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityWorkerOnceArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityWorkerOnceArgs {
    ledger_path: Option<PathBuf>,
    valkey_url: Option<String>,
    namespace: Option<String>,
    worker_id: Option<String>,
    task_queue: Option<String>,
    now_ms: Option<u64>,
    lease_ttl_ms: Option<u64>,
    executor: Option<ActivityExecutorKindArg>,
    outcome: Option<ActivitySettleOutcomeArg>,
    settled_at_ms: Option<u64>,
    output_ref_json: Option<String>,
    output_hash: Option<String>,
    output_artifact_path: Option<PathBuf>,
    output_artifact_content: Option<String>,
    output_artifact_id: Option<String>,
    output_artifact_kind: Option<String>,
    openai_compatible_base_url: Option<String>,
    openai_compatible_api_key: Option<String>,
    openai_compatible_timeout_ms: Option<u64>,
    error_code: Option<String>,
    message: Option<String>,
    retryable: Option<bool>,
    metadata: Option<String>,
    json: bool,
}

impl ActivityWorkerOnceArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
        let flag = args[*index].as_str();
        if self.parse_connection_flag(flag, args, index)?
            || self.parse_executor_flag(flag, args, index)?
            || self.parse_output_flag(flag, args, index)?
            || self.parse_failure_flag(flag, args, index)?
        {
            return Ok(());
        }
        if flag == "--json" {
            self.json = true;
            return Ok(());
        }
        Err(invalid_input(format!(
            "`control activity-worker-once` does not accept argument `{flag}`"
        )))
    }

    fn parse_connection_flag(
        &mut self,
        flag: &str,
        args: &[String],
        index: &mut usize,
    ) -> io::Result<bool> {
        match flag {
            "--ledger" => {
                self.ledger_path = Some(PathBuf::from(parse_flag_value(args, index, "--ledger")?));
            }
            "--valkey-url" => {
                self.valkey_url = Some(parse_flag_value(args, index, "--valkey-url")?);
            }
            "--namespace" => {
                self.namespace = Some(parse_flag_value(args, index, "--namespace")?);
            }
            "--worker-id" => {
                self.worker_id = Some(parse_flag_value(args, index, "--worker-id")?);
            }
            "--task-queue" => {
                self.task_queue = Some(parse_flag_value(args, index, "--task-queue")?);
            }
            "--now-ms" => {
                self.now_ms = Some(parse_u64(
                    "now-ms",
                    "control activity-worker-once",
                    &parse_flag_value(args, index, "--now-ms")?,
                )?);
            }
            "--lease-ttl-ms" => {
                self.lease_ttl_ms = Some(parse_u64(
                    "lease-ttl-ms",
                    "control activity-worker-once",
                    &parse_flag_value(args, index, "--lease-ttl-ms")?,
                )?);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn parse_executor_flag(
        &mut self,
        flag: &str,
        args: &[String],
        index: &mut usize,
    ) -> io::Result<bool> {
        match flag {
            "--executor" => {
                self.executor = Some(activity_executor::parse_executor(&parse_flag_value(
                    args,
                    index,
                    "--executor",
                )?)?);
            }
            "--outcome" => {
                self.outcome = Some(parse_outcome(&parse_flag_value(args, index, "--outcome")?)?);
            }
            "--settled-at-ms" => {
                self.settled_at_ms = Some(parse_u64(
                    "settled-at-ms",
                    "control activity-worker-once",
                    &parse_flag_value(args, index, "--settled-at-ms")?,
                )?);
            }
            "--openai-compatible-base-url" => {
                self.openai_compatible_base_url = Some(parse_flag_value(
                    args,
                    index,
                    "--openai-compatible-base-url",
                )?);
            }
            "--openai-compatible-api-key" => {
                self.openai_compatible_api_key = Some(parse_flag_value(
                    args,
                    index,
                    "--openai-compatible-api-key",
                )?);
            }
            "--openai-compatible-timeout-ms" => {
                self.openai_compatible_timeout_ms = Some(parse_u64(
                    "openai-compatible-timeout-ms",
                    "control activity-worker-once",
                    &parse_flag_value(args, index, "--openai-compatible-timeout-ms")?,
                )?);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn parse_output_flag(
        &mut self,
        flag: &str,
        args: &[String],
        index: &mut usize,
    ) -> io::Result<bool> {
        match flag {
            "--output-hash" => {
                self.output_hash = Some(parse_flag_value(args, index, "--output-hash")?);
            }
            "--output-ref-json" => {
                self.output_ref_json = Some(parse_flag_value(args, index, "--output-ref-json")?);
            }
            "--output-artifact-path" => {
                self.output_artifact_path = Some(PathBuf::from(parse_flag_value(
                    args,
                    index,
                    "--output-artifact-path",
                )?));
            }
            "--output-artifact-content" => {
                self.output_artifact_content =
                    Some(parse_flag_value(args, index, "--output-artifact-content")?);
            }
            "--output-artifact-id" => {
                self.output_artifact_id =
                    Some(parse_flag_value(args, index, "--output-artifact-id")?);
            }
            "--output-artifact-kind" => {
                self.output_artifact_kind =
                    Some(parse_flag_value(args, index, "--output-artifact-kind")?);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn parse_failure_flag(
        &mut self,
        flag: &str,
        args: &[String],
        index: &mut usize,
    ) -> io::Result<bool> {
        match flag {
            "--error-code" => {
                self.error_code = Some(parse_flag_value(args, index, "--error-code")?);
            }
            "--message" => {
                self.message = Some(parse_flag_value(args, index, "--message")?);
            }
            "--retryable" => {
                self.retryable = Some(parse_bool(&parse_flag_value(args, index, "--retryable")?)?);
            }
            "--metadata" => {
                self.metadata = Some(parse_flag_value(args, index, "--metadata")?);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        let executor = self.executor.ok_or_else(|| {
            invalid_input("missing `--executor fixture` for `control activity-worker-once`")
        })?;
        let outcome = self.outcome.ok_or_else(|| {
            invalid_input("missing `--outcome <complete|fail>` for `control activity-worker-once`")
        })?;
        validate_outcome_args(executor, outcome, &self)?;
        Ok(ControlCliCommand::ActivityWorkerOnce {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control activity-worker-once`")
            })?,
            valkey_url: self.valkey_url.ok_or_else(|| {
                invalid_input("missing `--valkey-url <url>` for `control activity-worker-once`")
            })?,
            namespace: self.namespace,
            worker_id: self.worker_id.ok_or_else(|| {
                invalid_input("missing `--worker-id <id>` for `control activity-worker-once`")
            })?,
            task_queue: self.task_queue,
            now_ms: self.now_ms.ok_or_else(|| {
                invalid_input("missing `--now-ms <ms>` for `control activity-worker-once`")
            })?,
            lease_ttl_ms: self.lease_ttl_ms.ok_or_else(|| {
                invalid_input("missing `--lease-ttl-ms <ms>` for `control activity-worker-once`")
            })?,
            executor,
            outcome,
            settled_at_ms: self.settled_at_ms.ok_or_else(|| {
                invalid_input("missing `--settled-at-ms <ms>` for `control activity-worker-once`")
            })?,
            output_ref_json: self.output_ref_json,
            output_hash: self.output_hash,
            output_artifact_path: self.output_artifact_path,
            output_artifact_content: self.output_artifact_content,
            output_artifact_id: self.output_artifact_id,
            output_artifact_kind: self.output_artifact_kind,
            openai_compatible_base_url: self.openai_compatible_base_url,
            openai_compatible_api_key: self.openai_compatible_api_key,
            openai_compatible_timeout_ms: self.openai_compatible_timeout_ms,
            error_code: self.error_code,
            message: self.message,
            retryable: self.retryable,
            metadata: self.metadata,
            json: self.json,
        })
    }
}

fn parse_outcome(value: &str) -> io::Result<ActivitySettleOutcomeArg> {
    match value {
        "complete" => Ok(ActivitySettleOutcomeArg::Complete),
        "fail" => Ok(ActivitySettleOutcomeArg::Fail),
        other => Err(invalid_input(format!(
            "invalid `--outcome` for `control activity-worker-once`; expected `complete` or `fail`, got `{other}`"
        ))),
    }
}

fn validate_outcome_args(
    executor: ActivityExecutorKindArg,
    outcome: ActivitySettleOutcomeArg,
    args: &ActivityWorkerOnceArgs,
) -> io::Result<()> {
    if executor == ActivityExecutorKindArg::OpenAiCompatibleLlm {
        return validate_openai_compatible_outcome_args(outcome, args);
    }
    if executor == ActivityExecutorKindArg::FlowhubService {
        return validate_flowhub_service_outcome_args(outcome, args);
    }
    if has_openai_compatible_args(args) {
        return Err(invalid_input(
            "`control activity-worker-once` OpenAI-compatible flags require `--executor openai-compatible-llm`",
        ));
    }
    match outcome {
        ActivitySettleOutcomeArg::Complete => {
            if args.error_code.is_some() || args.message.is_some() || args.retryable.is_some() {
                return Err(invalid_input(
                    "`control activity-worker-once --outcome complete` cannot be combined with `--error-code`, `--message`, or `--retryable`",
                ));
            }
            validate_complete_output_artifact_args(args)?;
        }
        ActivitySettleOutcomeArg::Fail => {
            if args.output_hash.is_some()
                || args.output_ref_json.is_some()
                || has_output_artifact_args(args)
            {
                return Err(invalid_input(
                    "`control activity-worker-once --outcome fail` cannot be combined with output artifact or output reference arguments",
                ));
            }
            if args.error_code.is_none() {
                return Err(invalid_input(
                    "missing `--error-code <code>` for `control activity-worker-once --outcome fail`",
                ));
            }
            if args.message.is_none() {
                return Err(invalid_input(
                    "missing `--message <text>` for `control activity-worker-once --outcome fail`",
                ));
            }
            if args.retryable.is_none() {
                return Err(invalid_input(
                    "missing `--retryable <true|false>` for `control activity-worker-once --outcome fail`",
                ));
            }
        }
    }
    Ok(())
}

fn validate_openai_compatible_outcome_args(
    outcome: ActivitySettleOutcomeArg,
    args: &ActivityWorkerOnceArgs,
) -> io::Result<()> {
    if outcome != ActivitySettleOutcomeArg::Complete {
        return Err(invalid_input(
            "`control activity-worker-once --executor openai-compatible-llm` uses `--outcome complete`; provider failures are recorded by the executor",
        ));
    }
    if args.error_code.is_some() || args.message.is_some() || args.retryable.is_some() {
        return Err(invalid_input(
            "`control activity-worker-once --executor openai-compatible-llm` cannot be combined with `--error-code`, `--message`, or `--retryable`",
        ));
    }
    if args.output_ref_json.is_some() || args.output_hash.is_some() {
        return Err(invalid_input(
            "`control activity-worker-once --executor openai-compatible-llm` derives output refs and hashes from `--output-artifact-path`",
        ));
    }
    if args.output_artifact_path.is_none() {
        return Err(invalid_input(
            "missing `--output-artifact-path <path>` for `control activity-worker-once --executor openai-compatible-llm`",
        ));
    }
    if args.output_artifact_content.is_some() {
        return Err(invalid_input(
            "`control activity-worker-once --executor openai-compatible-llm` writes provider output and does not accept `--output-artifact-content`",
        ));
    }
    if args
        .openai_compatible_base_url
        .as_deref()
        .is_none_or(|base_url| base_url.trim().is_empty())
    {
        return Err(invalid_input(
            "missing `--openai-compatible-base-url <url>` for `control activity-worker-once --executor openai-compatible-llm`",
        ));
    }
    Ok(())
}

fn validate_flowhub_service_outcome_args(
    outcome: ActivitySettleOutcomeArg,
    args: &ActivityWorkerOnceArgs,
) -> io::Result<()> {
    if outcome != ActivitySettleOutcomeArg::Complete {
        return Err(invalid_input(
            "`control activity-worker-once --executor flowhub-service` derives successful completion data; execution failures should be recorded through retry/fail settlement",
        ));
    }
    if args.error_code.is_some() || args.message.is_some() || args.retryable.is_some() {
        return Err(invalid_input(
            "`control activity-worker-once --executor flowhub-service` cannot be combined with `--error-code`, `--message`, or `--retryable`",
        ));
    }
    if args.output_ref_json.is_some()
        || args.output_hash.is_some()
        || has_output_artifact_args(args)
        || args.metadata.is_some()
    {
        return Err(invalid_input(
            "`control activity-worker-once --executor flowhub-service` derives completion metadata from the BPMN task contract",
        ));
    }
    Ok(())
}

fn validate_complete_output_artifact_args(args: &ActivityWorkerOnceArgs) -> io::Result<()> {
    if !has_output_artifact_args(args) {
        return Ok(());
    }
    if args.output_artifact_path.is_none() {
        return Err(invalid_input(
            "missing `--output-artifact-path <path>` for `control activity-worker-once`",
        ));
    }
    if args.output_artifact_content.is_none() {
        return Err(invalid_input(
            "missing `--output-artifact-content <text>` for `control activity-worker-once`",
        ));
    }
    if args.output_ref_json.is_some() || args.output_hash.is_some() {
        return Err(invalid_input(
            "`control activity-worker-once --outcome complete` cannot combine `--output-artifact-path` with `--output-ref-json` or `--output-hash`",
        ));
    }
    Ok(())
}

fn has_output_artifact_args(args: &ActivityWorkerOnceArgs) -> bool {
    args.output_artifact_path.is_some()
        || args.output_artifact_content.is_some()
        || args.output_artifact_id.is_some()
        || args.output_artifact_kind.is_some()
}

fn has_openai_compatible_args(args: &ActivityWorkerOnceArgs) -> bool {
    args.openai_compatible_base_url.is_some()
        || args.openai_compatible_api_key.is_some()
        || args.openai_compatible_timeout_ms.is_some()
}

fn parse_u64(field: &'static str, command: &'static str, value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{field}` for `{command}`; expected u64: {error}"
        ))
    })
}

fn parse_bool(value: &str) -> io::Result<bool> {
    value.parse::<bool>().map_err(|error| {
        invalid_input(format!(
            "invalid `--retryable` for `control activity-worker-once`; expected bool: {error}"
        ))
    })
}
