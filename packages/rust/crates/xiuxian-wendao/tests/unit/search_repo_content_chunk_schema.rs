use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;
use xiuxian_wendao::search::repo_content_chunk::schema::{
    COLUMN_ID, repo_content_chunk_engine_schema,
};

#[test]
fn repo_content_chunk_engine_schema_uses_db_store_table_metadata() {
    let schema = repo_content_chunk_engine_schema();

    assert_eq!(
        schema
            .metadata()
            .get(WENDAO_TABLE_METADATA_KEY)
            .map(String::as_str),
        Some("repo_content_chunk")
    );
    assert_eq!(schema.field(0).name(), COLUMN_ID);
}
