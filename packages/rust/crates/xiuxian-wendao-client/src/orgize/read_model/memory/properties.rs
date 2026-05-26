//! Org property adapter for memory object inference.

use super::super::model::AgentOrgTaskListRow;
use xiuxian_memory_engine::{
    InferredMemoryObject, infer_memory_lifecycle_facts_from_properties,
    infer_memory_objects_from_properties,
};

pub(in crate::orgize::read_model) fn property_memory_objects_for_row(
    row: &AgentOrgTaskListRow,
) -> Vec<InferredMemoryObject> {
    if row_memory_projection_is_blocked(row) {
        return Vec::new();
    }
    infer_memory_objects_from_properties(
        row.properties
            .iter()
            .map(|property| (property.key.as_str(), property.value.as_str())),
    )
}

pub(super) fn row_memory_projection_is_blocked(row: &AgentOrgTaskListRow) -> bool {
    !infer_memory_lifecycle_facts_from_properties(
        row.properties
            .iter()
            .map(|property| (property.key.as_str(), property.value.as_str())),
    )
    .evaluate()
    .projection_allowed
}
