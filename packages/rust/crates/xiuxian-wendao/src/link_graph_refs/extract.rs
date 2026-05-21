//! `link_graph_refs::extract` owns Wendao link graph refs extract behavior.

use super::model::LinkGraphEntityRef;
use std::collections::HashSet;
use xiuxian_wendao_parsers::{extract_wikilinks, parse_wikilink_literal};

/// Extract all entity references from note content.
///
/// Supports:
/// - `[[EntityName]]` - reference by name
/// - `[[EntityName#Heading]]` - reference with Obsidian-style heading address
/// - `[[EntityName#^block-id]]` - reference with Obsidian-style block address
/// - `[[EntityName|alias]]` - reference with alias (alias is ignored)
///
/// # Arguments
///
/// * `content` - The note body content to search
///
/// # Returns
///
/// Vector of extracted entity references (deduplicated)
#[must_use]
pub fn extract_entity_refs(content: &str) -> Vec<LinkGraphEntityRef> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut refs: Vec<LinkGraphEntityRef> = Vec::new();

    for wikilink in extract_wikilinks(content) {
        let Some(name) = wikilink.addressed_target.target else {
            continue;
        };
        let dedupe_key = match &wikilink.addressed_target.target_address {
            Some(address) => format!("{name}{address}"),
            None => name.clone(),
        };

        if !seen.contains(&dedupe_key) {
            seen.insert(dedupe_key);
            refs.push(LinkGraphEntityRef::new(
                name,
                wikilink.addressed_target.target_address,
                wikilink.original,
            ));
        }
    }

    refs
}

/// Extract entity references from multiple notes (batch processing).
///
/// More efficient than calling `extract_entity_refs` individually
/// when processing many notes.
///
/// # Arguments
///
/// * `notes` - Vector of (`note_id`, content) tuples
///
/// # Returns
///
/// Vector of (`note_id`, `entity_references`) tuples
#[must_use]
pub fn extract_entity_refs_batch<'a>(
    notes: &[(&'a str, &'a str)],
) -> Vec<(&'a str, Vec<LinkGraphEntityRef>)> {
    notes
        .iter()
        .map(|(note_id, content)| (*note_id, extract_entity_refs(content)))
        .collect()
}

/// Find notes that reference a given entity name.
///
/// # Arguments
///
/// * `entity_name` - The entity name to search for
/// * `contents` - Vector of (`note_id`, content) tuples to search
///
/// # Returns
///
/// Vector of note IDs that reference the entity
#[must_use]
pub fn find_notes_referencing_entity<'a>(
    entity_name: &str,
    contents: &[(&'a str, &'a str)],
) -> Vec<&'a str> {
    let lower_name = entity_name.to_lowercase();

    contents
        .iter()
        .filter(|(_, content)| {
            extract_entity_refs(content)
                .iter()
                .any(|entity_ref| entity_ref.name.to_lowercase() == lower_name)
        })
        .map(|(note_id, _)| *note_id)
        .collect()
}

/// Count entity references in content.
#[must_use]
pub fn count_entity_refs(content: &str) -> usize {
    extract_entity_refs(content).len()
}

/// Validate entity reference format.
#[must_use]
pub fn is_valid_entity_ref(text: &str) -> bool {
    parse_entity_ref(text).is_some()
}

/// Parse a single entity reference string.
#[must_use]
pub fn parse_entity_ref(text: &str) -> Option<LinkGraphEntityRef> {
    let parsed = parse_wikilink_literal(text)?;
    let name = parsed.addressed_target.target?;
    Some(LinkGraphEntityRef::new(
        name,
        parsed.addressed_target.target_address,
        text.to_string(),
    ))
}
