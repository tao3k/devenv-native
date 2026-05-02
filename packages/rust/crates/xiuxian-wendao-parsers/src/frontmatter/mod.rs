//! Markdown frontmatter parsing and SKILL.md frontmatter contracts.

mod api;
mod raw;
mod types;

pub use self::api::SkillFrontmatterParseError;
pub use self::api::{
    discover_skill_documents, frontmatter_kind, is_skill_descriptor_path, parse_frontmatter,
    parse_skill_frontmatter, uses_skill_frontmatter,
};
pub use self::raw::{RawFrontmatter, split_frontmatter, split_frontmatter_raw};
pub use self::types::NoteFrontmatter;
