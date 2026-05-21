//! `link_graph::index::scoring::lexical` owns Wendao index scoring lexical behavior.

#[path = "document.rs"]
mod document;
#[path = "helpers.rs"]
mod helpers;
#[path = "path.rs"]
mod path;

pub(in crate::link_graph::index) use document::score_document;
pub(in crate::link_graph::index) use helpers::token_match_ratio;
pub(in crate::link_graph::index) use path::score_path_fields;
