use super::{
    DOCUMENT_EXTRACT_STATUS_TABLE, DOCUMENT_RESOURCE_TABLE, document_extract_status_schema,
    document_resource_schema,
};
use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;

#[test]
fn document_extract_cache_schemas_use_db_store_table_metadata() {
    let cases = [
        (
            document_resource_schema(),
            DOCUMENT_RESOURCE_TABLE,
            "sourcePath",
        ),
        (
            document_extract_status_schema(),
            DOCUMENT_EXTRACT_STATUS_TABLE,
            "jobId",
        ),
    ];

    for (schema, table_name, first_column) in cases {
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
