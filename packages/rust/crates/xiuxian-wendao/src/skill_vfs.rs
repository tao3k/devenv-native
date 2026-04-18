//! Skill VFS resolver for `wendao://` resource addresses.
//!
//! Supported addressing modes:
//! - `wendao://skills/<semantic_name>/references/<entity_name>`
//! - `wendao://skills-internal/<skill_name>/<relative_path>`

/// Asset request handle and callback types.
#[path = "skill_vfs/asset_request/mod.rs"]
pub mod asset_request;
/// Error types for skill VFS operations.
#[path = "skill_vfs/error.rs"]
pub mod error;
/// Semantic namespace indexing and preloading.
#[path = "skill_vfs/index/mod.rs"]
pub mod index;
/// Authority auditing for internal skill manifests.
#[path = "skill_vfs/internal_authority/mod.rs"]
pub mod internal_authority;
/// Internal skill manifest loading and scanning.
#[path = "skill_vfs/internal_manifest/mod.rs"]
pub mod internal_manifest;
/// Skill VFS resolver core implementation.
#[path = "skill_vfs/resolver.rs"]
pub mod resolver;
/// Zhixing domain specific indexing and address constants.
#[path = "skill_vfs/zhixing/mod.rs"]
pub mod zhixing;

pub use asset_request::{AssetRequest, WendaoAssetHandle};
pub use error::SkillVfsError;
pub use index::{SkillNamespaceIndex, SkillNamespaceMount};
pub use internal_manifest::{InternalSkillManifest, InternalSkillWorkflowType};
pub use resolver::core::SkillVfsResolver;
pub use xiuxian_skills::InternalSkillManifestScan;
pub use zhixing::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED, Error, Result,
    ZHIXING_SKILL_DOC_PATH, ZhixingIndexSummary, ZhixingWendaoIndexer,
    build_embedded_wendao_registry, embedded_discover_canonical_uris, embedded_resource_text,
    embedded_resource_text_from_wendao_uri, embedded_skill_links_for_id,
    embedded_skill_links_for_reference_type, embedded_skill_links_index, embedded_skill_markdown,
};
