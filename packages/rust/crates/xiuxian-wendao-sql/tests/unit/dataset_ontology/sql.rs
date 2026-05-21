use super::validate_dataset_ontology_select_only_sql;

#[test]
fn dataset_ontology_select_only_sql_accepts_select_and_with_queries() {
    assert!(
        validate_dataset_ontology_select_only_sql(
            "with rows as (select 1 as id) select id from rows"
        )
        .is_ok()
    );
    assert!(validate_dataset_ontology_select_only_sql("select 1;").is_ok());
    assert!(validate_dataset_ontology_select_only_sql("select '-- drop table x' as text").is_ok());
}

#[test]
fn dataset_ontology_select_only_sql_rejects_mutation_and_multiple_statements() {
    assert!(validate_dataset_ontology_select_only_sql("delete from raw_patients").is_err());
    assert!(validate_dataset_ontology_select_only_sql("select 1; select 2").is_err());
    assert!(validate_dataset_ontology_select_only_sql("").is_err());
}
