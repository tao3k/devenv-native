//! Agentic expansion planning interface.

mod api;
#[path = "candidates.rs"]
mod candidates;
mod types;
#[path = "workers.rs"]
mod workers;

pub(super) use api::agentic_expansion_plan_with_config;
use types::ExpansionCandidateDoc;
