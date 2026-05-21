//! Coordinates the Studio contracts search responses branch and keeps its child modules behind one documented reasoning-tree boundary.

#[cfg(feature = "local-runtime")]
mod conversions;
mod hits;
mod responses;

#[cfg(all(test, feature = "zhenfa-router"))]
pub(crate) use conversions::domain_ast_hits_for_search_plane;
pub use hits::{
    AstSearchHit, AttachmentSearchHit, DefinitionSearchHit, IntentSearchHit, KnowledgeSearchHit,
    ObservationHint, ReferenceSearchHit, SearchBacklinkItem, SearchHit,
};
pub use responses::{
    AstSearchResponse, AttachmentSearchResponse, DefinitionResolveResponse,
    ReferenceSearchResponse, SearchResponse,
};
