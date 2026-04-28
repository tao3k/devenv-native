use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use arrow::array::{
    Array, ArrayRef, Float64Array, Int64Array, ListArray, ListBuilder, StringArray, StringBuilder,
    UInt64Array,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};

use super::super::super::pattern_symbols;
use crate::link_graph::index::{IndexedSection, LinkGraphIndex, SymbolRef};
use crate::link_graph::models::{
    LinkGraphAttachment, LinkGraphDocument, LinkGraphPassage, MarkdownBlock, MarkdownBlockKind,
    PageIndexMeta, PageIndexNode,
};
use crate::parsers::markdown::{CodeObservation, LogbookEntry};

/// Schema version for the DuckDB-local Arrow cache payload.
pub(in crate::link_graph::index::build::cache) const LINK_GRAPH_DUCKDB_ARROW_CACHE_SCHEMA_VERSION:
    &str = "xiuxian_wendao.link_graph.local_duckdb.arrow_snapshot.v1";

const LINK_GRAPH_DUCKDB_ARROW_CACHE_SCHEMA_CONTRACT: &str = "\
docs(id,stem,path,title,lead,doc_type,tags,word_count,search_text,saliency_base,decay_rate,created_ts,modified_ts);\
sections(doc_id,heading_title,heading_path,heading_path_lower,heading_level,line_start,line_end,byte_start,byte_end,section_text,section_text_lower,entities,attributes_json,logbook_json,observations_json);\
edges(source_id,target_id);\
aliases(alias,doc_id);\
residuals(passages_json,attachments_json,page_index_json)";

static LINK_GRAPH_DUCKDB_ARROW_CACHE_SCHEMA_FINGERPRINT: OnceLock<String> = OnceLock::new();

/// Return the fingerprint for the DuckDB-local Arrow cache schema.
pub(in crate::link_graph::index::build::cache) fn duckdb_arrow_cache_schema_fingerprint()
-> &'static str {
    LINK_GRAPH_DUCKDB_ARROW_CACHE_SCHEMA_FINGERPRINT.get_or_init(|| {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        LINK_GRAPH_DUCKDB_ARROW_CACHE_SCHEMA_CONTRACT.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    })
}

pub(in crate::link_graph::index::build::cache) struct LinkGraphArrowSnapshotPayload {
    pub(in crate::link_graph::index::build::cache) docs_ipc: Vec<u8>,
    pub(in crate::link_graph::index::build::cache) sections_ipc: Vec<u8>,
    pub(in crate::link_graph::index::build::cache) edges_ipc: Vec<u8>,
    pub(in crate::link_graph::index::build::cache) aliases_ipc: Vec<u8>,
    pub(in crate::link_graph::index::build::cache) passages_json: String,
    pub(in crate::link_graph::index::build::cache) attachments_json: String,
    pub(in crate::link_graph::index::build::cache) page_index_json: String,
}

/// Encode the link-graph index into DuckDB-local Arrow cache payloads.
///
/// # Errors
///
/// Returns an error when Arrow batch construction, IPC encoding, or bounded
/// residual JSON encoding fails.
pub(in crate::link_graph::index::build::cache) fn encode_arrow_cached_index_payload(
    index: &LinkGraphIndex,
) -> Result<LinkGraphArrowSnapshotPayload, String> {
    let docs_batch = build_docs_batch(index)?;
    let sections_batch = build_sections_batch(index)?;
    let edges_batch = build_edges_batch(index)?;
    let aliases_batch = build_aliases_batch(index)?;
    Ok(LinkGraphArrowSnapshotPayload {
        docs_ipc: encode_batch(&docs_batch)?,
        sections_ipc: encode_batch(&sections_batch)?,
        edges_ipc: encode_batch(&edges_batch)?,
        aliases_ipc: encode_batch(&aliases_batch)?,
        passages_json: serde_json::to_string(&index.passages_by_id)
            .map_err(|error| format!("serialize link-graph passage residuals: {error}"))?,
        attachments_json: serde_json::to_string(&index.attachments_by_doc)
            .map_err(|error| format!("serialize link-graph attachment residuals: {error}"))?,
        page_index_json: encode_cached_page_indices(&index.trees_by_doc)?,
    })
}

/// Decode a DuckDB-local Arrow cache payload into a `LinkGraphIndex`.
///
/// # Errors
///
/// Returns an error when Arrow IPC decoding or bounded residual JSON decoding
/// fails.
pub(in crate::link_graph::index::build::cache) fn decode_arrow_cached_index_payload(
    payload: &LinkGraphArrowSnapshotPayload,
    root: PathBuf,
    include_dirs: Vec<String>,
    excluded_dirs: Vec<String>,
) -> Result<LinkGraphIndex, String> {
    let docs_by_id = decode_docs(payload.docs_ipc.as_slice())?;
    let sections_by_doc = decode_sections(payload.sections_ipc.as_slice())?;
    let (outgoing, incoming, edge_count) = decode_edges(payload.edges_ipc.as_slice())?;
    let alias_to_doc_id = decode_aliases(payload.aliases_ipc.as_slice())?;
    let passages_by_id =
        serde_json::from_str::<HashMap<String, LinkGraphPassage>>(&payload.passages_json)
            .map_err(|error| format!("decode link-graph passage residuals: {error}"))?;
    let attachments_by_doc = serde_json::from_str::<HashMap<String, Vec<LinkGraphAttachment>>>(
        &payload.attachments_json,
    )
    .map_err(|error| format!("decode link-graph attachment residuals: {error}"))?;
    let trees_by_doc = decode_cached_page_indices(&payload.page_index_json)?;
    let rank_by_id = LinkGraphIndex::compute_rank_by_id(&docs_by_id, &incoming, &outgoing);
    let mut index = LinkGraphIndex {
        root,
        include_dirs,
        excluded_dirs,
        docs_by_id,
        passages_by_id,
        sections_by_doc,
        attachments_by_doc,
        trees_by_doc,
        node_parent_map: HashMap::new(),
        explicit_id_registry: HashMap::new(),
        alias_to_doc_id,
        outgoing,
        incoming,
        rank_by_id,
        edge_count,
        virtual_nodes: HashMap::new(),
        symbol_to_docs: HashMap::new(),
    };
    rebuild_cached_page_index_maps(&mut index);
    Ok(index)
}

fn build_docs_batch(index: &LinkGraphIndex) -> Result<RecordBatch, String> {
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

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("stem", DataType::Utf8, false),
        Field::new("path", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("lead", DataType::Utf8, false),
        Field::new("doc_type", DataType::Utf8, true),
        Field::new(
            "tags",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            false,
        ),
        Field::new("word_count", DataType::UInt64, false),
        Field::new("search_text", DataType::Utf8, false),
        Field::new("saliency_base", DataType::Float64, false),
        Field::new("decay_rate", DataType::Float64, false),
        Field::new("created_ts", DataType::Int64, true),
        Field::new("modified_ts", DataType::Int64, true),
    ]));
    RecordBatch::try_new(
        schema,
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
    .map_err(|error| format!("build link-graph Arrow docs batch: {error}"))
}

fn build_sections_batch(index: &LinkGraphIndex) -> Result<RecordBatch, String> {
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

    let schema = Arc::new(Schema::new(vec![
        Field::new("doc_id", DataType::Utf8, false),
        Field::new("heading_title", DataType::Utf8, false),
        Field::new("heading_path", DataType::Utf8, false),
        Field::new("heading_path_lower", DataType::Utf8, false),
        Field::new("heading_level", DataType::UInt64, false),
        Field::new("line_start", DataType::UInt64, false),
        Field::new("line_end", DataType::UInt64, false),
        Field::new("byte_start", DataType::UInt64, false),
        Field::new("byte_end", DataType::UInt64, false),
        Field::new("section_text", DataType::Utf8, false),
        Field::new("section_text_lower", DataType::Utf8, false),
        Field::new(
            "entities",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            false,
        ),
        Field::new("attributes_json", DataType::Utf8, false),
        Field::new("logbook_json", DataType::Utf8, false),
        Field::new("observations_json", DataType::Utf8, false),
    ]));
    RecordBatch::try_new(
        schema,
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
    .map_err(|error| format!("build link-graph Arrow sections batch: {error}"))
}

fn build_edges_batch(index: &LinkGraphIndex) -> Result<RecordBatch, String> {
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

    let schema = Arc::new(Schema::new(vec![
        Field::new("source_id", DataType::Utf8, false),
        Field::new("target_id", DataType::Utf8, false),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(sources)) as ArrayRef,
            Arc::new(StringArray::from(targets)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build link-graph Arrow edges batch: {error}"))
}

fn build_aliases_batch(index: &LinkGraphIndex) -> Result<RecordBatch, String> {
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

    let schema = Arc::new(Schema::new(vec![
        Field::new("alias", DataType::Utf8, false),
        Field::new("doc_id", DataType::Utf8, false),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(alias_values)) as ArrayRef,
            Arc::new(StringArray::from(doc_ids)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build link-graph Arrow aliases batch: {error}"))
}

fn decode_docs(payload: &[u8]) -> Result<HashMap<String, LinkGraphDocument>, String> {
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

fn decode_sections(payload: &[u8]) -> Result<HashMap<String, Vec<IndexedSection>>, String> {
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

type EdgeDecodeTables = (
    HashMap<String, HashSet<String>>,
    HashMap<String, HashSet<String>>,
    usize,
);

fn decode_edges(payload: &[u8]) -> Result<EdgeDecodeTables, String> {
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

fn decode_aliases(payload: &[u8]) -> Result<HashMap<String, String>, String> {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedPageIndexNode {
    node_id: String,
    parent_id: Option<String>,
    title: String,
    level: usize,
    text: String,
    summary: Option<String>,
    children: Vec<CachedPageIndexNode>,
    metadata: CachedPageIndexMeta,
    blocks: Vec<CachedMarkdownBlock>,
}

impl From<&PageIndexNode> for CachedPageIndexNode {
    fn from(value: &PageIndexNode) -> Self {
        Self {
            node_id: value.node_id.clone(),
            parent_id: value.parent_id.clone(),
            title: value.title.clone(),
            level: value.level,
            text: value.text.to_string(),
            summary: value.summary.clone(),
            children: value.children.iter().map(Self::from).collect(),
            metadata: CachedPageIndexMeta::from(&value.metadata),
            blocks: value.blocks.iter().map(CachedMarkdownBlock::from).collect(),
        }
    }
}

impl CachedPageIndexNode {
    fn into_node(self) -> PageIndexNode {
        PageIndexNode {
            node_id: self.node_id,
            parent_id: self.parent_id,
            title: self.title,
            level: self.level,
            text: Arc::<str>::from(self.text),
            summary: self.summary,
            children: self
                .children
                .into_iter()
                .map(CachedPageIndexNode::into_node)
                .collect(),
            metadata: self.metadata.into_meta(),
            blocks: self
                .blocks
                .into_iter()
                .map(CachedMarkdownBlock::into_block)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedPageIndexMeta {
    line_range: (usize, usize),
    byte_range: Option<(usize, usize)>,
    structural_path: Vec<String>,
    content_hash: Option<String>,
    attributes: HashMap<String, String>,
    token_count: usize,
    is_thinned: bool,
    logbook: Vec<LogbookEntry>,
    observations: Vec<CodeObservation>,
}

impl From<&PageIndexMeta> for CachedPageIndexMeta {
    fn from(value: &PageIndexMeta) -> Self {
        Self {
            line_range: value.line_range,
            byte_range: value.byte_range,
            structural_path: value.structural_path.clone(),
            content_hash: value.content_hash.clone(),
            attributes: value.attributes.clone(),
            token_count: value.token_count,
            is_thinned: value.is_thinned,
            logbook: value.logbook.clone(),
            observations: value.observations.clone(),
        }
    }
}

impl CachedPageIndexMeta {
    fn into_meta(self) -> PageIndexMeta {
        PageIndexMeta {
            line_range: self.line_range,
            byte_range: self.byte_range,
            structural_path: self.structural_path,
            content_hash: self.content_hash,
            attributes: self.attributes,
            token_count: self.token_count,
            is_thinned: self.is_thinned,
            logbook: self.logbook,
            observations: self.observations,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedMarkdownBlock {
    block_id: String,
    kind: CachedMarkdownBlockKind,
    byte_range: (usize, usize),
    line_range: (usize, usize),
    content_hash: String,
    content: String,
    id: Option<String>,
    structural_path: Vec<String>,
}

impl From<&MarkdownBlock> for CachedMarkdownBlock {
    fn from(value: &MarkdownBlock) -> Self {
        Self {
            block_id: value.block_id.clone(),
            kind: CachedMarkdownBlockKind::from(&value.kind),
            byte_range: value.byte_range,
            line_range: value.line_range,
            content_hash: value.content_hash.clone(),
            content: value.content.to_string(),
            id: value.id.clone(),
            structural_path: value.structural_path.clone(),
        }
    }
}

impl CachedMarkdownBlock {
    fn into_block(self) -> MarkdownBlock {
        MarkdownBlock {
            block_id: self.block_id,
            kind: self.kind.into_kind(),
            byte_range: self.byte_range,
            line_range: self.line_range,
            content_hash: self.content_hash,
            content: Arc::<str>::from(self.content),
            id: self.id,
            structural_path: self.structural_path,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CachedMarkdownBlockKind {
    Paragraph,
    CodeFence { language: String },
    List { ordered: bool },
    BlockQuote,
    ThematicBreak,
    Table,
    HtmlBlock,
}

impl From<&MarkdownBlockKind> for CachedMarkdownBlockKind {
    fn from(value: &MarkdownBlockKind) -> Self {
        match value {
            MarkdownBlockKind::Paragraph => Self::Paragraph,
            MarkdownBlockKind::CodeFence { language } => Self::CodeFence {
                language: language.clone(),
            },
            MarkdownBlockKind::List { ordered } => Self::List { ordered: *ordered },
            MarkdownBlockKind::BlockQuote => Self::BlockQuote,
            MarkdownBlockKind::ThematicBreak => Self::ThematicBreak,
            MarkdownBlockKind::Table => Self::Table,
            MarkdownBlockKind::HtmlBlock => Self::HtmlBlock,
        }
    }
}

impl CachedMarkdownBlockKind {
    fn into_kind(self) -> MarkdownBlockKind {
        match self {
            Self::Paragraph => MarkdownBlockKind::Paragraph,
            Self::CodeFence { language } => MarkdownBlockKind::CodeFence { language },
            Self::List { ordered } => MarkdownBlockKind::List { ordered },
            Self::BlockQuote => MarkdownBlockKind::BlockQuote,
            Self::ThematicBreak => MarkdownBlockKind::ThematicBreak,
            Self::Table => MarkdownBlockKind::Table,
            Self::HtmlBlock => MarkdownBlockKind::HtmlBlock,
        }
    }
}

fn encode_cached_page_indices(
    trees_by_doc: &HashMap<String, Vec<PageIndexNode>>,
) -> Result<String, String> {
    let cached = trees_by_doc
        .iter()
        .map(|(doc_id, nodes)| {
            (
                doc_id.clone(),
                nodes.iter().map(CachedPageIndexNode::from).collect(),
            )
        })
        .collect::<BTreeMap<_, Vec<_>>>();
    serde_json::to_string(&cached)
        .map_err(|error| format!("serialize link-graph page-index residuals: {error}"))
}

fn decode_cached_page_indices(raw: &str) -> Result<HashMap<String, Vec<PageIndexNode>>, String> {
    let cached = serde_json::from_str::<BTreeMap<String, Vec<CachedPageIndexNode>>>(raw)
        .map_err(|error| format!("decode link-graph page-index residuals: {error}"))?;
    Ok(cached
        .into_iter()
        .map(|(doc_id, nodes)| {
            (
                doc_id,
                nodes
                    .into_iter()
                    .map(CachedPageIndexNode::into_node)
                    .collect(),
            )
        })
        .collect())
}

fn rebuild_cached_page_index_maps(index: &mut LinkGraphIndex) {
    let mut node_parent_map = HashMap::new();
    let mut explicit_id_registry = HashMap::new();
    let mut symbol_to_docs = HashMap::new();
    for nodes in index.trees_by_doc.values() {
        index_cached_page_index_nodes(
            &mut node_parent_map,
            &mut explicit_id_registry,
            &mut symbol_to_docs,
            nodes,
            None,
        );
    }
    index.node_parent_map = node_parent_map;
    index.explicit_id_registry = explicit_id_registry;
    index.symbol_to_docs = symbol_to_docs;
}

fn index_cached_page_index_nodes(
    node_parent_map: &mut HashMap<String, Option<String>>,
    explicit_id_registry: &mut HashMap<String, PageIndexNode>,
    symbol_to_docs: &mut HashMap<String, Vec<SymbolRef>>,
    nodes: &[PageIndexNode],
    parent_id: Option<&str>,
) {
    for node in nodes {
        node_parent_map.insert(node.node_id.clone(), parent_id.map(str::to_string));
        if let Some(id) = node.metadata.attributes.get("ID")
            && !id.trim().is_empty()
        {
            explicit_id_registry
                .entry(node.node_id.clone())
                .or_insert_with(|| node.clone());
        }
        for obs in &node.metadata.observations {
            for symbol in pattern_symbols::extract_pattern_symbols(&obs.pattern) {
                let symbol_ref = SymbolRef {
                    doc_id: node.node_id.split('#').next().unwrap_or("").to_string(),
                    node_id: node.node_id.clone(),
                    pattern: obs.pattern.clone(),
                    language: obs.language.clone(),
                    line_number: obs.line_number,
                    scope: obs.scope.clone(),
                };
                symbol_to_docs.entry(symbol).or_default().push(symbol_ref);
            }
        }
        index_cached_page_index_nodes(
            node_parent_map,
            explicit_id_registry,
            symbol_to_docs,
            &node.children,
            Some(node.node_id.as_str()),
        );
    }
}

fn encode_batch(batch: &RecordBatch) -> Result<Vec<u8>, String> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = StreamWriter::try_new(&mut buffer, batch.schema().as_ref())
            .map_err(|error| format!("open link-graph Arrow IPC writer: {error}"))?;
        writer
            .write(batch)
            .map_err(|error| format!("write link-graph Arrow IPC batch: {error}"))?;
        writer
            .finish()
            .map_err(|error| format!("finish link-graph Arrow IPC stream: {error}"))?;
    }
    Ok(buffer.into_inner())
}

fn decode_single_batch(payload: &[u8], stream_name: &str) -> Result<RecordBatch, String> {
    let reader = StreamReader::try_new(Cursor::new(payload), None)
        .map_err(|error| format!("open link-graph Arrow {stream_name} stream: {error}"))?;
    let batches = reader
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("decode link-graph Arrow {stream_name} stream: {error}"))?;
    let [batch] = batches.as_slice() else {
        return Err(format!(
            "expected one link-graph Arrow {stream_name} batch, got {}",
            batches.len()
        ));
    };
    Ok(batch.clone())
}

fn required_column<'a, T: Array + 'static>(
    batch: &'a RecordBatch,
    column_name: &str,
) -> Result<&'a T, String> {
    batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<T>())
        .ok_or_else(|| format!("missing link-graph Arrow column `{column_name}`"))
}

fn string_at<'a>(array: &'a StringArray, row: usize, column_name: &str) -> Result<&'a str, String> {
    if array.is_null(row) {
        return Err(format!(
            "unexpected null in link-graph Arrow column `{column_name}`"
        ));
    }
    Ok(array.value(row))
}

fn optional_string_at(array: &StringArray, row: usize) -> Option<String> {
    if array.is_null(row) {
        None
    } else {
        Some(array.value(row).to_string())
    }
}

fn optional_i64_at(array: &Int64Array, row: usize) -> Option<i64> {
    if array.is_null(row) {
        None
    } else {
        Some(array.value(row))
    }
}

fn string_list_at(array: &ListArray, row: usize, column_name: &str) -> Result<Vec<String>, String> {
    if array.is_null(row) {
        return Ok(Vec::new());
    }
    let values = array.value(row);
    let Some(strings) = values.as_any().downcast_ref::<StringArray>() else {
        return Err(format!(
            "expected Utf8 values in link-graph Arrow list column `{column_name}`"
        ));
    };
    Ok((0..strings.len())
        .map(|index| {
            if strings.is_null(index) {
                String::new()
            } else {
                strings.value(index).to_string()
            }
        })
        .collect())
}

fn usize_to_u64_saturating(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn u64_to_usize_saturating(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}
