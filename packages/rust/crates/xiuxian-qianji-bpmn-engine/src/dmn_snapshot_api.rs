//! Public dmn snapshot api contracts for BPMN/DMN engine integration.

use crate::dmn::snapshot_dmn_source_sync;
use crate::dmn_model_api::{DmnDocumentSnapshot, DmnSourceFile};
use crate::error::Result;

/// Scans one DMN source into a non-executable document snapshot.
///
/// This scan preserves bounded document metadata even when the source later
/// fails the stricter executable decision-table contract.
///
/// # Errors
///
/// Returns an error when the source is not well-formed XML, has no root
/// element, or omits required identifiers such as a decision `id`.
pub fn snapshot_dmn_source(source: &DmnSourceFile) -> Result<DmnDocumentSnapshot> {
    snapshot_dmn_source_sync(source)
}
