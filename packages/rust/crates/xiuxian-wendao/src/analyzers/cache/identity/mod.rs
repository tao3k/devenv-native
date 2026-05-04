mod classify;
mod fingerprint;
#[cfg(feature = "search-runtime")]
mod semantic;

#[cfg(feature = "search-runtime")]
pub(crate) use classify::change_affects_analysis_identity;
#[cfg(feature = "search-runtime")]
pub(crate) use classify::{FingerprintMode, analysis_fingerprint_mode};
pub(crate) use fingerprint::collect_repository_analysis_identity;
#[cfg(feature = "search-runtime")]
pub(crate) use semantic::{plugin_ids_support_semantic_owner_reuse, semantic_fingerprint_for_file};
