//! Agent task-memory helpers for Org read-model recovery.

mod properties;
mod reflection;
mod temporary;

use super::model::AgentOrgTaskListRow;
use xiuxian_memory_engine::InferredMemoryObject;

pub(super) use temporary::{ProbeRecallScope, rank_probe_rows};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum MemoryObjectSourceKind {
    Property,
    Reflection,
}

impl MemoryObjectSourceKind {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Property => "property",
            Self::Reflection => "reflection",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct OrgInferredMemoryObject {
    pub(super) source_kind: MemoryObjectSourceKind,
    pub(super) source_key: String,
    pub(super) object: InferredMemoryObject,
}

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
