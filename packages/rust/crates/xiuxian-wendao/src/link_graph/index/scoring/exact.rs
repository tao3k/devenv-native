use crate::link_graph::index::LinkGraphDocument;

pub(in crate::link_graph::index) fn score_document_exact(
    doc: &LinkGraphDocument,
    query: &str,
    case_sensitive: bool,
) -> f64 {
    if query.is_empty() {
        return 0.0;
    }
    let (id_value, stem_value, title_value, path_value, tags_value): (
        &str,
        &str,
        &str,
        &str,
        &[String],
    ) = if case_sensitive {
        (
            doc.id.as_str(),
            doc.stem.as_str(),
            doc.title.as_str(),
            doc.path.as_str(),
            doc.tags.as_slice(),
        )
    } else {
        (
            doc.id_lower.as_str(),
            doc.stem_lower.as_str(),
            doc.title_lower.as_str(),
            doc.path_lower.as_str(),
            doc.tags_lower.as_slice(),
        )
    };

    if id_value == query || stem_value == query {
        return 1.0;
    }
    if title_value == query {
        return 0.95;
    }
    if tags_value.iter().any(|tag| tag == query) {
        return 0.85;
    }
    if path_value == query {
        return 0.8;
    }
    if (case_sensitive && doc.search_text.contains(query))
        || (!case_sensitive && doc.search_text_lower.contains(query))
    {
        return 0.75;
    }
    0.0
}
