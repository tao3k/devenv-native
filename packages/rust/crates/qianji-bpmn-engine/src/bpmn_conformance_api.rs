//! Public bpmn conformance api contracts for BPMN/DMN engine integration.

use std::fmt;

/// Conformance status used by the `BPMN` coverage registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BpmnConformanceStatus {
    /// Fully supported by the current engine contract.
    Supported,
    /// Executable within an explicit bounded subset.
    BoundedExecutable,
    /// Parsed and preserved as metadata without runtime execution.
    MetadataOnly,
    /// Recognized by lint as a deferred executable surface.
    LintDeferred,
    /// Not implemented or recognized by this engine slice.
    Missing,
}

impl BpmnConformanceStatus {
    /// Returns the canonical coverage-matrix spelling for this status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::BoundedExecutable => "bounded executable",
            Self::MetadataOnly => "metadata-only",
            Self::LintDeferred => "lint-deferred",
            Self::Missing => "missing",
        }
    }
}

impl fmt::Display for BpmnConformanceStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Machine-checkable coverage row for one `BPMN` family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BpmnConformanceEntry {
    /// Human-readable `BPMN` family name from the coverage matrix.
    pub family: &'static str,
    /// Overall conformance status for the family.
    pub status: BpmnConformanceStatus,
    /// Parser-layer coverage status.
    pub parser: BpmnConformanceStatus,
    /// Snapshot-layer coverage status.
    pub snapshot: BpmnConformanceStatus,
    /// Lint-layer coverage status.
    pub lint: BpmnConformanceStatus,
    /// Runtime-layer coverage status.
    pub runtime: BpmnConformanceStatus,
    /// Host-surface coverage status.
    pub host_surface: BpmnConformanceStatus,
    /// Stable package-doc anchor that explains the family.
    pub docs_anchor: &'static str,
    /// Next milestone that should promote or maintain this family.
    pub next_milestone: &'static str,
}

/// Returns the crate-owned `BPMN` conformance registry.
#[must_use]
pub const fn bpmn_conformance_registry() -> &'static [BpmnConformanceEntry] {
    crate::bpmn_conformance::BPMN_CONFORMANCE_REGISTRY
}
