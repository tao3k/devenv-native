//! Repository Intelligence endpoint handlers for Studio API.

#[path = "repo/analysis.rs"]
pub(crate) mod analysis;
#[path = "repo/command_service.rs"]
mod command_service;
#[path = "repo/family.rs"]
pub(crate) mod family;
#[path = "repo/index.rs"]
pub(crate) mod index;
#[path = "repo/pages.rs"]
pub(crate) mod pages;
#[path = "repo/parse.rs"]
pub(crate) mod parse;
#[path = "repo/projected_service.rs"]
mod projected_service;
#[path = "repo/query.rs"]
pub(crate) mod query;
#[path = "repo/refine.rs"]
pub(crate) mod refine;
#[path = "repo/retrieval.rs"]
pub(crate) mod retrieval;
#[path = "repo/shared.rs"]
pub(super) mod shared;
