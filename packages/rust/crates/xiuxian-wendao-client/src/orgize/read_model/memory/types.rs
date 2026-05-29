//! Agent task-memory types for Org read-model recovery.

use xiuxian_memory_engine::InferredMemoryObject;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::orgize::read_model) enum MemoryObjectSourceKind {
    Property,
    Reflection,
}

impl MemoryObjectSourceKind {
    pub(in crate::orgize::read_model) const fn as_str(self) -> &'static str {
        match self {
            Self::Property => "property",
            Self::Reflection => "reflection",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(in crate::orgize::read_model) struct OrgInferredMemoryObject {
    pub(in crate::orgize::read_model) source_kind: MemoryObjectSourceKind,
    pub(in crate::orgize::read_model) source_key: String,
    pub(in crate::orgize::read_model) object: InferredMemoryObject,
}
