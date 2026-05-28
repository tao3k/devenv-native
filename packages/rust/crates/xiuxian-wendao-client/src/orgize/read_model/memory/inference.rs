//! Agent task-memory inference from Org read-model rows.

use super::model::AgentOrgTaskListRow;
use super::types::{MemoryObjectSourceKind, OrgInferredMemoryObject};
use crate::orgize::read_model::memory::{properties, reflection};
use xiuxian_memory_engine::InferredMemoryObject;

pub(super) fn inferred_memory_objects_for_row(
    row: &AgentOrgTaskListRow,
) -> Vec<InferredMemoryObject> {
    org_inferred_memory_objects_for_row(row)
        .into_iter()
        .map(|projection| projection.object)
        .collect()
}

pub(super) fn org_inferred_memory_objects_for_row(
    row: &AgentOrgTaskListRow,
) -> Vec<OrgInferredMemoryObject> {
    if properties::row_memory_projection_is_blocked(row) {
        return Vec::new();
    }
    let mut objects = properties::property_memory_objects_for_row(row)
        .into_iter()
        .map(|object| OrgInferredMemoryObject {
            source_kind: MemoryObjectSourceKind::Property,
            source_key: object.question.clone(),
            object,
        })
        .collect::<Vec<_>>();
    objects.extend(
        reflection::reflection_memory_objects_for_row(row)
            .into_iter()
            .map(|object| OrgInferredMemoryObject {
                source_kind: MemoryObjectSourceKind::Reflection,
                source_key: object.question.clone(),
                object,
            }),
    );
    objects
}
