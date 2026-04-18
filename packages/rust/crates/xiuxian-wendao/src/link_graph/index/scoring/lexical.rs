#[path = "lexical/document.rs"]
mod document;
#[path = "lexical/helpers.rs"]
mod helpers;
#[path = "lexical/path.rs"]
mod path;

pub(in crate::link_graph::index) use document::score_document;
pub(in crate::link_graph::index) use helpers::token_match_ratio;
pub(in crate::link_graph::index) use path::score_path_fields;
