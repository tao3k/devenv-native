//! Stable shared contracts for the Wendao package split.
//!
//! Ownership rule:
//! - put stable identifiers, descriptors, traits, and payload records here
//! - do not put runtime config resolution, transport negotiation, host
//!   lifecycle, or Wendao business logic here
//!
//! `xiuxian-wendao-core` is intended to be consumable by runtime helpers and
//! plugin crates without pulling in deployment-dependent behavior.

/// Stable artifact payload and launch-spec records.
pub mod artifacts;
/// Stable capability-binding and contract-version records.
pub mod capabilities;
/// Stable contract-feedback projection helpers.
pub mod contract_feedback;
/// Stable entity and relation records shared across Wendao consumers.
pub mod entity;
/// Stable plugin, capability, and artifact identifiers.
pub mod ids;
/// Stable knowledge payload records shared across Wendao consumers.
pub mod knowledge;
/// Stable link-graph query contracts shared across Wendao consumers.
pub mod link_graph_query;
/// Stable link-graph refresh-mode contract shared across Wendao consumers.
pub mod link_graph_refresh;
/// Stable repo-intelligence contracts shared by Wendao plugins.
pub mod repo_intelligence;
/// Stable semantic resource URI parsing and normalization contracts.
pub mod resource_uri;
/// Stable semantic-document and cognitive-trace payload records.
pub mod semantic_document;
/// Stable semantic-scope metadata DTOs shared with runtime consumers.
pub mod semantic_scope;
/// Stable SQL result DTOs shared across Wendao consumers.
pub mod sql_query;
/// Stable transport endpoint and transport kind records.
pub mod transport;

pub use artifacts::{PluginArtifactPayload, PluginArtifactSelector, PluginLaunchSpec};
pub use capabilities::{ContractVersion, PluginCapabilityBinding, PluginProviderSelector};
pub use contract_feedback::{
    ContractFindingConfidence, ContractFindingSeverity, ContractKnowledgeBatch,
    ContractKnowledgeDecision, ContractKnowledgeEntryId, ContractKnowledgeEnvelope, ContractRuleId,
    ContractRulePackId, ContractSuiteId, WendaoContractFeedbackAdapter,
};
pub use entity::{
    Entity, EntityType, GraphEntity, GraphEntityId, GraphRelation, GraphStats, Relation,
    RelationType,
};
pub use ids::{ArtifactId, CapabilityId, PluginId};
pub use knowledge::{KnowledgeEntry, KnowledgeEntryId};
pub use link_graph_query::{
    LinkGraphDirection, LinkGraphEdgeType, LinkGraphLinkFilter, LinkGraphMatchStrategy,
    LinkGraphPprSubgraphMode, LinkGraphRelatedFilter, LinkGraphRelatedPprOptions, LinkGraphScope,
    LinkGraphSearchFilters, LinkGraphSearchOptions, LinkGraphSortField, LinkGraphSortOrder,
    LinkGraphSortTerm, LinkGraphTagFilter,
};
pub use link_graph_refresh::LinkGraphRefreshMode;
pub use resource_uri::{WENDAO_URI_SCHEME, WendaoResourceUri, WendaoResourceUriError};
pub use semantic_document::{
    CognitiveNodeId, CognitiveSessionId, CognitiveTraceId, CognitiveTraceRecord,
    LinkGraphSemanticDocument, LinkGraphSemanticDocumentKind, SemanticAnchorId, SemanticDocId,
    SemanticDocumentPath,
};
pub use semantic_scope::{
    SEMANTIC_PROJECTION_POLICY_EVIDENCE_METADATA_KEY, SEMANTIC_SCOPE_BUNDLE_METADATA_KEY,
    SEMANTIC_SQL_GUARD_EVIDENCE_METADATA_KEY, SemanticProjectionFreshnessPolicyEntry,
    SemanticProjectionFreshnessPolicyReport, SemanticScopeBundle, SemanticScopeChangeIntent,
    SemanticScopeMetadataEnvelope, SemanticScopeObject, SemanticScopeObjectKind,
    SemanticScopeProjectionPolicyStatus, SemanticScopeProjectionStaleness,
    SemanticScopeRelationEdge, SemanticScopeRelationKind, SemanticScopeStatus,
    parse_semantic_scope_metadata_envelope_json, semantic_scope_metadata_envelope,
    semantic_scope_metadata_envelope_to_vec,
};
pub use sql_query::{SqlBatchPayload, SqlColumnPayload, SqlQueryMetadata, SqlQueryPayload};
pub use transport::{PluginTransportEndpoint, PluginTransportKind};
pub use xiuxian_types::KnowledgeCategory;

#[cfg(test)]
#[path = "../tests/unit/lib_policy.rs"]
mod rust_project_harness_gate;
