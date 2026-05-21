//! `link_graph::index::scoring` owns Wendao link graph index scoring behavior.

mod basics;
mod exact;
#[path = "lexical/mod.rs"]
mod lexical;
mod regex_score;

pub(super) use basics::{normalize_with_case, section_tree_distance, tokenize};
pub(super) use exact::score_document_exact;
pub(super) use lexical::{score_document, score_path_fields, token_match_ratio};
pub(super) use regex_score::score_document_regex;
