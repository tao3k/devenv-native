//! Instance construction and runtime-state shells.

use super::{JoinRuntimeState, PendingHostWork, TokenRecord, WaitRegistration};
use crate::error::{BpmnEngineError, Result};
use crate::ir::{BpmnNodeIndex, BpmnPackage, BpmnProcessSpec, ProcessKey};
use serde::{Deserialize, Deserializer};
use std::borrow::Borrow;
use std::sync::Arc;

/// Initial values required to create one workflow instance shell.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BpmnInstanceInit {
    /// Stable instance identifier.
    pub instance_id: Arc<str>,
    /// Initial workflow variables.
    pub initial_variables: serde_json::Value,
    /// Initial timestamp in Unix milliseconds.
    pub initial_timestamp_ms: u64,
}

impl BpmnInstanceInit {
    /// Creates instance-initialization data.
    #[must_use]
    pub fn new(
        instance_id: impl AsRef<str>,
        initial_variables: serde_json::Value,
        initial_timestamp_ms: u64,
    ) -> Self {
        Self {
            instance_id: Arc::<str>::from(instance_id.as_ref()),
            initial_variables,
            initial_timestamp_ms,
        }
    }
}

/// High-level node runtime status shell.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeRuntimeStatus {
    /// Initial dormant state.
    #[default]
    Idle,
    /// Runnable or enqueued state.
    Queued,
    /// External or host-side execution in progress.
    Executing,
    /// Finished state.
    Completed,
    /// Cancelled by an interrupting boundary event.
    Cancelled,
    /// Terminal failure state.
    Failed,
}

/// Per-node runtime record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct NodeRuntimeState {
    /// Node index.
    pub node_index: BpmnNodeIndex,
    /// Current runtime status.
    pub status: NodeRuntimeStatus,
}

/// High-level instance lifecycle shell.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceLifecycle {
    /// Ready to execute.
    Ready,
    /// Currently progressing.
    Running,
    /// Waiting on external or host-side progress.
    Waiting,
    /// Suspended intentionally.
    Suspended,
    /// Completed successfully.
    Completed,
    /// Failed terminally.
    Failed,
}

/// Suspend reason shell.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuspendReason {
    /// Suspended by host request.
    HostRequested,
    /// Suspended due to external wait boundary.
    ExternalWait,
    /// Suspended because the runtime reached a reserved DMN placeholder task.
    DmnPlaceholder,
    /// Suspended by scaffold-only flow control.
    ScaffoldBoundary,
}

/// Snapshot of one active event-competition owner.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventCompetitionState {
    /// Owning gateway node index.
    pub gateway_node_index: BpmnNodeIndex,
    /// Candidate waiting node indices owned by the gateway.
    pub wait_node_indices: Vec<BpmnNodeIndex>,
}

/// Snapshot of one active standard-loop owner.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StandardLoopState {
    /// Owning loop node index.
    pub node_index: BpmnNodeIndex,
    /// Completed iteration count.
    pub completed_iterations: u32,
}

/// Snapshot of one active sequential multi-instance owner.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SequentialMultiInstanceState {
    /// Owning multi-instance node index.
    pub node_index: BpmnNodeIndex,
    /// Total planned sequential iterations.
    pub total_iterations: u32,
    /// Completed iteration count.
    pub completed_iterations: u32,
}

/// Snapshot of one parent execution frame while a bounded call activity runs.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CallActivityFrame {
    /// Suspended parent process identity.
    pub process: ProcessKey,
    /// Cached parent process position inside the owning package.
    #[serde(default)]
    pub process_index: u32,
    /// Parent call-activity node that should resume after the child process completes.
    pub return_node_index: BpmnNodeIndex,
    /// Suspended parent node runtime state.
    pub node_states: Vec<NodeRuntimeState>,
    /// Suspended parent active runtime tokens.
    pub active_tokens: Vec<TokenRecord>,
    /// Suspended parent join progress records.
    pub joins: Vec<JoinRuntimeState>,
    /// Suspended parent standard-loop progress records.
    #[serde(default)]
    pub standard_loops: Vec<StandardLoopState>,
    /// Suspended parent sequential multi-instance owner records.
    #[serde(default)]
    pub sequential_multi_instances: Vec<SequentialMultiInstanceState>,
    /// Suspended parent waiting registrations.
    pub waits: Vec<WaitRegistration>,
    /// Suspended parent event-competition state.
    #[serde(default)]
    pub event_competition: Option<EventCompetitionState>,
    /// Suspended parent pending host work references.
    #[serde(default, deserialize_with = "deserialize_pending_host_work_collection")]
    pub pending_host_work: Vec<PendingHostWork>,
    /// Suspended parent suspend reason.
    pub suspend_reason: Option<SuspendReason>,
}

/// Mutable BPMN instance state shell.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BpmnInstanceState {
    /// Stable instance identifier.
    pub instance_id: Arc<str>,
    /// Process identity.
    pub process: ProcessKey,
    /// Cached process position inside the owning package.
    ///
    /// Runtime resolution validates and repairs this index against
    /// `process_id`, so checkpoints written before this field existed remain
    /// recoverable.
    #[serde(default)]
    pub process_index: u32,
    /// Suspended parent frames while bounded call activities run.
    #[serde(default)]
    pub call_stack: Vec<CallActivityFrame>,
    /// Monotonic checkpoint sequence.
    pub sequence: u64,
    /// High-level lifecycle state.
    pub lifecycle: InstanceLifecycle,
    /// Workflow variables.
    pub variables: serde_json::Value,
    /// Per-node runtime state.
    pub node_states: Vec<NodeRuntimeState>,
    /// Active runtime tokens.
    pub active_tokens: Vec<TokenRecord>,
    /// Join progress records.
    pub joins: Vec<JoinRuntimeState>,
    /// Active standard-loop progress records.
    #[serde(default)]
    pub standard_loops: Vec<StandardLoopState>,
    /// Active sequential multi-instance owner records.
    #[serde(default)]
    pub sequential_multi_instances: Vec<SequentialMultiInstanceState>,
    /// Waiting registrations.
    pub waits: Vec<WaitRegistration>,
    /// Active event-competition state for multi-wait ownership.
    #[serde(default)]
    pub event_competition: Option<EventCompetitionState>,
    /// Pending host work references.
    #[serde(default, deserialize_with = "deserialize_pending_host_work_collection")]
    pub pending_host_work: Vec<PendingHostWork>,
    /// Optional suspend reason.
    pub suspend_reason: Option<SuspendReason>,
    /// Last update timestamp in Unix milliseconds.
    pub updated_at_ms: u64,
}

/// Creates a workflow instance shell for one process.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingProcess`] when the target process does not
/// exist in the provided package.
pub fn create_instance(
    package: impl Borrow<BpmnPackage>,
    process_id: &str,
    init: BpmnInstanceInit,
) -> Result<BpmnInstanceState> {
    let package = package.borrow();
    let (process_index, process) = package.find_process_position(process_id).ok_or_else(|| {
        BpmnEngineError::MissingProcess {
            process_id: process_id.to_string(),
        }
    })?;
    let node_states = process
        .nodes
        .iter()
        .map(|node| NodeRuntimeState {
            node_index: node.index,
            status: NodeRuntimeStatus::Idle,
        })
        .collect();
    Ok(BpmnInstanceState {
        instance_id: init.instance_id,
        process: process.key.clone(),
        process_index,
        call_stack: Vec::new(),
        sequence: 0,
        lifecycle: InstanceLifecycle::Ready,
        variables: init.initial_variables,
        node_states,
        active_tokens: Vec::new(),
        joins: Vec::new(),
        standard_loops: Vec::new(),
        sequential_multi_instances: Vec::new(),
        waits: Vec::new(),
        event_competition: None,
        pending_host_work: Vec::new(),
        suspend_reason: None,
        updated_at_ms: init.initial_timestamp_ms,
    })
}

fn deserialize_pending_host_work_collection<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<PendingHostWork>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PendingHostWorkField {
        None(Option<PendingHostWork>),
        One(PendingHostWork),
        Many(Vec<PendingHostWork>),
    }

    let value = Option::<PendingHostWorkField>::deserialize(deserializer)?;
    Ok(match value {
        None | Some(PendingHostWorkField::None(None)) => Vec::new(),
        Some(PendingHostWorkField::None(Some(pending)) | PendingHostWorkField::One(pending)) => {
            vec![pending]
        }
        Some(PendingHostWorkField::Many(pending)) => pending,
    })
}

pub(crate) fn build_node_states(process: &BpmnProcessSpec) -> Vec<NodeRuntimeState> {
    process
        .nodes
        .iter()
        .map(|node| NodeRuntimeState {
            node_index: node.index,
            status: NodeRuntimeStatus::Idle,
        })
        .collect()
}

pub(crate) fn push_call_activity_frame(
    instance: &mut BpmnInstanceState,
    return_node_index: BpmnNodeIndex,
) {
    instance.call_stack.push(CallActivityFrame {
        process: instance.process.clone(),
        process_index: instance.process_index,
        return_node_index,
        node_states: std::mem::take(&mut instance.node_states),
        active_tokens: std::mem::take(&mut instance.active_tokens),
        joins: std::mem::take(&mut instance.joins),
        standard_loops: std::mem::take(&mut instance.standard_loops),
        sequential_multi_instances: std::mem::take(&mut instance.sequential_multi_instances),
        waits: std::mem::take(&mut instance.waits),
        event_competition: instance.event_competition.take(),
        pending_host_work: std::mem::take(&mut instance.pending_host_work),
        suspend_reason: instance.suspend_reason.take(),
    });
}

pub(crate) fn pop_call_activity_frame(
    instance: &mut BpmnInstanceState,
) -> Option<CallActivityFrame> {
    instance.call_stack.pop()
}

pub(crate) fn install_process_state(
    instance: &mut BpmnInstanceState,
    process: &BpmnProcessSpec,
    process_index: u32,
) {
    instance.process = process.key.clone();
    instance.process_index = process_index;
    instance.node_states = build_node_states(process);
    instance.active_tokens.clear();
    instance.joins.clear();
    instance.standard_loops.clear();
    instance.sequential_multi_instances.clear();
    instance.waits.clear();
    instance.event_competition = None;
    instance.pending_host_work.clear();
    instance.suspend_reason = None;
}

pub(crate) fn restore_call_activity_frame(
    instance: &mut BpmnInstanceState,
    frame: CallActivityFrame,
) -> BpmnNodeIndex {
    let return_node_index = frame.return_node_index;
    instance.process = frame.process;
    instance.process_index = frame.process_index;
    instance.node_states = frame.node_states;
    instance.active_tokens = frame.active_tokens;
    instance.joins = frame.joins;
    instance.standard_loops = frame.standard_loops;
    instance.sequential_multi_instances = frame.sequential_multi_instances;
    instance.waits = frame.waits;
    instance.event_competition = frame.event_competition;
    instance.pending_host_work = frame.pending_host_work;
    instance.suspend_reason = frame.suspend_reason;
    return_node_index
}

pub(crate) fn resolve_process_for_instance<'a>(
    package: &'a BpmnPackage,
    instance: &mut BpmnInstanceState,
) -> Result<&'a BpmnProcessSpec> {
    if let Some(process) = package
        .processes
        .get(instance.process_index as usize)
        .filter(|process| process.key.process_id == instance.process.process_id)
    {
        return Ok(process);
    }

    let (process_index, process) = package
        .find_process_position(instance.process.process_id.as_ref())
        .ok_or_else(|| BpmnEngineError::MissingProcess {
            process_id: instance.process.process_id.to_string(),
        })?;
    instance.process_index = process_index;
    Ok(process)
}

pub(crate) fn standard_loop_completed_iterations(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> u32 {
    instance
        .standard_loops
        .iter()
        .find(|state| state.node_index == node_index)
        .map_or(0, |state| state.completed_iterations)
}

pub(crate) fn ensure_standard_loop_state(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    if instance
        .standard_loops
        .iter()
        .any(|state| state.node_index == node_index)
    {
        return;
    }
    instance.standard_loops.push(StandardLoopState {
        node_index,
        completed_iterations: 0,
    });
}

pub(crate) fn increment_standard_loop_iterations(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> u32 {
    if let Some(state) = instance
        .standard_loops
        .iter_mut()
        .find(|state| state.node_index == node_index)
    {
        state.completed_iterations += 1;
        return state.completed_iterations;
    }

    instance.standard_loops.push(StandardLoopState {
        node_index,
        completed_iterations: 1,
    });
    1
}

pub(crate) fn clear_standard_loop_state(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    instance
        .standard_loops
        .retain(|state| state.node_index != node_index);
}

pub(crate) fn sequential_multi_instance_progress(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Option<(u32, u32)> {
    instance
        .sequential_multi_instances
        .iter()
        .find(|state| state.node_index == node_index)
        .map(|state| (state.completed_iterations, state.total_iterations))
}

pub(crate) fn ensure_sequential_multi_instance_state(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    total_iterations: u32,
) {
    if instance
        .sequential_multi_instances
        .iter()
        .any(|state| state.node_index == node_index)
    {
        return;
    }
    instance
        .sequential_multi_instances
        .push(SequentialMultiInstanceState {
            node_index,
            total_iterations,
            completed_iterations: 0,
        });
}

pub(crate) fn increment_sequential_multi_instance_iterations(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Option<(u32, u32)> {
    let state = instance
        .sequential_multi_instances
        .iter_mut()
        .find(|state| state.node_index == node_index)?;
    state.completed_iterations += 1;
    Some((state.completed_iterations, state.total_iterations))
}

pub(crate) fn clear_sequential_multi_instance_state(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    instance
        .sequential_multi_instances
        .retain(|state| state.node_index != node_index);
}
