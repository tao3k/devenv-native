//! Agent task-memory types for Org read-model recovery.

use xiuxian_memory_engine::InferredMemoryObject;

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
