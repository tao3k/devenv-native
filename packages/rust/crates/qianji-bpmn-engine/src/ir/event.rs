//! BPMN event specification types.

use super::BpmnNodeIndex;
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
    /// Conditional event.
    Conditional,
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
    /// Optional resolved event name or fallback label.
    pub name: Option<Arc<str>>,
    /// Optional timer-definition snapshot for timer waits.
    pub timer: Option<BpmnTimerSpec>,
}

impl BpmnEventSpec {
    /// Creates an event specification.
    #[must_use]
    pub fn new(node_index: BpmnNodeIndex, kind: BpmnEventKind) -> Self {
        Self {
            node_index,
            kind,
            reference_id: None,
            name: None,
            timer: None,
        }
    }

    /// Attaches an optional source-level event reference id.
    #[must_use]
    pub fn with_reference_id(mut self, reference_id: impl AsRef<str>) -> Self {
        self.reference_id = Some(Arc::<str>::from(reference_id.as_ref()));
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
}
