//! DMN lint entrypoint and error-to-guidance mapping.

mod api;
mod contract_constructs;
mod contract_metadata;
mod contract_requirements;
mod contract_shape;
mod contract_subset;
mod decision;
mod document_business_context;
mod document_dispatch;
mod document_missing;
mod document_namespace;
mod document_root;
mod document_root_artifacts;
mod document_structures;
mod evidence;
mod snapshot_classify;
mod snapshot_count;
mod unexpected;

pub(crate) use api::lint_dmn_source_impl;
