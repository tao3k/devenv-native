//! Parser-owned Markdown section-create planning and rendering helpers.

mod building;
mod insertion;
mod types;

pub use building::{
    build_new_sections_content_with_options, compute_content_hash, generate_section_id,
};
pub use insertion::{ParsedHeadingLine, find_insertion_point, parse_heading_line};
pub use types::{BuildSectionOptions, InsertionInfo, SiblingInfo};
