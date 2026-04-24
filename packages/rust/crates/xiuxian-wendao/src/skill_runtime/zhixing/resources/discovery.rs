use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use xiuxian_wendao_core::WendaoResourceUri;

use crate::KnowledgeGraph;
use crate::skill_runtime::zhixing::{Error, Result};
use xiuxian_wendao_parsers::parse_frontmatter;

use super::registry::{
    embedded_skill_links_for_id, embedded_skill_links_for_reference_type,
    embedded_skill_links_index,
};
use super::text::embedded_resource_text_from_wendao_uri;

static EMBEDDED_DISCOVERY_RECORDS: OnceLock<
    std::result::Result<Vec<EmbeddedDiscoveryRecord>, String>,
> = OnceLock::new();

#[derive(Debug, Clone, Default)]
struct EmbeddedDiscoveryRecord {
    uri: String,
    reference_ids: std::collections::HashSet<String>,
    reference_types: std::collections::HashSet<String>,
    search_blob: String,
}

/// Discovers canonical semantic URIs from one runtime query expression.
///
/// Supported query forms:
/// - `reference_type:<type>` (or `type:<type>`, `ref_type:<type>`)
/// - `id:<config_id>`
/// - free semantic query (for example `carryover:>=1`)
///
/// Free semantic queries perform token matching across canonical URI, markdown
/// content, and frontmatter-derived hints.
///
/// # Errors
///
/// Returns an error when embedded registry construction fails.
pub fn embedded_discover_canonical_uris(query: &str) -> Result<Vec<String>> {
    let normalized_query = query.trim();
    if normalized_query.is_empty() {
        return Ok(Vec::new());
    }
    let normalized_query = normalized_query
        .strip_prefix("query:")
        .map_or(normalized_query, str::trim);
    if normalized_query.is_empty() {
        return Ok(Vec::new());
    }

    let records = embedded_discovery_records()?;

    if let Some(reference_type) =
        parse_prefixed_value(normalized_query, &["reference_type", "type", "ref_type"])
    {
        let hits = discover_by_reference_type(records, reference_type);
        if !hits.is_empty() {
            return Ok(hits);
        }
        return embedded_skill_links_for_reference_type(reference_type);
    }
    if let Some(config_id) = parse_prefixed_value(normalized_query, &["id"]) {
        let hits = discover_by_config_id(records, config_id);
        if !hits.is_empty() {
            return Ok(hits);
        }
        return embedded_skill_links_for_id(config_id);
    }

    let terms = semantic_query_terms(normalized_query);
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let hits = discover_by_semantic_terms(records, terms.as_slice());
    if !hits.is_empty() {
        return Ok(hits);
    }

    discover_by_registry_scan(terms.as_slice())
}

fn parse_prefixed_value<'a>(query: &'a str, keys: &[&str]) -> Option<&'a str> {
    let lowered = query.to_ascii_lowercase();
    for key in keys {
        let prefix = format!("{key}:");
        if lowered.starts_with(prefix.as_str()) {
            let value = query[prefix.len()..].trim();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn semantic_query_terms(query: &str) -> Vec<String> {
    let mut terms = query
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_')
        .map(str::trim)
        .filter(|term| term.len() >= 2)
        .filter(|term| term.chars().any(|ch| !ch.is_ascii_digit()))
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms
}

fn embedded_discovery_records() -> Result<&'static [EmbeddedDiscoveryRecord]> {
    match EMBEDDED_DISCOVERY_RECORDS.get_or_init(build_embedded_discovery_records) {
        Ok(records) => Ok(records.as_slice()),
        Err(reason) => Err(Error::Internal(format!(
            "failed to build embedded discovery graph cache: {reason}"
        ))),
    }
}

fn build_embedded_discovery_records() -> std::result::Result<Vec<EmbeddedDiscoveryRecord>, String> {
    let graph = Arc::new(KnowledgeGraph::new());
    let indexer = crate::skill_runtime::zhixing::indexer::ZhixingWendaoIndexer::new(
        Arc::clone(&graph),
        PathBuf::new(),
    );
    let _ = indexer
        .index_embedded_skill_references_only()
        .map_err(|error: Error| error.to_string())?;

    let mut by_uri: HashMap<String, EmbeddedDiscoveryRecord> = HashMap::new();
    for relation in graph.get_all_relations() {
        let Some(uri) = relation
            .metadata
            .get("reference_uri")
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        let Ok(parsed_uri) = WendaoResourceUri::parse(uri) else {
            continue;
        };
        let canonical_uri = parsed_uri.canonical_uri();
        let record =
            by_uri
                .entry(canonical_uri.clone())
                .or_insert_with(|| EmbeddedDiscoveryRecord {
                    uri: canonical_uri,
                    ..EmbeddedDiscoveryRecord::default()
                });
        if let Some(reference_id) = relation
            .metadata
            .get("reference_id")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            record
                .reference_ids
                .insert(reference_id.to_ascii_lowercase());
        }
        if let Some(reference_type) = relation
            .metadata
            .get("reference_type")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            record
                .reference_types
                .insert(reference_type.to_ascii_lowercase());
        }
    }

    for record in by_uri.values_mut() {
        let mut haystack = String::new();
        haystack.push_str(record.uri.to_ascii_lowercase().as_str());
        for reference_id in &record.reference_ids {
            haystack.push('\n');
            haystack.push_str(reference_id.as_str());
        }
        for reference_type in &record.reference_types {
            haystack.push('\n');
            haystack.push_str(reference_type.as_str());
        }

        if let Some(content) = embedded_resource_text_from_wendao_uri(record.uri.as_str()) {
            haystack.push('\n');
            haystack.push_str(content.to_ascii_lowercase().as_str());
            let frontmatter = parse_frontmatter(content);
            if let Some(name) = frontmatter.name.as_deref() {
                haystack.push('\n');
                haystack.push_str(name.to_ascii_lowercase().as_str());
            }
            for keyword in frontmatter.routing_keywords {
                haystack.push('\n');
                haystack.push_str(keyword.to_ascii_lowercase().as_str());
            }
            for intent in frontmatter.intents {
                haystack.push('\n');
                haystack.push_str(intent.to_ascii_lowercase().as_str());
            }
        }
        record.search_blob = haystack;
    }

    let mut records = by_uri.into_values().collect::<Vec<_>>();
    records.sort_by(|left, right| left.uri.cmp(&right.uri));
    Ok(records)
}

fn discover_by_reference_type(
    records: &[EmbeddedDiscoveryRecord],
    reference_type: &str,
) -> Vec<String> {
    let normalized = reference_type.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Vec::new();
    }
    let mut hits = records
        .iter()
        .filter(|record| record.reference_types.contains(normalized.as_str()))
        .map(|record| record.uri.clone())
        .collect::<Vec<_>>();
    hits.sort();
    hits.dedup();
    hits
}

fn discover_by_config_id(records: &[EmbeddedDiscoveryRecord], config_id: &str) -> Vec<String> {
    let normalized = config_id.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Vec::new();
    }
    let mut hits = records
        .iter()
        .filter(|record| record.reference_ids.contains(normalized.as_str()))
        .map(|record| record.uri.clone())
        .collect::<Vec<_>>();
    hits.sort();
    hits.dedup();
    hits
}

fn discover_by_semantic_terms(
    records: &[EmbeddedDiscoveryRecord],
    terms: &[String],
) -> Vec<String> {
    let mut hits = records
        .iter()
        .filter(|record| {
            terms
                .iter()
                .all(|term| record.search_blob.contains(term.as_str()))
        })
        .map(|record| record.uri.clone())
        .collect::<Vec<_>>();
    hits.sort();
    hits.dedup();
    hits
}

fn discover_by_registry_scan(terms: &[String]) -> Result<Vec<String>> {
    let mut candidates = embedded_skill_links_index()?
        .into_values()
        .flatten()
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();

    let mut hits = Vec::new();
    for uri in candidates {
        let Some(content) = embedded_resource_text_from_wendao_uri(uri.as_str()) else {
            continue;
        };
        let frontmatter = parse_frontmatter(content);
        let mut haystacks = Vec::with_capacity(5);
        haystacks.push(uri.to_ascii_lowercase());
        haystacks.push(content.to_ascii_lowercase());
        if let Some(name) = frontmatter.name.as_deref() {
            haystacks.push(name.to_ascii_lowercase());
        }
        haystacks.extend(
            frontmatter
                .routing_keywords
                .iter()
                .map(|value| value.to_ascii_lowercase()),
        );
        haystacks.extend(
            frontmatter
                .intents
                .iter()
                .map(|value| value.to_ascii_lowercase()),
        );
        if terms
            .iter()
            .all(|term| haystacks.iter().any(|entry| entry.contains(term.as_str())))
        {
            hits.push(uri);
        }
    }
    hits.sort();
    hits.dedup();
    Ok(hits)
}
