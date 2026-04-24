//! Zhixing domain specific indexing and address constants.

mod indexer;
mod resources;
mod types;

pub use indexer::{ZhixingIndexSummary, ZhixingWendaoIndexer};
pub use types::{ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED, Error, Result};

pub use resources::{
    ZHIXING_SKILL_DOC_PATH, build_embedded_wendao_registry, embedded_discover_canonical_uris,
    embedded_resource_text, embedded_resource_text_from_wendao_uri, embedded_skill_links_for_id,
    embedded_skill_links_for_reference_type, embedded_skill_links_index, embedded_skill_markdown,
};

pub(crate) use resources::{
    ZHIXING_EMBEDDED_CRATE_ID, embedded_resource_dir, embedded_semantic_reference_mounts,
};
