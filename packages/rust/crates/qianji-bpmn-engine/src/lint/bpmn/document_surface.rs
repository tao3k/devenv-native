//! Canonical api seam for BPMN document-surface lint checks.

mod api;
mod collaboration;
mod data;
mod di_anchor;
mod di_anchor_kind;
mod di_boolean;
mod di_completeness;
mod di_enum;
mod di_identity;
mod di_namespace;
mod di_numeric;
mod di_reference;
mod di_semantic;
mod di_topology;
mod issue;
mod metadata;
mod model;
mod shared;
mod summary;
mod xml;

pub(super) use api::deferred_document_surface_issue;
