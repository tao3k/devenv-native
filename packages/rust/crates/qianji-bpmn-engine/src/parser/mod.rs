//! Parse-surface entrypoints for BPMN source ingestion.

mod import;
mod normalize;
mod service;
mod validate;

pub(crate) use service::parse_bpmn_bundle_impl;
