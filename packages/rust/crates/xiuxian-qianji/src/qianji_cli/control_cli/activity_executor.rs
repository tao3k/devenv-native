use std::io;

use crate::qianji_cli::invalid_input;

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
use super::activity_args::ActivitySettleOutcomeArg;

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
const FIXTURE_ACTIVITY_TYPES: &[&str] = &[
    "llm.plan",
    "llm.tool_select",
    "llm.repair",
    "episteme.ontology.reasoning_fill",
    "tool.github",
    "wendao.search",
];

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
const FIXTURE_TASK_QUEUES: &[&str] = &[
    "llm.openai",
    "llm.anthropic",
    "llm.openrouter",
    "llm.local",
    "episteme.ontology.reasoning",
    "tool.github",
    "wendao.search",
];

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
const OPENAI_COMPATIBLE_LLM_ACTIVITY_TYPES: &[&str] = &[
    "llm.plan",
    "llm.tool_select",
    "llm.repair",
    "episteme.ontology.reasoning_fill",
];

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
const OPENAI_COMPATIBLE_LLM_TASK_QUEUES: &[&str] = &[
    "llm.openai",
    "llm.openrouter",
    "llm.local",
    "episteme.ontology.reasoning",
];

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
const LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY: &str = "qianji_llm_activity_request";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivityExecutorKindArg {
    Fixture,
    #[serde(rename = "openai_compatible_llm")]
    OpenAiCompatibleLlm,
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
    pub(crate) output_ref_json: Option<&'a str>,
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
    #[serde(rename = "openai_compatible_llm")]
    OpenAiCompatibleLlm,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(crate) struct ActivityExecutorContract {
    pub(crate) executor: ActivityExecutorKindArg,
    pub(crate) adapter: ActivityExecutorAdapterKind,
    pub(crate) allowed_activity_types: &'static [&'static str],
    pub(crate) allowed_task_queues: &'static [&'static str],
    pub(crate) requires_input_ref: bool,
    pub(crate) requires_request_audit: bool,
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
            ActivityExecutorKindArg::OpenAiCompatibleLlm => Err(invalid_input(
                "activity executor `openai-compatible-llm` is an admission gate only in this slice",
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

    pub(crate) const fn can_execute(self, executor: ActivityExecutorKindArg) -> bool {
        match executor {
            ActivityExecutorKindArg::Fixture => self.fixture_enabled,
            ActivityExecutorKindArg::OpenAiCompatibleLlm => {
                cfg!(any(feature = "qianji-full", test))
            }
        }
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
            ActivityExecutorKindArg::OpenAiCompatibleLlm => {
                Ok(ActivityExecutorContract::openai_compatible_llm())
            }
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
    pub(crate) const fn executor_label(self) -> &'static str {
        executor_label(self.executor)
    }

    const fn fixture() -> Self {
        Self {
            executor: ActivityExecutorKindArg::Fixture,
            adapter: ActivityExecutorAdapterKind::Fixture,
            allowed_activity_types: FIXTURE_ACTIVITY_TYPES,
            allowed_task_queues: FIXTURE_TASK_QUEUES,
            requires_input_ref: false,
            requires_request_audit: false,
        }
    }

    const fn openai_compatible_llm() -> Self {
        Self {
            executor: ActivityExecutorKindArg::OpenAiCompatibleLlm,
            adapter: ActivityExecutorAdapterKind::OpenAiCompatibleLlm,
            allowed_activity_types: OPENAI_COMPATIBLE_LLM_ACTIVITY_TYPES,
            allowed_task_queues: OPENAI_COMPATIBLE_LLM_TASK_QUEUES,
            requires_input_ref: true,
            requires_request_audit: true,
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
        )?;
        if self.requires_input_ref && task.input_ref.is_none() {
            return Err(invalid_input(format!(
                "activity executor `{}` requires task input_ref",
                executor_label(self.executor)
            )));
        }
        if self.requires_request_audit {
            validate_llm_request_audit(task, self.executor)?;
        }
        Ok(())
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
fn validate_llm_request_audit(
    task: &xiuxian_qianji_control::WorkerActivityTask,
    executor: ActivityExecutorKindArg,
) -> io::Result<()> {
    let audit = task
        .metadata
        .get(LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY)
        .ok_or_else(|| {
            invalid_input(format!(
                "activity executor `{}` requires `{}` metadata",
                executor_label(executor),
                LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY
            ))
        })?;
    let prompt_ref = audit.get("prompt_ref").ok_or_else(|| {
        invalid_input(format!(
            "activity executor `{}` requires request audit prompt_ref",
            executor_label(executor)
        ))
    })?;
    let prompt_ref: xiuxian_qianji_control::ArtifactRef =
        serde_json::from_value(prompt_ref.clone()).map_err(|error| {
            invalid_input(format!(
                "activity executor `{}` has invalid request audit prompt_ref: {error}",
                executor_label(executor)
            ))
        })?;
    if task.input_ref.as_ref() != Some(&prompt_ref) {
        return Err(invalid_input(format!(
            "activity executor `{}` requires task input_ref to match request audit prompt_ref",
            executor_label(executor)
        )));
    }
    let model = audit
        .get("model")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim();
    if model.is_empty() {
        return Err(invalid_input(format!(
            "activity executor `{}` requires request audit model",
            executor_label(executor)
        )));
    }
    Ok(())
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
const fn executor_label(executor: ActivityExecutorKindArg) -> &'static str {
    match executor {
        ActivityExecutorKindArg::Fixture => "fixture",
        ActivityExecutorKindArg::OpenAiCompatibleLlm => "openai-compatible-llm",
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn execute_fixture(request: ActivityExecutionRequest<'_>) -> io::Result<ActivityExecutorOutcome> {
    match request.outcome {
        ActivitySettleOutcomeArg::Complete => Ok(ActivityExecutorOutcome::Complete {
            result: xiuxian_qianji_control::ActivityResult {
                output_ref: parse_output_ref_json(request.output_ref_json)?,
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

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn parse_output_ref_json(
    output_ref_json: Option<&str>,
) -> io::Result<Option<xiuxian_qianji_control::ArtifactRef>> {
    let Some(output_ref_json) = output_ref_json else {
        return Ok(None);
    };
    let output_ref: xiuxian_qianji_control::ArtifactRef = serde_json::from_str(output_ref_json)
        .map_err(|error| {
            invalid_input(format!(
                "invalid `--output-ref-json` for activity executor fixture: {error}"
            ))
        })?;
    validate_output_ref(&output_ref)?;
    Ok(Some(output_ref))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn validate_output_ref(output_ref: &xiuxian_qianji_control::ArtifactRef) -> io::Result<()> {
    xiuxian_qianji_control::ArtifactId::new(output_ref.artifact_id.as_str())
        .map_err(|error| control_error(&error))?;
    xiuxian_qianji_control::ArtifactKind::new(output_ref.artifact_kind.as_str())
        .map_err(|error| control_error(&error))?;
    if output_ref.uri.trim().is_empty() {
        return Err(invalid_input(
            "activity output ArtifactRef uri must not be blank",
        ));
    }
    if output_ref
        .content_digest
        .as_ref()
        .is_some_and(|digest| digest.trim().is_empty())
    {
        return Err(invalid_input(
            "activity output ArtifactRef content_digest must not be blank when supplied",
        ));
    }
    Ok(())
}

pub(crate) fn parse_executor(value: &str) -> io::Result<ActivityExecutorKindArg> {
    match value {
        "fixture" => Ok(ActivityExecutorKindArg::Fixture),
        "openai-compatible-llm" | "openai_compatible_llm" => {
            Ok(ActivityExecutorKindArg::OpenAiCompatibleLlm)
        }
        other => Err(invalid_input(format!(
            "invalid `--executor` for `control activity-worker-once`; expected `fixture` or `openai-compatible-llm`, got `{other}`"
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
