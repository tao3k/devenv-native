use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;
use xiuxian_wendao::query_core::execute::backends::{
    GRAPH_NEIGHBORS_RELATION_TABLE, graph_neighbors_relation_contract,
    graph_neighbors_relation_schema_ref,
};

#[test]
fn graph_neighbors_relation_schema_uses_db_store_table_metadata() {
    let contract = graph_neighbors_relation_contract();
    let schema = graph_neighbors_relation_schema_ref(&contract);

    assert_eq!(
        schema
            .metadata()
            .get(WENDAO_TABLE_METADATA_KEY)
            .map(String::as_str),
        Some(GRAPH_NEIGHBORS_RELATION_TABLE)
    );
    assert_eq!(schema.field(0).name(), "node_id");
}
