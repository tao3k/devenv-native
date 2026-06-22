//! Public BPMN index contract owner.

/// Compact node index type used by the runtime scaffold.
pub type BpmnNodeIndex = u32;

/// Dense range into precomputed adjacency tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct BpmnIndexRange {
    /// Inclusive start offset in the backing table.
    pub start: u32,
    /// Exclusive end offset in the backing table.
    pub end: u32,
}

impl BpmnIndexRange {
    /// Creates an index range.
    #[must_use]
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}
