//! Public cache snapshot API assembled from Arrow batch codecs.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::models::{LinkGraphAttachment, LinkGraphPassage};

use super::decode::{decode_aliases, decode_docs, decode_edges, decode_sections};
use super::encode::{
    build_aliases_batch, build_docs_batch, build_edges_batch, build_sections_batch,
};
use super::ipc::encode_batch;
use super::page_index::{
    decode_cached_page_indices, encode_cached_page_indices, rebuild_cached_page_index_maps,
};

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
