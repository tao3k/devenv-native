//! Parser-owned semantic-check grammar helpers.

use super::types::HashReference;

/// Extract ID references from text content.
///
/// Looks for wiki-style links like `[[#id]]` or `[[id]]`.
#[must_use]
pub(crate) fn extract_id_references(text: &str) -> Vec<String> {
    wiki_link_contents(text)
        .map(str::trim)
        .filter(|link| link.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// Extract hash-annotated references from text content.
///
/// Format: `[[#id@hash]]` where `@hash` is the expected content hash.
#[must_use]
pub(crate) fn extract_hash_references(text: &str) -> Vec<HashReference> {
    wiki_link_contents(text)
        .map(str::trim)
        .filter_map(|link| link.strip_prefix('#'))
        .map(hash_reference_from_id_part)
        .collect()
}

fn wiki_link_contents(text: &str) -> impl Iterator<Item = &str> {
    text.match_indices("[[").filter_map(|(start, _)| {
        let content_start = start + 2;
        text[content_start..]
            .find("]]")
            .map(|end| &text[content_start..content_start + end])
    })
}

fn hash_reference_from_id_part(id_part: &str) -> HashReference {
    if let Some(at_pos) = id_part.find('@') {
        HashReference {
            target_id: id_part[..at_pos].to_string(),
            expect_hash: Some(id_part[at_pos + 1..].to_string()),
        }
    } else {
        HashReference {
            target_id: id_part.to_string(),
            expect_hash: None,
        }
    }
}

/// Validate a contract expression against content.
///
/// Supported contract formats:
/// - `must_contain("term1", "term2", ...)`
/// - `must_not_contain("term")`
/// - `min_length(N)`
#[must_use]
pub(crate) fn validate_contract(contract: &str, content: &str) -> Option<String> {
    let contract = contract.trim();

    if let Some(args) = extract_function_args(contract, "must_contain") {
        let terms: Vec<&str> = args
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim())
            .filter(|s| !s.is_empty())
            .collect();

        return terms
            .into_iter()
            .find(|term| !content.contains(*term))
            .map(|term| format!("missing required term '{term}'"));
    }

    if let Some(args) = extract_function_args(contract, "must_not_contain") {
        let term = args.trim().trim_matches('"').trim();
        if content.contains(term) {
            return Some(format!("contains forbidden term '{term}'"));
        }
        return None;
    }

    if let Some(args) = extract_function_args(contract, "min_length") {
        if let Ok(min_len) = args.trim().parse::<usize>()
            && content.len() < min_len
        {
            return Some(format!(
                "content length {} is less than required {}",
                content.len(),
                min_len
            ));
        }
        return None;
    }

    None
}

/// Extract arguments from a function-like contract expression.
#[must_use]
pub(crate) fn extract_function_args<'a>(contract: &'a str, function_name: &str) -> Option<&'a str> {
    let prefix = format!("{function_name}(");
    if contract.starts_with(&prefix) && contract.ends_with(')') {
        Some(&contract[prefix.len()..contract.len() - 1])
    } else {
        None
    }
}

/// Generate a suggested ID from a title.
#[must_use]
pub(crate) fn generate_suggested_id(title: &str) -> String {
    title
        .to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
        .trim_matches('-')
        .to_string()
}
