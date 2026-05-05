//! Arrow IPC decoders for the link-graph snapshot cache.

use std::collections::{HashMap, HashSet};

use arrow::array::{Float64Array, Int64Array, ListArray, StringArray, UInt64Array};

use crate::link_graph::index::IndexedSection;
use crate::link_graph::models::LinkGraphDocument;
use crate::parsers::markdown::{CodeObservation, LogbookEntry};

use super::ipc::{
    decode_single_batch, optional_i64_at, optional_string_at, required_column, string_at,
    string_list_at,
};
use super::primitive::u64_to_usize_saturating;

pub(super) fn decode_docs(payload: &[u8]) -> Result<HashMap<String, LinkGraphDocument>, String> {
    let batch = decode_single_batch(payload, "docs")?;
    let ids = required_column::<StringArray>(&batch, "id")?;
    let stems = required_column::<StringArray>(&batch, "stem")?;
    let paths = required_column::<StringArray>(&batch, "path")?;
    let titles = required_column::<StringArray>(&batch, "title")?;
    let leads = required_column::<StringArray>(&batch, "lead")?;
    let doc_types = required_column::<StringArray>(&batch, "doc_type")?;
    let tags = required_column::<ListArray>(&batch, "tags")?;
    let word_counts = required_column::<UInt64Array>(&batch, "word_count")?;
    let search_texts = required_column::<StringArray>(&batch, "search_text")?;
    let saliency_bases = required_column::<Float64Array>(&batch, "saliency_base")?;
    let decay_rates = required_column::<Float64Array>(&batch, "decay_rate")?;
    let created_values = required_column::<Int64Array>(&batch, "created_ts")?;
    let modified_values = required_column::<Int64Array>(&batch, "modified_ts")?;

    let mut docs_by_id = HashMap::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        let id = string_at(ids, row, "id")?.to_string();
        let stem = string_at(stems, row, "stem")?.to_string();
        let path = string_at(paths, row, "path")?.to_string();
        let title = string_at(titles, row, "title")?.to_string();
        let tag_values = string_list_at(tags, row, "tags")?;
        let search_text = string_at(search_texts, row, "search_text")?.to_string();
        let doc = LinkGraphDocument {
            id: id.clone(),
            id_lower: id.to_lowercase(),
            stem: stem.clone(),
            stem_lower: stem.to_lowercase(),
            path: path.clone(),
            path_lower: path.to_lowercase(),
            title: title.clone(),
            title_lower: title.to_lowercase(),
            tags_lower: tag_values.iter().map(|tag| tag.to_lowercase()).collect(),
            tags: tag_values,
            lead: string_at(leads, row, "lead")?.to_string(),
            doc_type: optional_string_at(doc_types, row),
            word_count: u64_to_usize_saturating(word_counts.value(row)),
            search_text_lower: search_text.to_lowercase(),
            search_text,
            saliency_base: saliency_bases.value(row),
            decay_rate: decay_rates.value(row),
            created_ts: optional_i64_at(created_values, row),
            modified_ts: optional_i64_at(modified_values, row),
        };
        docs_by_id.insert(id, doc);
    }
    Ok(docs_by_id)
}

pub(super) fn decode_sections(
    payload: &[u8],
) -> Result<HashMap<String, Vec<IndexedSection>>, String> {
    let batch = decode_single_batch(payload, "sections")?;
    let doc_ids = required_column::<StringArray>(&batch, "doc_id")?;
    let heading_titles = required_column::<StringArray>(&batch, "heading_title")?;
    let heading_paths = required_column::<StringArray>(&batch, "heading_path")?;
    let heading_paths_lower = required_column::<StringArray>(&batch, "heading_path_lower")?;
    let heading_levels = required_column::<UInt64Array>(&batch, "heading_level")?;
    let line_starts = required_column::<UInt64Array>(&batch, "line_start")?;
    let line_ends = required_column::<UInt64Array>(&batch, "line_end")?;
    let byte_starts = required_column::<UInt64Array>(&batch, "byte_start")?;
    let byte_ends = required_column::<UInt64Array>(&batch, "byte_end")?;
    let section_texts = required_column::<StringArray>(&batch, "section_text")?;
    let section_texts_lower = required_column::<StringArray>(&batch, "section_text_lower")?;
    let entities = required_column::<ListArray>(&batch, "entities")?;
    let attributes_json = required_column::<StringArray>(&batch, "attributes_json")?;
    let logbook_json = required_column::<StringArray>(&batch, "logbook_json")?;
    let observations_json = required_column::<StringArray>(&batch, "observations_json")?;

    let mut sections_by_doc = HashMap::<String, Vec<IndexedSection>>::new();
    for row in 0..batch.num_rows() {
        let doc_id = string_at(doc_ids, row, "doc_id")?.to_string();
        let section = IndexedSection {
            heading_title: string_at(heading_titles, row, "heading_title")?.to_string(),
            heading_path: string_at(heading_paths, row, "heading_path")?.to_string(),
            heading_path_lower: string_at(heading_paths_lower, row, "heading_path_lower")?
                .to_string(),
            heading_level: u64_to_usize_saturating(heading_levels.value(row)),
            line_start: u64_to_usize_saturating(line_starts.value(row)),
            line_end: u64_to_usize_saturating(line_ends.value(row)),
            byte_start: u64_to_usize_saturating(byte_starts.value(row)),
            byte_end: u64_to_usize_saturating(byte_ends.value(row)),
            section_text: string_at(section_texts, row, "section_text")?.to_string(),
            section_text_lower: string_at(section_texts_lower, row, "section_text_lower")?
                .to_string(),
            entities: string_list_at(entities, row, "entities")?,
            attributes: serde_json::from_str::<HashMap<String, String>>(string_at(
                attributes_json,
                row,
                "attributes_json",
            )?)
            .map_err(|error| format!("decode section attributes: {error}"))?,
            logbook: serde_json::from_str::<Vec<LogbookEntry>>(string_at(
                logbook_json,
                row,
                "logbook_json",
            )?)
            .map_err(|error| format!("decode section logbook: {error}"))?,
            observations: serde_json::from_str::<Vec<CodeObservation>>(string_at(
                observations_json,
                row,
                "observations_json",
            )?)
            .map_err(|error| format!("decode section observations: {error}"))?,
        };
        sections_by_doc.entry(doc_id).or_default().push(section);
    }
    Ok(sections_by_doc)
}

pub(super) type EdgeDecodeTables = (
    HashMap<String, HashSet<String>>,
    HashMap<String, HashSet<String>>,
    usize,
);

pub(super) fn decode_edges(payload: &[u8]) -> Result<EdgeDecodeTables, String> {
    let batch = decode_single_batch(payload, "edges")?;
    let sources = required_column::<StringArray>(&batch, "source_id")?;
    let targets = required_column::<StringArray>(&batch, "target_id")?;
    let mut outgoing = HashMap::<String, HashSet<String>>::new();
    let mut incoming = HashMap::<String, HashSet<String>>::new();
    let mut edge_count = 0_usize;
    for row in 0..batch.num_rows() {
        let source = string_at(sources, row, "source_id")?.to_string();
        let target = string_at(targets, row, "target_id")?.to_string();
        if outgoing
            .entry(source.clone())
            .or_default()
            .insert(target.clone())
        {
            incoming.entry(target).or_default().insert(source);
            edge_count = edge_count.saturating_add(1);
        }
    }
    Ok((outgoing, incoming, edge_count))
}

pub(super) fn decode_aliases(payload: &[u8]) -> Result<HashMap<String, String>, String> {
    let batch = decode_single_batch(payload, "aliases")?;
    let aliases = required_column::<StringArray>(&batch, "alias")?;
    let doc_ids = required_column::<StringArray>(&batch, "doc_id")?;
    let mut alias_to_doc_id = HashMap::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        alias_to_doc_id.insert(
            string_at(aliases, row, "alias")?.to_string(),
            string_at(doc_ids, row, "doc_id")?.to_string(),
        );
    }
    Ok(alias_to_doc_id)
}
