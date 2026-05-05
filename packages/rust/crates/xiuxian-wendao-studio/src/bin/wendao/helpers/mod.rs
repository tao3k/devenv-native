//! Shared CLI helpers (parsing, filters, formatting, index bootstrap).

mod filters;
mod index;
mod monitor;
mod output;
mod sort;

pub(crate) use filters::{
    build_optional_link_filter, build_optional_related_filter, build_optional_related_ppr_options,
    build_optional_tag_filter,
};
pub(crate) use index::build_index;
pub(crate) use monitor::{
    build_agentic_monitor_phases, build_agentic_monitor_summary, build_related_monitor_phases,
};
pub(crate) use output::emit;
pub(crate) use sort::parse_sort_terms;
