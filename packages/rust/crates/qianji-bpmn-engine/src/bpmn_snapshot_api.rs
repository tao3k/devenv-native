//! Public bpmn snapshot api contracts for BPMN/DMN engine integration.

use crate::bpmn_model_api::BpmnDocumentSnapshot;
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot::snapshot_bpmn_source_sync;
use crate::error::Result;

/// Scans one BPMN source into a non-executable document snapshot.
///
/// The snapshot surface preserves document-level metadata for analysis,
/// linting, and adapter tooling. It intentionally does not make those BPMN
/// constructs executable.
///
/// # Errors
///
/// Returns typed BPMN XML errors when the source payload is malformed or has
/// no root element.
pub fn snapshot_bpmn_source(source: &BpmnSourceFile) -> Result<BpmnDocumentSnapshot> {
    snapshot_bpmn_source_sync(source)
}
