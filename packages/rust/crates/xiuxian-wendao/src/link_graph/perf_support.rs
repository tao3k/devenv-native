//! `link_graph::perf_support` owns Wendao link graph perf support behavior.

use std::{collections::HashMap, io::Cursor, sync::Arc};

use arrow::array::{
    ArrayRef, Float64Array, Int64Array, ListBuilder, StringArray, StringBuilder, UInt64Array,
};
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_arrow_ipc_stream_with_options, validate_record_batch_schema_with_options,
};

use super::LinkGraphIndex;

const CORE_DOCS_TABLE: &str = "link_graph_perf_core_docs";
const CORE_EDGES_TABLE: &str = "link_graph_perf_core_edges";
const CORE_ALIASES_TABLE: &str = "link_graph_perf_core_aliases";

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
    validate_core_stream_payload(streams.docs.as_slice(), &core_docs_contract(), "docs")?;
    validate_core_stream_payload(streams.edges.as_slice(), &core_edges_contract(), "edges")?;
    validate_core_stream_payload(
        streams.aliases.as_slice(),
        &core_aliases_contract(),
        "aliases",
    )?;
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

    let contract = core_docs_contract();
    let batch = RecordBatch::try_new(
        core_stream_schema_ref(&contract),
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
    .map_err(|error| format!("build link-graph Arrow docs batch: {error}"))?;
    validate_core_stream_batch(&batch, &contract, "docs")?;
    Ok(batch)
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

    let contract = core_edges_contract();
    let batch = RecordBatch::try_new(
        core_stream_schema_ref(&contract),
        vec![
            Arc::new(StringArray::from(sources)) as ArrayRef,
            Arc::new(StringArray::from(targets)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build link-graph Arrow edges batch: {error}"))?;
    validate_core_stream_batch(&batch, &contract, "edges")?;
    Ok(batch)
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

    let contract = core_aliases_contract();
    let batch = RecordBatch::try_new(
        core_stream_schema_ref(&contract),
        vec![
            Arc::new(StringArray::from(alias_values)) as ArrayRef,
            Arc::new(StringArray::from(doc_ids)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build link-graph Arrow aliases batch: {error}"))?;
    validate_core_stream_batch(&batch, &contract, "aliases")?;
    Ok(batch)
}

fn core_docs_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        CORE_DOCS_TABLE,
        true,
        vec![
            utf8_column("id"),
            utf8_column("stem"),
            utf8_column("path"),
            utf8_column("title"),
            nullable_utf8_column("doc_type"),
            utf8_list_column("tags"),
            uint64_column("word_count"),
            float64_column("saliency_base"),
            float64_column("decay_rate"),
            nullable_int64_column("created_ts"),
            nullable_int64_column("modified_ts"),
        ],
    )
}

fn core_edges_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        CORE_EDGES_TABLE,
        true,
        vec![utf8_column("source_id"), utf8_column("target_id")],
    )
}

fn core_aliases_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        CORE_ALIASES_TABLE,
        true,
        vec![utf8_column("alias"), utf8_column("doc_id")],
    )
}

fn core_stream_schema_ref(contract: &ArrowSchemaContract) -> Arc<arrow::datatypes::Schema> {
    let mut metadata = HashMap::new();
    metadata.insert(
        WENDAO_TABLE_METADATA_KEY.to_string(),
        contract.table_name().to_string(),
    );
    Arc::new(build_arrow_schema(contract, metadata))
}

fn validate_core_stream_batch(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
    context: &str,
) -> Result<(), String> {
    validate_record_batch_schema_with_options(
        batch,
        contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| format!("validate link-graph Arrow {context} batch schema: {error}"))
}

fn validate_core_stream_payload(
    payload: &[u8],
    contract: &ArrowSchemaContract,
    context: &str,
) -> Result<(), String> {
    validate_arrow_ipc_stream_with_options(
        payload,
        contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| format!("validate link-graph Arrow {context} IPC schema: {error}"))
}

fn utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

fn nullable_utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

fn uint64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::UInt64)
}

fn nullable_int64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Int64)
}

fn float64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Float64)
}

fn utf8_list_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8List)
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        CORE_ALIASES_TABLE, CORE_DOCS_TABLE, CORE_EDGES_TABLE, LinkGraphIndex,
        core_aliases_contract, core_docs_contract, core_edges_contract, core_stream_schema_ref,
        decode_link_graph_arrow_core_stream_stats, encode_link_graph_arrow_core_streams,
    };
    use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;

    #[test]
    fn link_graph_arrow_core_stream_schemas_use_db_store_table_metadata() {
        let cases = [
            (CORE_DOCS_TABLE, core_docs_contract(), "id"),
            (CORE_EDGES_TABLE, core_edges_contract(), "source_id"),
            (CORE_ALIASES_TABLE, core_aliases_contract(), "alias"),
        ];

        for (table_name, contract, first_column) in cases {
            let schema = core_stream_schema_ref(&contract);

            assert_eq!(
                schema
                    .metadata()
                    .get(WENDAO_TABLE_METADATA_KEY)
                    .map(String::as_str),
                Some(table_name)
            );
            assert_eq!(schema.field(0).name(), first_column);
        }
    }

    #[test]
    fn link_graph_arrow_core_stream_roundtrip_validates_contract_payloads() -> Result<(), String> {
        let root = tempfile::tempdir()
            .map_err(|error| format!("create link-graph Arrow fixture: {error}"))?;
        fs::write(root.path().join("alpha.md"), "# Alpha\n\nSee [[beta]].\n")
            .map_err(|error| format!("write alpha fixture: {error}"))?;
        fs::write(root.path().join("beta.md"), "# Beta\n\nBody.\n")
            .map_err(|error| format!("write beta fixture: {error}"))?;

        let index = LinkGraphIndex::build(root.path())?;
        let streams = encode_link_graph_arrow_core_streams(&index)?;
        let stats = decode_link_graph_arrow_core_stream_stats(&streams)?;

        assert_eq!(stats.doc_count, 2);
        assert!(stats.edge_count >= 1);
        assert!(stats.total_bytes > 0);
        Ok(())
    }
}
