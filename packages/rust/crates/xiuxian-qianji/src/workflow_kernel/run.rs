//! Workflow execution runner.

use std::{
    any::Any,
    fmt,
    future::Future,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use futures::{StreamExt, stream};
use xiuxian_qianji_control::{ControlError, ControlResult};

use super::{
    WorkflowCheckpointError, WorkflowCheckpointId, WorkflowCheckpointRef, WorkflowCompletionError,
    WorkflowControlRecorder, WorkflowControlRecordingOutcome, WorkflowExecutionReport,
    WorkflowMemoryCheckpointStore, WorkflowStage, WorkflowStageCheckpointMiss, WorkflowStageFacts,
    WorkflowStageId, WorkflowStageStatus, WorkflowStageTrace, WorkflowTopology,
    WorkflowTopologyError, WorkflowTrace,
};

/// Error returned when a workflow stage fails.
#[derive(Debug, Clone, thiserror::Error)]
#[error("workflow `{workflow_id}` stage `{stage_id}` failed: {message}")]
pub struct WorkflowExecutionError {
    /// Stable workflow identifier.
    pub workflow_id: String,
    /// Stable stage identifier.
    pub stage_id: String,
    /// Stage error message.
    pub message: String,
    /// Trace captured through the failed stage.
    pub trace: WorkflowTrace,
}

/// Control-recording failure that preserves the completed workflow report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowControlRecordingFailure<T> {
    /// Normal workflow execution report produced before recording failed.
    pub workflow: Box<WorkflowExecutionReport<T>>,
    /// Control-plane recording error.
    pub source: ControlError,
}

impl<T> fmt::Display for WorkflowControlRecordingFailure<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workflow control recording failed after workflow completion: {}",
            self.source
        )
    }
}

impl<T: fmt::Debug> std::error::Error for WorkflowControlRecordingFailure<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl<T> WorkflowControlRecordingFailure<T> {
    /// Retries control recording from the retained workflow report.
    ///
    /// # Errors
    ///
    /// Returns another recoverable control-recording failure when the supplied
    /// recorder also rejects or cannot persist the retained workflow trace.
    pub fn retry_control_recording(
        self,
        recorder: WorkflowControlRecorder<'_>,
    ) -> Result<WorkflowControlRecordedReport<T>, Self> {
        (*self.workflow).record_control_recoverable(recorder)
    }
}

/// Error returned by topology-checked recoverable control recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowCheckedControlRecordingFailure<T> {
    /// The workflow trace failed the bound topology completion gate.
    Completion {
        /// Topology completion error.
        source: Box<WorkflowCompletionError>,
    },
    /// The workflow trace passed completion but control recording failed.
    Control {
        /// Recoverable control-recording failure.
        failure: Box<WorkflowControlRecordingFailure<T>>,
    },
}

impl<T> fmt::Display for WorkflowCheckedControlRecordingFailure<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completion { source } => {
                write!(
                    formatter,
                    "workflow completion validation failed before control recording: {source}"
                )
            }
            Self::Control { failure } => failure.fmt(formatter),
        }
    }
}

impl<T: fmt::Debug + 'static> std::error::Error for WorkflowCheckedControlRecordingFailure<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Completion { source } => Some(source.as_ref()),
            Self::Control { failure } => Some(failure.as_ref()),
        }
    }
}

impl<T> WorkflowCheckedControlRecordingFailure<T> {
    /// Retries control recording when topology validation has already passed.
    ///
    /// # Errors
    ///
    /// Returns the original completion failure when topology validation failed
    /// before control recording. Returns another control failure when the
    /// supplied recorder also rejects or cannot persist the retained workflow
    /// trace.
    pub fn retry_control_recording(
        self,
        recorder: WorkflowControlRecorder<'_>,
    ) -> Result<WorkflowControlRecordedReport<T>, Self> {
        match self {
            Self::Completion { source } => Err(Self::Completion { source }),
            Self::Control { failure } => {
                failure
                    .retry_control_recording(recorder)
                    .map_err(|failure| Self::Control {
                        failure: Box::new(failure),
                    })
            }
        }
    }
}

/// Error returned by topology-checked control recording.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WorkflowCheckedControlRecordingError {
    /// The workflow trace failed the bound topology completion gate.
    #[error(transparent)]
    Completion(#[from] WorkflowCompletionError),
    /// The workflow trace passed completion but control recording failed.
    #[error(transparent)]
    Control(#[from] ControlError),
}

/// Request object for bounded fan-out workflow stages.
pub struct WorkflowBoundedFanoutStageRequest<I, OutputFacts, F> {
    /// Stable stage identifier.
    pub stage_id: WorkflowStageId,
    /// Input items in desired output order.
    pub inputs: Vec<I>,
    /// Maximum number of concurrently executing item futures.
    pub max_concurrency: usize,
    /// Facts attached to the fan-out input edge.
    pub input_facts: WorkflowStageFacts,
    /// Facts builder for the ordered output edge.
    pub output_facts: OutputFacts,
    /// Per-item operation.
    pub operation: F,
}

/// Request object for memory checkpoint recording.
pub struct WorkflowMemoryCheckpointRecord<T> {
    /// Stage that produced the checkpoint.
    pub stage_id: WorkflowStageId,
    /// Stable checkpoint identifier.
    pub checkpoint_id: WorkflowCheckpointId,
    /// Facts attached to the checkpointed edge.
    pub facts: WorkflowStageFacts,
    /// Optional producer-supplied fingerprint.
    pub content_fingerprint: Option<String>,
    /// Same-process payload handle.
    pub payload: Arc<T>,
}

/// Workflow report paired with a control-plane recording outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowControlRecordedReport<T> {
    /// Normal workflow execution report.
    pub workflow: WorkflowExecutionReport<T>,
    /// Control-plane recording outcome.
    pub control: WorkflowControlRecordingOutcome,
}

/// Mutable state for one Rust-native workflow execution.
#[derive(Debug, Clone)]
pub struct WorkflowRun {
    workflow_id: String,
    trace: WorkflowTrace,
    topology: Option<WorkflowTopology>,
    memory_checkpoints: WorkflowMemoryCheckpointStore,
}

impl WorkflowRun {
    /// Creates a new workflow execution.
    #[must_use]
    pub fn new(workflow_id: impl Into<String>) -> Self {
        let workflow_id = workflow_id.into();
        Self {
            trace: WorkflowTrace::new(workflow_id.clone()),
            workflow_id,
            topology: None,
            memory_checkpoints: WorkflowMemoryCheckpointStore::default(),
        }
    }

    /// Creates a new topology-bound workflow execution.
    ///
    /// # Errors
    ///
    /// Returns an error when the supplied topology is invalid.
    pub fn new_with_topology(topology: WorkflowTopology) -> Result<Self, WorkflowTopologyError> {
        topology.validate()?;
        let workflow_id = topology.workflow_id.clone();
        Ok(Self {
            trace: WorkflowTrace::new(workflow_id.clone()),
            workflow_id,
            topology: Some(topology),
            memory_checkpoints: WorkflowMemoryCheckpointStore::default(),
        })
    }

    /// Returns the workflow identifier.
    #[must_use]
    pub fn workflow_id(&self) -> &str {
        self.workflow_id.as_str()
    }

    /// Borrows the current execution trace.
    #[must_use]
    pub fn trace(&self) -> &WorkflowTrace {
        &self.trace
    }

    /// Borrows the bound topology, when this run has one.
    #[must_use]
    pub fn topology(&self) -> Option<&WorkflowTopology> {
        self.topology.as_ref()
    }

    /// Borrows the in-process memory checkpoint store.
    #[must_use]
    pub fn memory_checkpoints(&self) -> &WorkflowMemoryCheckpointStore {
        &self.memory_checkpoints
    }

    /// Executes one typed stage and appends a trace row.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowExecutionError`] when the stage fails. The error
    /// includes the trace captured through the failed stage.
    pub async fn run_stage<C, I, S>(
        &mut self,
        context: &mut C,
        stage: &S,
        input: I,
    ) -> Result<S::Output, WorkflowExecutionError>
    where
        C: Send,
        I: Send,
        S: WorkflowStage<C, I>,
    {
        let stage_id = stage.id();
        let input_facts = stage.input_facts(&input);
        let started_unix_ms = unix_millis_now();
        let started = Instant::now();
        let typed_stage_id = WorkflowStageId::new(stage_id);
        if let Some(topology) = &self.topology
            && !topology.contains_stage(&typed_stage_id)
        {
            let message = format!("stage `{stage_id}` is not declared by workflow topology");
            self.trace.stages.push(failure_trace(
                stage_id,
                started_unix_ms,
                started.elapsed(),
                input_facts,
                message.clone(),
            ));
            return Err(WorkflowExecutionError {
                workflow_id: self.workflow_id.clone(),
                stage_id: stage_id.to_owned(),
                message,
                trace: self.trace.clone(),
            });
        }
        match stage.run(context, input).await {
            Ok(output) => {
                let output_facts = stage.output_facts(&output);
                self.trace.stages.push(success_trace(
                    stage_id,
                    started_unix_ms,
                    started.elapsed(),
                    input_facts,
                    output_facts,
                ));
                Ok(output)
            }
            Err(error) => {
                let message = error.to_string();
                self.trace.stages.push(failure_trace(
                    stage_id,
                    started_unix_ms,
                    started.elapsed(),
                    input_facts,
                    message.clone(),
                ));
                Err(WorkflowExecutionError {
                    workflow_id: self.workflow_id.clone(),
                    stage_id: stage_id.to_owned(),
                    message,
                    trace: self.trace.clone(),
                })
            }
        }
    }

    /// Executes homogeneous shard work with bounded concurrency and
    /// order-preserving fan-in.
    ///
    /// The returned outputs preserve the same order as the supplied inputs
    /// even when individual futures complete out of order.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowExecutionError`] when the stage is not declared by the
    /// bound topology or when any item returns an error. Item failures include
    /// the failing item index in the error message and append a failed stage
    /// trace.
    pub async fn run_bounded_fanout_stage<I, O, E, F, Fut, OutputFacts>(
        &mut self,
        request: WorkflowBoundedFanoutStageRequest<I, OutputFacts, F>,
    ) -> Result<Vec<O>, WorkflowExecutionError>
    where
        I: Send,
        O: Send,
        E: std::fmt::Display + Send + Sync + 'static,
        F: Fn(usize, I) -> Fut + Send + Sync,
        Fut: Future<Output = Result<O, E>> + Send,
        OutputFacts: Fn(&[O]) -> WorkflowStageFacts,
    {
        let stage_id = request.stage_id;
        let input_facts = request.input_facts;
        let started_unix_ms = unix_millis_now();
        let started = Instant::now();
        if let Some(topology) = &self.topology
            && !topology.contains_stage(&stage_id)
        {
            let message = format!("stage `{stage_id}` is not declared by workflow topology");
            self.trace.stages.push(failure_trace(
                stage_id.as_str(),
                started_unix_ms,
                started.elapsed(),
                input_facts.clone(),
                message.clone(),
            ));
            return Err(WorkflowExecutionError {
                workflow_id: self.workflow_id.clone(),
                stage_id: stage_id.as_str().to_owned(),
                message,
                trace: self.trace.clone(),
            });
        }

        let outputs =
            match run_ordered_fanout(request.inputs, request.max_concurrency, request.operation)
                .await
            {
                Ok(outputs) => outputs,
                Err((index, error)) => {
                    let message = format!("fan-out item `{index}` failed: {error}");
                    self.trace.stages.push(failure_trace(
                        stage_id.as_str(),
                        started_unix_ms,
                        started.elapsed(),
                        input_facts,
                        message.clone(),
                    ));
                    return Err(WorkflowExecutionError {
                        workflow_id: self.workflow_id.clone(),
                        stage_id: stage_id.as_str().to_owned(),
                        message,
                        trace: self.trace.clone(),
                    });
                }
            };
        let output_facts = (request.output_facts)(outputs.as_slice());
        self.trace.stages.push(success_trace(
            stage_id.as_str(),
            started_unix_ms,
            started.elapsed(),
            input_facts,
            output_facts,
        ));
        Ok(outputs)
    }

    /// Finishes this workflow execution with a final typed output.
    #[must_use]
    pub fn finish<T>(self, output: T) -> WorkflowExecutionReport<T> {
        WorkflowExecutionReport {
            output,
            trace: self.trace,
            memory_checkpoints: self.memory_checkpoints,
        }
    }

    /// Finishes this workflow execution and records the trace through a
    /// caller-supplied control recorder.
    ///
    /// # Errors
    ///
    /// Returns a control error when the recorder rejects or cannot persist
    /// the workflow trace. Use [`WorkflowExecutionReport::record_control`] on
    /// an already finished report when the caller must retain the workflow
    /// report after a recording error.
    pub fn finish_with_control_recording<T>(
        self,
        output: T,
        recorder: WorkflowControlRecorder<'_>,
    ) -> ControlResult<WorkflowControlRecordedReport<T>> {
        let workflow = self.finish(output);
        let control = workflow.record_control(recorder)?;
        Ok(WorkflowControlRecordedReport { workflow, control })
    }

    /// Finishes this workflow execution and attempts to record the trace while
    /// preserving the completed report if recording fails.
    ///
    /// # Errors
    ///
    /// Returns a recoverable control-recording failure when the recorder
    /// rejects or cannot persist the workflow trace. The error carries the
    /// normal workflow report so callers can retry recording or preserve the
    /// typed output.
    pub fn finish_with_recoverable_control_recording<T>(
        self,
        output: T,
        recorder: WorkflowControlRecorder<'_>,
    ) -> Result<WorkflowControlRecordedReport<T>, WorkflowControlRecordingFailure<T>> {
        self.finish(output).record_control_recoverable(recorder)
    }

    /// Finishes this workflow execution after validating the bound topology.
    ///
    /// # Errors
    ///
    /// Returns an error when this run has a topology and the captured trace
    /// contains undeclared stages, missing required stages, duplicate
    /// successful stages, or dependency order violations.
    pub fn finish_checked<T>(
        self,
        output: T,
    ) -> Result<WorkflowExecutionReport<T>, WorkflowCompletionError> {
        if let Some(topology) = &self.topology {
            topology.validate_trace(&self.trace)?;
        }
        Ok(self.finish(output))
    }

    /// Finishes this workflow execution after validating the bound topology,
    /// then records the trace through a caller-supplied control recorder.
    ///
    /// # Errors
    ///
    /// Returns a completion error when the bound topology rejects the trace.
    /// Returns a control error when validation passes but the recorder rejects
    /// or cannot persist the workflow trace.
    pub fn finish_checked_with_control_recording<T>(
        self,
        output: T,
        recorder: WorkflowControlRecorder<'_>,
    ) -> Result<WorkflowControlRecordedReport<T>, WorkflowCheckedControlRecordingError> {
        if let Some(topology) = &self.topology {
            topology.validate_trace(&self.trace)?;
        }
        let workflow = self.finish(output);
        let control = workflow.record_control(recorder)?;
        Ok(WorkflowControlRecordedReport { workflow, control })
    }

    /// Finishes this workflow execution after validating the bound topology,
    /// then attempts recoverable control recording.
    ///
    /// # Errors
    ///
    /// Returns a completion failure when the bound topology rejects the trace.
    /// Returns a recoverable control-recording failure when validation passes
    /// but the recorder rejects or cannot persist the workflow trace.
    pub fn finish_checked_with_recoverable_control_recording<T>(
        self,
        output: T,
        recorder: WorkflowControlRecorder<'_>,
    ) -> Result<WorkflowControlRecordedReport<T>, WorkflowCheckedControlRecordingFailure<T>> {
        if let Some(topology) = &self.topology {
            topology.validate_trace(&self.trace).map_err(|source| {
                WorkflowCheckedControlRecordingFailure::Completion {
                    source: Box::new(source),
                }
            })?;
        }
        self.finish(output)
            .record_control_recoverable(recorder)
            .map_err(|failure| WorkflowCheckedControlRecordingFailure::Control {
                failure: Box::new(failure),
            })
    }

    /// Records a same-process memory checkpoint for a successful stage.
    ///
    /// # Errors
    ///
    /// Returns an error when the stage has not succeeded or the checkpoint id
    /// already exists in this run.
    pub fn record_memory_checkpoint<T>(
        &mut self,
        request: WorkflowMemoryCheckpointRecord<T>,
    ) -> Result<WorkflowCheckpointRef, WorkflowCheckpointError>
    where
        T: Any + Send + Sync + 'static,
    {
        let stage_id = request.stage_id;
        let checkpoint_id = request.checkpoint_id;
        let stage_index = self
            .trace
            .stages
            .iter()
            .rposition(|trace| {
                trace.stage_id == stage_id.as_str()
                    && trace.status == WorkflowStageStatus::Succeeded
            })
            .ok_or_else(|| {
                WorkflowCheckpointError::StageNotSucceeded(WorkflowStageCheckpointMiss {
                    stage_id: stage_id.clone(),
                    checkpoint_id: checkpoint_id.clone(),
                })
            })?;
        let mut checkpoint_ref =
            WorkflowCheckpointRef::memory(checkpoint_id.as_str(), stage_id.as_str(), request.facts);
        if let Some(content_fingerprint) = request.content_fingerprint {
            checkpoint_ref = checkpoint_ref.with_content_fingerprint(content_fingerprint);
        }
        let checkpoint_ref = self
            .memory_checkpoints
            .insert(checkpoint_ref, request.payload)?;
        self.trace.stages[stage_index]
            .checkpoints
            .push(checkpoint_ref.clone());
        Ok(checkpoint_ref)
    }
}

impl<T> WorkflowExecutionReport<T> {
    /// Records this completed workflow report through a caller-supplied
    /// control recorder.
    ///
    /// # Errors
    ///
    /// Returns a control error when the recorder rejects or cannot persist
    /// the workflow trace.
    pub fn record_control(
        &self,
        recorder: WorkflowControlRecorder<'_>,
    ) -> ControlResult<WorkflowControlRecordingOutcome> {
        recorder.record_trace(&self.trace)
    }

    /// Records this completed workflow report while preserving the report if
    /// control recording fails.
    ///
    /// # Errors
    ///
    /// Returns a recoverable control-recording failure when the recorder
    /// rejects or cannot persist the workflow trace.
    pub fn record_control_recoverable(
        self,
        recorder: WorkflowControlRecorder<'_>,
    ) -> Result<WorkflowControlRecordedReport<T>, WorkflowControlRecordingFailure<T>> {
        match recorder.record_trace(&self.trace) {
            Ok(control) => Ok(WorkflowControlRecordedReport {
                workflow: self,
                control,
            }),
            Err(source) => Err(WorkflowControlRecordingFailure {
                workflow: Box::new(self),
                source,
            }),
        }
    }
}

async fn run_ordered_fanout<I, O, E, F, Fut>(
    inputs: Vec<I>,
    max_concurrency: usize,
    operation: F,
) -> Result<Vec<O>, (usize, E)>
where
    I: Send,
    O: Send,
    E: std::fmt::Display + Send + Sync + 'static,
    F: Fn(usize, I) -> Fut + Send + Sync,
    Fut: Future<Output = Result<O, E>> + Send,
{
    let mut indexed_outputs = Vec::new();
    let mut stream = stream::iter(inputs.into_iter().enumerate().map(|(index, input)| {
        let future = operation(index, input);
        async move {
            future
                .await
                .map(|output| (index, output))
                .map_err(|error| (index, error))
        }
    }))
    .buffer_unordered(max_concurrency.max(1));

    while let Some(result) = stream.next().await {
        indexed_outputs.push(result?);
    }
    indexed_outputs.sort_by_key(|(index, _)| *index);
    Ok(indexed_outputs
        .into_iter()
        .map(|(_, output)| output)
        .collect())
}

fn success_trace(
    stage_id: &str,
    started_unix_ms: u64,
    elapsed: Duration,
    input: WorkflowStageFacts,
    output: WorkflowStageFacts,
) -> WorkflowStageTrace {
    WorkflowStageTrace {
        stage_id: stage_id.to_owned(),
        status: WorkflowStageStatus::Succeeded,
        started_unix_ms,
        duration_nanos: duration_nanos(elapsed),
        input,
        output,
        error: None,
        checkpoints: Vec::new(),
    }
}

fn failure_trace(
    stage_id: &str,
    started_unix_ms: u64,
    elapsed: Duration,
    input: WorkflowStageFacts,
    error: String,
) -> WorkflowStageTrace {
    WorkflowStageTrace {
        stage_id: stage_id.to_owned(),
        status: WorkflowStageStatus::Failed,
        started_unix_ms,
        duration_nanos: duration_nanos(elapsed),
        input,
        output: WorkflowStageFacts::default(),
        error: Some(error),
        checkpoints: Vec::new(),
    }
}

fn duration_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn unix_millis_now() -> u64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}
