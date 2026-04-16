//! Markdown section parsing for link-graph indexing.

mod extract;
mod types;

pub(in crate::parsers::markdown) use extract::adapt_sections;
pub use types::ParsedSection;
pub use xiuxian_wendao_parsers::sections::LogbookEntry;

#[cfg(test)]
pub(crate) use extract::extract_sections;
#[cfg(test)]
pub(crate) use xiuxian_wendao_parsers::sections::{extract_logbook_entries, parse_logbook_entry};
#[cfg(test)]
pub(crate) use xiuxian_wendao_parsers::sections::{
    extract_property_drawers, parse_property_drawer,
};

#[cfg(test)]
#[path = "../../../../tests/unit/parsers/markdown/sections/mod.rs"]
mod tests;
