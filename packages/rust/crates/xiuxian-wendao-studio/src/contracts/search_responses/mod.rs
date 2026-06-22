//! Coordinates the Studio contracts search responses branch and keeps its child modules behind one documented reasoning-tree boundary.

#[cfg(feature = "local-runtime")]
mod conversions;
mod hits;
mod responses;

#[cfg(all(test, feature = "zhenfa-router"))]
pub(crate) use conversions::domain_source_symbol_hits_for_search_plane;
pub use hits::{
    AttachmentSearchHit, DefinitionSearchHit, IntentSearchHit, KnowledgeSearchHit, ObservationHint,
    ReferenceSearchHit, SearchBacklinkItem, SearchHit, SourceSymbolHit,
};
pub use responses::{
    AttachmentSearchResponse, DefinitionResolveResponse, ReferenceSearchResponse, SearchResponse,
};
