//! Flowhub BPMN `serviceTask` completion contract for runtime workers.
//!
//! The adapter validates replay-derived worker task metadata, checks required
//! BPMN output fields, derives deterministic completion data, and builds the
//! durable `ActivityResult` metadata consumed by Qianji control-plane ledgers.

use std::path::PathBuf;

use serde_json::{Map, Value, json};
use xiuxian_qianji_control::{ActivityResult, ControlError, ControlResult, WorkerActivityTask};

use super::{FLOWHUB_SERVICE_ACTIVITY_METADATA_KEY, FLOWHUB_SERVICE_ACTIVITY_SCHEMA};

/// Activity result metadata key used by the bounded Flowhub service executor.
pub const FLOWHUB_SERVICE_COMPLETION_METADATA_KEY: &str = "qianji_flowhub_service_completion";
/// Activity result metadata schema used by the bounded Flowhub service executor.
pub const FLOWHUB_SERVICE_COMPLETION_SCHEMA: &str = "xiuxian_qianji.flowhub.service_completion.v1";

/// Runtime-neutral Flowhub service-task completion facts.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowhubServiceTaskCompletion {
    /// Runtime token identifier for the pending host work.
    pub token_id: QianjiRuntimeBpmnTokenId,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: QianjiRuntimeBpmnProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: QianjiRuntimeBpmnActivityId,
    /// User- or operator-supplied payload merged into workflow variables.
    pub data: Value,
    /// Optional claimant supplied by the host when completing claimed work.
    pub claimant: Option<String>,
}

/// Runtime BPMN token identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QianjiRuntimeBpmnTokenId(u64);

impl QianjiRuntimeBpmnTokenId {
    /// Creates a runtime BPMN token id.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the serialized token id.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

macro_rules! runtime_bpmn_string_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Creates a runtime BPMN identifier.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Borrows the serialized identifier.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            /// Returns the owned serialized identifier.
            #[must_use]
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

runtime_bpmn_string_id!(
    QianjiRuntimeBpmnProcessId,
    "Runtime BPMN process identifier."
);
runtime_bpmn_string_id!(
    QianjiRuntimeBpmnActivityId,
    "Runtime BPMN activity identifier."
);

/// Builds runtime-neutral completion facts for a completed Flowhub worker task.
///
/// # Errors
///
/// Returns a control error when the worker task lacks Flowhub service metadata,
/// the metadata schema is unsupported, required BPMN identity fields are
/// missing, the task is not a service task, or the supplied completion data is
/// missing required BPMN output fields.
pub fn build_flowhub_service_task_completion(
    task: &WorkerActivityTask,
    data: Value,
) -> ControlResult<FlowhubServiceTaskCompletion> {
    let metadata = flowhub_service_metadata(task)?;
    require_schema(metadata)?;
    require_service_work_kind(metadata)?;
    validate_required_outputs(metadata, &data)?;
    Ok(FlowhubServiceTaskCompletion {
        token_id: QianjiRuntimeBpmnTokenId::new(required_u64(metadata, "tokenId")?),
        process_id: QianjiRuntimeBpmnProcessId::new(required_str(metadata, "processId")?),
        activity_id: QianjiRuntimeBpmnActivityId::new(required_str(metadata, "activityId")?),
        data,
        claimant: None,
    })
}

/// Returns the BPMN source path recorded in a replay-derived Flowhub task.
///
/// # Errors
///
/// Returns a control error when the task lacks valid Flowhub service metadata
/// or the `bpmnSource` metadata field is missing.
pub fn flowhub_service_task_bpmn_source_path(task: &WorkerActivityTask) -> ControlResult<PathBuf> {
    let metadata = flowhub_service_metadata(task)?;
    require_schema(metadata)?;
    require_service_work_kind(metadata)?;
    Ok(PathBuf::from(required_str(metadata, "bpmnSource")?))
}

/// Builds deterministic completion data for a Flowhub service task from its
/// declared BPMN output bindings.
///
/// This executor-side helper is intentionally bounded: it does not infer
/// domain state or mutate files. It only turns required BPMN output names into
/// boolean `true` values after validating Flowhub service-task metadata.
///
/// # Errors
///
/// Returns a control error when the worker task is not valid Flowhub service
/// work or when the task declares no required output fields.
pub fn build_flowhub_service_task_contract_completion_data(
    task: &WorkerActivityTask,
) -> ControlResult<Value> {
    let metadata = flowhub_service_metadata(task)?;
    require_schema(metadata)?;
    require_service_work_kind(metadata)?;
    let outputs = required_outputs(metadata);
    let mut data = Map::new();
    for output in outputs {
        if output
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            let name = required_str(output, "name")?;
            data.insert(name.to_owned(), Value::Bool(true));
        }
    }
    if data.is_empty() {
        return Err(invalid_flowhub_completion(format!(
            "Flowhub service task `{}` declares no required output fields",
            task.activity_id
        )));
    }
    Ok(Value::Object(data))
}

/// Builds a durable activity result for the bounded Flowhub service executor.
///
/// # Errors
///
/// Returns a control error when the worker task cannot produce deterministic
/// Flowhub completion data.
pub fn build_flowhub_service_task_contract_activity_result(
    task: &WorkerActivityTask,
) -> ControlResult<ActivityResult> {
    let data = build_flowhub_service_task_contract_completion_data(task)?;
    Ok(ActivityResult {
        output_ref: None,
        output_hash: None,
        metadata: json!({
            FLOWHUB_SERVICE_COMPLETION_METADATA_KEY: {
                "schema": FLOWHUB_SERVICE_COMPLETION_SCHEMA,
                "data": data
            }
        }),
    })
}

fn flowhub_service_metadata(task: &WorkerActivityTask) -> ControlResult<&Value> {
    task.metadata
        .get(FLOWHUB_SERVICE_ACTIVITY_METADATA_KEY)
        .ok_or_else(|| {
            invalid_flowhub_completion(format!(
                "worker task `{}` is missing Flowhub service metadata",
                task.activity_id
            ))
        })
}

fn require_schema(metadata: &Value) -> ControlResult<()> {
    let schema = required_str(metadata, "schema")?;
    if schema == FLOWHUB_SERVICE_ACTIVITY_SCHEMA {
        return Ok(());
    }
    Err(invalid_flowhub_completion(format!(
        "unsupported Flowhub service metadata schema `{schema}`"
    )))
}

fn require_service_work_kind(metadata: &Value) -> ControlResult<()> {
    let work_kind = required_str(metadata, "workKind")?;
    if work_kind == "service" {
        return Ok(());
    }
    Err(invalid_flowhub_completion(format!(
        "Flowhub completion adapter only accepts service work, got `{work_kind}`"
    )))
}

fn validate_required_outputs(metadata: &Value, data: &Value) -> ControlResult<()> {
    for output in required_outputs(metadata) {
        validate_required_output(output, data)?;
    }
    Ok(())
}

fn required_outputs(metadata: &Value) -> &[Value] {
    metadata
        .get("completion")
        .and_then(|completion| completion.get("requiredOutputs"))
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn validate_required_output(output: &Value, data: &Value) -> ControlResult<()> {
    let required = output
        .get("required")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if !required {
        return Ok(());
    }
    let name = required_str(output, "name")?;
    if data.get(name).is_some() {
        return Ok(());
    }
    Err(invalid_flowhub_completion(format!(
        "Flowhub service completion data is missing required output `{name}`"
    )))
}

fn required_str<'a>(value: &'a Value, field: &str) -> ControlResult<&'a str> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| {
        invalid_flowhub_completion(format!(
            "Flowhub service metadata requires string field `{field}`"
        ))
    })
}

fn required_u64(value: &Value, field: &str) -> ControlResult<u64> {
    value.get(field).and_then(Value::as_u64).ok_or_else(|| {
        invalid_flowhub_completion(format!(
            "Flowhub service metadata requires integer field `{field}`"
        ))
    })
}

fn invalid_flowhub_completion(message: String) -> ControlError {
    ControlError::InvalidEventSequence { message }
}
