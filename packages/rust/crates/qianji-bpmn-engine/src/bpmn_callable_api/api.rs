//! Public bpmn callable api contracts for BPMN/DMN engine integration.

use std::sync::Arc;

/// Package-owned registry of BPMN callable definitions and bindings.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCallableRegistry {
    /// Callable definitions discovered in the same BPMN package.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definitions: Vec<BpmnCallableDefinition>,
    /// `callActivity` bindings discovered in executable process graphs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub call_activity_bindings: Vec<BpmnCallActivityBinding>,
}

impl BpmnCallableRegistry {
    /// Returns true when no callable definition or binding has been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty() && self.call_activity_bindings.is_empty()
    }

    /// Finds one callable definition by BPMN identifier.
    #[must_use]
    pub fn find_definition(&self, callable_id: &str) -> Option<&BpmnCallableDefinition> {
        self.definitions
            .iter()
            .find(|definition| definition.callable_id.as_ref() == callable_id)
    }
}

/// BPMN callable definition kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnCallableKind {
    /// A top-level BPMN `process` callable.
    Process,
    /// A top-level BPMN `globalTask` callable.
    GlobalTask,
    /// A top-level BPMN `globalBusinessRuleTask` callable.
    GlobalBusinessRuleTask,
    /// A top-level BPMN `globalManualTask` callable.
    GlobalManualTask,
    /// A top-level BPMN `globalScriptTask` callable.
    GlobalScriptTask,
    /// A top-level BPMN `globalUserTask` callable.
    GlobalUserTask,
}

impl BpmnCallableKind {
    pub(super) fn from_global_task_tag(tag: &str) -> Option<Self> {
        match tag {
            "globalTask" => Some(Self::GlobalTask),
            "globalBusinessRuleTask" => Some(Self::GlobalBusinessRuleTask),
            "globalManualTask" => Some(Self::GlobalManualTask),
            "globalScriptTask" => Some(Self::GlobalScriptTask),
            "globalUserTask" => Some(Self::GlobalUserTask),
            _ => None,
        }
    }
}

/// One callable definition in the same parsed BPMN package.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCallableDefinition {
    /// Stable BPMN callable identifier.
    pub callable_id: Arc<str>,
    /// BPMN callable kind.
    pub kind: BpmnCallableKind,
    /// Optional human-readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Arc<str>>,
    /// Source document id that declared the callable.
    pub source_id: Arc<str>,
    /// Optional BPMN `isExecutable` marker for process callables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_executable: Option<bool>,
    /// Whether this callable already has bounded runtime execution in this package.
    #[serde(default)]
    pub runtime_available: bool,
    /// Optional BPMN `processType` marker for process callables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_type: Option<Arc<str>>,
    /// Optional BPMN `isClosed` marker for process callables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_closed: Option<bool>,
    /// Optional BPMN implementation marker for global task callables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<Arc<str>>,
    /// Optional BPMN script language marker for global script tasks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_language: Option<Arc<str>>,
    /// Optional BPMN script payload for global script tasks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<Arc<str>>,
    /// Direct supported interface references preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_interface_refs: Vec<Arc<str>>,
    /// Callable data input declarations preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<BpmnCallableDataRef>,
    /// Callable data output declarations preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<BpmnCallableDataRef>,
    /// Direct callable IO bindings preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub io_bindings: Vec<BpmnCallableIoBinding>,
}

/// One callable data input/output declaration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCallableDataRef {
    /// Optional BPMN data identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_id: Option<Arc<str>>,
    /// Optional human-readable data name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Arc<str>>,
    /// Optional BPMN item definition reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_subject_ref: Option<Arc<str>>,
    /// Optional BPMN collection marker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_collection: Option<bool>,
}

/// One direct callable IO binding.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCallableIoBinding {
    /// Optional BPMN IO-binding identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_id: Option<Arc<str>>,
    /// Referenced callable operation identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_ref: Option<Arc<str>>,
    /// Referenced input data identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_data_ref: Option<Arc<str>>,
    /// Referenced output data identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_data_ref: Option<Arc<str>>,
}

/// Runtime execution policy for one `callActivity` binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnCallableBindingExecutionPolicy {
    /// Existing bounded runtime path: call another process in the same package.
    BoundedProcessCall,
}

/// One `callActivity` binding owned by a parsed process graph.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCallActivityBinding {
    /// Process that owns the `callActivity` node.
    pub process_id: Arc<str>,
    /// `callActivity` BPMN identifier.
    pub activity_id: Arc<str>,
    /// Referenced callable identifier.
    pub target_id: Arc<str>,
    /// Resolved callable kind.
    pub target_kind: BpmnCallableKind,
    /// Bounded execution policy for this binding.
    pub execution_policy: BpmnCallableBindingExecutionPolicy,
}
