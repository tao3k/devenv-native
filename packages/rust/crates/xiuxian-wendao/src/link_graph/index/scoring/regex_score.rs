use crate::link_graph::index::LinkGraphDocument;
use regex::Regex;

pub(in crate::link_graph::index) fn score_document_regex(
    doc: &LinkGraphDocument,
    regex: &Regex,
) -> f64 {
    if regex.is_match(&doc.id) || regex.is_match(&doc.stem) {
        return 1.0;
    }
    if regex.is_match(&doc.title) {
        return 0.95;
    }
    if regex.is_match(&doc.path) {
        return 0.8;
    }
    if doc.tags.iter().any(|tag| regex.is_match(tag)) {
        return 0.85;
    }
    if regex.is_match(&doc.search_text) {
        return 0.75;
    }
    0.0
}
