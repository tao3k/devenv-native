//! Parser-owned docs governance line/path grammar helpers.

#[path = "api.rs"]
mod api;
#[path = "types.rs"]
pub mod types;

#[cfg(any(test, feature = "zhenfa-router"))]
pub use self::api::{
    collect_index_body_links, collect_lines, derive_opaque_doc_id, extract_hidden_path_links,
    extract_wikilinks, is_canonical_repo_doc, is_opaque_doc_id, is_package_local_crate_doc,
    parse_footer_block, parse_relations_links_line, parse_top_properties_drawer,
};

#[cfg(test)]
#[path = "../../../tests/unit/parsers/docs_governance.rs"]
mod tests;
