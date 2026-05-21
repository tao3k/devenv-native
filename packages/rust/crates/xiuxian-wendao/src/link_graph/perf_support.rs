//! `link_graph::perf_support` owns Wendao link graph perf support behavior.

use std::io::Cursor;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, Float64Array, Int64Array, ListBuilder, StringArray, StringBuilder, UInt64Array,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;

use super::LinkGraphIndex;

/// Native Arrow IPC streams for the core link-graph cache shape.
pub struct LinkGraphArrowCoreStreams {
    /// Document rows as an Arrow IPC stream.
    pub docs: Vec<u8>,
    /// Directed edge rows as an Arrow IPC stream.
    pub edges: Vec<u8>,
    /// Alias-to-document rows as an Arrow IPC stream.
    pub aliases: Vec<u8>,
}

impl LinkGraphArrowCoreStreams {
    /// Total IPC payload bytes across all core streams.
    #[must_use]
    pub fn total_bytes(&self) -> usize {
        self.docs.len() + self.edges.len() + self.aliases.len()
    }
}

/// Decoded row counts for Arrow core-stream profile validation.
#[derive(Debug, Clone, Copy)]
pub struct LinkGraphArrowCoreStreamStats {
    /// Document row count.
    pub doc_count: usize,
    /// Directed edge row count.
    pub edge_count: usize,
    /// Alias row count.
    pub alias_count: usize,
    /// Total IPC payload bytes across all core streams.
    pub total_bytes: usize,
}

/// Encode the `LinkGraph` core cache shape into native Arrow IPC streams.
///
/// This profile intentionally covers the high-cardinality cache core first:
/// documents, directed edges, and aliases. Residual rich section payloads stay
/// out of this probe until the core stream proves worthwhile.
///
/// # Errors
///
/// Returns an error when Arrow batch construction or IPC encoding fails.
pub fn encode_link_graph_arrow_core_streams(
    index: &LinkGraphIndex,
) -> Result<LinkGraphArrowCoreStreams, String> {
    let docs_batch = build_docs_batch(index)?;
    let edges_batch = build_edges_batch(index)?;
    let aliases_batch = build_aliases_batch(index)?;
    Ok(LinkGraphArrowCoreStreams {
        docs: encode_batch(&docs_batch)?,
        edges: encode_batch(&edges_batch)?,
        aliases: encode_batch(&aliases_batch)?,
    })
}

/// Decode Arrow core streams enough to verify row counts.
///
/// # Errors
///
/// Returns an error when any stream fails Arrow IPC decoding.
pub fn decode_link_graph_arrow_core_stream_stats(
    streams: &LinkGraphArrowCoreStreams,
) -> Result<LinkGraphArrowCoreStreamStats, String> {
    Ok(LinkGraphArrowCoreStreamStats {
        doc_count: decode_row_count(streams.docs.as_slice())?,
        edge_count: decode_row_count(streams.edges.as_slice())?,
        alias_count: decode_row_count(streams.aliases.as_slice())?,
        total_bytes: streams.total_bytes(),
    })
}

fn build_docs_batch(index: &LinkGraphIndex) -> Result<RecordBatch, String> {
    let mut docs = index.docs_by_id.values().collect::<Vec<_>>();
    docs.sort_by(|left, right| left.id.cmp(&right.id));

    let mut ids = Vec::with_capacity(docs.len());
    let mut stems = Vec::with_capacity(docs.len());
    let mut paths = Vec::with_capacity(docs.len());
    let mut titles = Vec::with_capacity(docs.len());
    let mut doc_types = Vec::with_capacity(docs.len());
    let mut word_counts = Vec::with_capacity(docs.len());
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
        doc_types.push(doc.doc_type.clone());
        word_counts.push(u64::try_from(doc.word_count).unwrap_or(u64::MAX));
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
        Field::new("doc_type", DataType::Utf8, true),
        Field::new(
            "tags",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            false,
        ),
        Field::new("word_count", DataType::UInt64, false),
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
            Arc::new(StringArray::from(doc_types)) as ArrayRef,
            Arc::new(tags_builder.finish()) as ArrayRef,
            Arc::new(UInt64Array::from(word_counts)) as ArrayRef,
            Arc::new(Float64Array::from(saliency_bases)) as ArrayRef,
            Arc::new(Float64Array::from(decay_rates)) as ArrayRef,
            Arc::new(Int64Array::from(created_values)) as ArrayRef,
            Arc::new(Int64Array::from(modified_values)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build link-graph Arrow docs batch: {error}"))
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

fn decode_row_count(payload: &[u8]) -> Result<usize, String> {
    let reader = StreamReader::try_new(Cursor::new(payload), None)
        .map_err(|error| format!("open link-graph Arrow IPC reader: {error}"))?;
    let batches = reader
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("decode link-graph Arrow IPC stream: {error}"))?;
    Ok(batches.iter().map(RecordBatch::num_rows).sum())
}
