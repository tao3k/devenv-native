use std::fs;

use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;
use xiuxian_wendao::link_graph::perf_support::{
    CORE_ALIASES_TABLE, CORE_DOCS_TABLE, CORE_EDGES_TABLE, LinkGraphIndex, core_aliases_contract,
    core_docs_contract, core_edges_contract, core_stream_schema_ref,
    decode_link_graph_arrow_core_stream_stats, encode_link_graph_arrow_core_streams,
};

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
    let root =
        tempfile::tempdir().map_err(|error| format!("create link-graph Arrow fixture: {error}"))?;
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
