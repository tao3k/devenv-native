//! Zhixing runtime artifact ownership for mounted package outputs.

mod mounts;
mod paths;
mod text;

pub use mounts::{embedded_semantic_reference_mounts, embedded_skill_mount_index};
pub use paths::{
    ZHIXING_EMBEDDED_CRATE_ID, ZHIXING_SKILL_DOC_PATH, embedded_resource_dir,
    normalize_embedded_resource_path,
};
pub use text::{
    embedded_resource_text, embedded_resource_text_from_wendao_uri, embedded_skill_markdown,
};
