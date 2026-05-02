//! Skill runtime resolver for `wendao://` resource addresses.
//!
//! Supported addressing modes:
//! - `wendao://skills/<semantic_name>/references/<entity_name>`

/// Asset request handle and callback types.
#[path = "skill_runtime/asset_request/mod.rs"]
pub mod asset_request;
/// Authority auditing for skill runtime manifests.
#[path = "skill_runtime/authority/mod.rs"]
pub mod authority;
/// Error types for skill runtime operations.
#[path = "skill_runtime/error.rs"]
pub mod error;
/// Compatibility re-export for the older `index` namespace.
#[path = "skill_runtime/legacy_index.rs"]
pub mod index;
/// Inventory discovery and semantic mount preloading for skill documents.
#[path = "skill_runtime/index/mod.rs"]
pub mod inventory;
/// Skill runtime manifest loading and scanning.
#[path = "skill_runtime/manifest/mod.rs"]
pub mod manifest;
/// Skill runtime resolver core implementation.
#[path = "skill_runtime/resolver.rs"]
pub mod resolver;
/// Zhixing domain specific indexing and address constants.
#[path = "skill_runtime/zhixing/mod.rs"]
pub mod zhixing;

pub use asset_request::{AssetRequest, WendaoAssetHandle};
pub use error::SkillRuntimeError;
pub use inventory::{
    SkillInventory, SkillInventoryMount, SkillNamespaceIndex, SkillNamespaceMount,
};
pub use manifest::{
    SkillManifest, SkillManifestScan, SkillNativeAliasMountReport, SkillNativeAliasSpec,
    SkillWorkflowType, ToolAnnotations,
};
pub use resolver::core::SkillRuntimeResolver;
pub use zhixing::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED, Error, Result,
    ZHIXING_SKILL_DOC_PATH, ZhixingIndexSummary, ZhixingWendaoIndexer,
    build_embedded_wendao_registry, embedded_discover_canonical_uris, embedded_resource_text,
    embedded_resource_text_from_wendao_uri, embedded_skill_links_for_id,
    embedded_skill_links_for_reference_type, embedded_skill_links_index, embedded_skill_markdown,
};
