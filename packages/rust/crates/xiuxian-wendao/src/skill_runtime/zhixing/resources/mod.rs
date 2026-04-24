//! Embedded `zhixing` resource accessors backed by Wendao AST parsing.

mod discovery;
mod mounts;
mod paths;
mod registry;
mod text;

pub use discovery::embedded_discover_canonical_uris;
pub use paths::ZHIXING_SKILL_DOC_PATH;
pub use registry::{
    build_embedded_wendao_registry, embedded_skill_links_for_id,
    embedded_skill_links_for_reference_type, embedded_skill_links_index,
};
pub use text::{
    embedded_resource_text, embedded_resource_text_from_wendao_uri, embedded_skill_markdown,
};

pub(crate) use mounts::embedded_semantic_reference_mounts;
pub(crate) use paths::{ZHIXING_EMBEDDED_CRATE_ID, embedded_resource_dir};

#[cfg(test)]
#[path = "../../../../tests/unit/skill_runtime/zhixing/resources/mod.rs"]
mod tests;
