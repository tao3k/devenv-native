//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use super::text::{normalize_boolean_operators, split_top_level, strip_outer_parens};
use super::values::{parse_tag_atom, push_unique_many};

pub(in crate::parsers::link_graph::query) fn parse_tag_expression(
    raw: &str,
    tags_all: &mut Vec<String>,
    tags_any: &mut Vec<String>,
    tags_not: &mut Vec<String>,
) {
    let normalized = strip_outer_parens(&normalize_boolean_operators(raw));
    if normalized.trim().is_empty() {
        return;
    }
    let or_groups = split_top_level(&normalized, &['|']);
    let has_or = or_groups.len() > 1;
    or_groups
        .into_iter()
        .flat_map(|group| split_top_level(&group, &[',', '&']))
        .filter_map(|part| parse_tag_atom(&part))
        .for_each(|(is_not, cleaned)| {
            if is_not {
                push_unique_many(tags_not, vec![cleaned]);
            } else if has_or {
                push_unique_many(tags_any, vec![cleaned]);
            } else {
                push_unique_many(tags_all, vec![cleaned]);
            }
        });
}
