//! Workflow execution runner.

use std::{
    any::Any,
    future::Future,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use futures::{StreamExt, stream};

use super::{
    WorkflowCheckpointError, WorkflowCheckpointRef, WorkflowCompletionError,
    WorkflowExecutionReport, WorkflowMemoryCheckpointStore, WorkflowStage, WorkflowStageFacts,
    WorkflowStageStatus, WorkflowStageTrace, WorkflowTopology, WorkflowTopologyError,
    WorkflowTrace,
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
        if let Some(topology) = &self.topology
            && !topology.contains_stage(stage_id)
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
        stage_id: &'static str,
        inputs: Vec<I>,
        max_concurrency: usize,
        input_facts: WorkflowStageFacts,
        output_facts: OutputFacts,
        operation: F,
    ) -> Result<Vec<O>, WorkflowExecutionError>
    where
        I: Send,
        O: Send,
        E: std::fmt::Display + Send + Sync + 'static,
        F: Fn(usize, I) -> Fut + Send + Sync,
        Fut: Future<Output = Result<O, E>> + Send,
        OutputFacts: Fn(&[O]) -> WorkflowStageFacts,
    {
        let started_unix_ms = unix_millis_now();
        let started = Instant::now();
        if let Some(topology) = &self.topology
            && !topology.contains_stage(stage_id)
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

        let concurrency = max_concurrency.max(1);
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
        .buffer_unordered(concurrency);

        while let Some(result) = stream.next().await {
            match result {
                Ok(indexed_output) => indexed_outputs.push(indexed_output),
                Err((index, error)) => {
                    let message = format!("fan-out item `{index}` failed: {error}");
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
            }
        }

        indexed_outputs.sort_by_key(|(index, _)| *index);
        let outputs = indexed_outputs
            .into_iter()
            .map(|(_, output)| output)
            .collect::<Vec<_>>();
        let output_facts = output_facts(outputs.as_slice());
        self.trace.stages.push(success_trace(
            stage_id,
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

    /// Records a same-process memory checkpoint for a successful stage.
    ///
    /// # Errors
    ///
    /// Returns an error when the stage has not succeeded or the checkpoint id
    /// already exists in this run.
    pub fn record_memory_checkpoint<T>(
        &mut self,
        stage_id: &str,
        checkpoint_id: impl Into<String>,
        facts: WorkflowStageFacts,
        content_fingerprint: Option<String>,
        payload: Arc<T>,
    ) -> Result<WorkflowCheckpointRef, WorkflowCheckpointError>
    where
        T: Any + Send + Sync + 'static,
    {
        let checkpoint_id = checkpoint_id.into();
        let stage_index = self
            .trace
            .stages
            .iter()
            .rposition(|trace| {
                trace.stage_id == stage_id && trace.status == WorkflowStageStatus::Succeeded
            })
            .ok_or_else(|| WorkflowCheckpointError::StageNotSucceeded {
                stage_id: stage_id.to_owned(),
                checkpoint_id: checkpoint_id.clone(),
            })?;
        let mut checkpoint_ref = WorkflowCheckpointRef::memory(&checkpoint_id, stage_id, facts);
        if let Some(content_fingerprint) = content_fingerprint {
            checkpoint_ref = checkpoint_ref.with_content_fingerprint(content_fingerprint);
        }
        let checkpoint_ref = self.memory_checkpoints.insert(checkpoint_ref, payload)?;
        self.trace.stages[stage_index]
            .checkpoints
            .push(checkpoint_ref.clone());
        Ok(checkpoint_ref)
    }
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
