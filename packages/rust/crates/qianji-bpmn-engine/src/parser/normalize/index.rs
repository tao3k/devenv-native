//! Shared BPMN normalization index conversions.

use crate::error::{BpmnEngineError, Result};

pub(in crate::parser::normalize) fn normalize_node_index(
    index: usize,
    operation: &'static str,
) -> Result<u32> {
    u32::try_from(index).map_err(|_| BpmnEngineError::UnsupportedOperation { operation })
}
