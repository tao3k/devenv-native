//! Repository Intelligence endpoint handlers for Studio API.

#[path = "analysis/mod.rs"]
pub(crate) mod analysis;
#[path = "analysis_support/mod.rs"]
pub(super) mod analysis_support;
#[path = "command_service.rs"]
mod command_service;
#[path = "family/mod.rs"]
pub(crate) mod family;
#[path = "index.rs"]
pub(crate) mod index;
#[path = "pages/mod.rs"]
pub(crate) mod pages;
#[path = "parse/mod.rs"]
pub(crate) mod parse;
#[path = "projected_service/mod.rs"]
mod projected_service;
#[path = "query/mod.rs"]
pub(crate) mod query;
#[path = "refine.rs"]
pub(crate) mod refine;
#[path = "retrieval/mod.rs"]
pub(crate) mod retrieval;
