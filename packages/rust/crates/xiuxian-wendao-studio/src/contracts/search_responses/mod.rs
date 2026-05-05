#[cfg(feature = "local-runtime")]
mod conversions;
mod hits;
mod responses;

#[cfg(all(test, feature = "local-runtime"))]
pub(crate) use conversions::domain_ast_hits_for_search_plane;
pub use hits::{
    AstSearchHit, AttachmentSearchHit, DefinitionSearchHit, IntentSearchHit, KnowledgeSearchHit,
    ObservationHint, ReferenceSearchHit, SearchBacklinkItem, SearchHit,
};
pub use responses::{
    AstSearchResponse, AttachmentSearchResponse, DefinitionResolveResponse,
    ReferenceSearchResponse, SearchResponse,
};
