use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;
use xiuxian_wendao::search::contracts::search_index::diagnostics::relations::{
    QUERY_TELEMETRY_DIAGNOSTICS_TABLE, REPO_READ_PRESSURE_DIAGNOSTICS_TABLE,
    STATUS_DIAGNOSTICS_TABLE, STATUS_REASON_DIAGNOSTICS_TABLE, diagnostics_schema_ref,
    query_telemetry_contract, repo_read_pressure_contract, status_reason_contract,
    status_snapshot_contract,
};

#[test]
fn diagnostics_relation_schemas_use_db_store_table_metadata() {
    let cases = [
        (
            STATUS_DIAGNOSTICS_TABLE,
            status_snapshot_contract(),
            "corpus",
        ),
        (
            QUERY_TELEMETRY_DIAGNOSTICS_TABLE,
            query_telemetry_contract(),
            "captured_at",
        ),
        (
            STATUS_REASON_DIAGNOSTICS_TABLE,
            status_reason_contract(),
            "code",
        ),
        (
            REPO_READ_PRESSURE_DIAGNOSTICS_TABLE,
            repo_read_pressure_contract(),
            "budget",
        ),
    ];

    for (table_name, contract, first_column) in cases {
        let schema = diagnostics_schema_ref(&contract);

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
