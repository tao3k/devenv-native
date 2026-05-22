use std::io;

use crate::qianji_cli::invalid_input;

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
use super::activity_args::ActivitySettleOutcomeArg;

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
const FIXTURE_ACTIVITY_TYPES: &[&str] = &[
    "llm.plan",
    "llm.tool_select",
    "llm.repair",
    "tool.github",
    "wendao.search",
];

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
const FIXTURE_TASK_QUEUES: &[&str] = &[
    "llm.openai",
    "llm.anthropic",
    "llm.openrouter",
    "llm.local",
    "tool.github",
    "wendao.search",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivityExecutorKindArg {
    Fixture,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ActivityExecutorOutcome {
    Complete {
        result: xiuxian_qianji_control::ActivityResult,
    },
    Fail {
        error_code: xiuxian_qianji_control::ErrorCode,
        message: String,
        retryable: bool,
        metadata: serde_json::Value,
    },
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Clone, Copy)]
pub(crate) struct ActivityExecutionRequest<'a> {
    pub(crate) task: Option<&'a xiuxian_qianji_control::WorkerActivityTask>,
    pub(crate) executor: ActivityExecutorKindArg,
    pub(crate) outcome: ActivitySettleOutcomeArg,
    pub(crate) output_hash: Option<&'a str>,
    pub(crate) error_code: Option<&'a str>,
    pub(crate) message: Option<&'a str>,
    pub(crate) retryable: Option<bool>,
    pub(crate) metadata: Option<&'a str>,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct ActivityExecutorRegistry {
    fixture_enabled: bool,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivityExecutorAdapterKind {
    Fixture,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(crate) struct ActivityExecutorContract {
    pub(crate) executor: ActivityExecutorKindArg,
    pub(crate) adapter: ActivityExecutorAdapterKind,
    pub(crate) allowed_activity_types: &'static [&'static str],
    pub(crate) allowed_task_queues: &'static [&'static str],
    pub(crate) requires_input_ref: bool,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
impl ActivityExecutorRegistry {
    pub(crate) const fn fixture_only() -> Self {
        Self {
            fixture_enabled: true,
        }
    }

    pub(crate) fn execute(
        self,
        request: ActivityExecutionRequest<'_>,
    ) -> io::Result<ActivityExecutorOutcome> {
        self.validate_task(request.executor, request.task)?;
        match request.executor {
            ActivityExecutorKindArg::Fixture if self.fixture_enabled => execute_fixture(request),
            ActivityExecutorKindArg::Fixture => Err(invalid_input(
                "activity executor registry does not enable the fixture executor",
            )),
        }
    }

    pub(crate) fn validate_task(
        self,
        executor: ActivityExecutorKindArg,
        task: Option<&xiuxian_qianji_control::WorkerActivityTask>,
    ) -> io::Result<ActivityExecutorContract> {
        let task = validate_executor_task(task)?;
        let contract = self.route_contract(executor)?;
        contract.validate(task)?;
        Ok(contract)
    }

    fn route_contract(
        self,
        executor: ActivityExecutorKindArg,
    ) -> io::Result<ActivityExecutorContract> {
        match executor {
            ActivityExecutorKindArg::Fixture if self.fixture_enabled => {
                Ok(ActivityExecutorContract::fixture())
            }
            ActivityExecutorKindArg::Fixture => Err(invalid_input(
                "activity executor registry does not enable the fixture executor",
            )),
        }
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn validate_executor_task(
    task: Option<&xiuxian_qianji_control::WorkerActivityTask>,
) -> io::Result<&xiuxian_qianji_control::WorkerActivityTask> {
    let Some(task) = task else {
        return Err(invalid_input(
            "activity executor requires a claimed worker activity task",
        ));
    };
    if task.next_attempt == 0 {
        return Err(invalid_input(
            "activity executor worker task must have a positive next_attempt",
        ));
    }
    Ok(task)
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
impl ActivityExecutorContract {
    const fn fixture() -> Self {
        Self {
            executor: ActivityExecutorKindArg::Fixture,
            adapter: ActivityExecutorAdapterKind::Fixture,
            allowed_activity_types: FIXTURE_ACTIVITY_TYPES,
            allowed_task_queues: FIXTURE_TASK_QUEUES,
            requires_input_ref: false,
        }
    }

    fn validate(self, task: &xiuxian_qianji_control::WorkerActivityTask) -> io::Result<()> {
        validate_allowed_route(
            "activity_type",
            task.activity_type.as_str(),
            self.allowed_activity_types,
            self.executor,
        )?;
        validate_allowed_route(
            "task_queue",
            task.task_queue.as_str(),
            self.allowed_task_queues,
            self.executor,
        )
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn validate_allowed_route(
    field: &'static str,
    value: &str,
    allowed: &[&str],
    executor: ActivityExecutorKindArg,
) -> io::Result<()> {
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(invalid_input(format!(
        "activity executor `{executor:?}` does not allow {field} `{value}`"
    )))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn execute_fixture(request: ActivityExecutionRequest<'_>) -> io::Result<ActivityExecutorOutcome> {
    match request.outcome {
        ActivitySettleOutcomeArg::Complete => Ok(ActivityExecutorOutcome::Complete {
            result: xiuxian_qianji_control::ActivityResult {
                output_ref: None,
                output_hash: request.output_hash.map(str::to_owned),
                metadata: parse_metadata(request.metadata)?,
            },
        }),
        ActivitySettleOutcomeArg::Fail => {
            let error_code = request.error_code.ok_or_else(|| {
                invalid_input("missing fixture `error_code` for failed activity execution")
            })?;
            let message = request.message.ok_or_else(|| {
                invalid_input("missing fixture `message` for failed activity execution")
            })?;
            let retryable = request.retryable.ok_or_else(|| {
                invalid_input("missing fixture `retryable` for failed activity execution")
            })?;
            Ok(ActivityExecutorOutcome::Fail {
                error_code: xiuxian_qianji_control::ErrorCode::new(error_code)
                    .map_err(|error| control_error(&error))?,
                message: message.to_string(),
                retryable,
                metadata: parse_metadata(request.metadata)?,
            })
        }
    }
}

pub(crate) fn parse_executor(value: &str) -> io::Result<ActivityExecutorKindArg> {
    match value {
        "fixture" => Ok(ActivityExecutorKindArg::Fixture),
        other => Err(invalid_input(format!(
            "invalid `--executor` for `control activity-worker-once`; expected `fixture`, got `{other}`"
        ))),
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn parse_metadata(metadata: Option<&str>) -> io::Result<serde_json::Value> {
    match metadata {
        Some(value) => serde_json::from_str(value).map_err(|error| {
            invalid_input(format!(
                "invalid `--metadata` JSON for activity executor fixture: {error}"
            ))
        }),
        None => Ok(serde_json::Value::Null),
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
