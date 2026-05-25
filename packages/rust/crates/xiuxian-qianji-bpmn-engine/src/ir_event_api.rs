//! Public BPMN event contract owner.

use crate::ir_index_api::BpmnNodeIndex;
use std::sync::Arc;

/// High-level event kinds relevant to the scaffold.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnEventKind {
    /// Timer event.
    Timer,
    /// Message event.
    Message,
    /// Signal event.
    Signal,
    /// Error event.
    Error,
    /// Escalation event.
    Escalation,
    /// Cancel event.
    Cancel,
    /// Compensation event.
    Compensation,
    /// Conditional event.
    Conditional,
    /// Terminate end event.
    Terminate,
}

/// Snapshot-style timer discriminator for the bounded timer slice.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnTimerKind {
    /// Absolute point-in-time timer.
    Date,
    /// Relative one-shot duration timer.
    Duration,
    /// Repeating cycle timer.
    Cycle,
}

/// Immutable timer-definition snapshot bound to one BPMN event.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnTimerSpec {
    /// Timer expression discriminator.
    pub kind: BpmnTimerKind,
    /// Source-level timer expression text.
    pub expression: Arc<str>,
}

impl BpmnTimerSpec {
    /// Creates a timer-definition snapshot.
    #[must_use]
    pub fn new(kind: BpmnTimerKind, expression: impl AsRef<str>) -> Self {
        Self {
            kind,
            expression: Arc::<str>::from(expression.as_ref()),
        }
    }
}

/// Immutable BPMN event binding for a node.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnEventSpec {
    /// Owning node index.
    pub node_index: BpmnNodeIndex,
    /// Event kind.
    pub kind: BpmnEventKind,
    /// Optional source-level reference id such as `messageRef` or `signalRef`.
    pub reference_id: Option<Arc<str>>,
    /// Whether throw compensation waits for handler completion before routing.
    #[serde(default = "default_wait_for_completion")]
    pub wait_for_completion: bool,
    /// Optional resolved event name or fallback label.
    pub name: Option<Arc<str>>,
    /// Optional timer-definition snapshot for timer waits.
    pub timer: Option<BpmnTimerSpec>,
    /// Optional bounded condition expression for conditional events.
    #[serde(default)]
    pub condition_expression: Option<Arc<str>>,
}

impl BpmnEventSpec {
    /// Creates an event specification.
    #[must_use]
    pub fn new(node_index: BpmnNodeIndex, kind: BpmnEventKind) -> Self {
        Self {
            node_index,
            kind,
            reference_id: None,
            wait_for_completion: default_wait_for_completion(),
            name: None,
            timer: None,
            condition_expression: None,
        }
    }

    /// Attaches an optional source-level event reference id.
    #[must_use]
    pub fn with_reference_id(mut self, reference_id: impl AsRef<str>) -> Self {
        self.reference_id = Some(Arc::<str>::from(reference_id.as_ref()));
        self
    }

    /// Configures throw-compensation completion behavior.
    #[must_use]
    pub fn with_wait_for_completion(mut self, wait_for_completion: bool) -> Self {
        self.wait_for_completion = wait_for_completion;
        self
    }

    /// Attaches an optional resolved event name or fallback label.
    #[must_use]
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(Arc::<str>::from(name.as_ref()));
        self
    }

    /// Attaches an optional timer-definition snapshot.
    #[must_use]
    pub fn with_timer(mut self, timer: BpmnTimerSpec) -> Self {
        self.timer = Some(timer);
        self
    }

    /// Attaches an optional bounded conditional-event expression.
    #[must_use]
    pub fn with_condition_expression(mut self, condition_expression: impl AsRef<str>) -> Self {
        self.condition_expression = Some(Arc::<str>::from(condition_expression.as_ref()));
        self
    }
}

const fn default_wait_for_completion() -> bool {
    true
}
