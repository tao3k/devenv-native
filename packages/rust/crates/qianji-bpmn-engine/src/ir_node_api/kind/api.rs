/// Supported bounded BPMN gateway kinds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnGatewayKind {
    /// Parallel fan-out / synchronization gateway.
    Parallel,
    /// Exclusive merge / deterministic single-route gateway.
    Exclusive,
    /// Structured inclusive split / synchronization gateway.
    Inclusive,
    /// Event-based winner-takes-all gateway.
    EventBased,
}

/// Supported high-level BPMN node kinds for the scaffold slice.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnNodeKind {
    /// Start event node.
    StartEvent,
    /// End event node.
    EndEvent,
    /// Intermediate throw event node.
    IntermediateThrowEvent,
    /// Intermediate catch event node.
    IntermediateCatchEvent,
    /// Boundary event attached to one host-blocking task.
    BoundaryEvent,
    /// Message-bound send task.
    SendTask,
    /// Message-bound receive task.
    ReceiveTask,
    /// Service task node.
    ServiceTask,
    /// Script task node dispatched through the host seam.
    ScriptTask,
    /// User task node.
    UserTask,
    /// Manual task node.
    ManualTask,
    /// Business-rule task reserved for future DMN integration.
    BusinessRuleTask,
    /// Generic gateway node.
    Gateway,
    /// Subprocess or call-activity-like node.
    SubProcess,
}

/// Supported bounded subprocess ownership kinds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnSubProcessKind {
    /// One same-package call activity.
    CallActivity,
    /// One inline embedded subprocess body.
    Embedded,
    /// One inline transaction shell.
    Transaction,
    /// One interrupting event subprocess body.
    EventSubProcess,
}
