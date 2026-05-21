//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use super::text::{normalize_boolean_operators, split_top_level, strip_outer_parens};
use crate::link_graph::{LinkGraphEdgeType, LinkGraphScope};

pub(in crate::parsers::link_graph::query) fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub(in crate::parsers::link_graph::query) fn parse_scope(raw: &str) -> Option<LinkGraphScope> {
    match raw.trim().to_lowercase().as_str() {
        "doc" | "doc_only" => Some(LinkGraphScope::DocOnly),
        "section" | "section_only" => Some(LinkGraphScope::SectionOnly),
        "mixed" => Some(LinkGraphScope::Mixed),
        _ => None,
    }
}

pub(in crate::parsers::link_graph::query) fn parse_edge_type(
    raw: &str,
) -> Option<LinkGraphEdgeType> {
    match raw.trim().to_lowercase().as_str() {
        "structural" => Some(LinkGraphEdgeType::Structural),
        "semantic" => Some(LinkGraphEdgeType::Semantic),
        "provisional" => Some(LinkGraphEdgeType::Provisional),
        "verified" => Some(LinkGraphEdgeType::Verified),
        "property_drawer" | "propertydrawer" | "property-drawer" => {
            Some(LinkGraphEdgeType::PropertyDrawer)
        }
        _ => None,
    }
}

pub(in crate::parsers::link_graph::query) fn parse_list_values(raw: &str) -> Vec<String> {
    let normalized = normalize_boolean_operators(raw);
    split_top_level(&normalized, &[',', '|', ';', '&'])
        .into_iter()
        .map(|item| strip_outer_parens(&item))
        .map(|item| {
            item.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string()
        })
        .filter(|item| !item.is_empty())
        .collect()
}

pub(in crate::parsers::link_graph::query) fn parse_tag_atom(raw: &str) -> Option<(bool, String)> {
    let mut token = strip_outer_parens(raw).trim().to_string();
    if token.is_empty() {
        return None;
    }

    let mut is_not = false;
    if let Some(rest) = token.strip_prefix('!').or_else(|| token.strip_prefix('-')) {
        is_not = true;
        token = rest.trim().to_string();
    } else if token.len() >= 4 && token[..4].eq_ignore_ascii_case("not ") {
        is_not = true;
        token = token[4..].trim().to_string();
    }

    token = strip_outer_parens(&token)
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string();
    if token.is_empty() {
        return None;
    }
    Some((is_not, token))
}

pub(in crate::parsers::link_graph::query) fn push_unique_many(
    dst: &mut Vec<String>,
    values: Vec<String>,
) {
    for value in values {
        if !dst
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&value))
        {
            dst.push(value);
        }
    }
}

pub(in crate::parsers::link_graph::query) fn parse_directive_key(raw_key: &str) -> (bool, String) {
    let mut token = raw_key.trim();
    let mut negate = false;
    while let Some(rest) = token.strip_prefix('-').or_else(|| token.strip_prefix('!')) {
        negate = !negate;
        token = rest.trim_start();
    }
    (negate, token.to_lowercase())
}
