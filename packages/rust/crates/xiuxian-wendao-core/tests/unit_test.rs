//! Cargo entry point for `xiuxian-wendao-core` unit tests.

#[path = "unit/contract_feedback.rs"]
mod contract_feedback;
#[path = "unit/entity.rs"]
mod entity;
#[path = "unit/knowledge.rs"]
mod knowledge;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/link_graph_query.rs"]
mod link_graph_query;
#[path = "unit/link_graph_refresh.rs"]
mod link_graph_refresh;
#[path = "unit/resource_uri.rs"]
mod resource_uri;
#[path = "unit/semantic_document.rs"]
mod semantic_document;
#[path = "unit/sql_query.rs"]
mod sql_query;
