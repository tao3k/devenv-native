use xiuxian_wendao::{LinkGraphSortField, LinkGraphSortOrder, LinkGraphSortTerm};

fn parse_sort_field(raw: &str) -> Option<LinkGraphSortField> {
    match raw.trim().to_lowercase().as_str() {
        "score" => Some(LinkGraphSortField::Score),
        "path" => Some(LinkGraphSortField::Path),
        "title" => Some(LinkGraphSortField::Title),
        "stem" => Some(LinkGraphSortField::Stem),
        "created" => Some(LinkGraphSortField::Created),
        "modified" | "updated" => Some(LinkGraphSortField::Modified),
        "random" => Some(LinkGraphSortField::Random),
        "word_count" | "word-count" => Some(LinkGraphSortField::WordCount),
        _ => None,
    }
}

fn parse_sort_order(raw: &str) -> Option<LinkGraphSortOrder> {
    match raw.trim().to_lowercase().as_str() {
        "asc" | "+" => Some(LinkGraphSortOrder::Asc),
        "desc" | "-" => Some(LinkGraphSortOrder::Desc),
        _ => None,
    }
}

fn default_order_for_field(field: LinkGraphSortField) -> LinkGraphSortOrder {
    match field {
        LinkGraphSortField::Path
        | LinkGraphSortField::Title
        | LinkGraphSortField::Stem
        | LinkGraphSortField::Random => LinkGraphSortOrder::Asc,
        LinkGraphSortField::Score
        | LinkGraphSortField::Created
        | LinkGraphSortField::Modified
        | LinkGraphSortField::WordCount => LinkGraphSortOrder::Desc,
    }
}

pub(crate) fn parse_sort_term(raw: &str) -> LinkGraphSortTerm {
    let value = raw.trim().to_lowercase().replace('-', "_");
    if value.is_empty() {
        return LinkGraphSortTerm::default();
    }

    let pair = value
        .split_once(':')
        .or_else(|| value.split_once('/'))
        .or_else(|| value.rsplit_once('_'));
    if let Some((field_raw, order_raw)) = pair
        && let (Some(field), Some(order)) =
            (parse_sort_field(field_raw), parse_sort_order(order_raw))
    {
        return LinkGraphSortTerm { field, order };
    }

    if let Some(field) = parse_sort_field(&value) {
        return LinkGraphSortTerm {
            field,
            order: default_order_for_field(field),
        };
    }

    LinkGraphSortTerm::default()
}

pub(crate) fn parse_sort_terms(raw: &[String]) -> Vec<LinkGraphSortTerm> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.iter().map(|s| parse_sort_term(s)).collect()
}
