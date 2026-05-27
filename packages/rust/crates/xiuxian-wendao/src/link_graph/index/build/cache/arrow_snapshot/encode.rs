//! Arrow record-batch builders for the link-graph snapshot cache.

use std::sync::Arc;

use arrow::array::{
    ArrayRef, Float64Array, Int64Array, ListBuilder, StringArray, StringBuilder, UInt64Array,
};
use arrow::record_batch::RecordBatch;

use crate::link_graph::index::LinkGraphIndex;

use super::primitive::usize_to_u64_saturating;
use super::schema::{
    aliases_contract, docs_contract, edges_contract, sections_contract, snapshot_schema_ref,
    validate_snapshot_batch,
};

pub(super) fn build_docs_batch(index: &LinkGraphIndex) -> Result<RecordBatch, String> {
    let mut docs = index.docs_by_id.values().collect::<Vec<_>>();
    docs.sort_by(|left, right| left.id.cmp(&right.id));

    let mut ids = Vec::with_capacity(docs.len());
    let mut stems = Vec::with_capacity(docs.len());
    let mut paths = Vec::with_capacity(docs.len());
    let mut titles = Vec::with_capacity(docs.len());
    let mut leads = Vec::with_capacity(docs.len());
    let mut doc_types = Vec::with_capacity(docs.len());
    let mut word_counts = Vec::with_capacity(docs.len());
    let mut search_texts = Vec::with_capacity(docs.len());
    let mut saliency_bases = Vec::with_capacity(docs.len());
    let mut decay_rates = Vec::with_capacity(docs.len());
    let mut created_values = Vec::with_capacity(docs.len());
    let mut modified_values = Vec::with_capacity(docs.len());
    let mut tags_builder = ListBuilder::new(StringBuilder::new());

    for doc in docs {
        ids.push(doc.id.clone());
        stems.push(doc.stem.clone());
        paths.push(doc.path.clone());
        titles.push(doc.title.clone());
        leads.push(doc.lead.clone());
        doc_types.push(doc.doc_type.clone());
        word_counts.push(usize_to_u64_saturating(doc.word_count));
        search_texts.push(doc.search_text.clone());
        saliency_bases.push(doc.saliency_base);
        decay_rates.push(doc.decay_rate);
        created_values.push(doc.created_ts);
        modified_values.push(doc.modified_ts);
        for tag in &doc.tags {
            tags_builder.values().append_value(tag);
        }
        tags_builder.append(true);
    }

    let contract = docs_contract();
    let batch = RecordBatch::try_new(
        snapshot_schema_ref(&contract),
        vec![
            Arc::new(StringArray::from(ids)) as ArrayRef,
            Arc::new(StringArray::from(stems)) as ArrayRef,
            Arc::new(StringArray::from(paths)) as ArrayRef,
            Arc::new(StringArray::from(titles)) as ArrayRef,
            Arc::new(StringArray::from(leads)) as ArrayRef,
            Arc::new(StringArray::from(doc_types)) as ArrayRef,
            Arc::new(tags_builder.finish()) as ArrayRef,
            Arc::new(UInt64Array::from(word_counts)) as ArrayRef,
            Arc::new(StringArray::from(search_texts)) as ArrayRef,
            Arc::new(Float64Array::from(saliency_bases)) as ArrayRef,
            Arc::new(Float64Array::from(decay_rates)) as ArrayRef,
            Arc::new(Int64Array::from(created_values)) as ArrayRef,
            Arc::new(Int64Array::from(modified_values)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build link-graph Arrow docs batch: {error}"))?;
    validate_snapshot_batch(&batch, &contract, "validate link-graph Arrow docs batch")?;
    Ok(batch)
}

pub(super) fn build_sections_batch(index: &LinkGraphIndex) -> Result<RecordBatch, String> {
    let mut sections = Vec::new();
    for (doc_id, doc_sections) in &index.sections_by_doc {
        for section in doc_sections {
            sections.push((doc_id.as_str(), section));
        }
    }
    sections.sort_by(|left, right| {
        left.0
            .cmp(right.0)
            .then_with(|| left.1.line_start.cmp(&right.1.line_start))
            .then_with(|| left.1.byte_start.cmp(&right.1.byte_start))
            .then_with(|| left.1.heading_path.cmp(&right.1.heading_path))
    });

    let mut doc_ids = Vec::with_capacity(sections.len());
    let mut heading_titles = Vec::with_capacity(sections.len());
    let mut heading_paths = Vec::with_capacity(sections.len());
    let mut heading_paths_lower = Vec::with_capacity(sections.len());
    let mut heading_levels = Vec::with_capacity(sections.len());
    let mut line_starts = Vec::with_capacity(sections.len());
    let mut line_ends = Vec::with_capacity(sections.len());
    let mut byte_starts = Vec::with_capacity(sections.len());
    let mut byte_ends = Vec::with_capacity(sections.len());
    let mut section_texts = Vec::with_capacity(sections.len());
    let mut section_texts_lower = Vec::with_capacity(sections.len());
    let mut attributes_json = Vec::with_capacity(sections.len());
    let mut logbook_json = Vec::with_capacity(sections.len());
    let mut observations_json = Vec::with_capacity(sections.len());
    let mut entities_builder = ListBuilder::new(StringBuilder::new());

    for (doc_id, section) in sections {
        doc_ids.push(doc_id.to_string());
        heading_titles.push(section.heading_title.clone());
        heading_paths.push(section.heading_path.clone());
        heading_paths_lower.push(section.heading_path_lower.clone());
        heading_levels.push(usize_to_u64_saturating(section.heading_level));
        line_starts.push(usize_to_u64_saturating(section.line_start));
        line_ends.push(usize_to_u64_saturating(section.line_end));
        byte_starts.push(usize_to_u64_saturating(section.byte_start));
        byte_ends.push(usize_to_u64_saturating(section.byte_end));
        section_texts.push(section.section_text.clone());
        section_texts_lower.push(section.section_text_lower.clone());
        for entity in &section.entities {
            entities_builder.values().append_value(entity);
        }
        entities_builder.append(true);
        attributes_json.push(
            serde_json::to_string(&section.attributes)
                .map_err(|error| format!("serialize section attributes: {error}"))?,
        );
        logbook_json.push(
            serde_json::to_string(&section.logbook)
                .map_err(|error| format!("serialize section logbook: {error}"))?,
        );
        observations_json.push(
            serde_json::to_string(&section.observations)
                .map_err(|error| format!("serialize section observations: {error}"))?,
        );
    }

    let contract = sections_contract();
    let batch = RecordBatch::try_new(
        snapshot_schema_ref(&contract),
        vec![
            Arc::new(StringArray::from(doc_ids)) as ArrayRef,
            Arc::new(StringArray::from(heading_titles)) as ArrayRef,
            Arc::new(StringArray::from(heading_paths)) as ArrayRef,
            Arc::new(StringArray::from(heading_paths_lower)) as ArrayRef,
            Arc::new(UInt64Array::from(heading_levels)) as ArrayRef,
            Arc::new(UInt64Array::from(line_starts)) as ArrayRef,
            Arc::new(UInt64Array::from(line_ends)) as ArrayRef,
            Arc::new(UInt64Array::from(byte_starts)) as ArrayRef,
            Arc::new(UInt64Array::from(byte_ends)) as ArrayRef,
            Arc::new(StringArray::from(section_texts)) as ArrayRef,
            Arc::new(StringArray::from(section_texts_lower)) as ArrayRef,
            Arc::new(entities_builder.finish()) as ArrayRef,
            Arc::new(StringArray::from(attributes_json)) as ArrayRef,
            Arc::new(StringArray::from(logbook_json)) as ArrayRef,
            Arc::new(StringArray::from(observations_json)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build link-graph Arrow sections batch: {error}"))?;
    validate_snapshot_batch(
        &batch,
        &contract,
        "validate link-graph Arrow sections batch",
    )?;
    Ok(batch)
}

pub(super) fn build_edges_batch(index: &LinkGraphIndex) -> Result<RecordBatch, String> {
    let mut edges = Vec::new();
    for (source, targets) in &index.outgoing {
        for target in targets {
            edges.push((source.as_str(), target.as_str()));
        }
    }
    edges.sort_unstable();

    let mut sources = Vec::with_capacity(edges.len());
    let mut targets = Vec::with_capacity(edges.len());
    for (source, target) in edges {
        sources.push(source.to_string());
        targets.push(target.to_string());
    }

    let contract = edges_contract();
    let batch = RecordBatch::try_new(
        snapshot_schema_ref(&contract),
        vec![
            Arc::new(StringArray::from(sources)) as ArrayRef,
            Arc::new(StringArray::from(targets)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build link-graph Arrow edges batch: {error}"))?;
    validate_snapshot_batch(&batch, &contract, "validate link-graph Arrow edges batch")?;
    Ok(batch)
}

pub(super) fn build_aliases_batch(index: &LinkGraphIndex) -> Result<RecordBatch, String> {
    let mut aliases = index
        .alias_to_doc_id
        .iter()
        .map(|(alias, doc_id)| (alias.as_str(), doc_id.as_str()))
        .collect::<Vec<_>>();
    aliases.sort_unstable();

    let mut alias_values = Vec::with_capacity(aliases.len());
    let mut doc_ids = Vec::with_capacity(aliases.len());
    for (alias, doc_id) in aliases {
        alias_values.push(alias.to_string());
        doc_ids.push(doc_id.to_string());
    }

    let contract = aliases_contract();
    let batch = RecordBatch::try_new(
        snapshot_schema_ref(&contract),
        vec![
            Arc::new(StringArray::from(alias_values)) as ArrayRef,
            Arc::new(StringArray::from(doc_ids)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build link-graph Arrow aliases batch: {error}"))?;
    validate_snapshot_batch(&batch, &contract, "validate link-graph Arrow aliases batch")?;
    Ok(batch)
}
